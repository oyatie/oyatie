use super::*;

#[component]
pub(super) fn UtilityPanels() -> impl IntoView {
    view! {
        <div class="utility-panel-backdrop" data-utility-backdrop hidden></div>

        <section
            class="utility-panel activity-center"
            data-utility-panel="notifications"
            aria-label="Notification and activity center"
            aria-hidden="true"
        >
            <div class="utility-panel-head">
                <div>
                    <p class="screen-anchor">"ACTIVITY CENTER"</p>
                    <h2>"Notifications, approvals, and local events"</h2>
                </div>
                <button type="button" data-utility-close="true" aria-label="Close activity center">"×"</button>
            </div>
            <section class="utility-proof-strip" aria-label="Activity center FD-001 substrate proof">
                <article>
                    <p class="screen-anchor">"FD-001 OPERATIONS SIGNALS"</p>
                    <strong>"Notifications are workload control signals, not inbox noise"</strong>
                    <span data-activity-status="true">"Close, filing, vendor, and audit events are Oyatie Cloud tenant workload previews."</span>
                </article>
                <div class="utility-route-grid" aria-label="Activity center routes">
                    <button type="button" data-utility-route="work-hub"><span>"Comms"</span><strong>"Work Hub"</strong></button>
                    <button type="button" data-utility-route="evidence"><span>"Receipt"</span><strong>"Evidence spine"</strong></button>
                    <button type="button" data-utility-route="cloud"><span>"Substrate"</span><strong>"Cloud cells"</strong></button>
                </div>
            </section>
            <div class="utility-summary">
                    <span><strong data-activity-count="true">"3"</strong><small>"unread"</small></span>
                <span><strong>"12"</strong><small>"today"</small></span>
                <span><strong>"3"</strong><small>"blocking"</small></span>
            </div>
            <div class="utility-filter-row" role="toolbar" aria-label="Activity filters">
                <button type="button" class="active" data-activity-filter="all">"All"</button>
                <button type="button" data-activity-filter="unread">"Unread"</button>
                <button type="button" data-activity-filter="blocking">"Blocking"</button>
                <button type="button" data-activity-action="clear-read">"Clear read"</button>
            </div>
            <ol class="activity-list" data-activity-list="true" aria-live="polite">
                <li data-activity-item="true" data-activity-state="unread" data-activity-severity="blocking">
                    <time>"09:18"</time>
                    <span class="status-chip danger">"blocking"</span>
                    <strong>"4대보험 변동 확인 필요"</strong>
                    <p>"Payroll close cannot seal until Park Seo-jun's insurance delta is reviewed."</p>
                    <button type="button" data-activity-action="mark-read">"Mark read"</button>
                </li>
                <li data-activity-item="true" data-activity-state="unread" data-activity-severity="review">
                    <time>"09:42"</time>
                    <span class="status-chip warning">"review"</span>
                    <strong>"Withholding tax brief ready"</strong>
                    <p>"HomeTax transport is staged locally; reviewer must approve before send."</p>
                    <button type="button" data-activity-action="mark-read">"Mark read"</button>
                </li>
                <li data-activity-item="true" data-activity-state="unread" data-activity-severity="blocking">
                    <time>"10:05"</time>
                    <span class="status-chip danger">"vendor"</span>
                    <strong>"Stripe renewal needs owner"</strong>
                    <p>"Spend approval exceeds one-step threshold and requires CFO attestation."</p>
                    <button type="button" data-activity-action="mark-read">"Mark read"</button>
                </li>
                <li data-activity-item="true" data-activity-state="read" data-activity-severity="info">
                    <time>"10:21"</time>
                    <span class="status-chip success">"sealed"</span>
                    <strong>"Audit receipt staged"</strong>
                    <p>"REC-FIN-2026-05 was added to the local close package preview."</p>
                    <button type="button" data-activity-action="open-audit">"Open audit"</button>
                </li>
            </ol>
        </section>

        <section
            class="utility-panel settings-center"
            data-utility-panel="settings"
            aria-label="Workspace settings"
            aria-hidden="true"
        >
            <div class="utility-panel-head">
                <div>
                    <p class="screen-anchor">"SETTINGS"</p>
                    <h2>"Workspace, profile, appearance, and integrations"</h2>
                </div>
                <button type="button" data-utility-close="true" aria-label="Close settings">"×"</button>
            </div>
            <section class="utility-proof-strip settings-proof" aria-label="Settings FD-001 substrate proof">
                <article>
                    <p class="screen-anchor">"CONTROL PLANE SETTINGS"</p>
                    <strong>"Workspace preferences stay tied to FD-001, policy, and Oyatie Cloud posture"</strong>
                    <span>"Every preference is local visual state; no auth, IAM, billing, integration, mail, or cloud mutation occurs."</span>
                </article>
                <div class="utility-route-grid" aria-label="Settings connected routes">
                    <button type="button" data-utility-route="identity"><span>"Identity"</span><strong>"Role envelope"</strong></button>
                    <button type="button" data-utility-route="policy"><span>"Policy"</span><strong>"Access matrix"</strong></button>
                    <button type="button" data-utility-route="catalog"><span>"Catalog"</span><strong>"Tenant modules"</strong></button>
                </div>
            </section>
            <div class="settings-person-card">
                <span class="workspace-avatar" aria-hidden="true">"최"</span>
                <div><strong>"최유나 · Choi Yu-na"</strong><p>"Tenant admin · Finance owner · PIPA-safe contract envelope"</p></div>
            </div>
            // A-2: settings tablist — aria-orientation + id/aria-controls on tabs + role=tabpanel on panels
            <div class="settings-tabs" role="tablist" aria-label="Settings panels" aria-orientation="horizontal">
                <button type="button" id="settings-tab-profile" class="active" data-settings-tab="profile" role="tab" aria-selected="true" aria-controls="settings-panel-profile">"Profile"</button>
                <button type="button" id="settings-tab-appearance" data-settings-tab="appearance" role="tab" aria-selected="false" aria-controls="settings-panel-appearance">"Appearance"</button>
                <button type="button" id="settings-tab-integrations" data-settings-tab="integrations" role="tab" aria-selected="false" aria-controls="settings-panel-integrations">"Integrations"</button>
                <button type="button" id="settings-tab-audit" data-settings-tab="audit" role="tab" aria-selected="false" aria-controls="settings-panel-audit">"Audit"</button>
            </div>
            <article id="settings-panel-profile" class="settings-panel active" data-settings-panel="profile" role="tabpanel" aria-labelledby="settings-tab-profile">
                <dl class="settings-kv">
                    <div><dt>"Workspace"</dt><dd>"Oyatie Corp. · 118 employees"</dd></div>
                    <div><dt>"Role"</dt><dd>"Admin · payroll close approver"</dd></div>
                    <div><dt>"Region pack"</dt><dd>"US/EU/KR · Korean payroll enabled"</dd></div>
                </dl>
                <button type="button" data-settings-action="open-identity">"Open identity profile"</button>
            </article>
            <article id="settings-panel-appearance" class="settings-panel" data-settings-panel="appearance" role="tabpanel" aria-labelledby="settings-tab-appearance">
                <p>"Adjust local visual density and shell language without changing server state."</p>
                <div class="settings-action-grid">
                    <button type="button" data-settings-action="density-comfortable">"Comfortable"</button>
                    <button type="button" data-settings-action="density-compact">"Compact"</button>
                    <button type="button" data-settings-action="locale-ko">"한국어 우선"</button>
                    <button type="button" data-settings-action="locale-en">"English labels"</button>
                </div>
            </article>
            <article id="settings-panel-integrations" class="settings-panel" data-settings-panel="integrations" role="tabpanel" aria-labelledby="settings-tab-integrations">
                <ol class="integration-list">
                    <li><strong>"Shinhan Bank"</strong><span class="status-chip success">"verified"</span><small>"Bank transport staged locally; no money movement."</small></li>
                    <li><strong>"HomeTax"</strong><span class="status-chip warning">"review"</span><small>"Filing transport waits for human attestation."</small></li>
                    <li><strong>"Google Workspace"</strong><span class="status-chip">"local"</span><small>"Mail and community previews only."</small></li>
                </ol>
            </article>
            <article id="settings-panel-audit" class="settings-panel" data-settings-panel="audit" role="tabpanel" aria-labelledby="settings-tab-audit">
                <ol class="activity-list compact">
                    <li><time>"09:14"</time><strong>"Settings drawer opened"</strong><p>"Local shell state only."</p></li>
                    <li><time>"09:18"</time><strong>"Density preference staged"</strong><p>"Stored in this browser session."</p></li>
                    <li><time>"09:42"</time><strong>"Identity panel linked"</strong><p>"No auth mutation."</p></li>
                </ol>
            </article>
            <p class="settings-status" data-settings-status="true">"Local settings ready · no backend persistence."</p>
        </section>
    }
}

