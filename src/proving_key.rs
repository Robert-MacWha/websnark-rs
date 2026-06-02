use std::collections::HashMap;

use ark_bn254::{Fr, G1Affine, G2Affine};
use serde::{Deserialize, Serialize};

use crate::serde::{
    fr_map_vec_json_serde, fr_map_vec_serde, g1_json_serde, g1_serde, g1_vec_json_serde,
    g1_vec_serde, g2_json_serde, g2_serde, g2_vec_json_serde, g2_vec_serde,
};

/// CircomV1-compatible proving key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvingKey {
    #[serde(with = "g1_vec_serde")]
    pub a: Vec<G1Affine>,
    #[serde(with = "g1_vec_serde")]
    pub b_g1: Vec<G1Affine>,
    #[serde(with = "g2_vec_serde")]
    pub b_g2: Vec<G2Affine>,
    #[serde(with = "g1_vec_serde")]
    pub c: Vec<G1Affine>,
    pub n_vars: u64,
    pub n_public: u64,
    #[serde(with = "g1_serde")]
    pub vk_alpha_g1: G1Affine,
    #[serde(with = "g1_serde")]
    pub vk_beta_g1: G1Affine,
    #[serde(with = "g2_serde")]
    pub vk_beta_g2: G2Affine,
    #[serde(with = "g1_serde")]
    pub vk_delta_g1: G1Affine,
    #[serde(with = "g2_serde")]
    pub vk_delta_g2: G2Affine,
    #[serde(with = "g1_vec_serde")]
    pub h_exps: Vec<G1Affine>,
    pub domain_size: u64,
    #[serde(with = "fr_map_vec_serde")]
    pub pols_a: Vec<HashMap<u64, Fr>>,
    #[serde(with = "fr_map_vec_serde")]
    pub pols_b: Vec<HashMap<u64, Fr>>,
}

