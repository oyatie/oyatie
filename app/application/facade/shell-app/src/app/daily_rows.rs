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
            route: "#work-hub",
        });
    }
    rows.extend([
        ExecutionRow {
            id: "REC-WF-7741".to_string(),
            kind: "evidence",
            state: "sealed",
            title: "Workflow output receipts ready".to_string(),
            body: "Messenger, Mail, Community, and Action Inbox outputs share one local evidence packet.".to_string(),
            owner: "Evidence Spine".to_string(),
            due: "sealed draft".to_string(),
            route: "#evidence-spine",
        },
        ExecutionRow {
            id: "REC-PAY-2026-04-PARK".to_string(),
            kind: "evidence",
            state: "blocking",
            title: "Payroll delta evidence needs owner".to_string(),
            body: "Four-insurance change for Park Seo-jun is the current close blocker and routes to audit.".to_string(),
            owner: "Finance close".to_string(),
            due: "4.0h".to_string(),
            route: "#audit-ledger",
        },
    ]);
    rows
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
