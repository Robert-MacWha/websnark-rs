use ark_bn254::{G1Affine, G2Affine};
use serde::{Deserialize, Serialize};

use crate::serde::{
    g1_json_serde, g1_serde, g1_vec_json_serde, g1_vec_serde, g2_json_serde, g2_serde,
};

/// CircomV1-compatible verifying key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyingKey {
    #[serde(with = "g1_serde")]
    pub alpha: G1Affine,
    #[serde(with = "g2_serde")]
    pub beta: G2Affine,
    #[serde(with = "g2_serde")]
    pub gamma: G2Affine,
    #[serde(with = "g2_serde")]
    pub delta: G2Affine,
    #[serde(with = "g1_vec_serde")]
    pub ic: Vec<G1Affine>,
}

impl VerifyingKey {
    /// Deserialize from the snarkjs JSON format (decimal string encoded fields).
    pub fn from_json(s: &str) -> Result<Self, anyhow::Error> {
        let j: VerifyingKeyJson = serde_json::from_str(s)?;
        Ok(VerifyingKey {
            alpha: j.alpha,
            beta: j.beta,
            gamma: j.gamma,
            delta: j.delta,
            ic: j.ic,
        })
    }
}

#[derive(Deserialize)]
struct VerifyingKeyJson {
    #[serde(rename = "vk_alfa_1", with = "g1_json_serde")]
    alpha: G1Affine,
    #[serde(rename = "vk_beta_2", with = "g2_json_serde")]
    beta: G2Affine,
    #[serde(rename = "vk_gamma_2", with = "g2_json_serde")]
    gamma: G2Affine,
    #[serde(rename = "vk_delta_2", with = "g2_json_serde")]
    delta: G2Affine,
    #[serde(rename = "IC", with = "g1_vec_json_serde")]
    ic: Vec<G1Affine>,
}
