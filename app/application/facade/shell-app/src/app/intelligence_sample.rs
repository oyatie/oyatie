use super::*;
use crate::render_envelope::IntelligenceSampleId;

pub(super) fn intelligence_sample_valid(envelope: &TenantRenderEnvelope) -> bool {
    let mut ids = std::collections::BTreeSet::new();
    envelope.intelligence.iter().all(|item| {
        let route = item.id.route();
        item.id.context() == envelope.context
            && ids.insert(item.id.id())
            && item.destination.as_deref() == route.map(|(destination, _)| destination)
            && item.channel.as_deref() == route.and_then(|(_, channel)| channel)
    }) && ids.len()
        == IntelligenceSampleId::ALL
            .iter()
            .filter(|id| id.context() == envelope.context)
            .count()
}

pub(super) fn intelligence_sample_source(envelope: &TenantRenderEnvelope) -> String {
    serde_json::to_string(&(
        envelope.context.id(),
        daily_sample_label(envelope),
        HubSample::envelope_source(envelope),
        &envelope.intelligence,
        &envelope.tenant_class,
        &envelope.accreditation,
        &envelope.server_derivation_note,
    ))
    .expect("complete Intelligence sample source serializes")
}

fn intelligence_card_html(item: &IntelligenceSuggestion) -> String {
    let id = item.id.id();
    let route = match item.destination.as_deref() {
        Some(destination) => format!(
            r##"<a href="#{destination}" data-intelligence-route="{id}" data-intelligence-channel="{channel}" data-intelligence-focus="route:{id}" aria-describedby="intelligence-availability intelligence-route-reason-{id}">Open existing sample</a><small id="intelligence-route-reason-{id}" data-intelligence-route-reason="{id}" hidden>Existing sample destination unavailable in this render.</small>"##,
            destination = daily_escape(destination),
            channel = daily_escape(item.channel.as_deref().unwrap_or("")),
        ),
        None => format!(
            r#"<small id="intelligence-route-reason-{id}" data-intelligence-route-reason="{id}">No supported local destination is supplied for this sample.</small>"#
        ),
    };
    format!(
        r#"<article class="intelligence-suggestion-card" role="listitem" data-intelligence-card="{id}" data-intelligence-record="{record}">
<div><span class="status-chip">Illustrative sample</span><strong data-intelligence-field="title">{title}</strong></div>
<details data-intelligence-detail="{id}"><summary data-intelligence-focus="detail:{id}">Inspect sample advice</summary><p>Illustrative advice: <span data-intelligence-field="body">{body}</span></p><p>Illustrative guardrail: <span data-intelligence-field="guardrail">{guardrail}</span></p>
<details data-intelligence-trace="{id}"><summary data-intelligence-focus="trace:{id}">Inspect sample source</summary><dl><div><dt>Sample ID</dt><dd>{id}</dd></div><div><dt>Supplied destination</dt><dd>{destination}</dd></div><div><dt>Supplied channel</dt><dd>{channel}</dd></div></dl><p>Supplied by the anonymous contextual sample. No live model, analysis, privacy verdict, permission or verified receipt is supplied.</p><button type="button" data-intelligence-close="true" data-intelligence-focus="close-trace:{id}">Close sample source</button></details>
<button type="button" data-intelligence-close="true" data-intelligence-focus="close-detail:{id}">Close sample advice</button></details>
<div class="intelligence-card-actions">{route}<button type="button" data-intelligence-unavailable="true" disabled aria-describedby="intelligence-live-unavailable">Apply unavailable</button><button type="button" data-intelligence-dismiss="{id}" data-intelligence-focus="dismiss:{id}">Dismiss sample locally</button></div></article>"#,
        record = daily_escape(&serde_json::to_string(item).expect("sample advice serializes")),
        title = daily_escape(&item.title),
        body = daily_escape(&item.body),
        guardrail = daily_escape(&item.guardrail),
        destination = daily_escape(item.destination.as_deref().unwrap_or("Unavailable")),
        channel = daily_escape(item.channel.as_deref().unwrap_or("No channel")),
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn intelligence_channel_templates(envelope: &TenantRenderEnvelope) -> String {
    [
        HubChannel::Messenger,
        HubChannel::Mail,
        HubChannel::Community,
    ]
    .into_iter()
    .filter(|channel| {
        envelope
            .intelligence
            .iter()
            .any(|item| item.channel.as_deref() == Some(channel.label()))
    })
    .map(|channel| {
        format!(
            "<template data-intelligence-hub-template=\"{}\">{}</template>",
            channel.label(),
            communication_hub_html(envelope, channel)
        )
    })
    .collect()
}

#[cfg(target_arch = "wasm32")]
fn intelligence_channel_templates(_envelope: &TenantRenderEnvelope) -> String {
    String::new()
}

pub(super) fn intelligence_body_html(envelope: &TenantRenderEnvelope) -> String {
    let count = envelope.intelligence.len();
    format!(
        r#"<div class="panel-header"><p class="eyebrow">Intelligence · anonymous sample</p><h3 id="intelligence-title" data-intelligence-focus="heading" tabindex="-1">Illustrative sample advice</h3></div>
<div class="intelligence-command-console"><div class="intelligence-console-head"><div><p>Inspect supplied advice and its illustrative guardrail. Local dismissal and navigation never execute or authorize an action.</p><p id="intelligence-availability" data-intelligence-availability="true" role="status">Observed anonymous sample: {label}. Local controls only.</p></div><div class="intelligence-console-actions"><button type="button" data-intelligence-reset="true" data-intelligence-focus="reset">Reset local advice</button></div></div>
<p data-intelligence-status="true" role="status"><span data-intelligence-remaining="true">{count}</span> of <span data-intelligence-total="true">{count}</span> sample advice remaining.</p><p data-intelligence-empty="true" role="status" hidden>All sample advice dismissed locally. Reset to inspect it again.</p><p data-intelligence-source-reset="true" role="status" hidden>Sample source changed; previous local inspection and dismissal cleared.</p>
<div class="intelligence-layout"><div class="intelligence-suggestion-stack" role="list" aria-label="Illustrative sample advice">{cards}</div><aside class="intelligence-eval-panel" aria-label="Live Intelligence unavailable"><div class="intelligence-eval-intro"><h4 data-intelligence-eval-heading="true">Live evaluation unavailable</h4><p id="intelligence-live-unavailable" data-intelligence-eval-explanation="true">No verified model, current evaluation, tenant action permission or durable command is supplied. Sample advice does not establish confidence, policy, privacy or evidence results.</p></div><button type="button" data-intelligence-unavailable="true" disabled aria-describedby="intelligence-live-unavailable">Run live evaluation unavailable</button></aside></div><p data-intelligence-route-status="true" role="status">Existing sample destinations only; no action output is created.</p>{templates}</div>"#,
        label = daily_escape(&daily_sample_label(envelope)),
        cards = envelope
            .intelligence
            .iter()
            .map(intelligence_card_html)
            .collect::<String>(),
        templates = intelligence_channel_templates(envelope),
    )
}

pub(super) fn intelligence_command_console(
    envelope: TenantRenderEnvelope,
    available: Signal<bool>,
) -> impl IntoView {
    let source = intelligence_sample_source(&envelope);
    view! {
        <For each=move || [(source.clone(), envelope.clone())]
            key=|(source, _)| source.clone()
            children=move |(source, envelope)| intelligence_panel(envelope, available, source) />
    }
}

fn intelligence_panel(
    envelope: TenantRenderEnvelope,
    available: Signal<bool>,
    source: String,
) -> impl IntoView {
    let valid = intelligence_sample_valid(&envelope);
    view! {
        <section id="intelligence-command-console" class="panel intelligence-command-shell" aria-labelledby="intelligence-title"
            data-intelligence-root="true" data-intelligence-source=source
            data-intelligence-context=envelope.context.id() data-intelligence-label=daily_sample_label(&envelope)
            data-intelligence-hub-source=HubSample::envelope_source(&envelope)
            data-intelligence-mode=if cfg!(target_arch = "wasm32") { "island" } else { "ssr" }
            data-intelligence-admitted=valid.to_string()
            data-intelligence-available=move || (valid && intelligence_available(available)).to_string()
            inner_html=intelligence_body_html(&envelope)></section>
    }
}

fn intelligence_available(available: Signal<bool>) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        available.try_get().unwrap_or(false)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        available.try_get_untracked().unwrap_or(false)
    }
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_intelligence_command_console(envelope: &TenantRenderEnvelope) -> String {
    format!(
        r#"<section id="intelligence-command-console" class="panel intelligence-command-shell" aria-labelledby="intelligence-title" data-intelligence-root="true" data-intelligence-source="{source}" data-intelligence-context="{context}" data-intelligence-label="{label}" data-intelligence-hub-source="{hub}" data-intelligence-mode="ssr" data-intelligence-admitted="{valid}" data-intelligence-available="{valid}">{body}</section>"#,
        source = daily_escape(&intelligence_sample_source(envelope)),
        context = envelope.context.id(),
        label = daily_escape(&daily_sample_label(envelope)),
        hub = daily_escape(&HubSample::envelope_source(envelope)),
        valid = intelligence_sample_valid(envelope),
        body = intelligence_body_html(envelope),
    )
}

#[cfg(test)]
mod tests {
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
}
