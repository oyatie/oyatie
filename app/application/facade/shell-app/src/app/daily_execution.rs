use super::*;

pub(super) fn daily_execution_console(
    envelope: TenantRenderEnvelope,
    available: Signal<bool>,
) -> impl IntoView {
    let source = daily_sample_source(&envelope);
    let label = daily_sample_label(&envelope);
    let context = envelope.context.id();
    let body = daily_execution_body_html(&envelope);
    view! {
        <section id="daily-execution" tabindex="-1" class="daily-execution-console panel" aria-labelledby="daily-execution-title"
            data-daily-source=source data-daily-context=context data-daily-label=label
            data-daily-mode=if cfg!(target_arch = "wasm32") { "island" } else { "ssr" }
            data-daily-available=move || daily_sample_available(available).to_string()
            inner_html=body></section>
    }
}

fn daily_sample_available(available: Signal<bool>) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        available.try_get().unwrap_or(false)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        available.try_get_untracked().unwrap_or(false)
    }
}

pub(super) fn daily_execution_body_html(envelope: &TenantRenderEnvelope) -> String {
    let rows = daily_execution_rows(envelope);
    let filters = [
        ("all", "All"), ("blocking", "Blocking"), ("task", "Tasks"),
        ("approval", "Approvals"), ("schedule", "Schedule"), ("evidence", "Evidence"),
    ].into_iter().map(|(value, label)| format!(
        r#"<button type="button" data-daily-filter="{value}" data-daily-focus="filter:{value}" aria-pressed="{}" class="{}">{label}</button>"#,
        value == "all", if value == "all" { "active" } else { "" }
    )).collect::<String>();
    let row_markup = rows.iter().map(daily_row_html).collect::<String>();
    let schedule = envelope
        .schedule
        .iter()
        .map(|item| {
            format!(
                "<li><time>{}</time><strong>{}</strong><p>{}</p></li>",
                daily_escape(&item.time),
                daily_escape(&item.title),
                daily_escape(&item.detail)
            )
        })
        .collect::<String>();
    format!(
        r##"<div class="daily-execution-head"><div><p class="screen-anchor">ANONYMOUS SAMPLE WORK</p><h3 id="daily-execution-title">Sample tasks, approvals, schedule, and evidence</h3><p>Inspect and stage sample rows locally. No tenant, role, task outcome, or receipt is verified.</p></div><span class="status-chip">sample only</span></div>
<p data-daily-availability="true" role="status">Observed anonymous sample: {label}. Live Calendar access and capacity are unavailable.</p>
<div class="daily-execution-kpis" aria-label="Sample work summary"><span><strong>{task}</strong><small>sample tasks</small></span><span><strong>{approval}</strong><small>sample approvals</small></span><span><strong>{schedule_count}</strong><small>sample schedule rows</small></span><span><strong>{blocking}</strong><small>sample blocking rows</small></span><span><strong>{evidence}</strong><small>sample evidence links</small></span></div>
<div class="daily-execution-toolbar" aria-label="Sample work filters"><label><span aria-hidden="true">⌕</span><input data-daily-search="true" data-daily-focus="search" aria-label="Search sample daily work" placeholder="Search sample IDs, titles, owners, or due text" /></label><div class="daily-filter-pills" role="toolbar" aria-label="Sample work filters">{filters}</div><button type="button" data-daily-clear="true" data-daily-focus="clear">Clear filters</button><span data-daily-status="true" role="status">{count} of {count} sample rows shown</span></div>
<p data-daily-empty="true" role="status" hidden>No matching sample work. Clear filters to show all sample rows.</p><p data-daily-reset="true" role="status" hidden>Sample source changed; previous local filters, details, and staged rows cleared.</p>
<details data-daily-packet="true"><summary data-daily-preview="true" data-daily-focus="preview">Preview sample packet</summary><p>Browser-local staging only. No task, event, workflow, message, or receipt is created.</p><p data-daily-packet-empty="true">No sample rows staged.</p><ol data-daily-packet-rows="true"></ol></details>
<div class="daily-execution-layout"><article id="tasks-title" tabindex="-1" class="daily-execution-list" aria-labelledby="daily-list-title"><div class="daily-column-head"><p class="screen-anchor">SAMPLE QUEUE</p><h4 id="daily-list-title">Inspect actual sample records</h4></div><div role="list" aria-label="Sample daily work rows">{row_markup}</div><div class="daily-subroute-proof"><span>Navigation opens existing sample views; no outcome is created.</span><div class="daily-proof-routes" aria-label="Sample queue routes">{queue_routes}</div></div></article>
<aside id="schedule-title" tabindex="-1" class="daily-calendar-rail" aria-labelledby="daily-schedule-title"><div class="daily-column-head"><p class="screen-anchor">ANONYMOUS SAMPLE SCHEDULE</p><h4 id="daily-schedule-title">Schedule supplied by this sample</h4></div><ol class="daily-timeline">{schedule}</ol><p>Live Calendar access and capacity are unavailable. These time, title, and detail strings do not establish a date, timezone, availability, conflict, or tenant calendar.</p><div class="daily-route-matrix" aria-label="Sample schedule routes">{schedule_routes}</div></aside></div>
<p data-daily-route-status="true" role="status">Existing sample destinations only; no live work or Calendar action is available.</p>{templates}"##,
        label = daily_escape(&daily_sample_label(envelope)),
        count = rows.len(),
        task = rows.iter().filter(|row| row.kind == "task").count(),
        approval = rows.iter().filter(|row| row.kind == "approval").count(),
        schedule_count = rows.iter().filter(|row| row.kind == "schedule").count(),
        blocking = rows.iter().filter(|row| row.state == "blocking").count(),
        evidence = rows.iter().filter(|row| row.kind == "evidence").count(),
        queue_routes = daily_route("#command-center-workbench", "Open Inbox sample", None)
            + &daily_route("#workflow-studio", "Open Workflow sample", None)
            + &daily_route("#evidence-spine", "Open Evidence sample", None)
            + &daily_route("#business-logics", "Open Approvals sample", None)
            + &daily_route("#schedule-title", "Open Schedule sample", None),
        schedule_routes = daily_route("#cockpit-panel-policy", "Open Policy sample", None)
            + &daily_route("#cloud-ops-cockpit", "Open Cloud sample", None)
            + &daily_route("#resource-panel-audit", "Open Audit sample", None)
            + &daily_route("#work-hub", "Open Mail sample", Some("Mail"))
            + &daily_route("#work-hub", "Open Messenger sample", Some("Messenger"))
            + &daily_route("#work-hub", "Open Community sample", Some("Community")),
        templates = daily_channel_templates(envelope)
    )
}

