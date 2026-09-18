#[cfg(feature = "serde")]
use crate::serde::{G1, G2};
use ark_bn254::{G1Affine, G2Affine};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// CircomV1-compatible verifying key
#[cfg_attr(feature = "serde", cfg_eval::cfg_eval, serde_with::serde_as)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VerifyingKey {
    #[cfg_attr(feature = "serde", serde(rename = "vk_alfa_1"), serde_as(as = "G1"))]
    pub alpha: G1Affine,
    #[cfg_attr(feature = "serde", serde(rename = "vk_beta_2"), serde_as(as = "G2"))]
    pub beta: G2Affine,
    #[cfg_attr(feature = "serde", serde(rename = "vk_gamma_2"), serde_as(as = "G2"))]
    pub gamma: G2Affine,
    #[cfg_attr(feature = "serde", serde(rename = "vk_delta_2"), serde_as(as = "G2"))]
    pub delta: G2Affine,
    #[cfg_attr(feature = "serde", serde(rename = "IC"), serde_as(as = "Vec<G1>"))]
    pub ic: Vec<G1Affine>,
}
