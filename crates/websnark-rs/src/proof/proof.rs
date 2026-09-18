use ark_bn254::{G1Affine, G2Affine};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use crate::serde::{G1, G2};

/// CircomV1-compatible zk-SNARK proof
#[cfg_attr(feature = "serde", cfg_eval::cfg_eval, serde_with::serde_as)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Proof {
    #[cfg_attr(feature = "serde", serde(rename = "pi_a"), serde_as(as = "G1"))]
    pub a: G1Affine,
    #[cfg_attr(feature = "serde", serde(rename = "pi_b"), serde_as(as = "G2"))]
    pub b: G2Affine,
    #[cfg_attr(feature = "serde", serde(rename = "pi_c"), serde_as(as = "G1"))]
    pub c: G1Affine,
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;

    #[test]
    fn proof_postcard_roundtrip() {
        let proof_data = include_str!("../testdata/proof.json");
        let proof: Proof = serde_json::from_str(proof_data).unwrap();

        let bytes = postcard::to_stdvec(&proof).unwrap();
        let roundtripped: Proof = postcard::from_bytes(&bytes).unwrap();

        assert_eq!(proof, roundtripped);
    }
}
