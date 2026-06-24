use ark_bn254::{G1Affine, G2Affine};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use crate::serde::{g1_serde, g1_vec_serde, g2_serde};

/// CircomV1-compatible verifying key
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VerifyingKey {
    #[cfg_attr(feature = "serde", serde(rename = "vk_alfa_1", with = "g1_serde"))]
    pub alpha: G1Affine,
    #[cfg_attr(feature = "serde", serde(rename = "vk_beta_2", with = "g2_serde"))]
    pub beta: G2Affine,
    #[cfg_attr(feature = "serde", serde(rename = "vk_gamma_2", with = "g2_serde"))]
    pub gamma: G2Affine,
    #[cfg_attr(feature = "serde", serde(rename = "vk_delta_2", with = "g2_serde"))]
    pub delta: G2Affine,
    #[cfg_attr(feature = "serde", serde(rename = "IC", with = "g1_vec_serde"))]
    pub ic: Vec<G1Affine>,
}
