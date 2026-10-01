use super::*;

pub(super) fn command_center_workbench(envelope: TenantRenderEnvelope) -> impl IntoView {
    let tasks = envelope.daily_tasks.clone();
    let approvals = envelope.approvals.clone();
    let suggestions = envelope.intelligence.clone();
    let workflow_name = envelope.workflow.name.clone();
    let role_name = envelope.role_name.clone();

    view! {
        <section
            id="command-center-workbench"
            class="command-center-workbench"
            aria-labelledby="command-workbench-title"
        >
            <article class="priority-workbench panel" aria-labelledby="command-workbench-title">
                <div class="workbench-head">
                    <div>
                        <p class="screen-anchor">"ACTION INBOX"</p>
                        <h3 id="command-workbench-title">"Priority queue" <span>"8"</span></h3>
                    </div>
                    <div class="workbench-filters" role="toolbar" aria-label="Action inbox filters">
                        <button type="button" class="active" data-workbench-filter="all">"All"</button>
                        <button type="button" data-workbench-filter="mine">"Mine"</button>
                        <button type="button" data-workbench-filter="blocking">"Blocking"</button>
                    </div>
                </div>

                <div class="workbench-summary-strip" aria-label="Action inbox summary">
                    <span><strong>"3"</strong><small>"blocking"</small></span>
                    <span><strong>"5"</strong><small>"owned by you"</small></span>
                    <span><strong>"4.0h"</strong><small>"SLA pressure"</small></span>
                    <span><strong>"12"</strong><small>"evidence links"</small></span>
                </div>

                {action_inbox_proof_board()}

                <div class="workbench-bulkbar" aria-label="Action inbox bulk operations">
                    <label>
                        <input type="checkbox" data-inbox-select-all="true" aria-label="Select all visible inbox items" />
                        <span><strong data-inbox-selected-count="true">"0"</strong>" selected"</span>
                    </label>
                    <div class="workbench-bulk-actions">
                        <button type="button" data-inbox-bulk="approve" disabled>"Approve"</button>
                        <button type="button" data-inbox-bulk="defer" disabled>"Defer"</button>
                        <button type="button" data-inbox-bulk="mail" disabled>"Mail brief"</button>
                        <button type="button" data-inbox-bulk="evidence" disabled>"Attach evidence"</button>
                    </div>
                    <span class="workbench-status" data-inbox-status="true">"No items selected · local inbox only"</span>
                </div>

                <div class="workbench-list" role="list" aria-label="Operational priority queue">
                    {tasks.into_iter().enumerate().map(|(index, item)| {
                        let title = item.title.clone();
                        let title_attr = title.clone();
                        let detail = item.detail.clone();
                        let detail_attr = detail.clone();
                        let priority = item.priority.clone();
                        let priority_label = priority.clone();
                        let priority_risk = priority.clone();
                        let priority_chip_class = if priority.eq_ignore_ascii_case("high") { "status-chip danger" } else { "status-chip" };
                        let priority_key = if priority.eq_ignore_ascii_case("high") {
                            "blocking"
                        } else if index % 2 == 0 {
                            "mine"
                        } else {
                            "all"
                        };
                        let row_class = if priority_key == "blocking" {
                            "workbench-row blocking"
                        } else {
                            "workbench-row"
                        };
                        let receipt = format!("REC-OYATIE-2026-05-{index:02}");
                        view! {
                            <article
                                class=row_class
                                data-workbench-row=priority_key
                                data-inbox-row="true"
                            >
                                <label class="inbox-select-cell">
                                    <input type="checkbox" data-inbox-select="true" data-inbox-title=title.clone() aria-label=format!("Select {title}") />
                                </label>
                                <span class="workbench-row-id">{format!("ACT-78{:02}", index + 41)}</span>
                                <span class=priority_chip_class>
                                    {priority_label}
                                </span>
                                <button
                                    type="button"
                                    class="inbox-row-main"
                                    data-sidepeek-trigger="action-inbox"
                                    data-sidepeek-title=title_attr
                                    data-sidepeek-id=receipt
                                    data-sidepeek-desc=detail_attr
                                    data-sidepeek-owner=role_name.clone()
                                    data-sidepeek-risk=priority_risk
                                    data-sidepeek-sla="4.0h target · local data"
                                >
                                    <strong>{title}</strong>
                                    <p>{detail}</p>
                                </button>
                                <time>{if index == 0 { "오늘 18:00" } else if index == 1 { "내일 09:00" } else { "5월 10일" }}</time>
                                <span class="inbox-row-actions">
                                    <button type="button" data-inbox-row-action="workflow">"Flow"</button>
                                    <button type="button" data-inbox-row-action="mail">"Mail"</button>
                                    <button type="button" data-inbox-row-action="audit">"Audit"</button>
                                </span>
                            </article>
                        }
                    }).collect_view()}
                    {approvals.into_iter().enumerate().map(|(index, item)| {
                        let title = item.title.clone();
                        let title_attr = title.clone();
                        let requester = item.requester.clone();
                        let requester_attr = requester.clone();
                        let requester_text = requester.clone();
                        let risk_note = item.risk_note.clone();
                        let risk_attr = risk_note.clone();
                        view! {
                            <article
                                class="workbench-row approval"
                                data-workbench-row="mine"
                                data-inbox-row="true"
                            >
                                <label class="inbox-select-cell">
                                    <input type="checkbox" data-inbox-select="true" data-inbox-title=title.clone() aria-label=format!("Select {title}") />
                                </label>
                                <span class="workbench-row-id">{format!("APR-{}", index + 274)}</span>
                                <span class="status-chip warning">"approval"</span>
                                <button
                                    type="button"
                                    class="inbox-row-main"
                                    data-sidepeek-trigger="approval"
                                    data-sidepeek-title=title_attr
                                    data-sidepeek-id=format!("APR-{}-{}", workflow_name.replace(' ', "-").to_ascii_uppercase(), index + 1)
                                    data-sidepeek-desc=risk_attr
                                    data-sidepeek-owner=requester_attr
                                    data-sidepeek-risk="Review"
                                    data-sidepeek-sla="Reviewer queue · visual only"
                                >
                                    <strong>{title}</strong>
                                    <p>{requester_text}" · "{risk_note}</p>
                                </button>
                                <time>"대기"</time>
                                <span class="inbox-row-actions">
                                    <button type="button" data-inbox-row-action="workflow">"Flow"</button>
                                    <button type="button" data-inbox-row-action="mail">"Mail"</button>
                                    <button type="button" data-inbox-row-action="audit">"Audit"</button>
                                </span>
                            </article>
                        }
                    }).collect_view()}
                </div>
            </article>

            <aside class="governed-copilot panel" aria-labelledby="copilot-workbench-title">
                <div class="copilot-head">
                    <div>
                        <p class="screen-anchor">"COPILOT · GOVERNED"</p>
                        <h3 id="copilot-workbench-title">"Suggested next moves"</h3>
                    </div>
                    <span class="status-chip ai">"PIPA-safe"</span>
                </div>
                <div class="copilot-suggestions" aria-live="polite">
                    {suggestions.into_iter().enumerate().map(|(index, suggestion)| {
                        let title = suggestion.title.clone();
                        let body = suggestion.body.clone();
                        let guardrail = suggestion.guardrail.clone();
                        view! {
                            <article class="copilot-card">
                                <strong>{title}</strong>
                                <p>{body}</p>
                                <small>{guardrail}</small>
                                <div>
                                    <button type="button" data-copilot-action="apply">"Apply delegation"</button>
                                    <button type="button" data-copilot-action="dismiss">"Dismiss"</button>
                                    <button type="button" data-sidepeek-trigger="copilot-trace" data-sidepeek-title="Copilot trace" data-sidepeek-id=format!("AI-TRACE-{index}") data-sidepeek-desc="Shows why a governed local suggestion is visible in this render envelope." data-sidepeek-owner="Governed Copilot" data-sidepeek-risk="Advisory" data-sidepeek-sla="Never auto-executes">"Trace"</button>
                                </div>
                            </article>
                        }
                    }).collect_view()}
                </div>
                <p class="copilot-status" data-copilot-status="true">
                    "Read-only · scoped to roster + run + workflow data · suggestions never auto-execute."
                </p>
            </aside>
        </section>
    }
}

