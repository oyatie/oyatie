//! The executable contract: every adapter runs this suite unchanged.
//!
//! Checks return `Err(String)` naming the violated clause rather than
//! panicking, so an adapter's test can attribute a failure to the contract
//! line it broke.

use crate::{Caller, CallerVerifier};

fn fail(clause: &str, detail: String) -> String {
    format!("{clause}: {detail}")
}

/// Every probe is a credential the verifier must not know.
pub fn check_recognizes_nobody<V: CallerVerifier>(
    verifier: &V,
    probes: &[&str],
) -> Result<(), String> {
    for probe in probes {
        if let Some(caller) = verifier.verify(Some(probe)) {
            return Err(fail(
                "an unknown credential verifies nobody",
                format!("{probe:?} minted {caller:?}"),
            ));
        }
    }
    Ok(())
}

/// A recognized credential returns its bound caller whole: tenant,
/// principal and roles, an empty role list included.
pub fn check_recognized_credential_is_exactly<V: CallerVerifier>(
    verifier: &V,
    presented: &str,
    expected: &Caller,
) -> Result<(), String> {
    match verifier.verify(Some(presented)) {
        Some(caller) if caller == *expected => Ok(()),
        other => Err(fail(
            "a recognized credential returns its bound caller exactly",
            format!("want {expected:?}, got {other:?}"),
        )),
    }
}

/// No input reaches a panic, and an absent credential is nobody.
pub fn check_verification_is_total<V: CallerVerifier>(verifier: &V) -> Result<(), String> {
    if let Some(caller) = verifier.verify(None) {
        return Err(fail(
            "an absent credential verifies nobody",
            format!("minted {caller:?}"),
        ));
    }
    let long = "x".repeat(1 << 16);
    for garbage in ["", " ", "\t\n", "Bearer x", "\0", "\u{fffd}", long.as_str()] {
        let _ = verifier.verify(Some(garbage));
    }
    Ok(())
}
