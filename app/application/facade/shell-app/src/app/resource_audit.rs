use super::*;

pub(super) fn resource_audit_console(envelope: TenantRenderEnvelope) -> impl IntoView {
    let resources = resource_inventory_rows();
    let audit_events = audit_receipts();
    let gates = deployment_gates();
    let visible_modules = envelope.modules.len();
    let open_approvals = envelope.approvals.len();

    view! {
        <section
            id="resource-audit-console"
            class="resource-audit-console panel"
            aria-labelledby="resource-audit-title"
        >
            <div class="panel-header resource-console-header">
                <div>
                    <p class="eyebrow">"Operate · Trust"</p>
                    <h3 id="resource-audit-title">"Resource inventory, audit ledger, and deployment gates"</h3>
                </div>
                // A-2: resource-audit tablist — aria-orientation + id/aria-controls + role=tabpanel
                <div class="resource-tabs" role="tablist" aria-label="Resource and audit console views" aria-orientation="horizontal">
                    <button type="button" id="resource-tab-inventory" class="active" data-resource-tab="inventory" role="tab" aria-selected="true" aria-controls="resource-panel-inventory">"Inventory"</button>
                    <button type="button" id="resource-tab-audit" data-resource-tab="audit" role="tab" aria-selected="false" aria-controls="resource-panel-audit">"Audit ledger"</button>
                    <button type="button" id="resource-tab-gates" data-resource-tab="gates" role="tab" aria-selected="false" aria-controls="resource-panel-gates">"Deployment gates"</button>
                </div>
            </div>

            <div class="resource-console-spine" aria-label="Console summary">
                <span><strong>{resources.len()}</strong>" resources"</span>
                <span><strong>{audit_events.len()}</strong>" receipts staged"</span>
                <span><strong>{visible_modules}</strong>" services visible"</span>
                <span><strong>{open_approvals}</strong>" approvals linked"</span>
            </div>

            <div class="resource-toolbar" aria-label="Resource console controls">
                <label>
                    <span aria-hidden="true">"⌕"</span>
                    <input data-resource-search="true" aria-label="Search resources and receipts" placeholder="Search resource, owner, region, receipt..." />
                </label>
                <div class="resource-filter-pills" role="toolbar" aria-label="Resource state filters">
                    <button type="button" class="active" data-resource-filter="all">"All"</button>
                    <button type="button" data-resource-filter="attention">"Attention"</button>
                    <button type="button" data-resource-filter="review">"Review"</button>
                    <button type="button" data-resource-filter="active">"Active"</button>
                </div>
                <div class="resource-actions">
                    <button type="button" data-resource-action="refresh">"Refresh data"</button>
                    <button type="button" data-resource-action="export">"Export CSV"</button>
                </div>
                <span data-resource-status="true">"6 visible · local inventory only"</span>
            </div>

            <div class="resource-panels">
                <article
                    id="resource-panel-inventory"
                    class="resource-panel active"
                    data-resource-panel="inventory"
                    role="tabpanel"
                    aria-labelledby="resource-tab-inventory"
                >
                    <div class="cockpit-column-head">
                        <p class="screen-anchor">"RESOURCE INVENTORY"</p>
                        <h4 id="resource-inventory-title">"Tenant assets with ownership, region, cost, and risk"</h4>
                    </div>
                    <table class="resource-table">
                        <thead>
                            <tr>
                                <th>"Kind"</th>
                                <th>"Name"</th>
                                <th>"Region"</th>
                                <th>"Owner"</th>
                                <th>"State"</th>
                                <th>"Monthly"</th>
                                <th>"Action"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {resources.into_iter().map(|row| view! {
                                <tr data-resource-row="true" data-resource-state=row.state>
                                    <td><span class="resource-kind">{row.kind}</span></td>
                                    <td><strong>{row.name}</strong><small>{row.description}</small></td>
                                    <td>{row.region}</td>
                                    <td>{row.owner}</td>
                                    <td><span class=resource_status_class(row.state)>{row.state}</span></td>
                                    <td class="numeric">{row.monthly}</td>
                                    <td>
                                        <button
                                            type="button"
                                            data-sidepeek-trigger="resource"
                                            data-sidepeek-title=row.name
                                            data-sidepeek-id=row.side_id
                                            data-sidepeek-desc=row.description
                                            data-sidepeek-owner=row.owner
                                            data-sidepeek-risk=row.risk
                                            data-sidepeek-sla="Inventory staged · no live mutation"
                                        >
                                            "Inspect"
                                        </button>
                                    </td>
                                </tr>
                            }).collect_view()}
                        </tbody>
                    </table>
                    {resource_inventory_anchor_board()}
                </article>

                <article
                    id="resource-panel-audit"
                    class="resource-panel"
                    data-resource-panel="audit"
                    role="tabpanel"
                    aria-labelledby="resource-tab-audit"
                >
                    <div class="cockpit-column-head">
                        <p class="screen-anchor">"AUDIT LEDGER"</p>
                        <h4 id="audit-ledger-title">"Immutable tenant-workload proof stream"</h4>
                    </div>
                    {receipt_stitching_console()}
                    <div class="audit-proof-grid" aria-label="FD-001 tenant workload receipt proof">
                        <article class="audit-proof-card selected" data-audit-card="fd001">
                            <p class="screen-anchor">"FD-001 RECEIPTS"</p>
                            <h5>"Product delivery remains master plan"</h5>
                            <p>
                                "Every Messenger, Mail, Community, Workflow, Ontology, and Intelligence preview action creates "
                                "a visible receipt so FD-001 can be dogfooded as a real tenant workload."
                            </p>
                            <div class="audit-command-actions">
                                <button type="button" data-audit-anchor-action="open-evidence">"Open evidence"</button>
                                <button type="button" data-audit-anchor-action="route-mail">"Mail brief"</button>
                            </div>
                        </article>
                        <article class="audit-proof-card" data-audit-card="cloud">
                            <p class="screen-anchor">"OYATIE CLOUD"</p>
                            <h5>"Oyatie Cloud substrate proves hosting posture"</h5>
                            <p>
                                "The cloud substrate records residency, release, cost, policy, and rollback checks before a "
                                "tenant surface can claim production readiness."
                            </p>
                            <div class="audit-command-actions">
                                <button type="button" data-audit-anchor-action="route-cloud">"Cloud topology"</button>
                                <button type="button" data-audit-anchor-action="route-gates">"Release gates"</button>
                            </div>
                        </article>
                        <article class="audit-proof-card" data-audit-card="sealed">
                            <p class="screen-anchor">"SEALED PACKET"</p>
                            <h5>"Interactive local receipt vault"</h5>
                            <p>
                                "Operators can inspect, seal, route, and brief a receipt packet visually while backend, billing, "
                                "deploy, and cloud mutations remain disconnected."
                            </p>
                            <div class="audit-command-actions">
                                <button type="button" data-audit-anchor-action="seal-packet">"Seal packet"</button>
                                <button type="button" data-audit-anchor-action="route-policy">"Policy board"</button>
                            </div>
                        </article>
                    </div>
                    <ol class="audit-ledger-list">
                        {audit_events.into_iter().map(|item| view! {
                            <li>
                                <time>{item.time}</time>
                                <span class=resource_status_class(item.severity)>{item.severity}</span>
                                <strong>{item.event}</strong>
                                <p>{item.actor}</p>
                                <code>{item.receipt}</code>
                                <button type="button" data-audit-anchor-action="inspect-receipt">"Inspect"</button>
                            </li>
                        }).collect_view()}
                    </ol>
                    <div class="audit-anchor-footer">
                        <span data-audit-anchor-status="true">
                            "Audit ledger ready · FD-001 tenant workload receipts remain local visual evidence."
                        </span>
                        <div class="audit-command-actions" aria-label="Audit ledger connected routes">
                            <button type="button" data-audit-anchor-action="route-workflow">"Workflow proof"</button>
                            <button type="button" data-audit-anchor-action="route-community">"Community review"</button>
                            <button type="button" data-audit-anchor-action="open-evidence">"Evidence graph"</button>
                        </div>
                    </div>
                </article>

                <article
                    id="resource-panel-gates"
                    class="resource-panel"
                    data-resource-panel="gates"
                    role="tabpanel"
                    aria-labelledby="resource-tab-gates"
                >
                    <div class="cockpit-column-head">
                        <p class="screen-anchor">"DEPLOYMENT GATES"</p>
                        <h4 id="deployment-gates-title">"FD-001 tenant workload admission gates"</h4>
                    </div>
                    {deployment_gate_command_board()}
                    <div class="gate-grid">
                        {gates.into_iter().map(|gate| view! {
                            <article class="gate-card">
                                <div>
                                    <span class=resource_status_class(gate.state)>{gate.state}</span>
                                    <h5>{gate.label}</h5>
                                    <p>{gate.detail}</p>
                                </div>
                                <span
                                    class="gate-progress"
                                    role="progressbar"
                                    aria-valuemin="0"
                                    aria-valuemax="100"
                                    aria-valuenow=gate.progress.trim_end_matches('%')
                                    aria-label=format!("{} gate progress: {}", gate.label, gate.progress)
                                    style=format!("--bar: {}", gate.progress)
                                >
                                    <em aria-hidden="true">{gate.progress}</em>
                                </span>
                                <div class="gate-card-actions">
                                    <button type="button" data-gate-action="attach-evidence">"Attach evidence"</button>
                                    <button type="button" data-gate-action="open-evidence">"Evidence"</button>
                                    <button type="button" data-gate-action="route-owner">"Owner route"</button>
                                </div>
                            </article>
                        }).collect_view()}
                    </div>
                    <div class="deployment-gate-footer">
                        <span data-deployment-gate-status="true">
                            "Deployment gates ready · FD-001 microservices are tenant workloads on Oyatie Cloud."
                        </span>
                        <div class="deployment-gate-routes" aria-label="Deployment gate connected routes">
                            <button type="button" data-deployment-gate-action="route-policy">"Policy envelope"</button>
                            <button type="button" data-deployment-gate-action="route-audit">"Audit packet"</button>
                            <button type="button" data-deployment-gate-action="route-community">"Community note"</button>
                            <button type="button" data-deployment-gate-action="route-cloud">"Cloud cells"</button>
                        </div>
                    </div>
                </article>
            </div>
        </section>
    }
}
