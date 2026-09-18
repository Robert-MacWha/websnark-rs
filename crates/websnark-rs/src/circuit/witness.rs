use std::ops::Deref;

use ark_bn254::Fr;
#[cfg(feature = "serde")]
use crate::serde::FieldElement;

#[cfg_attr(feature = "serde", cfg_eval::cfg_eval, serde_with::serde_as)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Witness(#[cfg_attr(feature = "serde", serde_as(as = "Vec<FieldElement>"))] Vec<Fr>);

impl Witness {
    #[must_use]
    pub fn new(witness: Vec<Fr>) -> Self {
        Witness(witness)
    }
}

impl Deref for Witness {
    type Target = Vec<Fr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;

    #[test]
    fn witness_postcard_roundtrip() {
        let witness = Witness::new(vec![Fr::from(0u64), Fr::from(1u64), Fr::from(u64::MAX)]);

        let bytes = postcard::to_stdvec(&witness).unwrap();
        let roundtripped: Witness = postcard::from_bytes(&bytes).unwrap();

        assert_eq!(witness, roundtripped);
    }
}
