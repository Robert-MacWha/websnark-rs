use ark_bn254::{G1Affine, G2Affine};
use serde::{Deserialize, Serialize};

use crate::utils::{g1_to_string, g2_to_string, parse_f2, parse_g1};

/// CircomV1-compatible zk-SNARK proof
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof {
    pub a: G1Affine,
    pub b: G2Affine,
    pub c: G1Affine,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ProofJson {
    #[serde(rename = "pi_a")]
    pub a: [String; 3],
    #[serde(rename = "pi_b")]
    pub b: [[String; 2]; 3],
    #[serde(rename = "pi_c")]
    pub c: [String; 3],
}

impl Serialize for Proof {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let proof_json: ProofJson = self.clone().into();
        proof_json.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Proof {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let proof_json = ProofJson::deserialize(deserializer)?;
        proof_json.try_into().map_err(serde::de::Error::custom)
    }
}

impl From<Proof> for ProofJson {
    fn from(proof: Proof) -> Self {
        let a = g1_to_string(proof.a);
        let b = g2_to_string(proof.b);
        let c = g1_to_string(proof.c);

        ProofJson { a, b, c }
    }
}

impl TryFrom<ProofJson> for Proof {
    type Error = anyhow::Error;

    fn try_from(value: ProofJson) -> Result<Self, Self::Error> {
        let a = parse_g1(value.a)?;
        let b = parse_f2(value.b)?;
        let c = parse_g1(value.c)?;

        Ok(Proof { a, b, c })
    }
}
