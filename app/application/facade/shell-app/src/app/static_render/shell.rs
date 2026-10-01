use super::*;

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_rail_html() -> String {
    r##"<aside class="app-rail" aria-label="Product navigation">
    <div class="rail-brand"><span class="rail-mark" aria-hidden="true">O</span><div><strong>Oyatie</strong><span>Control Center</span></div><code>v0.1</code></div>
    <section class="rail-proof-card" aria-label="FD-001 and Oyatie Cloud shell proof">
      <p>FD-001 TENANT WORKLOADS</p>
      <strong>Service graph on Oyatie Cloud</strong>
      <span>Messenger · Mail · Community dogfood the substrate.</span>
      <small data-rail-status="true">REC-WF-7741 · cell-us-east-2 · local visual routes</small>
      <div class="rail-proof-actions" aria-label="Persistent shell proof routes">
        <button type="button" class="is-selected" data-rail-proof-action="service-graph">Service graph</button>
        <button type="button" data-rail-proof-action="cloud">Cloud</button>
        <button type="button" data-rail-proof-action="evidence">Evidence</button>
        <button type="button" data-rail-proof-action="work-hub">Work hub</button>
      </div>
      <div class="rail-comms-switcher" aria-label="Built-in Work Hub surface routes">
        <button type="button" class="is-selected" data-rail-comms-surface="Messenger">Messenger</button>
        <button type="button" data-rail-comms-surface="Mail">Mail</button>
        <button type="button" data-rail-comms-surface="Community">Community</button>
      </div>
    </section>
    <p class="rail-group">Run the company</p>
    <a class="rail-nav active" href="#console-shell"><span aria-hidden="true">⌂</span>Command center</a>
    <a class="rail-nav" href="#command-center-workbench"><span aria-hidden="true">▥</span>Action Inbox<em>8</em></a>
    <a class="rail-nav" href="#governance-analytics"><span aria-hidden="true">↟</span>Governance analytics</a>
    <p class="rail-group">Operate</p>
    <a class="rail-nav" href="#business-logics"><span aria-hidden="true">⌬</span>Business Logics<em>17</em></a>
    <a class="rail-nav" href="#tasks-title"><span aria-hidden="true">☑</span>Tasks<em>73</em></a>
    <a class="rail-nav" href="#schedule-title"><span aria-hidden="true">◷</span>Schedule</a>
    <a class="rail-nav" href="#workflow-studio"><span aria-hidden="true">⌘</span>Workflow Studio</a>
    <a class="rail-nav" href="#work-hub"><span aria-hidden="true">✉</span>Messenger · Mail · Community<em>18</em></a>
    <a class="rail-nav" href="#cloud-ops-cockpit"><span aria-hidden="true">◫</span>Cloud Ops</a>
    <p class="rail-group">Money</p>
    <a class="rail-nav" href="#payroll-cockpit"><span aria-hidden="true">₩</span>Payroll</a>
    <a class="rail-nav" href="#ledger-preview"><span aria-hidden="true">▤</span>Ledger</a>
    <a class="rail-nav" href="#vendors-spend"><span aria-hidden="true">◇</span>Vendors &amp; spend</a>
    <a class="rail-nav" href="#billing-tax"><span aria-hidden="true">▧</span>Billing &amp; tax</a>
    <a class="rail-nav" href="#finops-pane"><span aria-hidden="true">₩</span>FinOps</a>
    <p class="rail-group">Compliance</p>
    <a class="rail-nav" href="#filing-readiness"><span aria-hidden="true">□</span>Filing readiness<em>2</em></a>
    <a class="rail-nav" href="#audit-ledger"><span aria-hidden="true">◱</span>Audit ledger</a>
    <a class="rail-nav" href="#policy-access"><span aria-hidden="true">⚿</span>Policy &amp; access</a>
    <p class="rail-group">People</p>
            <a class="rail-nav" href="#identity-employees"><span aria-hidden="true">◎</span>Employees</a>
            <a class="rail-nav" href="#leave-time"><span aria-hidden="true">◫</span>Leave &amp; time</a>
            <a class="rail-nav" href="#identity-workforce-service"><span aria-hidden="true">⚿</span>Auth · Org</a>
            <p class="rail-group">Trust</p>
    <a class="rail-nav" href="#resource-inventory"><span aria-hidden="true">▤</span>Resource inventory</a>
    <a class="rail-nav" href="#modules-title"><span aria-hidden="true">▦</span>Service catalog</a>
    <a class="rail-nav" href="#evidence-spine"><span aria-hidden="true">▥</span>Evidence spine</a>
    <a class="rail-nav" href="#deployment-gates"><span aria-hidden="true">✓</span>Deployment gates</a>
    <a class="rail-nav" href="#ontology-title"><span aria-hidden="true">◎</span>Object graph</a>
    <a class="rail-nav" href="#intelligence-title"><span aria-hidden="true">✦</span>Copilot rail</a>
    <div class="workspace-switch"><span class="workspace-avatar" aria-hidden="true">N</span><div><strong>Northwind</strong><span>Enterprise · US/EU/KR</span></div></div>
  </aside>"##
        .to_string()
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_header_html() -> String {
    r#"<header class="app-header" role="banner">
    <div class="top-breadcrumb" aria-label="Breadcrumb"><span>Oyatie Cloud</span><span class="sep">/</span><span>Operations</span><span class="sep">/</span><strong>Control Center</strong></div>
    <div class="header-route-strip" aria-label="FD-001 and Oyatie Cloud quick routes"><button type="button" class="is-selected" data-header-route="fd001"><span>FD-001</span>Service graph</button><button type="button" data-header-route="cloud"><span>Cloud</span>Substrate</button><button type="button" data-header-route="work-hub"><span>Comms</span>Work hub</button><div class="header-comms-switcher" aria-label="Built-in communications quick routes"><button type="button" class="is-selected" data-header-comms-surface="Messenger">Messenger</button><button type="button" data-header-comms-surface="Mail">Mail</button><button type="button" data-header-comms-surface="Community">Community</button></div><button type="button" data-header-route="evidence"><span>Audit</span>Evidence</button><small data-header-route-status="true">REC-WF-7741 · local quick routes</small></div>
    <button class="command-trigger" type="button" data-command-trigger="true" aria-haspopup="dialog"><span aria-hidden="true">⌕</span><span>Search actions, objects, workflows</span><kbd>⌘K</kbd></button>
    <div class="header-actions" aria-label="Shell render status"><button type="button" class="header-status">SSR shell</button><button type="button" class="header-status muted">Selective WASM islands</button><button type="button" class="header-icon" data-header-action="notifications" aria-label="Open notifications">◔<span class="header-badge" data-activity-badge="true">3</span></button><button type="button" class="header-icon" data-header-action="settings" aria-label="Open settings">⚙</button></div>
  </header>"#
        .to_string()
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_hero_html() -> String {
    r#"<section class="hero-panel" aria-labelledby="console-title"><div class="hero-main"><div class="page-title-copy"><p class="screen-anchor">01 / Command Center</p><div class="hero-title-row"><h1 id="console-title">Operations · 2026 May, week 19</h1><span class="hero-lens-chip">● Lens: tenant admin · Finance · 1,000 ppl</span></div><p id="console-notice" class="scope-notice" role="note">Anonymous preview only: sample tenant and role; no PHI/PII. No tenant or role was verified, and live service status was not queried. Live Ontology status requires sign-in.</p></div><section class="hero-close-strip" aria-label="FD-001 close command proof"><div><p class="screen-anchor">FD-001 CLOSE COMMAND</p><strong>April close proves the product workload on Oyatie Cloud</strong><span data-hero-status="true">Ready · REC-CLOSE-2026-04 · cell-us-east-2 · local command only</span></div><div class="hero-close-actions" aria-label="Close package routes"><button type="button" data-hero-action="close-april">Stage close</button><button type="button" data-hero-action="route-ledger">Ledger</button><button type="button" data-hero-action="route-cloud">Cloud proof</button><button type="button" data-hero-action="route-evidence">Evidence</button></div></section><section class="render-architecture-strip" aria-label="SSR shell and selective WASM hydration model"><article class="selected" data-render-arch-card="ssr"><p class="screen-anchor">SSR SHELL</p><strong>Fast baseline, service graph visible first</strong><span>Navigation, proof copy, tenant posture, and core dashboards render before island hydration.</span><button type="button" class="is-selected" data-render-arch-action="ssr">Show shell</button></article><article data-render-arch-card="islands"><p class="screen-anchor">SELECTIVE WASM</p><strong>Only interactive product surfaces hydrate</strong><span>Workflow Studio, Work Hub, filters, canvas state, and local drafts become browser-only islands.</span><button type="button" data-render-arch-action="islands">Show islands</button></article><article data-render-arch-card="boundary"><p class="screen-anchor">LOCAL BOUNDARY</p><strong>Visually functional, deliberately unwired</strong><span data-render-arch-status="true">No workflow execution, external send, IAM, billing, deploy, or cloud mutation.</span><button type="button" data-render-arch-action="boundary">Show evidence</button></article></section></div><div class="hero-side"><div class="hero-copy page-actions"><button type="button" data-sidepeek-trigger="new-action" data-sidepeek-title="Create governed action" data-sidepeek-id="ACT-LOCAL-DRAFT" data-sidepeek-desc="Local visual-only action draft. Nothing is persisted or sent." data-sidepeek-owner="Current operator session" data-sidepeek-risk="Draft" data-sidepeek-sla="No live SLA">New action</button><button type="button" data-command-trigger="true">Search ⌘K</button><button type="button" class="primary" data-hero-action="close-april">Close April →</button></div></div></section>"#
        .to_string()
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_command_palette_html() -> String {
    r#"<div class="command-palette-backdrop" data-command-backdrop hidden>
    <section class="command-palette" role="dialog" aria-modal="true" aria-label="Command palette">
      <div class="command-input-row"><span aria-hidden="true">⌕</span><input aria-label="Search command palette" placeholder="Search actions, objects, workflows…" value="" /><kbd>ESC</kbd></div>
      <section class="command-proof-strip" aria-label="FD-001 command palette proof">
        <article><p class="screen-anchor">COMMAND GRAPH</p><strong>One launcher for FD-001 tenant workloads on Oyatie Cloud</strong><span data-command-status="true">15 commands · REC-WF-7741 · cell-us-east-2 · local visual routes only</span></article>
        <div class="command-proof-grid" aria-label="Command proof shortcuts"><button type="button" data-command-proof-action="fd001"><span>FD-001</span><strong>Service graph</strong></button><button type="button" data-command-proof-action="cloud"><span>Cloud</span><strong>Substrate cells</strong></button><button type="button" data-command-proof-action="receipt"><span>Receipt</span><strong>REC-WF-7741</strong></button></div>
      </section>
      <div class="command-results" role="listbox" aria-label="Local command routes">
        <button type="button" data-command-action="workflow"><strong>Open Workflow Studio</strong><span>Build governed no-code flows for FD-001 work</span><kbd>W</kbd></button>
        <button type="button" data-command-action="mail"><strong>Compose mail</strong><span>Draft formal work messages locally</span><kbd>M</kbd></button>
        <button type="button" data-command-action="community"><strong>Post to community</strong><span>Coordinate role-aware spaces</span><kbd>C</kbd></button>
        <button type="button" data-command-action="peek"><strong>Inspect audit chain</strong><span>Open object graph and evidence spine</span><kbd>A</kbd></button>
        <button type="button" data-command-action="business-logics"><strong>Open Business Logic OS</strong><span>Inspect cost, health, owners, dependencies, and workflow routes</span><kbd>B</kbd></button>
        <button type="button" data-command-action="topology"><strong>Open cloud topology</strong><span>Inspect Oyatie Cloud tenant runtime cells and services</span><kbd>T</kbd></button>
        <button type="button" data-command-action="policy"><strong>Review policy access</strong><span>Open role envelope matrix</span><kbd>P</kbd></button>
        <button type="button" data-command-action="inventory"><strong>Open resource inventory</strong><span>Inspect ownership, cost, and risk</span><kbd>R</kbd></button>
        <button type="button" data-command-action="audit"><strong>Open audit ledger</strong><span>Review staged immutable receipts</span><kbd>L</kbd></button>
        <button type="button" data-command-action="gates"><strong>Review deployment gates</strong><span>Check Jenkins, ArgoCD, cosign, and audit evidence</span><kbd>G</kbd></button>
        <button type="button" data-command-action="catalog"><strong>Open Service Catalog</strong><span>Inspect service graph, routes, owners, and module access</span><kbd>⌘S</kbd></button>
        <button type="button" data-command-action="identity"><strong>Open Identity &amp; Workforce</strong><span>Manage auth, org profile, roles, employees, onboarding</span><kbd>I</kbd></button>
        <button type="button" data-command-action="finance"><strong>Open Finance Control</strong><span>Inspect ledger, vendors, billing, tax, leave and time</span><kbd>F</kbd></button>
        <button type="button" data-command-action="notifications"><strong>Open Activity Center</strong><span>Review notifications, approvals, and local events</span><kbd>N</kbd></button>
        <button type="button" data-command-action="settings"><strong>Open Workspace Settings</strong><span>Profile, density, integrations, and audit preferences</span><kbd>S</kbd></button>
      </div>
      <div class="command-palette-footer"><span>Local-only: commands route the visual shell without backend, workflow, mail, IAM, billing, deploy, or cloud mutation.</span><button type="button" data-command-proof-action="local-boundary">Show boundary</button></div>
    </section>
  </div>"#
        .to_string()
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_utility_panels_html() -> String {
    r#"<div class="utility-panel-backdrop" data-utility-backdrop hidden></div><section class="utility-panel activity-center" data-utility-panel="notifications" aria-label="Notification and activity center" aria-hidden="true">
    <div class="utility-panel-head"><div><p class="screen-anchor">ACTIVITY CENTER</p><h2>Notifications, approvals, and local events</h2></div><button type="button" data-utility-close="true" aria-label="Close activity center">×</button></div><section class="utility-proof-strip" aria-label="Activity center FD-001 substrate proof"><article><p class="screen-anchor">FD-001 OPERATIONS SIGNALS</p><strong>Notifications are workload control signals, not inbox noise</strong><span data-activity-status="true">Close, filing, vendor, and audit events are Oyatie Cloud tenant workload previews.</span></article><div class="utility-route-grid" aria-label="Activity center routes"><button type="button" data-utility-route="work-hub"><span>Comms</span><strong>Work Hub</strong></button><button type="button" data-utility-route="evidence"><span>Receipt</span><strong>Evidence spine</strong></button><button type="button" data-utility-route="cloud"><span>Substrate</span><strong>Cloud cells</strong></button></div></section>
    <div class="utility-summary"><span><strong data-activity-count="true">3</strong><small>unread</small></span><span><strong>12</strong><small>today</small></span><span><strong>3</strong><small>blocking</small></span></div>
    <div class="utility-filter-row" role="toolbar" aria-label="Activity filters"><button type="button" class="active" data-activity-filter="all">All</button><button type="button" data-activity-filter="unread">Unread</button><button type="button" data-activity-filter="blocking">Blocking</button><button type="button" data-activity-action="clear-read">Clear read</button></div>
    <ol class="activity-list" data-activity-list="true" aria-live="polite"><li data-activity-item="true" data-activity-state="unread" data-activity-severity="blocking"><time>09:18</time><span class="status-chip danger">blocking</span><strong>4대보험 변동 확인 필요</strong><p>Payroll close cannot seal until Park Seo-jun's insurance delta is reviewed.</p><button type="button" data-activity-action="mark-read">Mark read</button></li><li data-activity-item="true" data-activity-state="unread" data-activity-severity="review"><time>09:42</time><span class="status-chip warning">review</span><strong>Withholding tax brief ready</strong><p>HomeTax transport is staged locally; reviewer must approve before send.</p><button type="button" data-activity-action="mark-read">Mark read</button></li><li data-activity-item="true" data-activity-state="unread" data-activity-severity="blocking"><time>10:05</time><span class="status-chip danger">vendor</span><strong>Stripe renewal needs owner</strong><p>Spend approval exceeds one-step threshold and requires CFO attestation.</p><button type="button" data-activity-action="mark-read">Mark read</button></li><li data-activity-item="true" data-activity-state="read" data-activity-severity="info"><time>10:21</time><span class="status-chip success">sealed</span><strong>Audit receipt staged</strong><p>REC-FIN-2026-05 was added to the local close package preview.</p><button type="button" data-activity-action="open-audit">Open audit</button></li></ol>
  </section>
  <section class="utility-panel settings-center" data-utility-panel="settings" aria-label="Workspace settings" aria-hidden="true">
    <div class="utility-panel-head"><div><p class="screen-anchor">SETTINGS</p><h2>Workspace, profile, appearance, and integrations</h2></div><button type="button" data-utility-close="true" aria-label="Close settings">×</button></div><section class="utility-proof-strip settings-proof" aria-label="Settings FD-001 substrate proof"><article><p class="screen-anchor">CONTROL PLANE SETTINGS</p><strong>Workspace preferences stay tied to FD-001, policy, and Oyatie Cloud posture</strong><span>Every preference is local visual state; no auth, IAM, billing, integration, mail, or cloud mutation occurs.</span></article><div class="utility-route-grid" aria-label="Settings connected routes"><button type="button" data-utility-route="identity"><span>Identity</span><strong>Role envelope</strong></button><button type="button" data-utility-route="policy"><span>Policy</span><strong>Access matrix</strong></button><button type="button" data-utility-route="catalog"><span>Catalog</span><strong>Tenant modules</strong></button></div></section>
    <div class="settings-person-card"><span class="workspace-avatar" aria-hidden="true">최</span><div><strong>최유나 · Choi Yu-na</strong><p>Tenant admin · Finance owner · PIPA-safe contract envelope</p></div></div>
    <div class="settings-tabs" role="tablist" aria-label="Settings panels" aria-orientation="horizontal"><button type="button" id="settings-tab-profile" class="active" data-settings-tab="profile" role="tab" aria-selected="true" aria-controls="settings-panel-profile">Profile</button><button type="button" id="settings-tab-appearance" data-settings-tab="appearance" role="tab" aria-selected="false" aria-controls="settings-panel-appearance">Appearance</button><button type="button" id="settings-tab-integrations" data-settings-tab="integrations" role="tab" aria-selected="false" aria-controls="settings-panel-integrations">Integrations</button><button type="button" id="settings-tab-audit" data-settings-tab="audit" role="tab" aria-selected="false" aria-controls="settings-panel-audit">Audit</button></div>
    <article id="settings-panel-profile" class="settings-panel active" data-settings-panel="profile" role="tabpanel" aria-labelledby="settings-tab-profile"><dl class="settings-kv"><div><dt>Workspace</dt><dd>Oyatie Corp. · 118 employees</dd></div><div><dt>Role</dt><dd>Admin · payroll close approver</dd></div><div><dt>Region pack</dt><dd>US/EU/KR · Korean payroll enabled</dd></div></dl><button type="button" data-settings-action="open-identity">Open identity profile</button></article>
    <article id="settings-panel-appearance" class="settings-panel" data-settings-panel="appearance" role="tabpanel" aria-labelledby="settings-tab-appearance"><p>Adjust local visual density and shell language without changing server state.</p><div class="settings-action-grid"><button type="button" data-settings-action="density-comfortable">Comfortable</button><button type="button" data-settings-action="density-compact">Compact</button><button type="button" data-settings-action="locale-ko">한국어 우선</button><button type="button" data-settings-action="locale-en">English labels</button></div></article>
    <article id="settings-panel-integrations" class="settings-panel" data-settings-panel="integrations" role="tabpanel" aria-labelledby="settings-tab-integrations"><ol class="integration-list"><li><strong>Shinhan Bank</strong><span class="status-chip success">verified</span><small>Bank transport staged locally; no money movement.</small></li><li><strong>HomeTax</strong><span class="status-chip warning">review</span><small>Filing transport waits for human attestation.</small></li><li><strong>Google Workspace</strong><span class="status-chip">local</span><small>Mail and community previews only.</small></li></ol></article>
    <article id="settings-panel-audit" class="settings-panel" data-settings-panel="audit" role="tabpanel" aria-labelledby="settings-tab-audit"><ol class="activity-list compact"><li><time>09:14</time><strong>Settings drawer opened</strong><p>Local shell state only.</p></li><li><time>09:18</time><strong>Density preference staged</strong><p>Stored in this browser session.</p></li><li><time>09:42</time><strong>Identity panel linked</strong><p>No auth mutation.</p></li></ol></article>
    <p class="settings-status" data-settings-status="true">Local settings ready · no backend persistence.</p>
  </section>"#
        .to_string()
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_side_peek_html() -> String {
    r#"<aside class="side-peek" data-side-peek="true" aria-label="Object quick view" aria-hidden="true">
    <div class="side-peek-head"><div><p class="screen-anchor">OBJECT QUICK VIEW</p><h2 data-sidepeek-title-target="true">Network hot split</h2></div><button type="button" data-sidepeek-close="true" aria-label="Close object quick view">×</button></div>
    <div class="side-peek-body">
      <section class="quick-identity"><span class="workspace-avatar" aria-hidden="true">N</span><div><strong data-sidepeek-id-target="true">CHG-NTW-4182</strong><p data-sidepeek-desc-target="true">Tenant network split awaiting residency and rollback evidence.</p></div></section>
      <dl class="peek-kv"><div><dt>Owner</dt><dd data-sidepeek-owner-target="true">Infrastructure operations</dd></div><div><dt>Risk</dt><dd><span class="status-chip danger" data-sidepeek-risk-target="true">High</span></dd></div><div><dt>SLA</dt><dd data-sidepeek-sla-target="true">4.0h target · +1.4h over</dd></div><div><dt>Execution</dt><dd>Visual-only until live integration</dd></div></dl>
      <section class="side-peek-proof" aria-label="FD-001 object proof"><p class="screen-anchor">OBJECT PROOF</p><strong>Selected objects resolve to FD-001 workload evidence on Oyatie Cloud</strong><span data-sidepeek-status="true">Inspector ready · REC-WF-7741 · cell-us-east-2 · local visual state only.</span><div class="side-peek-route-grid" aria-label="Object proof routes"><button type="button" data-sidepeek-route="workload"><span>Workload</span><strong>FD-001 graph</strong></button><button type="button" data-sidepeek-route="cloud"><span>Cloud</span><strong>cell-us-east-2</strong></button><button type="button" data-sidepeek-route="evidence"><span>Receipt</span><strong>REC-WF-7741</strong></button></div></section>
      <section><h3>Evidence trail</h3><ol class="peek-timeline"><li><time>09:18</time><span>Policy guardrail matched residency rule for FD-001 tenant workload.</span></li><li><time>09:42</time><span>Oyatie Cloud rollback plan requested from network owner.</span></li><li><time>10:05</time><span>Audit-chain receipt REC-WF-7741 drafted locally.</span></li></ol></section>
      <section class="peek-actions" aria-label="Object actions"><button type="button" data-sidepeek-action="assign-owner">Assign owner</button><button type="button" data-sidepeek-action="draft-note">Draft note</button><button type="button" class="primary" data-sidepeek-action="review-evidence">Review evidence</button></section>
    </div>
  </aside>"#
        .to_string()
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_dashboard_content(envelope: &TenantRenderEnvelope) -> String {
    format!(
        r#"{context_switcher}
{surface_commands}
{product_activity}
{envelope_banner}
<section class="metric-grid" aria-label="Dashboard metrics">{metrics}</section>
{command_shell}
{substrate_proof}
{command_workbench}
{tenant_rbac}
{identity_service}
{finance_service}
{operator_intelligence}
{operations_cockpit}
{resource_audit}
<section class="dashboard-grid" aria-label="Personalized dashboard">{daily_execution}<section id="work-hub" class="panel communications-panel"><div class="panel-header"><p class="eyebrow">Messenger · Mail · Community</p><h3>Work hub</h3></div>{communication_hub}</section>{service_catalog}</section>
<section class="studio-grid" aria-label="Workflow, ontology, and intelligence">{workflow_studio}<section id="ontology-command-console" class="panel ontology-command-shell"><div class="panel-header"><p class="eyebrow">Ontology</p><h3 id="ontology-title">Tenant workload graph</h3></div>{ontology}</section><section id="intelligence-command-console" class="panel intelligence-command-shell"><div class="panel-header"><p class="eyebrow">Intelligence</p><h3 id="intelligence-title">Governed AI command</h3></div>{suggestions}</section></section>"#,
        context_switcher = static_context_switcher(envelope.context),
        surface_commands = static_surface_commands(),
        product_activity = static_product_activity_spine(&envelope.product_activity),
        envelope_banner = static_envelope_banner(envelope),
        metrics = envelope
            .metrics
            .iter()
            .map(static_metric)
            .collect::<String>(),
        command_shell = static_command_shell_substrate(),
        substrate_proof = static_substrate_proof_command(envelope),
        command_workbench = static_command_center_workbench(envelope),
        tenant_rbac = static_tenant_rbac_board(envelope),
        identity_service = static_identity_workforce_service(),
        finance_service = static_finance_commercial_service(),
        operator_intelligence = static_operator_intelligence_strip(envelope),
        operations_cockpit = static_tenant_operations_cockpit(envelope),
        resource_audit = static_resource_audit_console(envelope),
        daily_execution = static_daily_execution_console(envelope),
        communication_hub = static_communication_hub(&envelope.messages, &envelope.community),
        service_catalog = static_service_catalog(envelope),
        workflow_studio = static_workflow_studio_panel(envelope),
        ontology = static_ontology_command_console(&envelope.ontology, envelope.context),
        suggestions = static_intelligence_command_console(&envelope.intelligence),
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_context_switcher(active: OperatorContext) -> String {
    let cards = OperatorContext::ALL
        .iter()
        .map(|context| {
            let selected = if *context == active { " context-card selected" } else { " context-card" };
            let pressed = if *context == active { "true" } else { "false" };
            format!(
                "<button type=\"button\" class=\"{class}\" aria-pressed=\"{pressed}\"><span class=\"context-icon\" aria-hidden=\"true\">{icon}</span><span class=\"context-label\">{label}</span><span class=\"context-role\">{role}</span></button>",
                class = selected.trim(),
                pressed = pressed,
                icon = escape(context_icon(*context)),
                label = escape(context.label()),
                role = escape(context.role())
            )
        })
        .collect::<String>();

    format!(
        r#"<section class="context-switcher island-frame" aria-labelledby="context-title"><div><p class="eyebrow">Context</p><h2 id="context-title">Switch render envelope</h2><span class="island-label">interactive island</span></div><div class="context-grid" role="list" aria-label="Tenant and role contexts">{cards}</div></section>"#
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_envelope_banner(envelope: &TenantRenderEnvelope) -> String {
    let accreditation_class = if envelope.accreditation.healthcare_enabled {
        "badge success"
    } else {
        "badge warning"
    };

    format!(
        r#"<section class="envelope-banner" aria-labelledby="envelope-title"><div><p class="eyebrow">Active context</p><h2 id="envelope-title">{tenant}</h2><p>{role}</p></div><div class="envelope-detail"><span class="badge">{tenant_class}</span><span class="{accreditation_class}">{accreditation}</span><p>{derivation}</p></div></section>"#,
        tenant = escape(&envelope.tenant_name),
        role = escape(&envelope.role_name),
        tenant_class = escape(&envelope.tenant_class),
        accreditation_class = accreditation_class,
        accreditation = escape(&envelope.accreditation.label),
        derivation = escape(&envelope.server_derivation_note),
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_metric(metric: &MetricCard) -> String {
    format!(
        "<article class=\"metric-card\"><p>{}</p><strong>{}</strong><span>{}</span></article>",
        escape(&metric.label),
        escape(&metric.value),
        escape(&metric.detail)
    )
}
