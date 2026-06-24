use std::collections::HashMap;

use ark_bn254::{Fr, G1Affine, G2Affine};
use serde::{Deserialize, Serialize};

use crate::serde::{fr_map_vec_serde, g1_serde, g1_vec_serde, g2_serde, g2_vec_serde};

/// CircomV1-compatible proving key
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvingKey {
    #[serde(rename = "A", with = "g1_vec_serde")]
    pub a: Vec<G1Affine>,
    #[serde(rename = "B1", with = "g1_vec_serde")]
    pub b_g1: Vec<G1Affine>,
    #[serde(rename = "B2", with = "g2_vec_serde")]
    pub b_g2: Vec<G2Affine>,
    #[serde(rename = "C", with = "g1_vec_serde")]
    pub c: Vec<G1Affine>,
    #[serde(rename = "nVars")]
    pub n_vars: u64,
    #[serde(rename = "nPublic")]
    pub n_public: u64,
    #[serde(rename = "vk_alfa_1", with = "g1_serde")]
    pub vk_alpha_g1: G1Affine,
    #[serde(rename = "vk_beta_1", with = "g1_serde")]
    pub vk_beta_g1: G1Affine,
    #[serde(rename = "vk_beta_2", with = "g2_serde")]
    pub vk_beta_g2: G2Affine,
    #[serde(rename = "vk_delta_1", with = "g1_serde")]
    pub vk_delta_g1: G1Affine,
    #[serde(rename = "vk_delta_2", with = "g2_serde")]
    pub vk_delta_g2: G2Affine,
    #[serde(rename = "hExps", with = "g1_vec_serde")]
    pub h_exps: Vec<G1Affine>,
    #[serde(rename = "domainSize")]
    pub domain_size: u64,
    #[serde(rename = "polsA", with = "fr_map_vec_serde")]
    pub pols_a: Vec<HashMap<u64, Fr>>,
    #[serde(rename = "polsB", with = "fr_map_vec_serde")]
    pub pols_b: Vec<HashMap<u64, Fr>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    use ark_ec::AffineRepr;

    #[test]
    fn pkey_roundtrip() {
        let pk_data = include_str!("./testdata/withdraw_proving_key.json");
        let pk: ProvingKey = serde_json::from_str(pk_data).unwrap();

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