pub(super) fn action_inbox_proof_board() -> impl IntoView {
    view! {
        <section class="daily-proof-board inbox-proof-board" aria-label="FD-001 Action Inbox and Oyatie Cloud execution proof">
            <div class="daily-proof-grid">
                <article class="daily-proof-card selected" data-daily-proof-card="inbox-fd001">
                    <p class="screen-anchor">"FD-001 ACTION INBOX"</p>
                    <h5>"Priority queue is the product control plane"</h5>
                    <p>
                        "Blocking payroll, vendor, policy, Workflow, Mail, Messenger, Community, and evidence items stay inside the FD-001 tenant workload graph."
                    </p>
                    <div class="daily-proof-actions">
                        <button type="button" data-daily-proof-action="route-daily">"Daily queue"</button>
                        <button type="button" data-daily-proof-action="route-workflow">"Workflow gate"</button>
                    </div>
                </article>
                <article class="daily-proof-card" data-daily-proof-card="inbox-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD ADMISSION"</p>
                    <h5>"Every item can prove tenant readiness"</h5>
                    <p>
                        "Oyatie Cloud substrate checks policy, residency, release gates, FinOps, and audit freshness before any FD-001 workload claims production readiness."
                    </p>
                    <div class="daily-proof-actions">
                        <button type="button" data-daily-proof-action="route-cloud">"Cloud cells"</button>
                        <button type="button" data-daily-proof-action="route-policy">"Policy board"</button>
                    </div>
                </article>
                <article class="daily-proof-card" data-daily-proof-card="inbox-local">
                    <p class="screen-anchor">"LOCAL-ONLY REVIEW"</p>
                    <h5>"Interactive, never wired"</h5>
                    <p>
                        "Operators can select, defer, brief, and attach receipts visually; no approval, auth, workflow execution, mail send, payroll, billing, or cloud mutation runs."
                    </p>
                    <div class="daily-proof-actions">
                        <button type="button" data-daily-proof-action="stage-packet">"Stage packet"</button>
                        <button type="button" data-daily-proof-action="route-audit">"Audit ledger"</button>
                    </div>
                </article>
            </div>
            <div class="daily-proof-footer">
                <span data-daily-proof-status="true">"Action Inbox ready · FD-001 priority work dogfoods Oyatie Cloud locally."</span>
                <div class="daily-proof-routes" aria-label="Action Inbox connected routes">
                    <button type="button" data-daily-proof-action="route-mail">"Reviewer Mail"</button>
                    <button type="button" data-daily-proof-action="route-community">"Community"</button>
                    <button type="button" data-daily-proof-action="route-evidence">"Evidence"</button>
                </div>
            </div>
        </section>
    }
}
