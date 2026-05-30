use std::ops::Deref;
use std::str::FromStr;

use ark_bn254::Fr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Witness(Vec<Fr>);

impl Witness {
    pub fn new(witness: Vec<Fr>) -> Self {
        Witness(witness)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct WitnessJson(Vec<String>);

impl<'de> Deserialize<'de> for Witness {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let witness_json = WitnessJson::deserialize(deserializer)?;
        let witness = witness_json
            .0
            .iter()
            .map(|v| Fr::from_str(v).map_err(|_| serde::de::Error::custom("Failed to parse Fr")))
            .collect::<Result<_, _>>()?;

        Ok(Witness(witness))
    }
}

impl Serialize for Witness {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let witness_json = WitnessJson(
            self.0
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>(),
        );
        witness_json.serialize(serializer)
    }
}

impl Deref for Witness {
    type Target = Vec<Fr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
