use ark_bn254::{Fq, Fq2, Fr, G1Affine, G2Affine};
use ark_ec::AffineRepr;
use ark_ff::PrimeField;
use std::array::TryFromSliceError;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
enum FqParseError {
    #[error("invalid length")]
    InvalidLength,
    #[error("invalid field")]
    InvalidField(),
    #[error("slice error")]
    SliceError(#[from] TryFromSliceError),
}

pub mod g1_serde {
    use ark_bn254::G1Affine;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(v: &G1Affine, s: S) -> Result<S::Ok, S::Error> {
        if s.is_human_readable() {
            super::g1_to_string(*v).serialize(s)
        } else {
            super::g1_to_bytes(*v).serialize(s)
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<G1Affine, D::Error> {
        if d.is_human_readable() {
            let arr = <[String; 3]>::deserialize(d)?;
            super::parse_g1(&arr).map_err(serde::de::Error::custom)
        } else {
            let bytes = Vec::<u8>::deserialize(d)?;
            super::g1_from_bytes(&bytes).map_err(serde::de::Error::custom)
        }
    }
}

pub mod g2_serde {
    use ark_bn254::G2Affine;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(v: &G2Affine, s: S) -> Result<S::Ok, S::Error> {
        if s.is_human_readable() {
            super::g2_to_string(*v).serialize(s)
        } else {
            super::g2_to_bytes(*v).serialize(s)
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<G2Affine, D::Error> {
        if d.is_human_readable() {
            let arr = <[[String; 2]; 3]>::deserialize(d)?;
            super::parse_f2(&arr).map_err(serde::de::Error::custom)
        } else {
            let bytes = Vec::<u8>::deserialize(d)?;
            super::g2_from_bytes(&bytes).map_err(serde::de::Error::custom)
        }
    }
}

pub mod g1_vec_serde {
    use ark_bn254::G1Affine;
    use ark_ec::AffineRepr;
    use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeSeq};

    pub fn serialize<S: Serializer>(v: &[G1Affine], s: S) -> Result<S::Ok, S::Error> {
        if s.is_human_readable() {
            let strs: Vec<[String; 3]> = v.iter().map(|p| super::g1_to_string(*p)).collect();
            strs.serialize(s)
        } else {
            let mut seq = s.serialize_seq(Some(v.len()))?;
            for p in v {
                seq.serialize_element(&super::g1_to_bytes(*p))?;
            }
            seq.end()
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<G1Affine>, D::Error> {
        if d.is_human_readable() {
            let arrs = Vec::<Option<[String; 3]>>::deserialize(d)?;
            arrs.into_iter()
                .map(|opt| match opt {
                    Some(arr) => super::parse_g1(&arr).map_err(serde::de::Error::custom),
                    None => Ok(G1Affine::zero()),
                })
                .collect()
        } else {
            let vecs = Vec::<Vec<u8>>::deserialize(d)?;
            vecs.into_iter()
                .map(|bytes| super::g1_from_bytes(&bytes).map_err(serde::de::Error::custom))
                .collect()
        }
    }
}

pub mod g2_vec_serde {
    use ark_bn254::G2Affine;
    use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeSeq};

    pub fn serialize<S: Serializer>(v: &[G2Affine], s: S) -> Result<S::Ok, S::Error> {
        if s.is_human_readable() {
            let strs: Vec<[[String; 2]; 3]> = v.iter().map(|p| super::g2_to_string(*p)).collect();
            strs.serialize(s)
        } else {
            let mut seq = s.serialize_seq(Some(v.len()))?;
            for p in v {
                seq.serialize_element(&super::g2_to_bytes(*p))?;
            }
            seq.end()
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<G2Affine>, D::Error> {
        if d.is_human_readable() {
            let arrs = Vec::<[[String; 2]; 3]>::deserialize(d)?;
            arrs.into_iter()
                .map(|arr| super::parse_f2(&arr).map_err(serde::de::Error::custom))
                .collect()
        } else {
            let vecs = Vec::<Vec<u8>>::deserialize(d)?;
            vecs.into_iter()
                .map(|bytes| super::g2_from_bytes(&bytes).map_err(serde::de::Error::custom))
                .collect()
        }
    }
}

pub mod fr_map_vec_serde {
    use ark_bn254::Fr;
    use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeSeq};
    use std::collections::HashMap;

    pub fn serialize<S: Serializer>(v: &[HashMap<usize, Fr>], s: S) -> Result<S::Ok, S::Error> {
        if s.is_human_readable() {
            let string_maps: Vec<HashMap<usize, String>> = v
                .iter()
                .map(|map| map.iter().map(|(&k, fr)| (k, fr.to_string())).collect())
                .collect();
            string_maps.serialize(s)
        } else {
            let mut outer = s.serialize_seq(Some(v.len()))?;
            for map in v {
                let pairs: Vec<(usize, [u8; 32])> = map
                    .iter()
                    .map(|(&k, fr)| (k, super::fr_to_bytes(*fr)))
                    .collect();
                outer.serialize_element(&pairs)?;
            }
            outer.end()
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Vec<HashMap<usize, Fr>>, D::Error> {
        if d.is_human_readable() {
            let string_maps = Vec::<HashMap<usize, String>>::deserialize(d)?;
            super::parse_pols(&string_maps).map_err(serde::de::Error::custom)
        } else {
            let outer = Vec::<Vec<(usize, [u8; 32])>>::deserialize(d)?;
            outer
                .into_iter()
                .map(|pairs| {
                    pairs
                        .into_iter()
                        .map(|(k, bytes)| {
                            super::fr_from_bytes(bytes)
                                .map(|fr| (k, fr))
                                .map_err(serde::de::Error::custom)
                        })
                        .collect::<Result<HashMap<usize, Fr>, _>>()
                })
                .collect()
        }
    }
}

// --- shared helpers ---

fn fr_to_bytes(fr: Fr) -> [u8; 32] {
    let limbs = fr.into_bigint().0;
    let mut bytes = [0u8; 32];
    for (i, limb) in limbs.iter().enumerate() {
        bytes[i * 8..(i + 1) * 8].copy_from_slice(&limb.to_le_bytes());
    }
    bytes
}

fn fr_from_bytes(bytes: [u8; 32]) -> Result<Fr, TryFromSliceError> {
    let mut limbs = [0u64; 4];
    for (i, limb) in limbs.iter_mut().enumerate() {
        *limb = u64::from_le_bytes(bytes[i * 8..(i + 1) * 8].try_into()?);
    }
    Ok(Fr::from(ark_ff::BigInt(limbs)))
}

fn g1_to_bytes(value: G1Affine) -> Vec<u8> {
    let Some(xy) = value.xy() else {
        return vec![0u8; 64];
    };
    let mut bytes = [0u8; 64];
    for (i, limb) in xy.0.into_bigint().0.iter().enumerate() {
        bytes[i * 8..(i + 1) * 8].copy_from_slice(&limb.to_le_bytes());
    }
    for (i, limb) in xy.1.into_bigint().0.iter().enumerate() {
        bytes[32 + i * 8..32 + (i + 1) * 8].copy_from_slice(&limb.to_le_bytes());
    }
    bytes.to_vec()
}

fn g1_from_bytes(bytes: &[u8]) -> Result<G1Affine, FqParseError> {
    if bytes.len() != 64 {
        return Err(FqParseError::InvalidLength);
    }
    if bytes.iter().all(|&b| b == 0) {
        return Ok(G1Affine::zero());
    }
    let x = fq_from_bytes(&bytes[..32])?;
    let y = fq_from_bytes(&bytes[32..])?;
    Ok(G1Affine::new_unchecked(x, y))
}

fn g2_to_bytes(value: G2Affine) -> Vec<u8> {
    let Some(xy) = value.xy() else {
        return vec![0u8; 128];
    };
    let mut bytes = [0u8; 128];
    for (i, limb) in xy.0.c0.into_bigint().0.iter().enumerate() {
        bytes[i * 8..(i + 1) * 8].copy_from_slice(&limb.to_le_bytes());
    }
    for (i, limb) in xy.0.c1.into_bigint().0.iter().enumerate() {
        bytes[32 + i * 8..32 + (i + 1) * 8].copy_from_slice(&limb.to_le_bytes());
    }
    for (i, limb) in xy.1.c0.into_bigint().0.iter().enumerate() {
        bytes[64 + i * 8..64 + (i + 1) * 8].copy_from_slice(&limb.to_le_bytes());
    }
    for (i, limb) in xy.1.c1.into_bigint().0.iter().enumerate() {
        bytes[96 + i * 8..96 + (i + 1) * 8].copy_from_slice(&limb.to_le_bytes());
    }
    bytes.to_vec()
}

fn g2_from_bytes(bytes: &[u8]) -> Result<G2Affine, FqParseError> {
    if bytes.len() != 128 {
        return Err(FqParseError::InvalidLength);
    }
    if bytes.iter().all(|&b| b == 0) {
        return Ok(G2Affine::zero());
    }
    let x = Fq2::new(fq_from_bytes(&bytes[..32])?, fq_from_bytes(&bytes[32..64])?);
    let y = Fq2::new(fq_from_bytes(&bytes[64..96])?, fq_from_bytes(&bytes[96..])?);
    Ok(G2Affine::new_unchecked(x, y))
}

fn fq_from_bytes(bytes: &[u8]) -> Result<Fq, TryFromSliceError> {
    let mut limbs = [0u64; 4];
    for (i, limb) in limbs.iter_mut().enumerate() {
        *limb = u64::from_le_bytes(bytes[i * 8..(i + 1) * 8].try_into()?);
    }
    Ok(Fq::from(ark_ff::BigInt(limbs)))
}

// --- string helpers ---

fn g1_to_string(value: G1Affine) -> [String; 3] {
    let Some(xy) = value.xy() else {
        return ["0".to_string(), "0".to_string(), "0".to_string()];
    };

    let x = xy.0.to_string();
    let y = xy.1.to_string();

    [x, y, "1".to_string()]
}

fn g2_to_string(value: G2Affine) -> [[String; 2]; 3] {
    let Some(xy) = value.xy() else {
        return [
            ["0".to_string(), "0".to_string()],
            ["0".to_string(), "0".to_string()],
            ["0".to_string(), "0".to_string()],
        ];
    };

    let x0 = xy.0.c0.to_string();
    let x1 = xy.0.c1.to_string();
    let y0 = xy.1.c0.to_string();
    let y1 = xy.1.c1.to_string();

    [[x0, x1], [y0, y1], ["1".to_string(), "0".to_string()]]
}

fn parse_g1(value: &[String; 3]) -> Result<G1Affine, FqParseError> {
    let is_zero = value[2] == "0";
    if is_zero {
        return Ok(G1Affine::zero());
    }

    let x = Fq::from_str(&value[0]).map_err(|()| FqParseError::InvalidField())?;
    let y = Fq::from_str(&value[1]).map_err(|()| FqParseError::InvalidField())?;

    Ok(G1Affine::new_unchecked(x, y))
}

fn parse_f2(value: &[[String; 2]; 3]) -> Result<G2Affine, FqParseError> {
    let is_zero = value[2][0] == "0" && value[2][1] == "0";
    if is_zero {
        return Ok(G2Affine::zero());
    }

    let x = Fq2::new(
        Fq::from_str(&value[0][0]).map_err(|()| FqParseError::InvalidField())?,
        Fq::from_str(&value[0][1]).map_err(|()| FqParseError::InvalidField())?,
    );
    let y = Fq2::new(
        Fq::from_str(&value[1][0]).map_err(|()| FqParseError::InvalidField())?,
        Fq::from_str(&value[1][1]).map_err(|()| FqParseError::InvalidField())?,
    );

    Ok(G2Affine::new_unchecked(x, y))
}

fn parse_pols(value: &[HashMap<usize, String>]) -> Result<Vec<HashMap<usize, Fr>>, FqParseError> {
    value
        .iter()
        .map(|map| {
            map.iter()
                .map(|(&k, v)| {
                    Ok((
                        k,
                        Fr::from_str(v).map_err(|()| FqParseError::InvalidField())?,
                    ))
                })
                .collect()
        })
        .collect()
}
