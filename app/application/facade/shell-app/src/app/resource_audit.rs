use super::*;

pub(super) fn resource_audit_console(
    envelope: TenantRenderEnvelope,
    available: Signal<bool>,
) -> impl IntoView {
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
            data-cloud-root="resources" data-cloud-source=cloud_resources_source(&envelope) data-cloud-context=envelope.context.id()
            data-cloud-label=daily_sample_label(&envelope) data-cloud-hub-source=HubSample::envelope_source(&envelope) data-cloud-mode=if cfg!(target_arch="wasm32") {"island"} else {"ssr"}
            data-ae-source=ae_source(&envelope)
            data-ae-context=envelope.context.id()
            data-ae-label=daily_sample_label(&envelope)
            data-ae-hub-source=HubSample::envelope_source(&envelope)
            data-ae-source-available=move || ae_available(available).to_string()
            data-ae-available=move || ae_available(available).to_string()
            data-ae-mode=if cfg!(target_arch = "wasm32") { "island" } else { "ssr" }
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
                <span><strong>{audit_events.len()}</strong>" sample receipts"</span>
                <span><strong>{visible_modules}</strong>" services visible"</span>
                <span><strong>{open_approvals}</strong>" approvals linked"</span>
            </div>

            <div class="resource-toolbar" aria-label="Resource console controls">
                <label>
                    <span aria-hidden="true">"⌕"</span>
                    <input data-resource-search="true" aria-label="Search inventory samples" placeholder="Search sample name, owner, region, state..." />
                </label>
                <div class="resource-filter-pills" role="toolbar" aria-label="Resource state filters">
                    <button type="button" class="active" data-resource-filter="all" aria-pressed="true">"All"</button>
                    <button type="button" data-resource-filter="attention" aria-pressed="false">"Attention"</button>
                    <button type="button" data-resource-filter="review" aria-pressed="false">"Review"</button>
                    <button type="button" data-resource-filter="active" aria-pressed="false">"Active"</button>
                </div>
                <div class="resource-actions">
                    <button type="button" data-resource-action="refresh">"Reset inventory samples"</button>
                    <button type="button" data-resource-action="export" disabled aria-disabled="true" title="Sample export unavailable; no file created.">"Export CSV"</button>
                </div>
                <span data-resource-status="true" role="status">{format!("{} of {} sample resources visible",resources.len(),resources.len())}</span>
            </div>

            <div class="resource-panels">
                <article id="resource-panel-inventory" class="resource-panel active" data-resource-panel="inventory" role="tabpanel" aria-labelledby="resource-tab-inventory">
                    <div class="cockpit-column-head"><p class="screen-anchor">"SAMPLE RESOURCE INVENTORY"</p><h4 id="resource-inventory-title">"Six illustrative resources"</h4></div>
                    <p data-cloud-availability="resources">"Fixed sample values; live inventory, spend and owner verification unavailable. Export and Cloud operations are unavailable."</p>
                    <div class="cloud-resource-table-scroll" role="region" aria-label="Illustrative resource inventory table" tabindex="0"><table class="resource-table"><thead><tr><th>"Kind"</th><th>"Name"</th><th>"Region"</th><th>"Owner example"</th><th>"State example"</th><th>"Sample monthly USD"</th><th>"Action"</th></tr></thead><tbody>
                        {resources.into_iter().map(|row| view! {
                            <tr data-resource-row="true" data-resource-state=row.state data-cloud-record-row="true">
                                <td><span class="resource-kind" data-cloud-value="Kind">{row.kind}</span></td><td><strong data-cloud-value="Name">{row.name}</strong><small data-cloud-value="Description">{row.description}</small></td>
                                <td data-cloud-value="Region">{row.region}</td><td data-cloud-value="Owner">{row.owner}</td><td><span class=resource_status_class(row.state) data-cloud-value="State">{row.state}</span></td><td class="numeric" data-cloud-value="Monthly">{row.monthly}</td>
                                <td><button type="button" data-cloud-select=format!("resource:{}",row.side_id) data-cloud-record=cloud_resources_record(&format!("resource:{}",row.side_id)) data-cloud-focus=format!("resource:{}",row.side_id) aria-pressed="false" aria-expanded="false" aria-controls="cloud-resources-inspection" aria-label=format!("Inspect illustrative resource {}",row.name)>"Inspect sample"</button></td>
                            </tr>
                        }).collect_view()}
                    </tbody></table></div><p data-cloud-empty="true" hidden role="status">"No sample resources match this query and filter."</p>
                    {cloud_inspection("resources")}<p data-cloud-status="resources" role="status">"Select a resource example to inspect its fields."</p>
                    {resource_inventory_anchor_board(&envelope)}
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
                        <h4 id="audit-ledger-title">"Illustrative Audit records; verification unavailable"</h4>
                    </div>
                    {ae_notice()}
                    {ae_inspection()}
                    {receipt_stitching_console()}
                    <div class="audit-proof-grid" aria-label="FD-001 tenant workload receipt proof">
                        <article class="audit-proof-card selected" data-audit-card="fd001">
                            <p class="screen-anchor">"FD-001 RECEIPTS"</p>
                            <h5>"Product delivery remains master plan"</h5>
                            <p>
                                "These fixed illustrative receipts describe example product routes. Preview actions create no receipt, draft or tenant workload."
                            </p>
                            <div class="audit-command-actions">
                                {ae_action("audit", "open-evidence", "Open evidence")}
                                {ae_action("audit", "route-mail", "Mail brief")}
                            </div>
                        </article>
                        <article class="audit-proof-card" data-audit-card="cloud">
                            <p class="screen-anchor">"OYATIE CLOUD"</p>
                            <h5>"Illustrative cloud posture; live proof unknown"</h5>
                            <p>
                                "The cloud substrate records residency, release, cost, policy, and rollback checks before a "
                                "tenant surface can claim production readiness."
                            </p>
                            <div class="audit-command-actions">
                                {ae_action("audit", "route-cloud", "Cloud topology")}
                                {ae_action("audit", "route-gates", "Release gates")}
                            </div>
                        </article>
                        <article class="audit-proof-card" data-audit-card="sealed">
                            <p class="screen-anchor">"SEALED PACKET"</p>
                            <h5>"Illustrative receipt inspection"</h5>
                            <p>
                                "Inspect exact sample fields and open existing routes. Sealing, briefing and all live operations remain unavailable."
                            </p>
                            <div class="audit-command-actions">
                                {ae_action("audit", "seal-packet", "Seal packet")}
                                {ae_action("audit", "route-policy", "Policy board")}
                            </div>
                        </article>
                    </div>
                    <ol class="audit-ledger-list">
                        {audit_events.into_iter().map(|item| view! {
                            <li data-ae-record-row="true">
                                <time data-ae-value="Time">{item.time}</time>
                                <span class=resource_status_class(item.severity)><span data-ae-value="State">{item.severity}</span>" example"</span>
                                <strong data-ae-value="Event">{item.event}</strong>
                                <p data-ae-value="Actor">{item.actor}</p>
                                <code data-ae-value="Receipt">{item.receipt}</code>
                                <button type="button" data-audit-anchor-action="inspect-receipt" data-ae-select=format!("audit:{}", item.receipt)
                                    data-ae-record=ae_record(&format!("audit:{}", item.receipt)) data-ae-focus=format!("audit:{}", item.receipt) aria-pressed="false">"Inspect sample"</button>
                            </li>
                        }).collect_view()}
                    </ol>
                    <div class="audit-anchor-footer">
                        <span data-audit-anchor-status="true">
                            "Illustrative Audit examples only; no receipt created, sealed or verified."
                        </span>
                        <div class="audit-command-actions" aria-label="Audit ledger connected routes">
                            {ae_action("audit", "route-workflow", "Workflow proof")}
                            {ae_action("audit", "route-community", "Community review")}
                            {ae_action_at("audit", "open-evidence", "Evidence graph", "footer")}
                        </div>
                    </div>
                </article>

                <article id="resource-panel-gates" class="resource-panel" data-resource-panel="gates" role="tabpanel" aria-labelledby="resource-tab-gates">
                    <div class="cockpit-column-head"><p class="screen-anchor">"SAMPLE DEPLOYMENT GATES"</p><h4 id="deployment-gates-title">"Four illustrative gate records"</h4></div>
                    <p data-cloud-availability="gates">"These fixed percentages are examples, not measured readiness. Attach, admit, seal and owner assignment are unavailable."</p>
                    {deployment_gate_command_board(&envelope)}
                    <div class="gate-grid">{gates.into_iter().map(|gate| view! {
                        <article class="gate-card" data-cloud-gate=gate.label><div><span class=resource_status_class(gate.state) data-cloud-value="State">{gate.state}</span><h5 data-cloud-value="Label">{gate.label}</h5><p data-cloud-value="Detail">{gate.detail}</p></div>
                            <span class="gate-progress" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow=gate.progress.trim_end_matches('%') aria-label=format!("Sample {} gate progress: {}; not measured readiness",gate.label,gate.progress) style=format!("--bar: {}",gate.progress)><em aria-hidden="true" data-cloud-value="Progress">{gate.progress}</em></span>
                            <div class="gate-card-actions">{cloud_unavailable(&format!("gate:{}:attach",gate.label),"Attach evidence")}{cloud_route(&envelope,&format!("gate:{}:evidence",gate.label),"Evidence sample")}{cloud_unavailable(&format!("gate:{}:owner",gate.label),"Owner assignment")}</div>
                        </article>
                    }).collect_view()}</div>{cloud_inspection("gates")}<p data-cloud-status="gates" role="status">"Select a gate example to inspect it; no admission performed."</p>
                    <div class="deployment-gate-routes" aria-label="Existing gate sample destinations">{cloud_route(&envelope,"gates:policy","Policy sample")}{cloud_route(&envelope,"gates:audit","Audit sample")}{cloud_route(&envelope,"gates:community","Community sample")}</div>
                </article>
            </div>
        </section>
    }
}
