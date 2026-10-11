use super::*;

pub(super) fn daily_execution_proof_board() -> impl IntoView {
    view! {
        <section class="daily-proof-board" aria-label="FD-001 daily execution and Oyatie Cloud tenant proof">
            <div class="daily-proof-grid">
                <article class="daily-proof-card selected" data-daily-proof-card="daily-fd001">
                    <p class="screen-anchor">"FD-001 DAILY WORKLOAD"</p>
                    <h5>"Today’s work proves product delivery"</h5>
                    <p>
                        "Tasks, approvals, schedule holds, Workflow routes, Mail, Messenger, Community, and evidence receipts are FD-001 tenant workload operations, not detached widgets."
                    </p>
                    <div class="daily-proof-actions">
                        <button type="button" data-daily-proof-action="stage-packet">"Stage work packet"</button>
                        <button type="button" data-daily-proof-action="route-workflow">"Workflow run"</button>
                    </div>
                </article>
                <article class="daily-proof-card" data-daily-proof-card="daily-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD SUBSTRATE"</p>
                    <h5>"The queue dogfoods tenant hosting posture"</h5>
                    <p>
                        "Oyatie Cloud proves the substrate can host real production tenants by tying daily FD-001 work to cell health, policy envelopes, FinOps, and audit freshness."
                    </p>
                    <div class="daily-proof-actions">
                        <button type="button" data-daily-proof-action="route-cloud">"Cloud cells"</button>
                        <button type="button" data-daily-proof-action="route-policy">"Policy envelope"</button>
                    </div>
                </article>
                <article class="daily-proof-card" data-daily-proof-card="daily-local">
                    <p class="screen-anchor">"LOCAL-ONLY COMMAND RAIL"</p>
                    <h5>"Visually functional without side effects"</h5>
                    <p>
                        "Operators can filter, stage, route, brief, and inspect work while backend writes, auth changes, workflow execution, mail sends, payroll, billing, and cloud mutations remain disconnected."
                    </p>
                    <div class="daily-proof-actions">
                        <button type="button" data-daily-proof-action="route-audit">"Audit ledger"</button>
                        <button type="button" data-daily-proof-action="route-mail">"Mail brief"</button>
                    </div>
                </article>
            </div>
            <div class="daily-proof-footer">
                <span data-daily-proof-status="true">"Daily execution ready · FD-001 work queue dogfoods Oyatie Cloud locally."</span>
                <div class="daily-proof-routes" aria-label="Daily execution connected routes">
                    <button type="button" data-daily-proof-action="route-inbox">"Action Inbox"</button>
                    <button type="button" data-daily-proof-action="route-schedule">"Schedule"</button>
                    <button type="button" data-daily-proof-action="route-community">"Community"</button>
                    <button type="button" data-daily-proof-action="route-evidence">"Evidence"</button>
                </div>
            </div>
        </section>
    }
}

pub(super) fn daily_queue_footer() -> impl IntoView {
    view! {
        <div class="daily-subroute-proof">
            <span data-daily-proof-status="true">"Execution queue ready · FD-001 work items dogfood Oyatie Cloud with no backend mutation."</span>
            <div class="daily-proof-routes" aria-label="Execution queue connected routes">
                <button type="button" data-daily-proof-action="route-inbox">"Action Inbox"</button>
                <button type="button" data-daily-proof-action="route-workflow">"Workflow"</button>
                <button type="button" data-daily-proof-action="route-evidence">"Evidence"</button>
            </div>
        </div>
    }
}

pub(super) fn daily_schedule_footer() -> impl IntoView {
    view! {
        <div class="daily-subroute-proof">
            <span data-daily-proof-status="true">"Schedule pressure ready · FD-001 calendar risk stays tenant-scoped on Oyatie Cloud with no calendar, workflow, mail, policy, or cloud mutation."</span>
            <div class="daily-proof-routes" aria-label="Schedule connected routes">
                <button type="button" data-daily-proof-action="route-policy">"Policy"</button>
                <button type="button" data-daily-proof-action="route-cloud">"Cloud cells"</button>
                <button type="button" data-daily-proof-action="route-mail">"Reviewer Mail"</button>
            </div>
        </div>
    }
}

