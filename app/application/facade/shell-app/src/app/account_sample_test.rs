use super::*;

#[test]
fn account_sample_truth_progress_and_native_compatibility_aliases_agree() {
    for context in OperatorContext::ALL {
        let envelope = server_derived_envelope(context);
        let owner = Owner::new();
        let native = owner.with(|| {
            identity_workforce_service(envelope.clone(), Signal::derive(|| true))
                .into_view()
                .to_html()
        });
        let compatibility = static_dashboard_html();
        owner.cleanup();
        for panel in ACCOUNT_PANELS {
            let body = account_panel_html(panel);
            assert!(native.contains(&body));
            assert!(compatibility.contains(&body));
            assert!(native.contains(&format!("id=\"identity-panel-{panel}\"")));
            assert!(compatibility.contains(&format!("id=\"identity-{panel}\"")));
            assert!(body.contains("sample") || body.contains("Sample"));
            for fabricated in [
                "94/100",
                "56%",
                "1원 검증 완료",
                "PIPA-safe",
                "REC-ID-2026-05",
                "passkey verified",
            ] {
                assert!(!body.contains(fabricated));
            }
        }
        for (panel, label) in [
            ("auth", "Sample Auth"),
            ("sessions", "Sample Sessions"),
            ("roles", "Sample Roles"),
            ("org", "Sample Org profile"),
            ("onboarding", "Sample Onboarding"),
        ] {
            let tab = native
                .split(&format!("id=\"identity-tab-{panel}\""))
                .nth(1)
                .unwrap()
                .split("</button>")
                .next()
                .unwrap();
            assert!(tab.contains(&format!("aria-label=\"{label}\"")));
            assert!(tab.ends_with(&format!(">{}", label.strip_prefix("Sample ").unwrap())));
            assert!(native.contains(&format!("aria-labelledby=\"identity-tab-{panel}\"")));
        }
        assert_eq!(SAMPLE_SETUP_STEPS.len(), 9);
        let setup = account_panel_html("onboarding");
        assert_eq!(setup.matches("Sample complete</small>").count(), 3);
        assert!(setup.contains("3 of 9 sample steps complete · 33%"));
        assert!(setup.contains("aria-valuenow=\"33\""));
        assert!(setup.contains("data-setup-step=\"review\""));
        assert!(native.contains("data-employee-directory=\"sample\""));
        assert!(compatibility.contains("data-employee-directory=\"sample\""));
        let settings = settings_body_html();
        let owner = Owner::new();
        let utilities = owner.with(|| UtilityPanels().into_view().to_html());
        owner.cleanup();
        assert!(utilities.contains(settings));
        assert!(static_dashboard_html().contains(settings));
        assert!(settings.contains("Static audit examples"));
        assert!(settings.contains("reset on reload"));
        assert!(!settings.contains("Stored in this browser session"));
    }
}

#[test]
fn account_source_binds_complete_owned_facts_context_provenance_and_hub() {
    let envelope = server_derived_envelope(OperatorContext::TenantAdmin);
    let source = account_sample_source(&envelope);
    let parsed: serde_json::Value = serde_json::from_str(&source).unwrap();
    assert_eq!(parsed[0], "tenant-admin");
    assert_eq!(parsed[2], HubSample::envelope_source(&envelope));
    assert_eq!(parsed[3][0], account_person_html());
    assert_eq!(parsed[3][1], account_context_html());
    for (index, panel) in ACCOUNT_PANELS.into_iter().enumerate() {
        assert_eq!(parsed[3][2][index][0], panel);
        assert_eq!(parsed[3][2][index][1], account_panel_html(panel));
    }
    let mut variations = Vec::new();
    let mut changed = envelope.clone();
    changed.context = OperatorContext::CorporateOffice;
    variations.push(changed);
    let mut changed = envelope.clone();
    changed.tenant_name.push('!');
    variations.push(changed);
    let mut changed = envelope.clone();
    changed.role_name.push('!');
    variations.push(changed);
    let mut changed = envelope.clone();
    changed.tenant_class.push('!');
    variations.push(changed);
    let mut changed = envelope.clone();
    changed.accreditation.explanation.push('!');
    variations.push(changed);
    let mut changed = envelope.clone();
    changed.server_derivation_note.push('!');
    variations.push(changed);
    let mut changed = envelope.clone();
    changed.messages[0].preview.push('!');
    variations.push(changed);
    for changed in variations {
        assert_ne!(source, account_sample_source(&changed));
    }
}
