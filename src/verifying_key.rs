use ark_bn254::{G1Affine, G2Affine};
use serde::{Deserialize, Serialize};

use crate::utils::{parse_f2, parse_g1};

/// CircomV1-compatible verifying key
#[derive(Debug, Clone)]
pub struct VerifyingKey {
    pub alpha: G1Affine,
    pub beta: G2Affine,
    pub gamma: G2Affine,
    pub delta: G2Affine,
    pub ic: Vec<G1Affine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VerifyingKeyJson {
    #[serde(rename = "vk_alfa_1")]
    pub alpha: [String; 3],
    #[serde(rename = "vk_beta_2")]
    pub beta: [[String; 2]; 3],
    #[serde(rename = "vk_gamma_2")]
    pub gamma: [[String; 2]; 3],
    #[serde(rename = "vk_delta_2")]
    pub delta: [[String; 2]; 3],
    #[serde(rename = "IC")]
    pub ic: Vec<[String; 3]>,
}

impl<'de> Deserialize<'de> for VerifyingKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let vk_json = VerifyingKeyJson::deserialize(deserializer)?;
        vk_json.try_into().map_err(serde::de::Error::custom)
    }
}

impl TryFrom<VerifyingKeyJson> for VerifyingKey {
    type Error = anyhow::Error;

    fn try_from(value: VerifyingKeyJson) -> Result<Self, Self::Error> {
        let alpha = parse_g1(value.alpha)?;
        let beta = parse_f2(value.beta)?;
        let gamma = parse_f2(value.gamma)?;
        let delta = parse_f2(value.delta)?;

        let mut ic = Vec::with_capacity(value.ic.len());
        for point in value.ic {
            ic.push(parse_g1(point)?);
        }

        Ok(VerifyingKey {
            alpha,
            beta,
            gamma,
            delta,
            ic,
        })
    }
}
