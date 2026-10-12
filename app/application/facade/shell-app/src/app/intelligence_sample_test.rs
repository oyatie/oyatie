use super::*;

#[test]
fn contextual_sample_identity_routes_and_serialization_refuse_invented_metadata() {
    for context in OperatorContext::ALL {
        let mut envelope = server_derived_envelope(context);
        assert!(intelligence_sample_valid(&envelope));
        let json = serde_json::to_string(&envelope).unwrap();
        assert_eq!(
            serde_json::from_str::<TenantRenderEnvelope>(&json).unwrap(),
            envelope
        );
        let records = envelope.intelligence.clone();
        envelope.intelligence.reverse();
        assert!(intelligence_sample_valid(&envelope));
        for item in records {
            assert_eq!(
                envelope
                    .intelligence
                    .iter()
                    .find(|other| other.id == item.id),
                Some(&item)
            );
        }
        envelope.intelligence[1].id = envelope.intelligence[0].id;
        assert!(!intelligence_sample_valid(&envelope));
        let mut envelope = server_derived_envelope(context);
        envelope.intelligence[0].destination = Some("invented-action".into());
        assert!(!intelligence_sample_valid(&envelope));
        let mut envelope = server_derived_envelope(context);
        envelope.intelligence[0].channel = Some("Unknown".into());
        assert!(!intelligence_sample_valid(&envelope));
    }
    let item = server_derived_envelope(OperatorContext::TenantAdmin)
        .intelligence
        .remove(0);
    let mut json = serde_json::to_value(item).unwrap();
    for id in ["", "unknown-advice"] {
        json["id"] = id.into();
        assert!(serde_json::from_value::<IntelligenceSuggestion>(json.clone()).is_err());
    }
    let item = server_derived_envelope(OperatorContext::TenantAdmin)
        .intelligence
        .remove(0);
    let mut json = serde_json::to_value(item).unwrap();
    json.as_object_mut().unwrap().remove("id");
    assert!(serde_json::from_value::<IntelligenceSuggestion>(json).is_err());
    let item = server_derived_envelope(OperatorContext::TenantAdmin)
        .intelligence
        .remove(0);
    let mut json = serde_json::to_value(item).unwrap();
    json["authorization"] = true.into();
    assert!(serde_json::from_value::<IntelligenceSuggestion>(json).is_err());
}

#[test]
fn shared_sample_markup_qualifies_supplied_text_and_complete_source_changes() {
    for context in OperatorContext::ALL {
        let mut envelope = server_derived_envelope(context);
        let source = intelligence_sample_source(&envelope);
        let body = intelligence_body_html(&envelope);
        let html = static_intelligence_command_console(&envelope);
        assert!(html.contains(&body));
        let owner = Owner::new();
        let native = owner.with(|| {
            intelligence_command_console(envelope.clone(), Signal::derive(|| true))
                .into_view()
                .to_html()
        });
        owner.cleanup();
        assert!(native.contains(&body));
        assert_eq!(body.matches("data-intelligence-card=\"").count(), 3);
        assert!(body.contains("Illustrative sample advice"));
        assert!(body.contains("Live evaluation unavailable</h4><p"));
        assert!(body.contains("data-intelligence-remaining=\"true\">3"));
        assert!(!body.contains("REC-AI-GUARD"));
        assert!(!body.contains("decision confidence"));
        assert!(!body.contains("Governed AI command"));
        assert_eq!(
            body.matches("disabled aria-describedby=\"intelligence-live-unavailable\"")
                .count(),
            4
        );
        envelope.intelligence[0].body = "<script>unsafe & \"quoted\"</script>".into();
        assert_ne!(source, intelligence_sample_source(&envelope));
        let body = intelligence_body_html(&envelope);
        assert!(!body.contains("<script>unsafe"));
        assert!(body.contains("&lt;script&gt;unsafe &amp; &quot;quoted&quot;&lt;/script&gt;"));
    }
}

#[test]
fn every_advice_and_context_source_field_changes_identity_without_merging_inbox_state() {
    let envelope = server_derived_envelope(OperatorContext::TenantAdmin);
    let source = intelligence_sample_source(&envelope);
    let inbox = inbox_sample_source(&envelope);
    assert_eq!(source, intelligence_sample_source(&envelope.clone()));
    assert_eq!(inbox, inbox_sample_source(&envelope.clone()));
    let mut changed = Vec::new();
    let mut value = envelope.clone();
    value.intelligence[0].id = IntelligenceSampleId::TenantTaxFiling;
    changed.push(value);
    let mut value = envelope.clone();
    value.intelligence[0].title.push('!');
    changed.push(value);
    let mut value = envelope.clone();
    value.intelligence[0].body.push('!');
    changed.push(value);
    let mut value = envelope.clone();
    value.intelligence[0].guardrail.push('!');
    changed.push(value);
    let mut value = envelope.clone();
    value.intelligence[0].destination = None;
    changed.push(value);
    let mut value = envelope.clone();
    value.intelligence[0].channel = Some("Mail".into());
    changed.push(value);
    let mut value = envelope.clone();
    value.context = OperatorContext::CorporateOffice;
    changed.push(value);
    let mut value = envelope.clone();
    value.tenant_name.push('!');
    changed.push(value);
    let mut value = envelope.clone();
    value.role_name.push('!');
    changed.push(value);
    let mut value = envelope.clone();
    value.tenant_class.push('!');
    changed.push(value);
    let mut value = envelope.clone();
    value.accreditation.label.push('!');
    changed.push(value);
    let mut value = envelope.clone();
    value.accreditation.explanation.push('!');
    changed.push(value);
    let mut value = envelope.clone();
    value.accreditation.healthcare_enabled = !value.accreditation.healthcare_enabled;
    changed.push(value);
    let mut value = envelope.clone();
    value.server_derivation_note.push('!');
    changed.push(value);
    let mut value = envelope.clone();
    value.messages[0].preview.push('!');
    changed.push(value);
    for value in changed {
        assert_ne!(source, intelligence_sample_source(&value));
        assert_ne!(inbox, inbox_sample_source(&value));
    }
}
