use std::ops::Deref;
#[cfg(feature = "serde")]
use std::str::FromStr;

use ark_bn254::Fr;
#[cfg(feature = "serde")]
use ark_ff::PrimeField;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize, de::Error, ser::SerializeSeq};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Witness(Vec<Fr>);

impl Witness {
    #[must_use] 
    pub fn new(witness: Vec<Fr>) -> Self {
        Witness(witness)
    }
}

#[cfg(feature = "serde")]
impl Serialize for Witness {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        if s.is_human_readable() {
            let mut seq = s.serialize_seq(Some(self.0.len()))?;
            for fr in &self.0 {
                seq.serialize_element(&fr.to_string())?;
            }
            seq.end()
        } else {
            let mut seq = s.serialize_seq(Some(self.0.len()))?;
            for fr in &self.0 {
                let limbs = fr.into_bigint().0;
                let mut bytes = [0u8; 32];
                for (i, limb) in limbs.iter().enumerate() {
                    bytes[i * 8..(i + 1) * 8].copy_from_slice(&limb.to_le_bytes());
                }
                seq.serialize_element(&bytes)?;
            }
            seq.end()
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Witness {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        if d.is_human_readable() {
            let strings = Vec::<String>::deserialize(d)?;
            let frs = strings
                .into_iter()
                .map(|v| {
                    Fr::from_str(&v)
                        .map_err(|()| D::Error::custom(format!("Failed to parse Fr from '{v}'")))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Witness(frs))
        } else {
            let vecs = Vec::<Vec<u8>>::deserialize(d)?;
            let frs =
                vecs.into_iter()
                    .map(|bytes| {
                        if bytes.len() != 32 {
                            return Err(D::Error::custom(format!(
                                "expected 32 bytes, got {}",
                                bytes.len()
                            )));
                        }
                        let mut limbs = [0u64; 4];
                        for (i, limb) in limbs.iter_mut().enumerate() {
                            *limb =
                                u64::from_le_bytes(bytes[i * 8..(i + 1) * 8].try_into().map_err(
                                    |_| D::Error::custom("Failed to convert bytes to u64"),
                                )?);
                        }
                        Ok(Fr::from(ark_ff::BigInt(limbs)))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
            Ok(Witness(frs))
        }
    }
}

impl Deref for Witness {
    type Target = Vec<Fr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
