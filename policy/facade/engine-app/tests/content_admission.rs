#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod support;

use policy_engine_app::{EngineLoadError, PolicyEngine};
use policy_pdp_kernel::PolicyDecisionPoint;
use support::*;

#[test]
fn signed_bundle_content_identity_is_verified_on_reload_and_restart() {
    let fixture = Fixture::new();
    let first = project().prepare(ids()).unwrap();
    first
        .publish(
            &fixture.store,
            &fixture.signer,
            &fixture.signer.public_key_bytes(),
        )
        .unwrap();
    let engine = PolicyEngine::load(&fixture.store, ids(), 4).unwrap();
    assert!(
        !engine
            .authorize(&request("alice"), &entities("alice"))
            .unwrap()
            .cache_hit
    );
    assert!(
        engine
            .authorize(&request("alice"), &entities("alice"))
            .unwrap()
            .cache_hit
    );
    let mut second = project();
    second.source.policies_src.push('\n');
    let second = second.prepare(ids()).unwrap();
    second
        .publish(
            &fixture.store,
            &fixture.signer,
            &fixture.signer.public_key_bytes(),
        )
        .unwrap();
    engine.reload(&fixture.store).unwrap();
    let mut inconsistent = second.bundle().clone();
    inconsistent.version = first.bundle().version.clone();
    fixture
        .store
        .write_signed_bundle(
            &inconsistent,
            &fixture.signer,
            &fixture.signer.public_key_bytes(),
        )
        .unwrap();
    assert!(matches!(
        engine.reload(&fixture.store),
        Err(EngineLoadError::ContentIdentity { .. })
    ));
    assert_eq!(engine.loaded_policy_version(), second.bundle().version);
    assert!(matches!(
        PolicyEngine::load(&fixture.store, ids(), 4),
        Err(EngineLoadError::ContentIdentity { .. })
    ));
    first
        .publish(
            &fixture.store,
            &fixture.signer,
            &fixture.signer.public_key_bytes(),
        )
        .unwrap();
    assert_eq!(
        engine.reload(&fixture.store).unwrap(),
        first.bundle().version
    );
    assert!(
        !engine
            .authorize(&request("alice"), &entities("alice"))
            .unwrap()
            .cache_hit
    );
}
