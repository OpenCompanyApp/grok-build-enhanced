//! Managed-identity claim integration with the gate decision.

use super::super::*;
use super::team;

#[test]
fn claim_refuses_stripped_policy_sidecar_even_with_forged_marker() {
    use crate::signed_policy::SignedVerdict;
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path();
    let forged = ManagedConfigCache {
        principal: Some("synthetic-principal".into()),
        fail_closed: false,
        ..Default::default()
    };

    assert!(managed_policy_compromised_decision(
        SignedVerdict::NoAuthenticSidecar,
        || true,
        false,
        Some(&forged),
        home,
        &team("synthetic-principal"),
    ));
    assert!(!managed_policy_compromised_decision(
        SignedVerdict::NoAuthenticSidecar,
        || false,
        false,
        Some(&forged),
        home,
        &team("synthetic-principal"),
    ));
}

#[test]
fn claim_is_not_consulted_for_policy_sidecar_read_blip() {
    use crate::signed_policy::SignedVerdict;
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path();
    std::fs::write(home.join("requirements.toml"), "[features]\n").unwrap();
    let served = ManagedConfigCache {
        principal: Some("synthetic-principal".into()),
        had_requirements: true,
        fail_closed: true,
        ..Default::default()
    };

    assert!(!managed_policy_compromised_decision(
        SignedVerdict::SidecarUnreadable,
        || true,
        false,
        Some(&served),
        home,
        &team("synthetic-principal"),
    ));
}
