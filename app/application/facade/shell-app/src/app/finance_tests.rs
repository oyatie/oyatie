use super::*;

#[test]
fn finance_anchor_controls_declare_sample_routes_or_unavailable_effects() {
    let owner = Owner::new();
    owner.with(|| {
        let mut counts = [0, 0];
        for body in [
            finance_vendors_anchor_board().into_view().to_html(),
            finance_billing_anchor_board().into_view().to_html(),
        ] {
            for tag in body
                .split('<')
                .filter(|tag| tag.contains("data-finance-anchor-action="))
                .map(|tag| tag.split_once('>').unwrap().0)
            {
                if tag.starts_with("a ") {
                    assert!(tag.contains("href=\"#"));
                    assert!(tag.contains("data-finance-route=\""));
                    assert!(!tag.contains("data-finance-unavailable="));
                    if tag.contains("href=\"#work-hub\"") {
                        assert!(
                            tag.contains("data-finance-channel=\"Mail\"")
                                || tag.contains("data-finance-channel=\"Community\"")
                        );
                    }
                    counts[0] += 1;
                } else {
                    assert!(tag.starts_with("button ") && tag.contains(" disabled"));
                    assert!(tag.contains("data-finance-unavailable=\"true\""));
                    assert!(tag.contains("aria-describedby=\"finance-unavailable-reason\""));
                    assert!(!tag.contains("data-finance-route="));
                    counts[1] += 1;
                }
            }
        }
        assert_eq!(counts, [17, 3]);
    });
    owner.cleanup();
}

#[test]
fn finance_projection_matches_complete_shared_native_body_for_every_context() {
    let owner = Owner::new();
    owner.with(|| {
        for context in OperatorContext::ALL {
            let envelope = server_derived_envelope(context);
            let expected = finance_service_markup(context).into_view().to_html();
            let projection = envelope.finance_body_projection.as_ref().unwrap();
            assert_eq!(projection.context, context);
            assert_eq!(projection.body, expected);
            assert!(!expected.is_empty());
            assert_eq!(finance_body_html(&envelope, true), expected);
            let source: serde_json::Value =
                serde_json::from_str(&finance_sample_source(&envelope)).unwrap();
            assert_eq!(source[4], expected);
            assert!(expected.contains("id=\"finance-panel-leave\""));
            assert!(expected.contains("id=\"payroll-finance-preview\""));
            assert_eq!(
                expected.contains("data-finance-record="),
                context != OperatorContext::HealthcareClinician
            );
        }
    });
    owner.cleanup();
}

#[test]
fn finance_projection_refuses_missing_empty_foreign_and_legacy_inputs() {
    let owner = Owner::new();
    owner.with(|| {
        let original = server_derived_envelope(OperatorContext::TenantAdmin);
        let mut missing = original.clone();
        missing.finance_body_projection = None;
        let mut empty = original.clone();
        empty.finance_body_projection.as_mut().unwrap().body.clear();
        let mut foreign = original.clone();
        foreign.finance_body_projection.as_mut().unwrap().context =
            OperatorContext::HealthcareClinician;
        let mut changed_context = original.clone();
        changed_context.context = OperatorContext::HealthcareClinician;
        let mut legacy_json = serde_json::to_value(&original).unwrap();
        legacy_json
            .as_object_mut()
            .unwrap()
            .remove("finance_body_projection");
        let legacy: TenantRenderEnvelope = serde_json::from_value(legacy_json).unwrap();
        assert!(legacy.finance_body_projection.is_none());
        for (case, envelope) in [
            ("missing", missing),
            ("empty", empty),
            ("foreign", foreign),
            ("changed context with retained carrier", changed_context),
            ("legacy JSON", legacy),
        ] {
            assert!(finance_body_html(&envelope, true).is_empty(), "{case}");
            assert!(finance_body_html(&envelope, false).is_empty(), "{case}");
            let source: serde_json::Value =
                serde_json::from_str(&finance_sample_source(&envelope)).unwrap();
            assert_eq!(source[0], envelope.context.id(), "{case}");
            assert_eq!(source[1], daily_sample_label(&envelope), "{case}");
            assert_eq!(source[3], HubSample::envelope_source(&envelope), "{case}");
            assert_eq!(source[4], "", "{case}");
        }
    });
    owner.cleanup();
}

