use super::*;

#[test]
fn governance_sample_scope_does_not_manufacture_roles() {
    let owner = Owner::new();
    owner.with(|| {
        for context in OperatorContext::ALL {
            let source = server_derived_envelope(context);
            let html = tenant_rbac_board(source.clone(), Signal::derive(|| true))
                .into_view()
                .to_html();
            let shown = context != OperatorContext::HealthcareClinician;
            for marker in [
                "class=\"business-logic-os\"",
                "id=\"governance-analytics\"",
                "id=\"governance-analytics-summary\"",
            ] {
                assert_eq!(html.contains(marker), shown, "{context:?}: {marker}");
            }
            for marker in ["id=\"payroll-cockpit\"", "id=\"employee-directory\""] {
                assert!(html.contains(marker), "preserve unrelated {marker}");
            }
            assert_eq!(
                html.contains("id=\"filing-readiness\""),
                finance_sample_scope(context).filing_examples
            );
            for logic in [false, true] {
                let rail = governance_rail_html(context, logic);
                assert_eq!(rail.is_empty(), !shown);
                if shown {
                    assert!(rail.contains("sample") || rail.contains("Sample"));
                    assert!(!rail.contains("17"));
                }
            }
            let compatibility = static_governance_preview(&source);
            assert_eq!(compatibility.contains("data-logic-row=\"true\""), shown);
            assert!(
                html.contains("No tenant or role is verified")
                    || html.contains("no tenant or role is verified")
            );
        }
    });
    owner.cleanup();
}

#[test]
fn governance_source_binds_semantic_fixtures_without_render_mode() {
    let original = server_derived_envelope(OperatorContext::TenantAdmin);
    let identity = governance_sample_source(&original);
    let facts: serde_json::Value = serde_json::from_str(&identity).unwrap();
    assert_eq!(facts[0], "tenant-admin");
    assert_eq!(facts[2], HubSample::envelope_source(&original));
    let body = facts[3].as_str().unwrap();
    assert!(body.contains("data-risk-detail="));
    assert!(body.contains("data-gov-calendar-description="));
    assert_eq!(
        body.matches("data-logic-record=\"").count(),
        BUSINESS_LOGIC_ROWS.len()
    );
    for (before, after) in [
        ("#vendors-spend", "#finance-panel-vendors"),
        ("#identity-onboarding", "#identity-panel-onboarding"),
    ] {
        let compatibility = business_logic_os_html(false);
        let native = business_logic_os_html(true);
        assert!(compatibility.contains(&format!("href=\"{before}\"")));
        assert!(native.contains(&format!("href=\"{after}\"")));
        for html in [compatibility, native] {
            let fields = html
                .split("data-logic-search-text=\"")
                .skip(1)
                .map(|part| part.split_once('"').unwrap().0)
                .collect::<Vec<_>>();
            assert!(fields.iter().any(|field| field.contains(after)));
            assert!(!fields.iter().any(|field| field.contains(before)));
        }
    }
    let mut changed = original.clone();
    changed.workflow.name.push_str(" changed");
    assert_ne!(identity, governance_sample_source(&changed));
    changed = original.clone();
    changed.workflow.goal.push_str(" changed");
    assert_ne!(identity, governance_sample_source(&changed));
    changed = original.clone();
    changed.role_name.push_str(" changed");
    assert_ne!(identity, governance_sample_source(&changed));
    changed = original.clone();
    changed.messages[0].preview.push_str(" changed");
    assert_ne!(identity, governance_sample_source(&changed));
    changed = original;
    changed.workflow.name = "<script>unsafe & \"quoted\"</script>".into();
    let html = governance_body_html(&changed, true);
    assert!(!html.contains("<script>unsafe"));
    assert!(html.contains("&lt;script&gt;unsafe &amp; &quot;quoted&quot;&lt;/script&gt;"));
}

#[test]
fn governance_calendar_risks_and_metrics_are_explicit_examples() {
    let source = server_derived_envelope(OperatorContext::CorporateOffice);
    let html = governance_body_html(&source, true);
    assert_eq!(html.matches("data-governance-risk=\"").count(), 5);
    let risks = html
        .split("data-governance-risk=\"")
        .skip(1)
        .map(|part| part.split_once("</button>").unwrap().0)
        .collect::<Vec<_>>();
    assert_eq!(
        risks
            .iter()
            .filter(|part| part.contains("aria-pressed=\"true\""))
            .count(),
        1
    );
    assert!(
        risks
            .iter()
            .all(|part| part.contains("aria-label=\"Inspect sample "))
    );
    assert_eq!(html.matches("data-gov-calendar-cell=\"").count(), 10);
    assert_eq!(html.matches("<td class=\"empty\" headers=").count(), 26);
    assert!(!html.contains("<button type=\"button\" class=\"empty\""));
    assert_eq!(html.matches("scope=\"row\" id=\"gov-family-").count(), 3);
    assert_eq!(html.matches("scope=\"col\" id=\"gov-month-").count(), 12);
    assert!(html.contains("January 2027") && html.contains("May 2026–April 2027"));
    assert_eq!(
        html.matches("data-governance-unavailable=\"true\" disabled")
            .count(),
        12
    );
    assert_eq!(html.matches("data-logic-detail=\"").count(), 7);
    assert!(html.contains("Sample logics</span><strong>7"));
    assert!(html.contains("Sample P0</span><strong>4"));
    assert!(html.contains("Sample attention</span><strong>2"));
    assert!(!html.contains("₩9.2M") && !html.contains("₩110M") && !html.contains("sealed draft"));
    assert!(html.contains(
        "Live review, execution, attestation, sealing and receipt creation are unavailable"
    ));
}