fn daily_route(target: &str, label: &str, channel: Option<&str>) -> String {
    let channel = channel.map_or_else(String::new, |value| {
        format!(" data-daily-channel=\"{value}\"")
    });
    format!(
        "<a href=\"{}\" data-daily-route=\"true\"{channel}>{}</a>",
        daily_escape(target),
        daily_escape(label)
    )
}

fn daily_row_html(row: &ExecutionRow) -> String {
    let mut metadata = String::new();
    for (field, value) in [
        ("ID", row.id.as_str()),
        ("Kind", row.kind),
        ("State", row.state),
        ("Owner", row.owner.as_str()),
        ("Due", row.due.as_str()),
        ("Route", row.route),
    ] {
        metadata.push_str(&format!(
            "<div><dt>{field}</dt><dd>{}</dd></div>",
            daily_escape(value)
        ));
    }
    let search = format!(
        "{} {} {} {} {} {} {}",
        row.id, row.title, row.body, row.owner, row.due, row.kind, row.state
    );
    format!(
        r#"<article class="daily-row" data-daily-row="true" data-daily-id="{id}" data-daily-kind="{kind}" data-daily-state="{state}" data-daily-record="{record}" data-daily-search-text="{search}" role="listitem"><details data-daily-detail="{id}"><summary class="daily-row-main" data-daily-focus="detail:{id}"><span class="{chip}">{state}</span><strong data-daily-title="true">{title}</strong><span>Inspect sample {id}</span></summary><p data-daily-body="true">{body}</p><dl>{metadata}</dl><button type="button" data-daily-close="true" data-daily-focus="close:{id}">Close sample details</button></details><div class="daily-row-actions">{route}{mail}{evidence}<button type="button" data-daily-stage="{id}" data-daily-focus="stage:{id}" aria-pressed="false">Stage sample</button></div></article>"#,
        id = daily_escape(&row.id),
        kind = daily_escape(row.kind),
        state = daily_escape(row.state),
        record = daily_escape(&daily_record(row)),
        search = daily_escape(&search),
        title = daily_escape(&row.title),
        body = daily_escape(&row.body),
        chip = daily_status_class(row.state),
        route = daily_route(row.route, "Open row destination", None),
        mail = daily_route("#work-hub", "Open Mail sample", Some("Mail")),
        evidence = daily_route("#evidence-spine", "Open Evidence sample", None),
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn daily_channel_templates(envelope: &TenantRenderEnvelope) -> String {
    HubChannel::ALL
        .into_iter()
        .map(|channel| {
            format!(
                "<template data-daily-hub-template=\"{}\">{}</template>",
                channel.label(),
                communication_hub_html(envelope, channel)
            )
        })
        .collect()
}

#[cfg(target_arch = "wasm32")]
fn daily_channel_templates(_envelope: &TenantRenderEnvelope) -> String {
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daily_markup_has_complete_escaped_details_and_no_capacity_claim() {
        let mut source = server_derived_envelope(OperatorContext::TenantAdmin);
        source.daily_tasks[0].title = "<script>unsafe & \"quoted\"</script>".into();
        source.daily_tasks[0].detail = "Full body <img src=x onerror=unsafe>".into();
        let rows = daily_execution_rows(&source);
        let html = daily_execution_body_html(&source);
        assert_eq!(html.matches("data-daily-row=\"true\"").count(), rows.len());
        assert_eq!(html.matches("data-daily-detail=\"").count(), rows.len());
        assert_eq!(html.matches("data-daily-filter=\"").count(), 6);
        let filters = html
            .split('<')
            .filter_map(|tag| tag.split_once('>'))
            .map(|(attributes, _)| attributes)
            .filter(|attributes| {
                attributes.starts_with("button ") && attributes.contains("data-daily-filter=\"")
            })
            .collect::<Vec<_>>();
        assert_eq!(filters.len(), 6);
        assert_eq!(
            filters
                .iter()
                .filter(|attributes| attributes.contains("aria-pressed=\"true\""))
                .count(),
            1
        );
        assert!(html.contains("&lt;script&gt;unsafe &amp; &quot;quoted&quot;&lt;/script&gt;"));
        assert!(html.contains("Full body &lt;img src=x onerror=unsafe&gt;"));
        assert!(!html.contains("<script>unsafe") && !html.contains("<img src=x"));
        assert!(!html.contains("role=\"progressbar\"") && !html.contains("sealed draft"));
        assert!(html.contains("Live Calendar access and capacity are unavailable"));
        assert!(html.contains("href=\"#resource-panel-audit\""));
        for row in rows {
            assert!(html.contains(&daily_escape(&daily_record(&row))));
            assert!(html.contains(&daily_escape(&row.body)));
        }
        for item in source.schedule {
            assert!(html.contains(&format!("<time>{}</time>", daily_escape(&item.time))));
            assert!(html.contains(&daily_escape(&item.title)));
            assert!(html.contains(&daily_escape(&item.detail)));
        }
    }
}
