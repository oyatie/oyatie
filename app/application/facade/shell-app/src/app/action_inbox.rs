use super::*;
use crate::render_envelope::InboxSampleMetadata;

struct InboxRow<'a> {
    metadata: &'a InboxSampleMetadata,
    kind: &'static str,
    title: &'a str,
    body: &'a str,
    priority: &'a str,
    requester: &'a str,
}

fn inbox_rows(envelope: &TenantRenderEnvelope) -> Vec<InboxRow<'_>> {
    envelope
        .daily_tasks
        .iter()
        .map(|item| InboxRow {
            metadata: &item.inbox,
            kind: "task",
            title: &item.title,
            body: &item.detail,
            priority: &item.priority,
            requester: "Unavailable for this sample task",
        })
        .chain(envelope.approvals.iter().map(|item| InboxRow {
            metadata: &item.inbox,
            kind: "approval",
            title: &item.title,
            body: &item.risk_note,
            priority: "Unavailable for this sample approval",
            requester: &item.requester,
        }))
        .collect()
}

pub(super) fn inbox_sample_valid(envelope: &TenantRenderEnvelope) -> bool {
    let rows = inbox_rows(envelope);
    let mut ids = std::collections::BTreeSet::new();
    rows.iter().all(|row| {
        !row.metadata.id.is_empty()
            && !row.metadata.owner.is_empty()
            && ids.insert(&row.metadata.id)
    })
}

pub(super) fn inbox_sample_source(envelope: &TenantRenderEnvelope) -> String {
    serde_json::to_string(&(
        envelope.context,
        &envelope.tenant_name,
        &envelope.role_name,
        &envelope.tenant_class,
        &envelope.accreditation,
        &envelope.server_derivation_note,
        &envelope.daily_tasks,
        &envelope.approvals,
        &envelope.intelligence,
        HubSample::envelope_source(envelope),
    ))
    .expect("complete native Inbox sample source is serializable")
}

fn inbox_record(row: &InboxRow<'_>) -> String {
    serde_json::to_string(&(
        row.metadata,
        row.kind,
        row.title,
        row.body,
        row.priority,
        row.requester,
    ))
    .expect("native Inbox sample row is serializable")
}

fn inbox_route(target: &str, label: &str, channel: Option<&str>) -> String {
    let channel = channel.map_or_else(String::new, |value| {
        format!(" data-inbox-channel=\"{value}\"")
    });
    format!(
        "<a href=\"{}\" data-inbox-route=\"true\"{channel}>{}</a>",
        daily_escape(target),
        daily_escape(label)
    )
}

