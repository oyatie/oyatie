use super::*;

pub(super) fn daily_execution_rows(envelope: &TenantRenderEnvelope) -> Vec<ExecutionRow> {
    let mut rows = Vec::new();
    for (index, item) in envelope.daily_tasks.iter().enumerate() {
        let state = if item.priority.eq_ignore_ascii_case("high") {
            "blocking"
        } else {
            "task"
        };
        rows.push(ExecutionRow {
            id: format!("TASK-{}", index + 741),
            kind: "task",
            state,
            title: item.title.clone(),
            body: item.detail.clone(),
            owner: envelope.role_name.clone(),
            due: if index == 0 { "today 18:00" } else { "today" }.to_string(),
            route: "#workflow-studio",
        });
    }
    for (index, item) in envelope.approvals.iter().enumerate() {
        rows.push(ExecutionRow {
            id: format!("APR-{}", index + 274),
            kind: "approval",
            state: if index == 0 { "blocking" } else { "review" },
            title: item.title.clone(),
            body: item.risk_note.clone(),
            owner: item.requester.clone(),
            due: "review queue".to_string(),
            route: "#business-logics",
        });
    }
    for (index, item) in envelope.schedule.iter().enumerate() {
        rows.push(ExecutionRow {
            id: format!("CAL-{}", index + 31),
            kind: "schedule",
            state: "scheduled",
            title: item.title.clone(),
            body: item.detail.clone(),
            owner: "Calendar".to_string(),
            due: item.time.clone(),
            route: "#schedule-title",
        });
    }
    rows.extend([
        ExecutionRow {
            id: "REC-WF-7741".to_string(),
            kind: "evidence",
            state: "sample",
            title: "Workflow output receipt example".to_string(),
            body: "Anonymous sample links to Messenger, Mail, Community, and Action Inbox; no receipt has been created or verified.".to_string(),
            owner: "Sample evidence".to_string(),
            due: "sample only".to_string(),
            route: "#evidence-spine",
        },
        ExecutionRow {
            id: "REC-PAY-2026-04-PARK".to_string(),
            kind: "evidence",
            state: "blocking",
            title: "Payroll delta evidence example".to_string(),
            body: "Sample payroll change illustrates a close blocker and links to the audit example; no tenant work or receipt is verified.".to_string(),
            owner: "Sample finance close".to_string(),
            due: "sample only".to_string(),
            route: "#resource-panel-audit",
        },
    ]);
    rows
}

pub(super) fn daily_record(row: &ExecutionRow) -> String {
    serde_json::to_string(&(
        &row.id, row.kind, row.state, &row.title, &row.body, &row.owner, &row.due, row.route,
    ))
    .expect("native sample work row is serializable")
}

pub(super) fn daily_sample_source(envelope: &TenantRenderEnvelope) -> String {
    let rows = daily_execution_rows(envelope)
        .iter()
        .map(daily_record)
        .collect::<Vec<_>>();
    serde_json::to_string(&(
        envelope.context,
        &envelope.tenant_name,
        &envelope.role_name,
        &envelope.tenant_class,
        &envelope.accreditation,
        &envelope.server_derivation_note,
        rows,
        &envelope.schedule,
    ))
    .expect("complete native Daily sample source is serializable")
}

pub(super) fn daily_sample_label(envelope: &TenantRenderEnvelope) -> String {
    format!(
        "{} · {} · {} sample",
        envelope.tenant_name,
        envelope.role_name,
        envelope.context.label()
    )
}

pub(super) fn daily_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub(super) fn daily_status_class(state: &str) -> &'static str {
    match state {
        "blocking" => "status-chip danger",
        "review" | "scheduled" => "status-chip warning",
        "sealed" => "status-chip success",
        _ => "status-chip",
    }
}

pub(super) fn work_list(items: Vec<WorkItem>) -> impl IntoView {
    view! {
        <div>
            <h4 id="tasks-title">"Task queue"</h4>
            <ul class="item-list">
                {items.into_iter().map(|item| view! {
                    <li>
                        <span class="priority">{item.priority}</span>
                        <strong>{item.title}</strong>
                        <p>{item.detail}</p>
                    </li>
                }).collect_view()}
            </ul>
        </div>
    }
}

pub(super) fn approval_list(items: Vec<ApprovalItem>) -> impl IntoView {
    view! {
        <div>
            <h4>"Approval queue"</h4>
            <ul class="item-list compact">
                {items.into_iter().map(|item| view! {
                    <li>
                        <strong>{item.title}</strong>
                        <p>{item.requester}</p>
                        <span>{item.risk_note}</span>
                    </li>
                }).collect_view()}
            </ul>
        </div>
    }
}

pub(super) fn schedule_list(items: Vec<ScheduleItem>) -> impl IntoView {
    view! {
        <ol class="timeline-list">
            {items.into_iter().map(|item| view! {
                <li>
                    <time>{item.time}</time>
                    <div>
                        <strong>{item.title}</strong>
                        <p>{item.detail}</p>
                    </div>
                </li>
            }).collect_view()}
        </ol>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daily_source_binds_complete_records_order_and_schedule() {
        let original = server_derived_envelope(OperatorContext::TenantAdmin);
        let key = daily_sample_source(&original);
        let changes: [fn(&mut TenantRenderEnvelope); 8] = [
            |value| value.daily_tasks[0].title.push_str(" changed"),
            |value| value.daily_tasks[0].detail.push_str(" changed"),
            |value| value.daily_tasks.reverse(),
            |value| value.schedule[0].time.push_str(" changed"),
            |value| value.schedule[0].title.push_str(" changed"),
            |value| value.schedule[0].detail.push_str(" changed"),
            |value| value.role_name.push_str(" changed"),
            |value| value.context = OperatorContext::CorporateOffice,
        ];
        for change in changes {
            let mut changed = original.clone();
            change(&mut changed);
            assert_ne!(daily_sample_source(&changed), key);
        }
        for row in daily_execution_rows(&original) {
            let record: Vec<String> = serde_json::from_str(&daily_record(&row)).unwrap();
            assert_eq!(
                record,
                [
                    row.id,
                    row.kind.into(),
                    row.state.into(),
                    row.title,
                    row.body,
                    row.owner,
                    row.due,
                    row.route.into()
                ]
            );
        }
    }
}
