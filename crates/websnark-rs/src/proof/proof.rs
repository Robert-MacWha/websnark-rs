use ark_bn254::{G1Affine, G2Affine};
use serde::{Deserialize, Serialize};

use crate::serde::{g1_serde, g2_serde};

/// CircomV1-compatible zk-SNARK proof
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proof {
    #[serde(rename = "pi_a", with = "g1_serde")]
    pub a: G1Affine,
    #[serde(rename = "pi_b", with = "g2_serde")]
    pub b: G2Affine,
    #[serde(rename = "pi_c", with = "g1_serde")]
    pub c: G1Affine,
}
