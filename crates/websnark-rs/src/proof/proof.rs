use ark_bn254::{G1Affine, G2Affine};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use crate::serde::{g1_serde, g2_serde};

/// CircomV1-compatible zk-SNARK proof
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Proof {
    #[cfg_attr(feature = "serde", serde(rename = "pi_a", with = "g1_serde"))]
    pub a: G1Affine,
    #[cfg_attr(feature = "serde", serde(rename = "pi_b", with = "g2_serde"))]
    pub b: G2Affine,
    #[cfg_attr(feature = "serde", serde(rename = "pi_c", with = "g1_serde"))]
    pub c: G1Affine,
}
