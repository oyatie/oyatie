use super::*;

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_rail_html() -> String {
    format!(
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
    <a class="rail-nav" href="#console-shell"><span aria-hidden="true">⌂</span>Command center</a>
    <a class="rail-nav" href="#command-center-workbench"><span aria-hidden="true">▥</span>Action Inbox</a>
    {governance_rail}
    <p class="rail-group">Operate</p>
    {logic_rail}
    <a class="rail-nav" href="#tasks-title"><span aria-hidden="true">☑</span>Tasks<em>73</em></a>
    <a class="rail-nav" href="#schedule-title"><span aria-hidden="true">◷</span>Schedule</a>
    <a class="rail-nav" href="#workflow-studio"><span aria-hidden="true">⌘</span>Workflow Studio</a>
    <a class="rail-nav" href="#work-hub"><span aria-hidden="true">✉</span>Messenger · Mail · Community<em>18</em></a>
    <a class="rail-nav" href="#cloud-ops-cockpit"><span aria-hidden="true">◫</span>Cloud Ops</a>
    <p class="rail-group">Money</p>
    <a class="rail-nav" href="#payroll-cockpit"><span aria-hidden="true">₩</span>Payroll</a>
<span data-finance-rail-slot="commercial"></span>
<span data-finance-rail-slot="finops"></span>
    <p class="rail-group">Compliance</p>
<span data-finance-rail-slot="filing"></span>
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
  </aside>"##,
        governance_rail = governance_rail_html(OperatorContext::TenantAdmin, false),
        logic_rail = governance_rail_html(OperatorContext::TenantAdmin, true)
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_header_html() -> String {
    r#"<header class="app-header" role="banner">
    <div class="top-breadcrumb" aria-label="Breadcrumb"><span>Oyatie Cloud</span><span class="sep">/</span><span>Operations</span><span class="sep">/</span><strong>Control Center</strong></div>
    <div class="header-route-strip" aria-label="FD-001 and Oyatie Cloud quick routes"><button type="button" class="is-selected" data-header-route="fd001"><span>FD-001</span>Service graph</button><button type="button" data-header-route="cloud"><span>Cloud</span>Substrate</button><button type="button" data-header-route="work-hub"><span>Comms</span>Work hub</button><div class="header-comms-switcher" aria-label="Built-in communications quick routes"><button type="button" class="is-selected" data-header-comms-surface="Messenger">Messenger</button><button type="button" data-header-comms-surface="Mail">Mail</button><button type="button" data-header-comms-surface="Community">Community</button></div><button type="button" data-header-route="evidence"><span>Audit</span>Evidence</button><small data-header-route-status="true">REC-WF-7741 · local quick routes</small></div>
    <button class="command-trigger" type="button" data-command-trigger="true" aria-haspopup="dialog"><span aria-hidden="true">⌕</span><span>Search actions, objects, workflows</span><kbd>⌘K</kbd></button>
    <div class="header-actions" aria-label="Shell render status"><button type="button" class="header-status">SSR shell</button><button type="button" class="header-status muted">Selective WASM islands</button><button type="button" class="header-icon" data-header-action="notifications" aria-label="Open sample notifications" aria-controls="notification-center" aria-haspopup="dialog" commandfor="notification-center" command="show-modal">◔<span class="header-badge" data-activity-badge="true" aria-hidden="true">NOTIFICATION_UNREAD</span></button><button type="button" class="header-icon" data-header-action="settings" aria-label="Open settings" aria-controls="settings-center" aria-haspopup="dialog" commandfor="settings-center" command="show-modal">⚙</button></div>
  </header>"#
        .replace("NOTIFICATION_UNREAD", &notification_unread_count().to_string())
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_hero_html() -> String {
    r#"<section class="hero-panel" aria-labelledby="console-title"><div class="hero-main"><div class="page-title-copy"><p class="screen-anchor">01 / Command Center</p><div class="hero-title-row"><h1 id="console-title">Operations · 2026 May, week 19</h1><span class="hero-lens-chip">● Lens: tenant admin · Finance · 1,000 ppl</span></div><p id="console-notice" class="scope-notice" role="note">Sample dashboard · anonymous preview only: sample tenant and role; no PHI/PII. This preview does not verify a tenant or role and does not query live service status. The separate live Ontology card requires sign-in.</p></div><section class="hero-close-strip" aria-label="FD-001 close command proof"><div><p class="screen-anchor">FD-001 CLOSE COMMAND</p><strong>April close proves the product workload on Oyatie Cloud</strong><span data-hero-status="true">Ready · REC-CLOSE-2026-04 · cell-us-east-2 · local command only</span></div><div class="hero-close-actions" aria-label="Close package routes"><button type="button" data-hero-action="close-april">Stage close</button><button type="button" data-hero-action="route-ledger">Ledger</button><button type="button" data-hero-action="route-cloud">Cloud proof</button><button type="button" data-hero-action="route-evidence">Evidence</button></div></section><section class="render-architecture-strip" aria-label="SSR shell and selective WASM hydration model"><article class="selected" data-render-arch-card="ssr"><p class="screen-anchor">SSR SHELL</p><strong>Fast baseline, service graph visible first</strong><span>Navigation, proof copy, tenant posture, and core dashboards render before island hydration.</span><button type="button" class="is-selected" data-render-arch-action="ssr">Show shell</button></article><article data-render-arch-card="islands"><p class="screen-anchor">SELECTIVE WASM</p><strong>Only interactive product surfaces hydrate</strong><span>Workflow Studio, Work Hub, filters, canvas state, and local drafts become browser-only islands.</span><button type="button" data-render-arch-action="islands">Show islands</button></article><article data-render-arch-card="boundary"><p class="screen-anchor">LOCAL BOUNDARY</p><strong>Visually functional, deliberately unwired</strong><span data-render-arch-status="true">Sample dashboard actions do not execute workflows, send messages, or change IAM, billing, deployments, or cloud resources.</span><button type="button" data-render-arch-action="boundary">Show evidence</button></article></section></div><div class="hero-side"><div class="hero-copy page-actions"><button type="button" data-sidepeek-trigger="new-action" data-sidepeek-title="Create governed action" data-sidepeek-id="ACT-LOCAL-DRAFT" data-sidepeek-desc="Local visual-only action draft. Nothing is persisted or sent." data-sidepeek-owner="Current operator session" data-sidepeek-risk="Draft" data-sidepeek-sla="No live SLA">New action</button><button type="button" data-command-trigger="true">Search ⌘K</button><button type="button" class="primary" data-hero-action="close-april">Close April →</button></div></div></section>"#
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
        <button type="button" data-command-action="notifications" aria-controls="notification-center" aria-haspopup="dialog" commandfor="notification-center" command="show-modal"><strong>Open sample notifications</strong><span>Read illustrative notification examples</span><kbd>N</kbd></button>
        <button type="button" data-command-action="settings"><strong>Open Workspace Settings</strong><span>Profile, density, integrations, and audit preferences</span><kbd>S</kbd></button>
      </div>
      <div class="command-palette-footer"><span>Local-only: commands route the visual shell without backend, workflow, mail, IAM, billing, deploy, or cloud mutation.</span><button type="button" data-command-proof-action="local-boundary">Show boundary</button></div>
    </section>
  </div>"#
        .to_string()
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_utility_panels_html() -> String {
    notification_dialog_html()
        + &format!(
            r#"<dialog id="settings-center" class="utility-panel settings-center" data-utility-panel="settings" data-settings-density="comfortable" aria-label="Settings · sample preview" closedby="any">{}</dialog>"#,
            settings_body_html()
        )
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
<section class="dashboard-grid" aria-label="Personalized dashboard">{daily_execution}<section id="work-hub" class="panel communications-panel"><div class="panel-header"><p class="eyebrow">Messenger · Mail · Community</p><h3>Work hub</h3></div>{payroll_mail_preview}{communication_hub}</section>{service_catalog}</section>
<section class="studio-grid" aria-label="Workflow, ontology, and intelligence">{workflow_studio}<section id="ontology-command-console" class="panel ontology-command-shell"><div class="panel-header"><p class="eyebrow">Ontology</p><h3 id="ontology-title">Tenant workload graph</h3></div>{ontology}</section>{suggestions}</section>"#,
        context_switcher = static_context_switcher(envelope),
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
        identity_service = static_identity_workforce_service(envelope),
        finance_service = static_finance_commercial_service(envelope),
        operator_intelligence = static_operator_intelligence_strip(envelope),
        operations_cockpit = static_tenant_operations_cockpit(envelope),
        resource_audit = static_resource_audit_console(envelope),
        daily_execution = static_daily_execution_console(envelope),
        payroll_mail_preview = static_payroll_preview_detail(PayrollPreview::Mail),
        communication_hub = static_communication_hub(envelope),
        service_catalog = static_service_catalog(envelope),
        workflow_studio = static_workflow_studio_panel(envelope),
        ontology = static_ontology_command_console(&envelope.ontology, envelope.context),
        suggestions = static_intelligence_command_console(envelope),
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_context_switcher(envelope: &TenantRenderEnvelope) -> String {
    let active = envelope.context;
    let cards = OperatorContext::ALL
        .iter()
        .map(|context| {
            let selected = if *context == active { " context-card selected" } else { " context-card" };
            let pressed = if *context == active { "true" } else { "false" };
            format!(
                "<button type=\"button\" data-sample-context=\"{id}\" disabled class=\"{class}\" aria-pressed=\"{pressed}\"><span class=\"context-icon\" aria-hidden=\"true\">{icon}</span><span class=\"context-label\">{label} sample</span></button>",
                id = context.id(),
                class = selected.trim(),
                pressed = pressed,
                icon = escape(context_icon(*context)),
                label = escape(context.label()),
            )
        })
        .collect::<String>();

    format!(
        r#"<section class="context-switcher sample-context-selector island-frame" aria-labelledby="context-title" aria-describedby="sample-context-notice sample-context-status" data-sample-context-requested="{id}" data-sample-context-displayed="{id}"><div><h2 id="context-title">Anonymous sample context</h2><p id="sample-context-notice" class="sample-context-notice">{notice}</p></div><div class="context-grid" role="group" aria-label="Anonymous sample contexts">{cards}</div><p id="sample-context-status" class="sample-context-status" role="status">{status}</p></section>"#,
        id = active.id(),
        notice = escape(sample_context_notice_text()),
        status = escape(&sample_context_status(
            active,
            Some(envelope),
            false,
            false,
            false
        )),
    )
}

#[cfg(test)]
#[test]
fn static_sample_selector_matches_displayed_source_and_binding_availability() {
    for active in OperatorContext::ALL {
        let source = server_derived_envelope(active);
        let html = static_context_switcher(&source);
        assert!(html.contains(sample_context_notice_text()));
        assert!(html.contains(&format!(
            "data-sample-context-requested=\"{}\"",
            active.id()
        )));
        assert!(html.contains(&format!(
            "data-sample-context-displayed=\"{}\"",
            active.id()
        )));
        assert!(
            html.contains(&escape(&source.tenant_name))
                && html.contains(&escape(&source.role_name))
        );
        assert_eq!(
            html.matches(" disabled ").count(),
            OperatorContext::ALL.len()
        );
        assert_eq!(html.matches("aria-pressed=\"true\"").count(), 1);
        assert!(!html.contains("again to refresh") && !html.contains("again to retry"));
        for context in OperatorContext::ALL {
            let card = html
                .split_once(&format!("data-sample-context=\"{}\"", context.id()))
                .unwrap()
                .1
                .split_once("</button>")
                .unwrap()
                .0;
            assert!(card.contains(&format!("aria-pressed=\"{}\"", context == active)));
            assert!(card.contains(&format!("{} sample", context.label())));
        }
    }
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
