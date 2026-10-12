use super::*;

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_surface_commands() -> String {
    let commands = ProductSurface::ALL
        .iter()
        .map(|surface| {
            let class = if *surface == ProductSurface::Workflow {
                "surface-command active"
            } else {
                "surface-command"
            };
            format!(
                r#"<a class="{class}" href="{href}"><span>{label}</span><small>{summary}</small></a>"#,
                class = class,
                href = surface.href(),
                label = escape(surface.label()),
                summary = escape(surface.summary())
            )
        })
        .collect::<String>();

    format!(
        r#"<nav class="surface-command-bar" aria-label="Open built-in product surface">{commands}</nav>"#
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_command_shell_substrate() -> String {
    let routes = COMMAND_SHELL_ROUTES
        .iter()
        .map(|(route, label, detail, target)| {
            let class = if *route == "fd001" {
                " class=\"selected\""
            } else {
                ""
            };
            format!(
                r#"<button type="button"{class} data-shell-context-route="{route}" data-shell-context-target="{target}"><strong>{label}</strong><span>{detail}</span></button>"#,
                class = class,
                route = escape(route),
                target = escape(target),
                label = escape(label),
                detail = escape(detail),
            )
        })
        .collect::<String>();

    format!(
        r##"<section id="command-shell-substrate" class="command-shell-substrate panel" aria-labelledby="command-shell-title" data-command-shell-substrate="true"><div class="command-shell-copy"><p class="screen-anchor">COMMAND SHELL SUBSTRATE</p><h3 id="command-shell-title">Every lower panel inherits the same active route, tenant lens, and local boundary</h3><span data-command-shell-status="true">FD-001 graph is active · lower surfaces will keep the route/status/inspector spine synchronized.</span></div><div class="command-shell-context" aria-live="polite"><span><small>Active route</small><strong data-command-shell-route="true">FD-001 graph</strong></span><span><small>Target</small><strong data-command-shell-target="true">#service-catalog</strong></span><span><small>Updated</small><strong data-command-shell-updated="true">SSR render</strong></span></div><div class="command-shell-routes" role="toolbar" aria-label="Lower dashboard product routes">{routes}</div></section>"##,
        routes = routes,
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_product_activity_spine(spine: &ProductActivitySpine) -> String {
    let active_label = spine
        .steps
        .iter()
        .find(|step| step.route_key == spine.active_route)
        .map(|step| step.label.as_str())
        .unwrap_or(&spine.active_route);
    let route_buttons = spine
        .steps
        .iter()
        .map(|step| {
            let class = if step.route_key == spine.active_route {
                " class=\"selected\""
            } else {
                ""
            };
            format!(
                r#"<button type="button"{class} data-activity-route="{route}" data-activity-target="{target}" data-activity-label="{label}" data-activity-detail="{detail}" data-activity-state="{state}"><strong>{label}</strong><span>{surface}</span></button>"#,
                class = class,
                route = escape(&step.route_key),
                target = escape(&step.target),
                label = escape(&step.label),
                detail = escape(&step.detail),
                state = escape(&step.state),
                surface = escape(&step.surface),
            )
        })
        .collect::<String>();

    let lane_steps = spine
        .steps
        .iter()
        .map(|step| {
            let class = if step.route_key == spine.active_route {
                "activity-step-card selected"
            } else {
                "activity-step-card"
            };
            format!(
                r#"<button type="button" class="{class}" data-activity-route="{route}" data-activity-target="{target}" data-activity-label="{label}" data-activity-detail="{detail}" data-activity-state="{state}" data-spine-step="{route}"><span>{surface}</span><strong>{label}</strong><small>{detail}</small><em>{state}</em></button>"#,
                class = class,
                route = escape(&step.route_key),
                target = escape(&step.target),
                label = escape(&step.label),
                detail = escape(&step.detail),
                state = escape(&step.state),
                surface = escape(&step.surface),
            )
        })
        .collect::<String>();

    format!(
        r#"<section id="product-activity-spine" class="product-activity-spine panel" aria-labelledby="product-activity-title" data-product-activity-spine="true"><div class="activity-spine-head"><div><p class="screen-anchor">PRODUCT ACTIVITY SPINE</p><h3 id="product-activity-title">One operating model for FD-001 tenant workloads on Oyatie Cloud</h3><span data-spine-active-context="true">{context}</span></div><div class="activity-spine-proof"><span data-spine-active-route="true">{route}</span><code data-spine-evidence-id="true">{evidence}</code><strong data-global-activity-status="true">{status}</strong></div></div><div class="activity-spine-grid"><aside class="activity-route-column" aria-label="Cross-surface routes"><p class="screen-anchor">ROUTES</p>{route_buttons}</aside><div class="activity-flow-lane" aria-label="FD-001 workload path">{lane_steps}</div><aside class="activity-inspector-card" aria-label="Selected route inspector"><p class="screen-anchor">INSPECTOR</p><h4 data-spine-inspector-title="true">FD-001 graph · product substrate</h4><p data-spine-inspector-body="true">Service catalog, workflow, Messenger, Mail, Community, cloud posture, and evidence receipts are one cohesive local operating graph.</p><dl><div><dt>Tenant</dt><dd data-spine-inspector-tenant="true">{context}</dd></div><div><dt>Boundary</dt><dd>Visual-only · no backend write</dd></div><div><dt>Receipt</dt><dd>{evidence}</dd></div></dl><div class="activity-inspector-actions"><button type="button" data-activity-route="workflow">Open Workflow</button><button type="button" data-activity-route="mail">Mail brief</button><button type="button" data-activity-route="evidence">Evidence</button></div></aside></div><div class="activity-spine-statusbar" aria-label="Current local shell state"><span>SSR shell</span><span>Selective WASM islands</span><span>Local-only actions</span><span data-spine-last-action="true">Ready · route and inspector state will update visually</span></div></section>"#,
        context = escape(&spine.active_context),
        route = escape(active_label),
        evidence = escape(&spine.evidence_id),
        status = escape(&spine.status_label),
        route_buttons = route_buttons,
        lane_steps = lane_steps,
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_workflow_studio_panel(envelope: &TenantRenderEnvelope) -> String {
    let owner = Owner::new();
    let html = owner.with(|| {
        let sample = RwSignal::new(WorkflowSample::from_envelope(envelope));
        workflow_studio_panel(envelope.clone(), sample)
            .into_view()
            .to_html()
    });
    owner.cleanup();
    html
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_substrate_proof_command(envelope: &TenantRenderEnvelope) -> String {
    format!(
        r#"<section id="substrate-proof" class="substrate-proof-command panel" data-substrate-proof="true" aria-labelledby="substrate-proof-title"><div class="substrate-proof-head"><div><p class="screen-anchor">OYATIE CLOUD · FD-001 DOGFOOD SUBSTRATE</p><h3 id="substrate-proof-title">Prove production tenancy by running FD-001 as real tenant workloads</h3><p>FD-001 remains the product delivery goal. Oyatie Cloud is the hyperscaler-grade substrate proving those microservices can host production tenants before any external claim.</p></div><div class="substrate-proof-actions"><span class="status-chip success" data-substrate-status="true">12 workloads · 3 cells · 0 external writes</span><button type="button" data-substrate-action="cloud">Cloud cells</button><button type="button" data-substrate-action="workflow">Workflow proof</button><button type="button" data-substrate-action="evidence">Evidence</button></div></div><div class="substrate-proof-grid" aria-label="Substrate proof metrics"><article class="substrate-proof-card primary"><p class="screen-anchor">PRODUCT GOAL</p><strong>FD-001 delivery</strong><span>Core, workflow, messenger, mail, community, finance, identity, intelligence, and ontology run as tenant workload previews.</span></article><article class="substrate-proof-card"><p class="screen-anchor">SUBSTRATE</p><strong>Oyatie Cloud</strong><span>Cellular runtime, policy, FinOps, resource inventory, deployment gates, and rollback evidence.</span></article><article class="substrate-proof-card"><p class="screen-anchor">TENANT LENS</p><strong>{tenant}</strong><span>{role} · server-derived envelope · local dogfood only</span></article><article class="substrate-proof-card warning"><p class="screen-anchor">READINESS</p><strong>84% proof</strong><span>3 blockers: payroll delta, cloud rollback receipt, PIPA review.</span></article></div><div class="substrate-workload-map" aria-label="FD-001 tenant workload deployment map"><div class="substrate-map-column substrate-product-column"><p class="screen-anchor">FD-001 WORKLOADS</p><button type="button" data-substrate-action="workflow"><strong>Workflow</strong><span>approval engine · no-code studio</span></button><button type="button" data-substrate-action="messenger"><strong>Messenger</strong><span>ops room thread · evidence extraction</span></button><button type="button" data-substrate-action="mail"><strong>Mail</strong><span>formal approval brief</span></button><button type="button" data-substrate-action="community"><strong>Community</strong><span>governance council post</span></button></div><div class="substrate-map-spine" aria-hidden="true"><span>tenant workload</span><i></i><span>cell runtime</span><i></i><span>evidence receipt</span></div><div class="substrate-map-column substrate-cloud-column"><p class="screen-anchor">OYATIE CLOUD CELLS</p><button type="button" data-substrate-action="cloud"><strong>cell-us-east-2</strong><span>primary · workload dogfood</span></button><button type="button" data-substrate-action="finops"><strong>kr-seoul-1</strong><span>localization pack · FinOps watch</span></button><button type="button" data-substrate-action="deployment"><strong>gitops promotion</strong><span>Jenkins · ArgoCD · cosign · audit</span></button><button type="button" data-substrate-action="evidence"><strong>evidence spine</strong><span>REC-FD001-CLOUD-009</span></button></div></div><div class="substrate-proof-footer" aria-label="Dogfood proof routes"><span>Proof loop: tenant workload → Oyatie Cloud cell → policy gate → human route → evidence receipt</span><button type="button" data-substrate-action="finance">Finance close</button><button type="button" data-substrate-action="identity">Identity policy</button><button type="button" data-substrate-action="catalog">Service catalog</button></div></section>"#,
        tenant = escape(&envelope.tenant_name),
        role = escape(&envelope.role_name),
    )
}
