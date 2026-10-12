use super::*;

#[test]
fn policy_preview_uses_one_markup_with_truthful_sources_and_counts() {
    let owner = Owner::new();
    for context in OperatorContext::ALL {
        let envelope = server_derived_envelope(context);
        let expected_count = envelope.modules.len();
        let html = owner.with(|| policy_preview_panel(envelope).into_view().to_html());
        assert!(html.contains("Sample policy preview"));
        assert!(html.contains("Live policy evaluation and verified grants are unavailable"));
        assert!(html.contains("no native policy check was performed"));
        assert!(html.contains("example decision rows"));
        assert!(html.contains(&format!("<strong>{expected_count}</strong>")));
        assert!(html.contains("sample module cards"));
        assert!(html.contains("<strong>0</strong><small>live policy checks</small>"));
        assert!(html.contains(&format!("data-policy-context=\"{}\"", context.id())));
        assert!(!html.contains("Cedar rules mirrored"));
        assert!(!html.contains("tenant pack grants"));
        if context == OperatorContext::TenantAdmin {
            assert!(static_dashboard_html().contains(&html));
        }
    }
}

#[test]
fn policy_preview_escapes_role_text_in_shared_native_markup() {
    let mut envelope = server_derived_envelope(OperatorContext::TenantAdmin);
    envelope.role_name = "<script>alert(1)</script>".to_owned();
    let html = Owner::new().with(|| policy_preview_panel(envelope).into_view().to_html());
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
}

#[test]
fn policy_channel_snapshots_keep_envelope_list_detail_and_status_together() {
    let mut envelope = server_derived_envelope(OperatorContext::TenantAdmin);
    envelope.messages = vec![MessageItem {
        from: "Tenant mail <owner>".to_owned(),
        channel: "Mail".to_owned(),
        preview: "Mail envelope body & scope".to_owned(),
    }];
    envelope.community = vec![CommunityItem {
        space: "Tenant council".to_owned(),
        topic: "Community <review>".to_owned(),
        activity: "Community envelope body & scope".to_owned(),
    }];
    for (surface, title, body, excluded) in [
        (
            ProductSurface::Mail,
            "Tenant mail &lt;owner&gt;",
            "Mail envelope body &amp; scope",
            "Community &lt;review&gt;",
        ),
        (
            ProductSurface::Community,
            "Community &lt;review&gt;",
            "Community envelope body &amp; scope",
            "Tenant mail &lt;owner&gt;",
        ),
    ] {
        let html = policy_hub_html(&envelope, surface);
        assert!(html.contains(&format!("<strong>{title}</strong>")));
        assert!(html.contains(&format!("<h4>{title}</h4>")));
        assert_eq!(html.matches(&format!("<p>{body}</p>")).count(), 2);
        assert!(html.contains(&format!("<dt>Route</dt><dd>{}</dd>", surface.label())));
        assert!(html.contains(&format!(
            "{} sample preview · no message sent or posted",
            surface.label()
        )));
        let visible_workspace = html.split_once("class=\"hub-list\"").unwrap().1;
        assert!(!visible_workspace.contains(excluded));
        assert!(!html.contains("<owner>") && !html.contains("<review>"));
    }
}
