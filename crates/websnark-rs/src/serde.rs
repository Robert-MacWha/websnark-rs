use std::collections::HashMap;

use anyhow::anyhow;
use ark_bn254::{Fq, Fq2, Fr, G1Affine, G2Affine};
use ark_ec::AffineRepr;
use std::str::FromStr;

pub mod g1_serde {
    use ark_bn254::G1Affine;
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &G1Affine, s: S) -> Result<S::Ok, S::Error> {
        let mut bytes = Vec::new();
        v.serialize_uncompressed(&mut bytes)
            .map_err(serde::ser::Error::custom)?;
        s.serialize_bytes(&bytes)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<G1Affine, D::Error> {
        let bytes = Vec::<u8>::deserialize(d)?;
        G1Affine::deserialize_uncompressed_unchecked(&bytes[..]).map_err(serde::de::Error::custom)
    }
}

pub mod g2_serde {
    use ark_bn254::G2Affine;
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &G2Affine, s: S) -> Result<S::Ok, S::Error> {
        let mut bytes = Vec::new();
        v.serialize_uncompressed(&mut bytes)
            .map_err(serde::ser::Error::custom)?;
        s.serialize_bytes(&bytes)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<G2Affine, D::Error> {
        let bytes = Vec::<u8>::deserialize(d)?;
        G2Affine::deserialize_uncompressed_unchecked(&bytes[..]).map_err(serde::de::Error::custom)
    }
}

pub mod g1_vec_serde {
    use ark_bn254::G1Affine;
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
    use serde::{Deserialize, Deserializer, Serializer, ser::SerializeSeq};

    pub fn serialize<S: Serializer>(v: &[G1Affine], s: S) -> Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(v.len()))?;
        for p in v {
            let mut bytes = Vec::new();
            p.serialize_uncompressed(&mut bytes)
                .map_err(serde::ser::Error::custom)?;
            seq.serialize_element(bytes.as_slice())?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<G1Affine>, D::Error> {
        let vecs = Vec::<Vec<u8>>::deserialize(d)?;
        vecs.into_iter()
            .map(|bytes| {
                G1Affine::deserialize_uncompressed_unchecked(&bytes[..])
                    .map_err(serde::de::Error::custom)
            })
            .collect()
    }
}

pub mod g2_vec_serde {
    use ark_bn254::G2Affine;
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
    use serde::{Deserialize, Deserializer, Serializer, ser::SerializeSeq};

    pub fn serialize<S: Serializer>(v: &[G2Affine], s: S) -> Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(v.len()))?;
        for p in v {
            let mut bytes = Vec::new();
            p.serialize_uncompressed(&mut bytes)
                .map_err(serde::ser::Error::custom)?;
            seq.serialize_element(bytes.as_slice())?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<G2Affine>, D::Error> {
        let vecs = Vec::<Vec<u8>>::deserialize(d)?;
        vecs.into_iter()
            .map(|bytes| {
                G2Affine::deserialize_uncompressed_unchecked(&bytes[..])
                    .map_err(serde::de::Error::custom)
            })
            .collect()
    }
}

pub mod fr_map_vec_serde {
    use std::collections::HashMap;

    use ark_bn254::Fr;
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
    use serde::{Deserialize, Deserializer, Serializer, ser::SerializeSeq};

