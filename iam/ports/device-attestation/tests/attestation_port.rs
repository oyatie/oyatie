#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use iam_device_attestation_api::{AttestationError, AttestationKind, DeviceAttestation, bind};

#[test]
fn closed_kinds_are_passkey_mdm_chrome_spiffe() {
    assert_eq!(
        AttestationKind::CLOSED,
        [
            AttestationKind::Passkey,
            AttestationKind::Mdm,
            AttestationKind::ChromeEnterprise,
            AttestationKind::SpiffeWorkload,
        ]
    );
}

#[test]
fn empty_token_fails_closed() {
    let port = bind(AttestationKind::Passkey);
    assert_eq!(port.verify(b"").unwrap_err(), AttestationError::TokenEmpty);
}

#[test]
fn unwired_adapters_fail_closed_and_are_not_a_browser() {
    for kind in AttestationKind::CLOSED {
        let port = bind(kind);
        assert_eq!(port.kind(), kind);
        assert_eq!(
            port.verify(b"token").unwrap_err(),
            AttestationError::AdapterNotWired(kind)
        );
    }
}