fn inbox_row_html(row: &InboxRow<'_>) -> String {
    let meta = row.metadata;
    let metadata = [
        ("Sample ID", meta.id.as_str()),
        ("Kind", row.kind),
        ("Sample owner", meta.owner.as_str()),
        ("Requester", row.requester),
        ("Sample priority", row.priority),
        (
            "Assigned to this sample role",
            if meta.owned_by_viewer { "Yes" } else { "No" },
        ),
        ("Sample blocker", if meta.blocking { "Yes" } else { "No" }),
        ("Due, SLA and verified evidence", "Unavailable"),
    ]
    .into_iter()
    .map(|(name, value)| format!("<div><dt>{name}</dt><dd>{}</dd></div>", daily_escape(value)))
    .collect::<String>();
    format!(
        r#"<article class="inbox-sample-row daily-row" role="listitem" data-inbox-row="true" data-inbox-id="{id}" data-inbox-owned="{owned}" data-inbox-blocking="{blocking}" data-inbox-record="{record}">
<label class="inbox-select-cell"><input type="checkbox" data-inbox-select="{id}" data-inbox-focus="select:{id}" aria-label="Select sample {title}" /></label>
<details data-inbox-detail="{id}"><summary class="daily-row-main" data-inbox-focus="detail:{id}"><span class="status-chip">sample {kind}</span><strong>{title}</strong><span>Inspect sample {id}</span></summary><p>{body}</p><dl>{metadata}</dl><button type="button" data-inbox-close="true" data-inbox-focus="close:{id}">Close sample details</button></details>
<div class="inbox-sample-routes">{routes}</div></article>"#,
        id = daily_escape(&meta.id),
        owned = meta.owned_by_viewer,
        blocking = meta.blocking,
        record = daily_escape(&inbox_record(row)),
        kind = row.kind,
        title = daily_escape(row.title),
        body = daily_escape(row.body),
        routes = inbox_route("#workflow-studio", "Open Workflow sample", None)
            + &inbox_route("#work-hub", "Open Mail sample", Some("Mail"))
            + &inbox_route("#resource-panel-audit", "Open Audit sample", None),
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn inbox_channel_templates(envelope: &TenantRenderEnvelope) -> String {
    [HubChannel::Mail, HubChannel::Community]
        .into_iter()
        .map(|channel| {
            format!(
                "<template data-inbox-hub-template=\"{}\">{}</template>",
                channel.label(),
                communication_hub_html(envelope, channel)
            )
        })
        .collect()
}

#[cfg(target_arch = "wasm32")]
fn inbox_channel_templates(_envelope: &TenantRenderEnvelope) -> String {
    String::new()
}

pub(super) fn inbox_body_html(envelope: &TenantRenderEnvelope) -> String {
    let rows = inbox_rows(envelope);
    let filters = [("all", "All"), ("mine", "Mine"), ("blocking", "Blocking")].into_iter().map(|(value, label)| format!(
        r#"<button type="button" data-inbox-filter="{value}" data-inbox-focus="filter:{value}" aria-pressed="{}" class="{}">{label}</button>"#,
        value == "all", if value == "all" { "active" } else { "" }
    )).collect::<String>();
    let advice = envelope.intelligence.iter().map(|item| {
        let record = serde_json::to_string(item).expect("native advisory sample is serializable");
        format!(r#"<article class="copilot-card" data-inbox-advice="{record}"><strong>{title}</strong><p>{body}</p><details data-inbox-trace="{record}"><summary>Inspect sample advice guardrail</summary><p>{guardrail}</p><p>This advice is supplied by the anonymous sample, not a live analysis or verified receipt.</p></details><button type="button" data-inbox-unavailable="true" disabled aria-describedby="inbox-live-unavailable">Apply delegation unavailable</button><button type="button" data-inbox-dismiss="true">Dismiss sample advice locally</button></article>"#,
            record = daily_escape(&record), title = daily_escape(&item.title), body = daily_escape(&item.body), guardrail = daily_escape(&item.guardrail))
    }).collect::<String>();
    format!(
        r##"<article class="priority-workbench panel" aria-labelledby="command-workbench-title"><div class="workbench-head"><div><p class="screen-anchor">ANONYMOUS SAMPLE ACTION INBOX</p><h3 id="command-workbench-title">Sample priority queue <span>{count}</span></h3></div><div class="workbench-filters" role="toolbar" aria-label="Sample inbox filters">{filters}</div></div>
<p>Inspect and select local sample records. Mine means assigned to this sample role; no tenant, assignee, policy or outcome is verified.</p><p data-inbox-availability="true" role="status">Observed anonymous sample: {label}. Live Inbox actions are unavailable.</p>
<div class="workbench-summary-strip" aria-label="Sample inbox summary"><span><strong>{blocking}</strong><small>sample blockers</small></span><span><strong>{mine}</strong><small>assigned to this sample role</small></span><span><strong>Unavailable</strong><small>live SLA and verified evidence</small></span></div>
<div class="workbench-bulkbar" aria-label="Sample inbox selection"><label><input type="checkbox" data-inbox-select-all="true" data-inbox-focus="select-all" aria-label="Select all visible sample inbox items" /><span><strong data-inbox-selected-count="true">0</strong> selected</span></label><div class="workbench-bulk-actions"><button type="button" data-inbox-preview="packet" data-inbox-focus="preview-packet" disabled>Preview selected sample packet</button><button type="button" data-inbox-preview="brief" data-inbox-focus="preview-brief" disabled>Preview selected sample brief</button><button type="button" data-inbox-unavailable="true" disabled aria-describedby="inbox-live-unavailable">Approve unavailable</button><button type="button" data-inbox-unavailable="true" disabled aria-describedby="inbox-live-unavailable">Defer unavailable</button><button type="button" data-inbox-unavailable="true" disabled aria-describedby="inbox-live-unavailable">Attach evidence unavailable</button></div></div>
<p id="inbox-live-unavailable">Live approval, deferral, delegation and receipt attachment are unavailable because no tenant or action permission has been verified. Selecting sample records does not grant permission.</p><p data-inbox-status="true" role="status">{count} of {count} sample rows shown · 0 selected</p><p>Changing filters clears selection of hidden rows. Checkbox selection does not approve or change any sample record.</p><p data-inbox-empty="true" role="status" hidden>No sample rows match this filter.</p><p data-inbox-reset="true" role="status" hidden>Sample source changed; previous local selection, inspection and previews cleared.</p>
<details data-inbox-packet="packet"><summary data-inbox-focus="packet">Selected sample packet</summary><p>Browser-local preview only. No workflow, message, approval or receipt is created.</p><p data-inbox-preview-empty="true">No visible sample records selected.</p><ol data-inbox-preview-rows="true"></ol></details><details data-inbox-packet="brief"><summary data-inbox-focus="brief">Selected sample brief</summary><p>Browser-local preview only; no Mail draft or delivery is created.</p><p data-inbox-preview-empty="true">No visible sample records selected.</p><ol data-inbox-preview-rows="true"></ol></details>
<div class="workbench-list" role="list" aria-label="Sample action inbox rows">{row_markup}</div><nav class="inbox-sample-routes" aria-label="Existing sample destinations">{routes}</nav><p data-inbox-route-status="true" role="status">Existing sample destinations only; no action output is created.</p>{templates}</article>
<aside class="governed-copilot panel" aria-labelledby="copilot-workbench-title"><p class="screen-anchor">SAMPLE ADVISORY</p><h3 id="copilot-workbench-title" tabindex="-1">Suggested sample moves</h3>{advice}<p>Local advice dismissal only. Suggestions never authorize or execute live actions.</p></aside>"##,
        count = rows.len(),
        blocking = rows.iter().filter(|row| row.metadata.blocking).count(),
        mine = rows
            .iter()
            .filter(|row| row.metadata.owned_by_viewer)
            .count(),
        label = daily_escape(&daily_sample_label(envelope)),
        row_markup = rows.iter().map(inbox_row_html).collect::<String>(),
        routes = inbox_route("#daily-execution", "Open Daily sample", None)
            + &inbox_route("#workflow-studio", "Open Workflow sample", None)
            + &inbox_route("#cloud-ops-cockpit", "Open Cloud sample", None)
            + &inbox_route("#cockpit-panel-policy", "Open Policy sample", None)
            + &inbox_route("#resource-panel-audit", "Open Audit sample", None)
            + &inbox_route("#evidence-spine", "Open Evidence sample", None)
            + &inbox_route("#work-hub", "Open Mail sample", Some("Mail"))
            + &inbox_route("#work-hub", "Open Community sample", Some("Community")),
        templates = inbox_channel_templates(envelope),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inbox_identity_metadata_and_source_survive_reorder_without_inventing_authority() {
        for context in OperatorContext::ALL {
            let mut sample = server_derived_envelope(context);
            assert!(inbox_sample_valid(&sample));
            let original = inbox_rows(&sample)
                .iter()
                .map(inbox_record)
                .collect::<Vec<_>>();
            sample.daily_tasks.reverse();
            sample.approvals.reverse();
            let mut reordered = inbox_rows(&sample)
                .iter()
                .map(inbox_record)
                .collect::<Vec<_>>();
            let mut sorted = original.clone();
            sorted.sort();
            reordered.sort();
            assert_eq!(sorted, reordered);
            let daily = daily_sample_source(&sample);
            let source = inbox_sample_source(&sample);
            sample.daily_tasks[0].inbox.blocking = !sample.daily_tasks[0].inbox.blocking;
            sample.approvals[0].inbox.owned_by_viewer = !sample.approvals[0].inbox.owned_by_viewer;
            assert_ne!(source, inbox_sample_source(&sample));
            assert_eq!(daily, daily_sample_source(&sample));
            sample.approvals[0].title = "<script>unsafe & \"quoted\"</script>".into();
            sample.approvals[0].requester = "requester is not the owner".into();
            let html = inbox_body_html(&sample);
            assert!(!html.contains("<script>unsafe"));
            assert!(html.contains("&lt;script&gt;unsafe &amp; &quot;quoted&quot;&lt;/script&gt;"));
            assert!(html.contains("requester is not the owner"));
            assert!(html.contains("Due, SLA and verified evidence</dt><dd>Unavailable"));
            assert_eq!(
                html.matches("data-inbox-row=\"true\"").count(),
                original.len()
            );
            assert!(html.contains(
                "disabled aria-describedby=\"inbox-live-unavailable\">Approve unavailable"
            ));
            let duplicate = sample.daily_tasks[0].inbox.id.clone();
            sample.approvals[0].inbox.id = duplicate;
            assert!(!inbox_sample_valid(&sample));
        }
    }
}