    pub fn serialize<S: Serializer>(v: &[HashMap<u64, Fr>], s: S) -> Result<S::Ok, S::Error> {
        let mut outer = s.serialize_seq(Some(v.len()))?;
        for map in v {
            let mut pairs: Vec<(u64, Vec<u8>)> = Vec::with_capacity(map.len());
            for (&k, fr) in map {
                let mut bytes = Vec::new();
                fr.serialize_uncompressed(&mut bytes)
                    .map_err(serde::ser::Error::custom)?;
                pairs.push((k, bytes));
            }
            outer.serialize_element(&pairs)?;
        }
        outer.end()
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<HashMap<u64, Fr>>, D::Error> {
        let outer = Vec::<Vec<(u64, Vec<u8>)>>::deserialize(d)?;
        outer
            .into_iter()
            .map(|pairs| {
                pairs
                    .into_iter()
                    .map(|(k, bytes)| {
                        Fr::deserialize_uncompressed_unchecked(&bytes[..])
                            .map(|fr| (k, fr))
                            .map_err(serde::de::Error::custom)
                    })
                    .collect::<Result<HashMap<u64, Fr>, _>>()
            })
            .collect()
    }
}

/// Parse snarkjs JSON string format for G1Affine
pub mod g1_json_serde {
    use ark_bn254::G1Affine;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(v: &G1Affine, s: S) -> Result<S::Ok, S::Error> {
        super::g1_to_string(*v).serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<G1Affine, D::Error> {
        let arr = <[String; 3]>::deserialize(d)?;
        super::parse_g1(arr).map_err(serde::de::Error::custom)
    }
}

/// Parse snarkjs JSON string format for G2Affine
pub mod g2_json_serde {
    use ark_bn254::G2Affine;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(v: &G2Affine, s: S) -> Result<S::Ok, S::Error> {
        super::g2_to_string(*v).serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<G2Affine, D::Error> {
        let arr = <[[String; 2]; 3]>::deserialize(d)?;
        super::parse_f2(arr).map_err(serde::de::Error::custom)
    }
}

/// Parse snarkjs JSON string format for `Vec<G1Affine>`
pub mod g1_vec_json_serde {
    use ark_bn254::G1Affine;
    use ark_ec::AffineRepr;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(v: &[G1Affine], s: S) -> Result<S::Ok, S::Error> {
        let strs: Vec<[String; 3]> = v.iter().map(|p| super::g1_to_string(*p)).collect();
        strs.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<G1Affine>, D::Error> {
        let arrs = Vec::<Option<[String; 3]>>::deserialize(d)?;
        arrs.into_iter()
            .map(|opt| match opt {
                Some(arr) => super::parse_g1(arr).map_err(serde::de::Error::custom),
                None => Ok(G1Affine::zero()),
            })
            .collect()
    }
}

/// Parse snarkjs JSON string format for `Vec<G2Affine>`
pub mod g2_vec_json_serde {
    use ark_bn254::G2Affine;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(v: &[G2Affine], s: S) -> Result<S::Ok, S::Error> {
        let strs: Vec<[[String; 2]; 3]> = v.iter().map(|p| super::g2_to_string(*p)).collect();
        strs.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<G2Affine>, D::Error> {
        let arrs = Vec::<[[String; 2]; 3]>::deserialize(d)?;
        arrs.into_iter()
            .map(|arr| super::parse_f2(arr).map_err(serde::de::Error::custom))
            .collect()
    }
}

/// Parse snarkjs JSON string format for `Vec<HashMap<u64, Fr>>`
pub mod fr_map_vec_json_serde {
    use std::collections::HashMap;

    use ark_bn254::Fr;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(v: &[HashMap<u64, Fr>], s: S) -> Result<S::Ok, S::Error> {
        let string_maps: Vec<HashMap<u64, String>> = v
            .iter()
            .map(|map| map.iter().map(|(&k, fr)| (k, fr.to_string())).collect())
            .collect();
        string_maps.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<HashMap<u64, Fr>>, D::Error> {
        let string_maps = Vec::<HashMap<u64, String>>::deserialize(d)?;
        super::parse_pols(string_maps).map_err(serde::de::Error::custom)
    }
}

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

fn parse_g1(value: [String; 3]) -> Result<G1Affine, anyhow::Error> {
    let is_zero = value[2] == "0";
    if is_zero {
        return Ok(G1Affine::zero());
    }

    let x = Fq::from_str(&value[0]).map_err(|_| anyhow!("Failed to parse x coord"))?;
    let y = Fq::from_str(&value[1]).map_err(|_| anyhow!("Failed to parse y coord"))?;

    Ok(G1Affine::new_unchecked(x, y))
}

fn parse_f2(value: [[String; 2]; 3]) -> Result<G2Affine, anyhow::Error> {
    let is_zero = value[2][0] == "0" && value[2][1] == "0";
    if is_zero {
        return Ok(G2Affine::zero());
    }

    let x = Fq2::new(
        Fq::from_str(&value[0][0]).map_err(|_| anyhow!("Failed to parse x0 coord"))?,
        Fq::from_str(&value[0][1]).map_err(|_| anyhow!("Failed to parse x1 coord"))?,
    );
    let y = Fq2::new(
        Fq::from_str(&value[1][0]).map_err(|_| anyhow!("Failed to parse y0 coord"))?,
        Fq::from_str(&value[1][1]).map_err(|_| anyhow!("Failed to parse y1 coord"))?,
    );

    Ok(G2Affine::new_unchecked(x, y))
}

fn parse_pols(value: Vec<HashMap<u64, String>>) -> Result<Vec<HashMap<u64, Fr>>, anyhow::Error> {
    value
        .iter()
        .map(|map| {
            map.iter()
                .map(|(&k, v)| {
                    Ok((
                        k,
                        Fr::from_str(v).map_err(|_| anyhow!("Failed to parse Fr"))?,
                    ))
                })
                .collect()
        })
        .collect()
}
