//! The roster adapter, driven through the port's conformance suite, beside
//! a verifier that breaks the contract so the suite is seen to bite.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use foundry_caller_draft::conformance::{
    check_recognized_credential_is_exactly, check_recognizes_nobody, check_verification_is_total,
};
use foundry_caller_draft::{Caller, CallerVerifier, OperatorCredential, RosterVerifier};

/// Every shape a forged credential can take against `alice-token`: unrelated,
/// a prefix, one trailing byte, whitespace, a scheme prefix, and a NUL tail
/// exactly 256 bytes long, the length a `u8` length fold cannot see.
fn foreign_probes() -> Vec<String> {
    let mut probes: Vec<String> = ["", "nobody", "alice-toke", "alice-tokenX", "alice-token "]
        .into_iter()
        .chain(["Bearer alice-token", "\0"])
        .map(str::to_owned)
        .collect();
    probes.push(format!("alice-token{}", "\0".repeat(256)));
    probes
}

fn check_foreign_probes_verify_nobody(verifier: &RosterVerifier) -> Result<(), String> {
    let probes = foreign_probes();
    let probes: Vec<&str> = probes.iter().map(String::as_str).collect();
    check_recognizes_nobody(verifier, &probes)
}

fn credential(token: &str, principal: &str, roles: &[&str]) -> OperatorCredential {
    OperatorCredential {
        token: token.into(),
        tenant_id: "ten_acme".into(),
        principal_id: principal.into(),
        roles: roles.iter().map(|role| (*role).to_owned()).collect(),
    }
}

fn caller(principal: &str, roles: &[&str]) -> Caller {
    Caller {
        tenant_id: "ten_acme".into(),
        principal_id: principal.into(),
        roles: roles.iter().map(|role| (*role).to_owned()).collect(),
    }
}

fn roster() -> RosterVerifier {
    RosterVerifier::new(vec![
        credential("alice-token", "prn_alice", &["foundry-operator"]),
        credential("nobody-token", "prn_nobody", &[]),
        credential("zed-token", "prn_zed", &["foundry-operator", "auditor"]),
    ])
}

#[test]
fn an_empty_roster_verifies_nobody() {
    let empty = RosterVerifier::new(Vec::new());
    check_foreign_probes_verify_nobody(&empty).unwrap();
    check_recognizes_nobody(&empty, &["alice-token"]).unwrap();
}

#[test]
fn an_unknown_credential_verifies_nobody() {
    check_foreign_probes_verify_nobody(&roster()).unwrap();
}

#[test]
fn a_recognized_credential_returns_its_caller_exactly() {
    let roster = roster();
    check_recognized_credential_is_exactly(
        &roster,
        "alice-token",
        &caller("prn_alice", &["foundry-operator"]),
    )
    .unwrap();
    check_recognized_credential_is_exactly(&roster, "nobody-token", &caller("prn_nobody", &[]))
        .unwrap();
    check_recognized_credential_is_exactly(
        &roster,
        "zed-token",
        &caller("prn_zed", &["foundry-operator", "auditor"]),
    )
    .unwrap();
}

#[test]
fn verification_is_total() {
    check_verification_is_total(&roster()).unwrap();
    check_verification_is_total(&RosterVerifier::new(Vec::new())).unwrap();
}

struct AcceptsAnything;

impl CallerVerifier for AcceptsAnything {
    fn verify(&self, _presented_bearer: Option<&str>) -> Option<Caller> {
        Some(caller("prn_anyone", &["foundry-operator"]))
    }
}

#[test]
fn the_suite_refuses_a_verifier_that_recognizes_everyone() {
    let refused = check_recognizes_nobody(&AcceptsAnything, &["nobody"]).unwrap_err();
    assert!(refused.contains("verifies nobody"), "{refused}");
    let refused = check_verification_is_total(&AcceptsAnything).unwrap_err();
    assert!(refused.contains("absent credential"), "{refused}");
}

#[test]
fn the_suite_refuses_a_caller_with_its_roles_dropped() {
    let refused =
        check_recognized_credential_is_exactly(&roster(), "alice-token", &caller("prn_alice", &[]))
            .unwrap_err();
    assert!(refused.contains("exactly"), "{refused}");
}
