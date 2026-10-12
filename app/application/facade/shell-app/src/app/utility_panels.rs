use super::*;

#[component]
pub(super) fn UtilityPanels() -> impl IntoView {
    view! {
        <dialog id="notification-center" class="utility-panel activity-center" data-utility-panel="notifications"
            data-notification-source=notification_source() aria-labelledby="notification-title"
            aria-describedby="notification-notice notification-status" closedby="any"
            inner_html=notification_body_html()></dialog>

        <dialog id="settings-center" class="utility-panel settings-center" data-utility-panel="settings"
            data-settings-density="comfortable" aria-label="Settings · sample preview" closedby="any"
            inner_html=settings_body_html()></dialog>
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

pub(super) fn settings_body_html() -> &'static str {
    r#"<div class="utility-panel-head"><div><p class="screen-anchor">SETTINGS · SAMPLE PREVIEW</p><h2 data-settings-label="title" lang="en">Settings · sample preview</h2></div><button type="button" data-utility-close="true" aria-label="Close settings" commandfor="settings-center" command="close">×</button></div>
<section class="utility-proof-strip settings-proof" aria-label="Sample Settings context"><article><p class="screen-anchor">PAGE MEMORY ONLY</p><strong>Settings preview; live account and workspace facts unavailable</strong><span>Preferences change this Settings drawer only and reset on reload. They never supply identity, grants, privacy, integration or tenant admission authority.</span></article><div class="utility-route-grid" aria-label="Existing sample Settings destinations"><button type="button" data-utility-route="identity"><span>Account</span><strong>Sample profile</strong></button><button type="button" data-utility-route="policy"><span>Policy</span><strong>Sample access matrix</strong></button><button type="button" data-utility-route="catalog"><span>Catalog</span><strong>Sample modules</strong></button></div></section>
<div class="settings-person-card" aria-label="Sample person and workspace"><span class="workspace-avatar" aria-hidden="true">최</span><div><strong>Sample: 최유나 · Choi Yu-na</strong><p>Sample Tenant Admin · Finance owner; live role and privacy status Unknown / unavailable.</p></div></div>
<div class="settings-tabs" role="tablist" aria-label="Sample Settings panels" aria-orientation="horizontal"><button type="button" id="settings-tab-profile" class="active" data-settings-tab="profile" data-settings-label="profile" lang="en" role="tab" aria-selected="true" aria-controls="settings-panel-profile">Sample profile</button><button type="button" id="settings-tab-appearance" data-settings-tab="appearance" data-settings-label="appearance" lang="en" role="tab" aria-selected="false" aria-controls="settings-panel-appearance">Drawer appearance</button><button type="button" id="settings-tab-integrations" data-settings-tab="integrations" data-settings-label="integrations" lang="en" role="tab" aria-selected="false" aria-controls="settings-panel-integrations">Sample integrations</button><button type="button" id="settings-tab-audit" data-settings-tab="audit" data-settings-label="audit" lang="en" role="tab" aria-selected="false" aria-controls="settings-panel-audit">Sample audit examples</button></div>
<article id="settings-panel-profile" class="settings-panel active" data-settings-panel="profile" role="tabpanel" aria-labelledby="settings-tab-profile"><dl class="settings-kv" aria-label="Sample workspace facts"><div><dt>Sample workspace</dt><dd>Oyatie Corp. · 118 example employees</dd></div><div><dt>Sample role</dt><dd>Admin · payroll approver example</dd></div><div><dt>Sample region pack</dt><dd>US/EU/KR example; live enablement unavailable</dd></div></dl><button type="button" data-settings-action="open-identity">Open sample account profile</button></article>
<article id="settings-panel-appearance" class="settings-panel" data-settings-panel="appearance" role="tabpanel" aria-labelledby="settings-tab-appearance"><p>Density changes only Settings drawer spacing. Language previews only its title and four tab labels. Both reset on reload; the rest of the shell is unchanged.</p><div class="settings-action-grid" aria-label="Settings drawer preview preferences"><button type="button" data-settings-action="density-comfortable" aria-pressed="true">Comfortable drawer</button><button type="button" data-settings-action="density-compact" aria-pressed="false">Compact drawer</button><button type="button" data-settings-action="locale-ko" aria-pressed="false">한국어 drawer label preview</button><button type="button" data-settings-action="locale-en" aria-pressed="true">English drawer label preview</button></div></article>
<article id="settings-panel-integrations" class="settings-panel" data-settings-panel="integrations" role="tabpanel" aria-labelledby="settings-tab-integrations"><ol class="integration-list" aria-label="Sample integration examples"><li><strong>Sample Shinhan Bank</strong><span class="status-chip">Unknown / unavailable</span><small>Example only; no account or transport verification supplied.</small></li><li><strong>Sample HomeTax</strong><span class="status-chip">Unknown / unavailable</span><small>Example only; no filing connection supplied.</small></li><li><strong>Sample Google Workspace</strong><span class="status-chip">Unknown / unavailable</span><small>Example only; no connected mail service supplied.</small></li></ol></article>
<article id="settings-panel-audit" class="settings-panel" data-settings-panel="audit" role="tabpanel" aria-labelledby="settings-tab-audit"><p>Static audit examples, not events from this page or a browser, session or server log.</p><ol class="activity-list compact" aria-label="Static sample Settings audit examples"><li><time>09:14 example</time><strong>Sample drawer inspection</strong><p>Illustrative event only.</p></li><li><time>09:18 example</time><strong>Sample density choice</strong><p>Illustrative event only; no persistence claimed.</p></li><li><time>09:42 example</time><strong>Sample account navigation</strong><p>Illustrative event only; no authentication occurred.</p></li></ol></article>
<p class="settings-status" data-settings-status="true" role="status">Settings drawer previews use page memory and reset on reload. Live account facts unavailable.</p>"#
}
