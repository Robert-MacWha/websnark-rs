use ark_bn254::{G1Affine, G2Affine};
use serde::{Deserialize, Serialize};

use crate::serde::{g1_json_serde, g1_serde, g2_json_serde, g2_serde};

/// CircomV1-compatible zk-SNARK proof
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proof {
    #[serde(with = "g1_serde")]
    pub a: G1Affine,
    #[serde(with = "g2_serde")]
    pub b: G2Affine,
    #[serde(with = "g1_serde")]
    pub c: G1Affine,
}

impl Proof {
    /// Deserialize from the snarkjs JSON format (decimal string encoded fields).
    pub fn from_json(s: &str) -> Result<Self, anyhow::Error> {
        let j: ProofJson = serde_json::from_str(s)?;
        Ok(Proof {
            a: j.a,
            b: j.b,
            c: j.c,
        })
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct ProofJson {
    #[serde(rename = "pi_a", with = "g1_json_serde")]
    a: G1Affine,
    #[serde(rename = "pi_b", with = "g2_json_serde")]
    b: G2Affine,
    #[serde(rename = "pi_c", with = "g1_json_serde")]
    c: G1Affine,
}
