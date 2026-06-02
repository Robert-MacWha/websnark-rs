use std::ops::Deref;
use std::str::FromStr;

use ark_bn254::Fr;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use serde::{Deserialize, Serialize, de::Error, ser::SerializeSeq};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Witness(Vec<Fr>);

impl Witness {
    pub fn new(witness: Vec<Fr>) -> Self {
        Witness(witness)
    }

    /// Deserialize from the snarkjs JSON format (array of decimal strings).
    pub fn from_json(s: &str) -> Result<Self, anyhow::Error> {
        let strings: Vec<String> = serde_json::from_str(s)?;
        let frs = strings
            .iter()
            .map(|v| Fr::from_str(v).map_err(|_| anyhow::anyhow!("Failed to parse Fr from '{v}'")))
            .collect::<Result<_, _>>()?;
        Ok(Witness(frs))
    }

    /// Serialize to the snarkjs JSON format (array of decimal strings).
    pub fn to_json(&self) -> String {
        let strings: Vec<String> = self.0.iter().map(|v| v.to_string()).collect();
        serde_json::to_string(&strings).expect("infallible")
    }
}

/// Binary serde: sequence of uncompressed Fr byte arrays.
impl Serialize for Witness {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(self.0.len()))?;
        for fr in &self.0 {
            let mut bytes = Vec::new();
            fr.serialize_uncompressed(&mut bytes)
                .map_err(serde::ser::Error::custom)?;
            seq.serialize_element(bytes.as_slice())?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Witness {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let vecs = Vec::<Vec<u8>>::deserialize(d)?;
        let frs = vecs
            .into_iter()
            .map(|bytes| {
                Fr::deserialize_uncompressed_unchecked(&bytes[..]).map_err(D::Error::custom)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Witness(frs))
    }
}

impl Deref for Witness {
    type Target = Vec<Fr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
