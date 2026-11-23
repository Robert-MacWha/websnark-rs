use ark_bn254::{Bn254, Fr, G1Affine, G1Projective};
use ark_ec::{AffineRepr, VariableBaseMSM, pairing::Pairing};
use ark_ff::One;
use tracing::instrument;

use crate::{
    proof::{ProofError, proof::Proof},
    verifying_key::VerifyingKey,
};

/// Verify a groth16 proof against a verifying key and the circuit's public signals.
///
/// Checks `e(A, B) = e(α, β) · e(vk_x, γ) · e(C, δ)`, where
/// `vk_x = IC₀ + Σ sᵢ·ICᵢ₊₁` over the public signals `sᵢ`.
///
/// Returns `Ok(false)` if the proof does not verify.
///
/// # Errors
/// Returns an error if the number of public signals does not match the
/// verifying key, or if a proof element is not a valid curve/subgroup point.
#[instrument(skip_all)]
pub fn verify(vk: &VerifyingKey, proof: &Proof, pub_signals: &[Fr]) -> Result<bool, ProofError> {
    if pub_signals.len() + 1 != vk.ic.len() {
        return Err(ProofError::InvalidPublicSignals {
            expected: vk.ic.len().saturating_sub(1),
            got: pub_signals.len(),
        });
    }

    if !(is_valid_g1(&proof.a) && is_valid_g1(&proof.c) && is_valid_g2(&proof.b)) {
        return Err(ProofError::InvalidProofPoint);
    }

    // vk_x = IC₀ + Σ sᵢ·ICᵢ₊₁
    let vk_x = G1Projective::msm_unchecked(&vk.ic[1..], pub_signals) + vk.ic[0];

    let out = Bn254::multi_pairing(
        [
            -proof.a.into_group(),
            vk.alpha.into_group(),
            vk_x,
            proof.c.into_group(),
        ],
        [proof.b, vk.beta, vk.gamma, vk.delta],
    );

    Ok(out.0.is_one())
}

fn is_valid_g1(p: &G1Affine) -> bool {
    p.is_on_curve() && p.is_in_correct_subgroup_assuming_on_curve()
}

fn is_valid_g2(p: &ark_bn254::G2Affine) -> bool {
    p.is_on_curve() && p.is_in_correct_subgroup_assuming_on_curve()
}

#[cfg(test)]
mod tests {
    use ark_bn254::G1Affine;
    use ark_ec::AffineRepr;
    use ark_ff::AdditiveGroup;

    use super::*;
    use crate::{circuit::Witness, proof::prove, proving_key::ProvingKey};

    struct Fixture {
        vk: VerifyingKey,
        proof: Proof,
        pub_signals: Vec<Fr>,
    }

    fn fixture(r: Fr, s: Fr) -> Fixture {
        let pk: ProvingKey =
            serde_json::from_str(include_str!("../testdata/withdraw_proving_key.json")).unwrap();
        let vk: VerifyingKey =
            serde_json::from_str(include_str!("../testdata/withdraw_verification_key.json"))
                .unwrap();
        let witness: Witness =
            serde_json::from_str(include_str!("../testdata/witness.json")).unwrap();

        let (proof, pub_signals) = prove(&pk, &witness, r, s).unwrap();
        Fixture {
            vk,
            proof,
            pub_signals,
        }
    }

    #[test]
    #[tracing_test::traced_test]
    fn prove_verify_roundtrip() {
        for (r, s) in [(Fr::ZERO, Fr::ZERO), (Fr::from(123u64), Fr::from(456u64))] {
            let f = fixture(r, s);
            assert!(verify(&f.vk, &f.proof, &f.pub_signals).unwrap());
        }
    }

    #[test]
    #[tracing_test::traced_test]
    fn verifies_stored_proof() {
        let f = fixture(Fr::ZERO, Fr::ZERO);
        let stored: Proof = serde_json::from_str(include_str!("../testdata/proof.json")).unwrap();
        assert!(verify(&f.vk, &stored, &f.pub_signals).unwrap());
    }

    #[test]
    #[tracing_test::traced_test]
    fn rejects_modified_public_signal() {
        let f = fixture(Fr::from(7u64), Fr::from(9u64));
        for i in 0..f.pub_signals.len() {
            let mut signals = f.pub_signals.clone();
            signals[i] += Fr::one();
            assert!(!verify(&f.vk, &f.proof, &signals).unwrap(), "signal {i}");
        }
    }

    #[test]
    #[tracing_test::traced_test]
    fn rejects_modified_proof_element() {
        let f = fixture(Fr::from(7u64), Fr::from(9u64));
        let g1 = G1Affine::generator();
        let g2 = ark_bn254::G2Affine::generator();

        let mut p = f.proof.clone();
        p.a = (p.a + g1).into();
        assert!(!verify(&f.vk, &p, &f.pub_signals).unwrap());

        let mut p = f.proof.clone();
        p.b = (p.b + g2).into();
        assert!(!verify(&f.vk, &p, &f.pub_signals).unwrap());

        let mut p = f.proof.clone();
        p.c = (p.c + g1).into();
        assert!(!verify(&f.vk, &p, &f.pub_signals).unwrap());
    }

    #[test]
    #[tracing_test::traced_test]
    fn rejects_wrong_public_signal_count() {
        let f = fixture(Fr::ZERO, Fr::ZERO);

        let mut short = f.pub_signals.clone();
        short.pop();
        assert!(matches!(
            verify(&f.vk, &f.proof, &short),
            Err(ProofError::InvalidPublicSignals { .. })
        ));

        let mut long = f.pub_signals.clone();
        long.push(Fr::ZERO);
        assert!(matches!(
            verify(&f.vk, &f.proof, &long),
            Err(ProofError::InvalidPublicSignals { .. })
        ));
    }

    #[test]
    #[tracing_test::traced_test]
    fn rejects_off_curve_proof_point() {
        let f = fixture(Fr::ZERO, Fr::ZERO);
        let mut p = f.proof.clone();
        p.a = G1Affine::new_unchecked(p.a.x, p.a.y + ark_bn254::Fq::one());
        assert!(matches!(
            verify(&f.vk, &p, &f.pub_signals),
            Err(ProofError::InvalidProofPoint)
        ));
    }
}
