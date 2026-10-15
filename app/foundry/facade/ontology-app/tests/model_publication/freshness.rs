use crate::{model_support::*, native::*};

#[tokio::test]
async fn fresh_canonical_membership_is_required_after_pure_replay_before_commit() {
    for (answer, status, code) in [
        (
            Ok(view(&["foundry-model-author"])),
            StatusCode::FORBIDDEN,
            ModelRefusalCode::Forbidden,
        ),
        (
            Ok(SessionView {
                user_id: "usr_changed_owner".into(),
                ..view(&["foundry-model-publisher"])
            }),
            StatusCode::FORBIDDEN,
            ModelRefusalCode::Forbidden,
        ),
        (
            Ok(SessionView {
                expires_at: now() - 1,
                ..view(&["foundry-model-publisher"])
            }),
            StatusCode::UNAUTHORIZED,
            ModelRefusalCode::Unauthenticated,
        ),
        (
            Err(SessionRefusal::MembershipUnavailable),
            StatusCode::SERVICE_UNAVAILABLE,
            ModelRefusalCode::ModelUnavailable,
        ),
    ] {
        let fixture = NativeFixture::new("fresh-publish");
        let state = fixture.state(&["foundry-model-author", "foundry-model-publisher"]);
        let saved = create(&state.router).await;
        let before = state.app.tenants[TENANT].lock().await.projection().clone();
        let after = state.sessions.calls.load(Ordering::SeqCst) + 3;
        *state.sessions.after_call.lock().unwrap() = Some((after, answer));
        refusal(
            send(&state.router, publication_request(&saved)).await,
            status,
            code,
        )
        .await;
        let tenant = state.app.tenants[TENANT].lock().await;
        assert_eq!(tenant.observed_model_generation(), Some(1));
        assert_eq!(tenant.projection(), &before);
        assert_eq!(tenant.action_log().head(TENANT).unwrap(), 0);
        assert_eq!(tenant.denial_log().head(TENANT).unwrap(), 0);
        drop(tenant);
        *state.sessions.after_call.lock().unwrap() = None;
        assert_eq!(lifecycle(&state).await.desired_generation, number(1));
    }
}