#[component]
pub(super) fn SidePeek() -> impl IntoView {
    view! {
        <aside
            class="side-peek"
            data-side-peek="true"
            aria-label="Object quick view"
            aria-hidden="true"
        >
            <div class="side-peek-head">
                <div>
                    <p class="screen-anchor">"OBJECT QUICK VIEW"</p>
                    <h2 data-sidepeek-title-target="true">"Network hot split"</h2>
                </div>
                <button type="button" data-sidepeek-close="true" aria-label="Close object quick view">"×"</button>
            </div>
            <div class="side-peek-body">
                <section class="quick-identity">
                    <span class="workspace-avatar" aria-hidden="true">"N"</span>
                    <div>
                        <strong data-sidepeek-id-target="true">"CHG-NTW-4182"</strong>
                        <p data-sidepeek-desc-target="true">"Tenant network split awaiting residency and rollback evidence."</p>
                    </div>
                </section>
                <dl class="peek-kv">
                    <div><dt>"Owner"</dt><dd data-sidepeek-owner-target="true">"Infrastructure operations"</dd></div>
                    <div><dt>"Risk"</dt><dd><span class="status-chip danger" data-sidepeek-risk-target="true">"High"</span></dd></div>
                    <div><dt>"SLA"</dt><dd data-sidepeek-sla-target="true">"4.0h target · +1.4h over"</dd></div>
                    <div><dt>"Execution"</dt><dd>"Visual-only until live integration"</dd></div>
                </dl>
                <section class="side-peek-proof" aria-label="FD-001 object proof">
                    <p class="screen-anchor">"OBJECT PROOF"</p>
                    <strong>"Selected objects resolve to FD-001 workload evidence on Oyatie Cloud"</strong>
                    <span data-sidepeek-status="true">"Inspector ready · REC-WF-7741 · cell-us-east-2 · local visual state only."</span>
                    <div class="side-peek-route-grid" aria-label="Object proof routes">
                        <button type="button" data-sidepeek-route="workload"><span>"Workload"</span><strong>"FD-001 graph"</strong></button>
                        <button type="button" data-sidepeek-route="cloud"><span>"Cloud"</span><strong>"cell-us-east-2"</strong></button>
                        <button type="button" data-sidepeek-route="evidence"><span>"Receipt"</span><strong>"REC-WF-7741"</strong></button>
                    </div>
                </section>
                <section>
                    <h3>"Evidence trail"</h3>
                    <ol class="peek-timeline">
                        <li><time>"09:18"</time><span>"Policy guardrail matched residency rule for FD-001 tenant workload."</span></li>
                        <li><time>"09:42"</time><span>"Oyatie Cloud rollback plan requested from network owner."</span></li>
                        <li><time>"10:05"</time><span>"Audit-chain receipt REC-WF-7741 drafted locally."</span></li>
                    </ol>
                </section>
                <section class="peek-actions" aria-label="Object actions">
                    <button type="button" data-sidepeek-action="assign-owner">"Assign owner"</button>
                    <button type="button" data-sidepeek-action="draft-note">"Draft note"</button>
                    <button type="button" class="primary" data-sidepeek-action="review-evidence">"Review evidence"</button>
                </section>
            </div>
        </aside>
    }
}
