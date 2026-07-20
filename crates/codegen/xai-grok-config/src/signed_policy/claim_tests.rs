//! Managed-identity claim verification, domain separation, and downgrade resistance.

use super::super::*;
use super::{keyset, payload, sign, test_keypair};

fn claim(principal: &str, fail_closed: bool, expires_at: u64) -> ManagedIdentityClaim {
    ManagedIdentityClaim {
        typ: MANAGED_IDENTITY_TYP.into(),
        principal: principal.into(),
        fail_closed,
        expires_at,
        key_id: "v1".into(),
    }
}

fn sign_claim(
    kp: &ring::signature::Ed25519KeyPair,
    claim: &ManagedIdentityClaim,
) -> SignatureEnvelope {
    let signed_payload = serde_json::to_string(claim).unwrap();
    let signature = kp.sign(signed_payload.as_bytes());
    SignatureEnvelope {
        signed_payload,
        signature: base64::engine::general_purpose::STANDARD.encode(signature.as_ref()),
        key_id: claim.key_id.clone(),
    }
}

fn write_claim(home: &std::path::Path, sidecar: &SignatureEnvelope) {
    write_managed_identity_sidecar(home, sidecar).unwrap();
}

#[test]
fn domain_separation_rejects_cross_type_substitution() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path();
    let (kp, pubkey) = test_keypair();
    let keys = keyset("v1", &pubkey);

    let claim_sidecar = sign_claim(&kp, &claim("synthetic-principal", true, 4_000_000_000));
    assert_eq!(
        verify_signed_payload(
            &claim_sidecar.signed_payload,
            &claim_sidecar.signature,
            &keys,
        ),
        Err(SigError::WrongType),
        "a managed-identity claim must not verify as policy"
    );

    std::fs::write(
        sidecar_path(home),
        serde_json::to_string(&claim_sidecar).unwrap(),
    )
    .unwrap();
    assert_eq!(
        signed_cache_compromised_with_keys(home, &keys, Some("synthetic-principal"), 1_000),
        SignedVerdict::NoAuthenticSidecar,
        "a substituted claim is not an authentic policy verdict"
    );

    let policy_sidecar = sign(&kp, &payload());
    assert_eq!(
        verify_managed_identity_claim(
            &policy_sidecar.signed_payload,
            &policy_sidecar.signature,
            &keys,
        ),
        Err(SigError::BadPayload),
        "a policy envelope must not verify as a managed-identity claim"
    );

    let mut wrong_type = claim("synthetic-principal", true, 4_000_000_000);
    wrong_type.typ = MANAGED_POLICY_TYP.into();
    let wrong_type = sign_claim(&kp, &wrong_type);
    assert_eq!(
        verify_managed_identity_claim(&wrong_type.signed_payload, &wrong_type.signature, &keys,),
        Err(SigError::WrongType)
    );
}

#[test]
fn identity_claim_removal_and_tamper_do_not_impose() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path();
    let (kp, pubkey) = test_keypair();
    let keys = keyset("v1", &pubkey);
    let sidecar = sign_claim(&kp, &claim("synthetic-principal", true, 4_000_000_000));

    write_claim(home, &sidecar);
    assert!(managed_identity_claim_imposes_with_keys(
        home,
        &keys,
        Some("synthetic-principal"),
        1_000,
    ));

    std::fs::remove_file(managed_identity_sidecar_path(home)).unwrap();
    assert!(!managed_identity_claim_imposes_with_keys(
        home,
        &keys,
        Some("synthetic-principal"),
        1_000,
    ));

    let mut tampered = sidecar;
    tampered.signature = base64::engine::general_purpose::STANDARD.encode([0u8; 64]);
    write_claim(home, &tampered);
    assert!(!managed_identity_claim_imposes_with_keys(
        home,
        &keys,
        Some("synthetic-principal"),
        1_000,
    ));
}

#[test]
fn identity_claim_requires_binding_opt_in_and_unexpired_time() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path();
    let (kp, pubkey) = test_keypair();
    let keys = keyset("v1", &pubkey);

    write_claim(
        home,
        &sign_claim(&kp, &claim("synthetic-principal", true, 2_000)),
    );
    assert!(managed_identity_claim_imposes_with_keys(
        home,
        &keys,
        Some("synthetic-principal"),
        1_000,
    ));
    assert!(!managed_identity_claim_imposes_with_keys(
        home,
        &keys,
        Some("synthetic-principal"),
        3_000,
    ));
    assert!(!managed_identity_claim_imposes_with_keys(
        home,
        &keys,
        Some("foreign-principal"),
        1_000,
    ));
    assert!(!managed_identity_claim_imposes_with_keys(
        home, &keys, None, 1_000,
    ));

    write_claim(
        home,
        &sign_claim(&kp, &claim("synthetic-principal", false, 4_000_000_000)),
    );
    assert!(!managed_identity_claim_imposes_with_keys(
        home,
        &keys,
        Some("synthetic-principal"),
        1_000,
    ));
}

#[test]
fn fetched_claim_rejects_expiry() {
    let (kp, pubkey) = test_keypair();
    let keys = keyset("v1", &pubkey);
    let sidecar = sign_claim(&kp, &claim("synthetic-principal", true, 2_000));
    assert!(verify_fetched_claim_with_keys(&sidecar, &keys, 1_000).is_ok());
    assert_eq!(
        verify_fetched_claim_with_keys(&sidecar, &keys, 3_000),
        Err(SigError::Expired)
    );
}
