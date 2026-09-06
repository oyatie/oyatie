use super::credentials::check_certificate;
use super::process::{LIMIT, run_with_timeout};
use super::sessions::{retry_bastion_auth, valid_session_id};

use super::*;
use sha2::{Digest, Sha256};

#[test]
fn bastion_auth_readiness_retry_is_bounded_and_never_relaxes_host_trust() {
    assert!(retry_bastion_auth(
        "permission denied (publickey)",
        Duration::from_secs(2)
    ));
    assert!(!retry_bastion_auth(
        "permission denied (publickey)",
        Duration::from_secs(90)
    ));
    assert!(!retry_bastion_auth(
        "host key verification failed; permission denied (publickey)",
        Duration::ZERO
    ));
    assert!(!retry_bastion_auth("connection refused", Duration::ZERO));
}

#[test]
fn session_ids_admit_oci_region_hyphens_but_not_other_regions_or_arguments() {
    assert!(valid_session_id(
        "ocid1.bastionsession.oc1.ap-chuncheon-1.abc",
        "ap-chuncheon-1"
    ));
    assert!(!valid_session_id(
        "ocid1.bastionsession.oc1.other.abc",
        "ap-chuncheon-1"
    ));
    assert!(!valid_session_id("--endpoint=other", "ap-chuncheon-1"));
}

#[test]
fn bounds_dependency_output() {
    assert_eq!(
        read_bounded(&vec![0; LIMIT + 1][..]).unwrap_err(),
        AccessError::OutputLimit
    );
    assert_eq!(read_bounded(&b"small"[..]).unwrap().as_slice(), b"small");
}

#[test]
fn dependency_failures_never_render_stderr_or_credentials() {
    let error = run(
        "/bin/sh",
        &strings(&["-c", "echo secret-fixture >&2; exit 7"]),
        &[],
        false,
    )
    .unwrap_err();
    assert_eq!(error.to_string(), "operator_access_dependency_failed");
}

#[test]
fn process_timeout_is_bounded_and_reaps_process_group() {
    let start = Instant::now();
    assert_eq!(
        run_with_timeout(
            "/bin/sh",
            &strings(&["-c", "sleep 60"]),
            &[],
            false,
            Duration::from_millis(50),
            &AtomicBool::new(false)
        )
        .unwrap_err(),
        AccessError::Timeout
    );
    assert!(start.elapsed() < Duration::from_secs(5));
}

#[test]
fn cancellation_reaps_the_command_but_does_not_prevent_cleanup() {
    let cancelled = AtomicBool::new(true);
    assert_eq!(
        run_with_timeout(
            "/bin/sh",
            &strings(&["-c", "sleep 60"]),
            &[],
            false,
            Duration::from_secs(2),
            &cancelled
        )
        .unwrap_err(),
        AccessError::Cancelled
    );
    assert!(
        run_with_timeout(
            "/bin/sh",
            &strings(&["-c", "exit 0"]),
            &[],
            true,
            Duration::from_secs(2),
            &cancelled
        )
        .is_ok()
    );
}

#[test]
fn cleanup_attempts_every_session_and_retains_failures_for_retry() {
    let mut ids = vec!["first".to_string(), "second".to_string()];
    let mut attempted = Vec::new();
    let result = cleanup_ids(&mut ids, |id| {
        attempted.push(id.to_string());
        if id == "first" {
            Err(AccessError::DependencyFailed)
        } else {
            Ok(())
        }
    });
    assert_eq!(result, Err(AccessError::CleanupFailed));
    assert_eq!(attempted, ["first", "second"]);
    assert_eq!(ids, ["first"]);
    assert_eq!(cleanup_ids(&mut ids, |_| Ok(())), Ok(()));
    assert!(ids.is_empty());
}

#[test]
fn certificates_reject_wrong_trust_and_expired_identity() {
    let key = rcgen::KeyPair::generate().unwrap();
    let mut params = rcgen::CertificateParams::default();
    params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    let ca = params.self_signed(&key).unwrap();
    let digest = format!("{:x}", Sha256::digest(ca.pem().as_bytes()));
    assert_eq!(
        check_certificate(ca.pem().as_bytes(), ca.pem().as_bytes(), &digest),
        Ok(())
    );
    assert_eq!(
        check_certificate(ca.pem().as_bytes(), ca.pem().as_bytes(), &"0".repeat(64)),
        Err(AccessError::InvalidCredentials)
    );
    params.not_before = rcgen::date_time_ymd(2000, 1, 1);
    params.not_after = rcgen::date_time_ymd(2001, 1, 1);
    let expired = params.self_signed(&key).unwrap();
    let digest = format!("{:x}", Sha256::digest(expired.pem().as_bytes()));
    assert_eq!(
        check_certificate(expired.pem().as_bytes(), expired.pem().as_bytes(), &digest),
        Err(AccessError::CertificateExpired)
    );
}