#[test]
fn finance_sample_scope_omits_commercial_children_and_preserves_people_consumers() {
    let owner = Owner::new();
    owner.with(|| {
        for context in OperatorContext::ALL {
            let envelope = server_derived_envelope(context);
            let visible = context != OperatorContext::HealthcareClinician;
            let body = finance_body_html(&envelope, true);
            for id in [
                "finance-panel-ledger",
                "finance-panel-vendors",
                "finance-panel-billing",
            ] {
                assert_eq!(
                    body.contains(&format!("id=\"{id}\"")),
                    visible,
                    "{context:?}: {id}"
                );
            }
            assert!(body.contains("id=\"finance-panel-leave\""));
            assert!(body.contains("id=\"payroll-finance-preview\""));
            assert_eq!(body.contains("data-finance-record="), visible);
            for group in ["commercial", "finops", "filing"] {
                assert_eq!(
                    finance_rail_html(finance_sample_scope(context), group, true).is_empty(),
                    !visible
                );
            }
            let cockpit = tenant_operations_cockpit(envelope.clone())
                .into_view()
                .to_html();
            assert_eq!(cockpit.contains("id=\"cockpit-panel-finops\""), visible);
            assert!(cockpit.contains("id=\"cockpit-panel-topology\""));
            let governance = tenant_rbac_board(envelope.clone(), Signal::derive(|| true))
                .into_view()
                .to_html();
            assert_eq!(governance.contains("id=\"filing-readiness\""), visible);
            assert!(governance.contains("id=\"payroll-cockpit\""));
            assert!(governance.contains("id=\"employee-directory\""));
            let compatibility = static_finance_preview(&envelope);
            assert_eq!(compatibility.contains("id=\"ledger-preview\""), visible);
            assert!(compatibility.contains("id=\"leave-time\""));
        }
    });
    owner.cleanup();
}

#[test]
fn finance_source_binds_complete_native_body_scope_label_and_hub_input() {
    let owner = Owner::new();
    owner.with(|| {
        let original = server_derived_envelope(OperatorContext::TenantAdmin);
        let identity = finance_sample_source(&original);
        let source: serde_json::Value = serde_json::from_str(&identity).unwrap();
        assert_eq!(source.as_array().unwrap().len(), 5);
        assert_eq!(source[0], "tenant-admin");
        assert_eq!(source[1], daily_sample_label(&original));
        assert_eq!(source[2]["commercial_examples"], true);
        assert_eq!(source[3], HubSample::envelope_source(&original));
        assert_eq!(source[4], finance_body_html(&original, true));
        assert_eq!(
            finance_body_html(&original, false),
            finance_compatible_html(source[4].as_str().unwrap().to_owned())
        );
        let mut changed = original.clone();
        changed.tenant_name.push_str(" changed");
        assert_ne!(identity, finance_sample_source(&changed));
        changed = original.clone();
        changed.messages[0].preview.push_str(" changed");
        assert_ne!(identity, finance_sample_source(&changed));
        let compatibility = static_finance_preview(&original);
        let encoded = compatibility
            .split_once("data-finance-source=\"")
            .unwrap()
            .1
            .split_once('"')
            .unwrap()
            .0;
        let decoded = encoded
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&");
        assert_eq!(decoded, identity);
        assert!(compatibility.contains("data-finance-native=\"false\""));
    });
    owner.cleanup();
}

#[test]
fn finance_records_and_effect_controls_report_samples_and_unavailability() {
    let owner = Owner::new();
    owner.with(|| {
        let envelope = server_derived_envelope(OperatorContext::CorporateOffice);
        let body = finance_body_html(&envelope, true);
        assert_eq!(body.matches("data-finance-journal=\"").count(), 4);
        assert_eq!(body.matches("data-finance-vendor=\"").count(), 3);
        assert_eq!(body.matches("data-finance-detail=\"").count(), 7);
        assert!(body.contains("1 high · 1 medium · 1 low"));
        assert!(!body.contains("close package ready") && !body.contains("18 staged"));
        assert!(body.contains("live close readiness unknown"));
        assert!(body.contains("Fixture as-of 2026-04-30; live observation unavailable"));
        assert!(body.contains("sample-tenant") && body.contains("sample-entity-kr"));
        assert!(body.contains("Sample KRW 4,820,000 minor units / month"));
        assert!(body.contains("verified evidence unavailable"));
        for action in [
            "reconcile",
            "export-pack",
            "add-vendor",
            "run-close",
            "attach-proof",
        ] {
            let marker = format!("=\"{action}\"");
            let tag = body
                .split(&marker)
                .next()
                .unwrap()
                .rsplit('<')
                .next()
                .unwrap();
            let tail = body
                .split_once(&marker)
                .unwrap()
                .1
                .split_once('>')
                .unwrap()
                .0;
            assert!(
                tag.starts_with("button") && tail.contains("disabled"),
                "{action}"
            );
        }
        assert!(body.contains("data-finance-channel=\"Mail\""));
        let finops = finance_finops_panel(&envelope).into_view().to_html();
        assert!(finops.contains("Sample monthly total USD 41,500"));
        assert_eq!(
            FINOPS_SAMPLE_COSTS
                .iter()
                .map(|(_, cost)| cost)
                .sum::<u64>(),
            41_500
        );
        let bars = finance_finops_bars_html();
        for share in [52, 24, 17, 7] {
            assert!(bars.contains(&format!("aria-valuenow=\"{share}\"")));
        }
        assert_eq!(
            bars.matches("data-finops-sample-total-usd=\"41500\"")
                .count(),
            4
        );
        assert!(finops.contains("Illustrative April 2026 USD spend fixtures"));
    });
    owner.cleanup();
}
