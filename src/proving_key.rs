use std::collections::HashMap;

use ark_bn254::{Fr, G1Affine, G2Affine};
use ark_ec::AffineRepr;
use serde::{Deserialize, Serialize};

use crate::utils::{parse_f2, parse_g1, parse_pols};

/// CircomV1-compatible proving key
#[derive(Debug, Clone)]
pub struct ProvingKey {
    pub a: Vec<G1Affine>,
    pub b_g1: Vec<G1Affine>,
    pub b_g2: Vec<G2Affine>,
    pub c: Vec<G1Affine>,
    pub n_vars: u64,
    pub n_public: u64,
    pub vk_alpha_g1: G1Affine,
    pub vk_beta_g1: G1Affine,
    pub vk_beta_g2: G2Affine,
    pub vk_delta_g1: G1Affine,
    pub vk_delta_g2: G2Affine,
    pub h_exps: Vec<G1Affine>,
    pub domain_size: u64,
    pub pols_a: Vec<HashMap<u64, Fr>>,
    pub pols_b: Vec<HashMap<u64, Fr>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProvingKeyJson {
    #[serde(rename = "A")]
    pub a: Vec<[String; 3]>,
    #[serde(rename = "B1")]
    pub b1: Vec<[String; 3]>,
    #[serde(rename = "B2")]
    pub b2: Vec<[[String; 2]; 3]>,
    #[serde(rename = "C")]
    pub c: Vec<Option<[String; 3]>>,
    pub protocol: String,
    #[serde(rename = "nVars")]
    pub n_vars: u64,
    #[serde(rename = "nPublic")]
    pub n_public: u64,
    #[serde(rename = "domainSize")]
    pub domain_size: u64,
    #[serde(rename = "domainBits")]
    pub domain_bits: u64,
    #[serde(rename = "hExps")]
    pub h_exps: Vec<[String; 3]>,
    #[serde(rename = "polsA")]
    pub pols_a: Vec<HashMap<u64, String>>,
    #[serde(rename = "polsB")]
    pub pols_b: Vec<HashMap<u64, String>>,
    #[serde(rename = "polsC")]
    pub pols_c: Vec<HashMap<u64, String>>,
    pub vk_alfa_1: [String; 3],
    pub vk_delta_1: [String; 3],
    pub vk_beta_1: [String; 3],
    pub vk_beta_2: [[String; 2]; 3],
    pub vk_delta_2: [[String; 2]; 3],
}

impl<'de> Deserialize<'de> for ProvingKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let pk_json = ProvingKeyJson::deserialize(deserializer)?;
        pk_json.try_into().map_err(serde::de::Error::custom)
    }
}

impl TryFrom<ProvingKeyJson> for ProvingKey {
    type Error = anyhow::Error;

    fn try_from(value: ProvingKeyJson) -> Result<Self, Self::Error> {
        let a = value
            .a
            .into_iter()
            .map(parse_g1)
            .collect::<Result<Vec<_>, _>>()?;

        let b_g1 = value
            .b1
            .into_iter()
            .map(parse_g1)
            .collect::<Result<Vec<_>, _>>()?;

        let b_g2 = value
            .b2
            .into_iter()
            .map(parse_f2)
            .collect::<Result<Vec<_>, _>>()?;

        let c = value
            .c
            .into_iter()
            .map(|opt| match opt {
                Some(v) => parse_g1(v),
                None => Ok(G1Affine::zero()),
            })
            .collect::<Result<Vec<_>, _>>()?;

        let n_vars = value.n_vars;
        let n_public = value.n_public;
        let domain_size = value.domain_size;

        let vk_alpha_g1 = parse_g1(value.vk_alfa_1)?;
        let vk_beta_g1 = parse_g1(value.vk_beta_1)?;
        let vk_beta_g2 = parse_f2(value.vk_beta_2)?;
        let vk_delta_g1 = parse_g1(value.vk_delta_1)?;
        let vk_delta_g2 = parse_f2(value.vk_delta_2)?;

        let h_exps = value
            .h_exps
            .into_iter()
            .map(parse_g1)
            .collect::<Result<Vec<_>, _>>()?;

        let pols_a = parse_pols(value.pols_a)?;
        let pols_b = parse_pols(value.pols_b)?;

        Ok(ProvingKey {
            a,
            b_g1,
            b_g2,
            c,
            n_vars,
            n_public,
            vk_alpha_g1,
            vk_beta_g1,
            vk_beta_g2,
            vk_delta_g1,
            vk_delta_g2,
            h_exps,
            domain_size,
            pols_a,
            pols_b,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkey_roundtrip() {
        let pk: ProvingKey =
            serde_json::from_str(include_str!("./testdata/withdraw_proving_key.json")).unwrap();

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