impl ProvingKey {
    /// Deserialize from the snarkjs JSON format (decimal string encoded fields).
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        let j: ProvingKeyJson = serde_json::from_str(s)?;
        Ok(ProvingKey {
            a: j.a,
            b_g1: j.b_g1,
            b_g2: j.b_g2,
            c: j.c,
            n_vars: j.n_vars,
            n_public: j.n_public,
            vk_alpha_g1: j.vk_alpha_g1,
            vk_beta_g1: j.vk_beta_g1,
            vk_beta_g2: j.vk_beta_g2,
            vk_delta_g1: j.vk_delta_g1,
            vk_delta_g2: j.vk_delta_g2,
            h_exps: j.h_exps,
            domain_size: j.domain_size,
            pols_a: j.pols_a,
            pols_b: j.pols_b,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct ProvingKeyJson {
    #[serde(rename = "A", with = "g1_vec_json_serde")]
    a: Vec<G1Affine>,
    #[serde(rename = "B1", with = "g1_vec_json_serde")]
    b_g1: Vec<G1Affine>,
    #[serde(rename = "B2", with = "g2_vec_json_serde")]
    b_g2: Vec<G2Affine>,
    #[serde(rename = "C", with = "g1_vec_json_serde")]
    c: Vec<G1Affine>,
    #[serde(rename = "nVars")]
    n_vars: u64,
    #[serde(rename = "nPublic")]
    n_public: u64,
    #[serde(rename = "domainSize")]
    domain_size: u64,
    #[serde(rename = "hExps", with = "g1_vec_json_serde")]
    h_exps: Vec<G1Affine>,
    #[serde(rename = "polsA", with = "fr_map_vec_json_serde")]
    pols_a: Vec<HashMap<u64, Fr>>,
    #[serde(rename = "polsB", with = "fr_map_vec_json_serde")]
    pols_b: Vec<HashMap<u64, Fr>>,
    #[serde(rename = "vk_alfa_1", with = "g1_json_serde")]
    vk_alpha_g1: G1Affine,
    #[serde(rename = "vk_beta_1", with = "g1_json_serde")]
    vk_beta_g1: G1Affine,
    #[serde(rename = "vk_beta_2", with = "g2_json_serde")]
    vk_beta_g2: G2Affine,
    #[serde(rename = "vk_delta_1", with = "g1_json_serde")]
    vk_delta_g1: G1Affine,
    #[serde(rename = "vk_delta_2", with = "g2_json_serde")]
    vk_delta_g2: G2Affine,
}

#[cfg(test)]
mod tests {
    use super::*;

    use ark_ec::AffineRepr;

    #[test]
    fn pkey_roundtrip() {
        let pk =
            ProvingKey::from_json(include_str!("./testdata/withdraw_proving_key.json")).unwrap();

        for (i, p) in pk.a.iter().enumerate() {
            if !p.is_zero() {
                assert!(p.is_on_curve(), "A[{}] is not on curve: {:?}", i, p);
                assert!(
                    p.is_in_correct_subgroup_assuming_on_curve(),
                    "A[{}] is not in correct subgroup: {:?}",
                    i,
                    p
                );
            }
        }

        for (i, p) in pk.b_g1.iter().enumerate() {
            if !p.is_zero() {
                assert!(p.is_on_curve(), "B_g1[{}] is not on curve: {:?}", i, p);
                assert!(
                    p.is_in_correct_subgroup_assuming_on_curve(),
                    "B_g1[{}] is not in correct subgroup: {:?}",
                    i,
                    p
                );
            }
        }

        for (i, p) in pk.b_g2.iter().enumerate() {
            if !p.is_zero() {
                assert!(p.is_on_curve(), "B_g2[{}] is not on curve: {:?}", i, p);
                assert!(
                    p.is_in_correct_subgroup_assuming_on_curve(),
                    "B_g2[{}] is not in correct subgroup: {:?}",
                    i,
                    p
                );
            }
        }

        for (i, p) in pk.c.iter().enumerate() {
            if !p.is_zero() {
                assert!(p.is_on_curve(), "C[{}] is not on curve: {:?}", i, p);
                assert!(
                    p.is_in_correct_subgroup_assuming_on_curve(),
                    "C[{}] is not in correct subgroup: {:?}",
                    i,
                    p
                );
            }
        }

        for (i, p) in pk.h_exps.iter().enumerate() {
            if !p.is_zero() {
                assert!(p.is_on_curve(), "h_exps[{}] is not on curve: {:?}", i, p);
                assert!(
                    p.is_in_correct_subgroup_assuming_on_curve(),
                    "h_exps[{}] is not in correct subgroup: {:?}",
                    i,
                    p
                );
            }
        }

        assert!(pk.vk_alpha_g1.is_on_curve(), "vk_alpha_g1 is not on curve");
        assert!(
            pk.vk_alpha_g1.is_in_correct_subgroup_assuming_on_curve(),
            "vk_alpha_g1 is not in correct subgroup"
        );

        assert!(pk.vk_beta_g1.is_on_curve(), "vk_beta_g1 is not on curve");
        assert!(
            pk.vk_beta_g1.is_in_correct_subgroup_assuming_on_curve(),
            "vk_beta_g1 is not in correct subgroup"
        );

        assert!(pk.vk_beta_g2.is_on_curve(), "vk_beta_g2 is not on curve");
        assert!(
            pk.vk_beta_g2.is_in_correct_subgroup_assuming_on_curve(),
            "vk_beta_g2 is not in correct subgroup"
        );

        assert!(pk.vk_delta_g1.is_on_curve(), "vk_delta_g1 is not on curve");
        assert!(
            pk.vk_delta_g1.is_in_correct_subgroup_assuming_on_curve(),
            "vk_delta_g1 is not in correct subgroup"
        );

        assert!(pk.vk_delta_g2.is_on_curve(), "vk_delta_g2 is not on curve");
        assert!(
            pk.vk_delta_g2.is_in_correct_subgroup_assuming_on_curve(),
            "vk_delta_g2 is not in correct subgroup"
        );
    }
}