pub(super) fn daily_execution_console(envelope: TenantRenderEnvelope) -> impl IntoView {
    let rows = daily_execution_rows(&envelope);
    let blocking_count = rows.iter().filter(|row| row.state == "blocking").count();
    let task_count = rows.iter().filter(|row| row.kind == "task").count();
    let approval_count = rows.iter().filter(|row| row.kind == "approval").count();
    let schedule_count = rows.iter().filter(|row| row.kind == "schedule").count();
    let evidence_count = rows.iter().filter(|row| row.kind == "evidence").count();

    view! {
        <section id="daily-execution" class="daily-execution-console panel" aria-labelledby="daily-execution-title">
            <div class="daily-execution-head">
                <div>
                    <p class="screen-anchor">"DAILY WORK · PERSONAL OPERATIONS"</p>
                    <h3 id="daily-execution-title">"Tasks, approvals, schedule, and evidence for today"</h3>
                    <p>"A single operator queue connects calendar pressure, approval risk, workflow routes, Mail/Messenger drafts, and receipt evidence."</p>
                </div>
                <span class="status-chip success">"local command surface"</span>
            </div>

            <div class="daily-execution-kpis" aria-label="Daily execution summary">
                <span><strong>{task_count}</strong><small>"tasks"</small></span>
                <span><strong>{approval_count}</strong><small>"approvals"</small></span>
                <span><strong>{schedule_count}</strong><small>"calendar holds"</small></span>
                <span><strong>{blocking_count}</strong><small>"blocking"</small></span>
                <span><strong>{evidence_count}</strong><small>"evidence links"</small></span>
            </div>

            <div class="daily-execution-toolbar" aria-label="Daily execution filters">
                <label>
                    <span aria-hidden="true">"⌕"</span>
                    <input data-daily-search="true" aria-label="Search daily work" placeholder="Search tasks, approvals, owners, receipts..." />
                </label>
                <div class="daily-filter-pills" role="toolbar" aria-label="Daily work filters">
                    <button type="button" class="active" data-daily-filter="all">"All"</button>
                    <button type="button" data-daily-filter="blocking">"Blocking"</button>
                    <button type="button" data-daily-filter="task">"Tasks"</button>
                    <button type="button" data-daily-filter="approval">"Approvals"</button>
                    <button type="button" data-daily-filter="schedule">"Schedule"</button>
                    <button type="button" data-daily-filter="evidence">"Evidence"</button>
                </div>
                <span data-daily-status="true">{format!("{} visible · all work · local only", rows.len())}</span>
            </div>

            {daily_execution_proof_board()}

            <div class="daily-execution-layout">
                <article id="tasks-title" class="daily-execution-list" aria-labelledby="daily-list-title">
                    <div class="daily-column-head">
                        <p class="screen-anchor">"EXECUTION QUEUE"</p>
                        <h4 id="daily-list-title">"One list for personal operations"</h4>
                    </div>
                    <div role="list" aria-label="Daily work rows">
                        {rows.clone().into_iter().map(|row| {
                            let chip = daily_status_class(row.state);
                            view! {
                                <article
                                    class="daily-row"
                                    data-daily-row="true"
                                    data-daily-kind=row.kind
                                    data-daily-state=row.state
                                    role="listitem"
                                >
                                    <button
                                        type="button"
                                        class="daily-row-main"
                                        data-sidepeek-trigger="daily-work"
                                        data-sidepeek-title=row.title.clone()
                                        data-sidepeek-id=row.id.clone()
                                        data-sidepeek-desc=row.body.clone()
                                        data-sidepeek-owner=row.owner.clone()
                                        data-sidepeek-risk=row.state
                                        data-sidepeek-sla=row.due.clone()
                                    >
                                        <span class=chip>{row.state}</span>
                                        <strong>{row.title.clone()}</strong>
                                        <p>{row.body.clone()}</p>
                                    </button>
                                    <dl>
                                        <div><dt>"Kind"</dt><dd>{row.kind}</dd></div>
                                        <div><dt>"Owner"</dt><dd>{row.owner.clone()}</dd></div>
                                        <div><dt>"Due"</dt><dd>{row.due.clone()}</dd></div>
                                    </dl>
                                    <div class="daily-row-actions">
                                        <button type="button" data-daily-action="workflow" data-daily-target=row.route>"Flow"</button>
                                        <button type="button" data-daily-action="mail">"Mail"</button>
                                        <button type="button" data-daily-action="evidence">"Evidence"</button>
                                        <button type="button" data-daily-action="stage">"Stage"</button>
                                    </div>
                                </article>
                            }
                        }).collect_view()}
                    </div>
                    {daily_queue_footer()}
                </article>

                <aside id="schedule-title" class="daily-calendar-rail" aria-label="Today schedule and capacity">
                    <div class="daily-column-head">
                        <p class="screen-anchor">"CALENDAR"</p>
                        <h4>"Today’s schedule pressure"</h4>
                    </div>
                    <ol class="daily-timeline">
                        {envelope.schedule.clone().into_iter().map(|item| view! {
                            <li>
                                <time>{item.time}</time>
                                <strong>{item.title}</strong>
                                <p>{item.detail}</p>
                            </li>
                        }).collect_view()}
                    </ol>
                    <div class="daily-capacity" aria-label="Daily capacity">
                        <span role="progressbar" aria-valuenow="73" aria-valuemin="0" aria-valuemax="100" aria-label="Close work: 73%" style="--bar: 73%"><em aria-hidden="true">"Close work · 73%"</em></span>
                        <span role="progressbar" aria-valuenow="64" aria-valuemin="0" aria-valuemax="100" aria-label="Approvals: 64%" style="--bar: 64%"><em aria-hidden="true">"Approvals · 64%"</em></span>
                        <span role="progressbar" aria-valuenow="41" aria-valuemin="0" aria-valuemax="100" aria-label="Context switching: 41%" style="--bar: 41%"><em aria-hidden="true">"Context switching · 41%"</em></span>
                    </div>
                    <div class="daily-route-matrix" aria-label="Daily route matrix">
                        <button type="button" data-daily-action="workflow" data-daily-target="#workflow-studio">"Workflow"</button>
                        <button type="button" data-daily-action="mail">"Mail brief"</button>
                        <button type="button" data-daily-action="messenger">"Messenger"</button>
                        <button type="button" data-daily-action="evidence">"Audit evidence"</button>
                    </div>
                    {daily_schedule_footer()}
                </aside>
            </div>
        </section>
    }
}
