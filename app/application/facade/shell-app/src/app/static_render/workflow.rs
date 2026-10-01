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
pub(super) fn static_workflow_palette() -> String {
    r#"<aside class="workflow-palette" aria-label="Workflow building blocks"><div class="palette-search"><span aria-hidden="true">⌕</span><input data-workflow-palette-search="true" aria-label="Search workflow blocks" placeholder="Search nodes..." /><kbd>⌘K</kbd></div><div class="palette-heading"><span>Primitives</span><em>12</em></div><button type="button" data-palette-item="primitive"><span>System task</span><small>Deterministic step · 0 ms</small><kbd>S</kbd></button><button type="button" data-palette-item="primitive"><span>Approval</span><small>Single, parallel, or quorum</small><kbd>A</kbd></button><button type="button" data-palette-item="primitive"><span>Validation</span><small>Rule check · halts on fail</small><kbd>V</kbd></button><button type="button" data-palette-item="primitive"><span>External call</span><small>HTTP, RPC, or connector</small><kbd>E</kbd></button><button type="button" data-palette-item="primitive"><span>Branch / Switch</span><small>Multi-way condition split</small><kbd>B</kbd></button><button type="button" data-palette-item="primitive"><span>Wait / Timer</span><small>Until time · or duration</small><kbd>W</kbd></button><button type="button" data-palette-item="primitive"><span>Loop / For-each</span><small>Iterate over collection</small><kbd>L</kbd></button><button type="button" data-palette-item="primitive"><span>AI step</span><small>Suggest · classify · extract</small><kbd>⌥A</kbd></button><button type="button" data-palette-item="primitive"><span>중단 / 에스컬레이트</span><small>CFO 알림 · 실행 중단</small><kbd>H</kbd></button><button type="button" data-palette-item="primitive"><span>Form / Input</span><small>Collect data from human</small><kbd>F</kbd></button><button type="button" data-palette-item="primitive"><span>Webhook trigger</span><small>Inbound event start</small><kbd>T</kbd></button><button type="button" data-palette-item="primitive"><span>End / Receipt</span><small>Emit immutable event</small><kbd>⌘E</kbd></button><div class="palette-heading"><span>Actions</span><em>6</em></div><button type="button" data-palette-item="action"><span>Task</span><small>Create a governed work item</small><kbd>T</kbd></button><button type="button" data-palette-item="action"><span>HTTP request</span><small>Call external REST/HTTP</small><kbd>H</kbd></button><button type="button" data-palette-item="action"><span>Database</span><small>Read/write a record</small><kbd>D</kbd></button><button type="button" data-palette-item="action"><span>Transform</span><small>Reshape the payload</small><kbd>X</kbd></button><button type="button" data-palette-item="action"><span>Filter</span><small>Drop failed items</small><kbd>F</kbd></button><button type="button" data-palette-item="action"><span>Write to doc</span><small>Append a row / line</small><kbd>W</kbd></button><div class="palette-heading"><span>Logic</span><em>5</em></div><button type="button" data-palette-item="logic"><span>If / Branch</span><small>Two-way condition split</small><kbd>I</kbd></button><button type="button" data-palette-item="logic"><span>Switch</span><small>Multi-way routing</small><kbd>S</kbd></button><button type="button" data-palette-item="logic"><span>Loop / For-each</span><small>Iterate collection</small><kbd>L</kbd></button><button type="button" data-palette-item="logic"><span>Wait</span><small>Delay or duration</small><kbd>W</kbd></button><button type="button" data-palette-item="logic"><span>Merge</span><small>Wait for branches</small><kbd>M</kbd></button><div class="palette-heading"><span>Built-in surfaces</span><em>3</em></div><button type="button" data-palette-item="surface"><span>Messenger post</span><small>Route run summary to Ops room</small><kbd>M</kbd></button><button type="button" data-palette-item="surface"><span>Mail draft</span><small>Formal approval note</small><kbd>⌘M</kbd></button><button type="button" data-palette-item="surface"><span>Community note</span><small>Publish governed update</small><kbd>C</kbd></button><div class="palette-heading"><span>Connectors</span><em>9</em></div><div class="workflow-connector-grid" aria-label="Workflow connector shortcuts"><button type="button" data-palette-item="connector"><strong>국세</strong><span>HomeTax</span></button><button type="button" data-palette-item="connector"><strong>국민</strong><span>NPS / 4대</span></button><button type="button" data-palette-item="connector"><strong>신한</strong><span>Shinhan</span></button><button type="button" data-palette-item="connector"><strong>T</strong><span>Toss</span></button><button type="button" data-palette-item="connector"><strong>K</strong><span>Kakao Work</span></button><button type="button" data-palette-item="connector"><strong>#</strong><span>Slack</span></button><button type="button" data-palette-item="connector"><strong>G</strong><span>Workspace</span></button><button type="button" data-palette-item="connector"><strong>Q</strong><span>QuickBooks</span></button><button type="button" data-palette-item="connector"><strong>N</strong><span>Notion</span></button></div></aside>"#.to_string()
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_workflow_process_chrome() -> &'static str {
    r#"<div class="workflow-process-chrome" aria-label="Workflow process command chrome"><div class="workflow-process-meta"><span>PROCESS</span><strong>PROC-PAYROLL-CLOSE</strong><span>OWNER</span><strong>Hyo-jin Park · #274</strong><span>SLA</span><strong>4.0d</strong></div><div class="workflow-process-actions"><button type="button" data-workflow-process-action="validate">✓ Validate</button><button type="button" data-workflow-process-action="simulate">◷ Simulate</button><button type="button" data-workflow-process-action="diff">↯ Diff v17 → v18</button><button type="button" class="dark-action" data-workflow-process-action="publish">게시 v18</button></div><span class="workflow-process-status" data-workflow-process-status="true">autosaved · local visual IDE</span></div>"#
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_workflow_lens_toolbar() -> &'static str {
    r#"<div class="workflow-lens-toolbar" aria-label="Workflow layout, overlay, and filter controls"><div class="workflow-lens-group"><span>LAYOUT</span><button type="button" class="active" data-workflow-lens="Graph">Graph</button><button type="button" data-workflow-lens="Swimlanes">Swimlanes</button><button type="button" data-workflow-lens="Timeline">Timeline</button><button type="button" data-workflow-lens="Tree">Tree</button></div><div class="workflow-lens-group"><span>OVERLAY</span><button type="button" data-workflow-overlay="Cycle">Cycle</button><button type="button" class="active" data-workflow-overlay="Cost">Cost</button><button type="button" data-workflow-overlay="Owner">Owner</button><button type="button" data-workflow-overlay="Risk">Risk</button><button type="button" data-workflow-overlay="Off">Off</button></div><div class="workflow-lens-group"><span>FILTER</span><button type="button" data-workflow-filter="All">All</button><button type="button" class="active" data-workflow-filter="Critical path">Critical path</button><button type="button" data-workflow-filter="Bottlenecks">Bottlenecks</button><button type="button" data-workflow-filter="AI suggestions">AI suggestions</button></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_workflow_canvas_metrics() -> &'static str {
    r#"<div class="workflow-canvas-metrics" aria-label="Workflow simulation metrics overlay"><span><small>CYCLE</small><strong>5.4d</strong><em>+1.4 vs target</em></span><span><small>TARGET</small><strong>4.0d</strong><em>SLA limit</em></span><span><small>COST</small><strong>₩2.18M</strong><em>delay cost</em></span><span><small>REWORK</small><strong>8%</strong><em>2 loops</em></span></div>"#
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_workflow_property_form() -> &'static str {
    r#"<form class="workflow-property-form" aria-label="Selected workflow node properties"><label><span>LABEL · KO</span><input data-workflow-prop="label-ko" value="재무 검토 · 사인오프" /></label><label><span>TYPE</span><select data-workflow-prop="type"><option>Single · auto-delegate</option><option>Parallel quorum</option><option>Human review stop</option></select></label><label><span>OWNER</span><select data-workflow-prop="owner"><option>Sarah Kim · EMP-188 · HR Manager</option><option>Choi Yu-na · CFO</option><option>David Chen · Delegate</option></select></label><div class="workflow-form-row"><label><span>SLA TARGET</span><input data-workflow-prop="sla" value="1.2d" /></label><label><span>ESCALATE AFTER</span><input data-workflow-prop="escalate" value="0.8d" /></label></div><fieldset class="workflow-rule-stack"><legend>승인 조건</legend><label><span>1</span><input data-workflow-prop="rule-1" value="payroll.gross > ₩500,000,000" /></label><label><span>2</span><input data-workflow-prop="rule-2" value="policy.P0 == active" /></label><button type="button" data-workflow-process-action="add-condition">+ Add condition</button></fieldset></form>"#
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_workflow_output_bus() -> &'static str {
    r#"<div class="workflow-output-bus" data-workflow-output-bus="true" aria-label="Workflow output bus for FD-001 tenant workload routes"><div class="workflow-output-head"><p class="screen-anchor">FD-001 OUTPUT BUS</p><strong>Run preview emits tenant workload drafts</strong><span data-workflow-output-status="true">Idle · run/validate/publish stays local until a route is selected</span></div><div class="workflow-output-flow" aria-label="Workflow output routes"><button type="button" class="selected" data-workflow-output-route="messenger"><span>01</span><strong>Messenger</strong><em>Ops room run note</em></button><button type="button" data-workflow-output-route="mail"><span>02</span><strong>Mail</strong><em>Approval brief</em></button><button type="button" data-workflow-output-route="community"><span>03</span><strong>Community</strong><em>Council digest</em></button><button type="button" data-workflow-output-route="evidence"><span>04</span><strong>Evidence</strong><em>Receipt spine</em></button></div><aside class="workflow-output-proof" aria-label="FD-001 and Oyatie Cloud proof context"><dl><div><dt>Product goal</dt><dd>FD-001 delivery</dd></div><div><dt>Substrate</dt><dd>Oyatie Cloud · cell-us-east-2</dd></div><div><dt>Receipt</dt><dd data-workflow-output-receipt="true">REC-FD001-WF-018 · draft</dd></div></dl></aside></div>"#
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_workflow_studio_panel(envelope: &TenantRenderEnvelope) -> String {
    let display_nodes = workflow_display_nodes(&envelope.workflow.nodes, 0);
    let nodes = display_nodes
        .iter()
        .map(|node| format!("<button type=\"button\">{}</button>", escape(&node.label)))
        .collect::<String>();
    let selected_node = display_nodes
        .first()
        .map(static_selected_node)
        .unwrap_or_default();

    format!(
        r#"<section id="workflow-studio" class="panel workflow-panel cohesive-workflow" aria-labelledby="workflow-title"><div class="workflow-topbar"><div><p class="eyebrow">Workflow Studio</p><h3 id="workflow-title">{name}</h3><div class="workflow-doc-meta"><span>v18 · draft</span><span>Owner · tenant admin</span><span>SLA · 4.0h</span></div></div><div class="workflow-run-chip"><span></span>draft · select mode</div><div class="workflow-actions"><button type="button">Fit</button><button type="button">Clear run</button><button type="button">Validate</button><button class="primary-action" type="button">Run</button><button type="button">Add block</button><button class="dark-action" type="button">Publish</button></div></div>{workflow_process_chrome}{workflow_output_bus}<p class="panel-intro">{goal}</p><div class="workflow-modebar" role="toolbar" aria-label="Workflow editor modes"><button type="button" class="active">Select</button><button type="button">Connect</button><button type="button">Simulate</button></div>{workflow_lens_toolbar}<div class="workflow-ide">{workflow_palette}<div class="workflow-canvas island-frame"><div class="workflow-toolbar"><button type="button">Select</button><button type="button">Connect</button><button type="button">Simulate</button><span class="island-label">interactive island</span></div>{workflow_board}<div class="canvas-footer"><div class="zoom-controls"><button type="button">−</button><span>82%</span><button type="button">+</button></div><div class="mini-map" aria-hidden="true"><span></span><span></span><span></span><span></span></div></div><div class="node-toolbar">{nodes}</div></div><aside class="workflow-inspector"><div class="inspector-tabs" aria-hidden="true"><span class="active">Inspector</span><span>Run log</span><span>Rules</span><span>History</span></div>{selected_node}{workflow_property_form}<dl class="inspector-fields"><div><dt>Guardrail</dt><dd>Human review before action</dd></div><div><dt>Output</dt><dd>Task · message · evidence draft</dd></div><div><dt>Execution</dt><dd>Disabled until live integration</dd></div></dl><div class="inspector-stat-grid" aria-label="Selected node run statistics"><div><span>Avg</span><strong>0.8s</strong></div><div><span>P95</span><strong>2.1s</strong></div><div><span>Errors</span><strong>0</strong></div><div><span>Cost</span><strong>$0.03</strong></div></div><div class="run-log-preview"><p class="eyebrow">Run log</p><ol><li><time>10:31</time><span>Validation preview passed</span></li><li><time>10:32</time><span>Messenger/Mail/Community drafts generated</span></li><li><time>10:33</time><span>Audit receipt staged locally</span></li></ol></div></aside></div><div class="workflow-statusbar"><span>Nodes: {node_count}</span><span>Local blocks: 0</span><span>Messenger/Mail/Community outputs are drafts</span><span>Ready · staged</span></div></section>"#,
        name = escape(&envelope.workflow.name),
        goal = escape(&envelope.workflow.goal),
        workflow_board = static_workflow_board(&display_nodes),
        workflow_palette = static_workflow_palette(),
        workflow_process_chrome = static_workflow_process_chrome(),
        workflow_output_bus = static_workflow_output_bus(),
        workflow_lens_toolbar = static_workflow_lens_toolbar(),
        workflow_property_form = static_workflow_property_form(),
        nodes = nodes,
        selected_node = selected_node,
        node_count = display_nodes.len(),
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_substrate_proof_command(envelope: &TenantRenderEnvelope) -> String {
    format!(
        r#"<section id="substrate-proof" class="substrate-proof-command panel" data-substrate-proof="true" aria-labelledby="substrate-proof-title"><div class="substrate-proof-head"><div><p class="screen-anchor">OYATIE CLOUD · FD-001 DOGFOOD SUBSTRATE</p><h3 id="substrate-proof-title">Prove production tenancy by running FD-001 as real tenant workloads</h3><p>FD-001 remains the product delivery goal. Oyatie Cloud is the hyperscaler-grade substrate proving those microservices can host production tenants before any external claim.</p></div><div class="substrate-proof-actions"><span class="status-chip success" data-substrate-status="true">12 workloads · 3 cells · 0 external writes</span><button type="button" data-substrate-action="cloud">Cloud cells</button><button type="button" data-substrate-action="workflow">Workflow proof</button><button type="button" data-substrate-action="evidence">Evidence</button></div></div><div class="substrate-proof-grid" aria-label="Substrate proof metrics"><article class="substrate-proof-card primary"><p class="screen-anchor">PRODUCT GOAL</p><strong>FD-001 delivery</strong><span>Core, workflow, messenger, mail, community, finance, identity, intelligence, and ontology run as tenant workload previews.</span></article><article class="substrate-proof-card"><p class="screen-anchor">SUBSTRATE</p><strong>Oyatie Cloud</strong><span>Cellular runtime, policy, FinOps, resource inventory, deployment gates, and rollback evidence.</span></article><article class="substrate-proof-card"><p class="screen-anchor">TENANT LENS</p><strong>{tenant}</strong><span>{role} · server-derived envelope · local dogfood only</span></article><article class="substrate-proof-card warning"><p class="screen-anchor">READINESS</p><strong>84% proof</strong><span>3 blockers: payroll delta, cloud rollback receipt, PIPA review.</span></article></div><div class="substrate-workload-map" aria-label="FD-001 tenant workload deployment map"><div class="substrate-map-column substrate-product-column"><p class="screen-anchor">FD-001 WORKLOADS</p><button type="button" data-substrate-action="workflow"><strong>Workflow</strong><span>approval engine · no-code studio</span></button><button type="button" data-substrate-action="messenger"><strong>Messenger</strong><span>ops room thread · evidence extraction</span></button><button type="button" data-substrate-action="mail"><strong>Mail</strong><span>formal approval brief</span></button><button type="button" data-substrate-action="community"><strong>Community</strong><span>governance council post</span></button></div><div class="substrate-map-spine" aria-hidden="true"><span>tenant workload</span><i></i><span>cell runtime</span><i></i><span>evidence receipt</span></div><div class="substrate-map-column substrate-cloud-column"><p class="screen-anchor">OYATIE CLOUD CELLS</p><button type="button" data-substrate-action="cloud"><strong>cell-us-east-2</strong><span>primary · workload dogfood</span></button><button type="button" data-substrate-action="finops"><strong>kr-seoul-1</strong><span>localization pack · FinOps watch</span></button><button type="button" data-substrate-action="deployment"><strong>gitops promotion</strong><span>Jenkins · ArgoCD · cosign · audit</span></button><button type="button" data-substrate-action="evidence"><strong>evidence spine</strong><span>REC-FD001-CLOUD-009</span></button></div></div><div class="substrate-proof-footer" aria-label="Dogfood proof routes"><span>Proof loop: tenant workload → Oyatie Cloud cell → policy gate → human route → evidence receipt</span><button type="button" data-substrate-action="finance">Finance close</button><button type="button" data-substrate-action="identity">Identity policy</button><button type="button" data-substrate-action="catalog">Service catalog</button></div></section>"#,
        tenant = escape(&envelope.tenant_name),
        role = escape(&envelope.role_name),
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_selected_node(node: &WorkflowNode) -> String {
    format!(
        "<aside class=\"node-inspector\"><p class=\"eyebrow\">Selected node</p><h4>{}</h4><p><strong>{}</strong> · {}</p></aside>",
        escape(&node.label),
        escape(&node.kind),
        escape(&node.explanation)
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_workflow_board(nodes: &[WorkflowNode]) -> String {
    let edges = board_edges(nodes)
        .into_iter()
        .map(|(from, to, path)| {
            format!(
                "<path class=\"workflow-edge workflow-board-edge\" data-edge-from=\"{}\" data-edge-to=\"{}\" d=\"{}\" marker-end=\"url(#workflow-board-arrow)\"></path>",
                escape(&from),
                escape(&to),
                escape(&path)
            )
        })
        .collect::<String>();
    let workflow_canvas_metrics = static_workflow_canvas_metrics();
    let cards = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let class = if index == 0 {
                "workflow-card active selectable"
            } else {
                "workflow-card selectable"
            };
            format!(
                r#"<button type="button" class="{class}" style="left: {x}px; top: {y}px" data-workflow-card="true" data-node-id="{id}" data-node-label="{label}" data-node-kind="{kind}" data-node-desc="{desc}"><span class="board-port in" aria-hidden="true"></span><span class="board-port out" aria-hidden="true"></span><span class="workflow-card-type">{kind}</span><strong>{label}</strong><small>{desc}</small></button>"#,
                class = class,
                x = workflow_board_x(index),
                y = workflow_board_y(index, node),
                id = escape(&node.id),
                label = escape(&node.label),
                kind = escape(&node.kind),
                desc = escape(&node.explanation)
            )
        })
        .collect::<String>();

    format!(
        r#"<div class="workflow-board selectable" data-workflow-board="true"><svg class="workflow-board-edges" viewBox="0 0 860 430" aria-hidden="true" focusable="false"><defs><marker id="workflow-board-arrow" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth"><path d="M0,0 L0,6 L9,3 z" class="workflow-arrow"></path></marker></defs>{edges}</svg>{workflow_canvas_metrics}{cards}<div class="workflow-ai-suggestion" aria-label="AI workflow suggestion"><p>AI · WORKFLOW SUGGESTION</p><strong>CFO 승인이 SLA를 초과할 때 자동 위임 조건을 추가</strong><span>conf 0.86 · model oyatie-flow-sense-1.4 · why →</span><div><button type="button" data-workflow-suggestion="dismiss">Dismiss</button><button type="button" data-workflow-suggestion="preview">Preview</button><button type="button" data-workflow-suggestion="apply">Apply</button></div></div><div class="canvas-drop-hint" aria-hidden="true">Drag blocks here · connect ports visually · local only</div></div>"#
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_workflow_svg(nodes: &[WorkflowNode]) -> String {
    let svg_nodes = nodes
        .iter()
        .map(|node| {
            format!(
                "<g class=\"workflow-node-group selectable\"><rect x=\"{x}\" y=\"{y}\" width=\"130\" height=\"56\" rx=\"10\" class=\"workflow-node\"></rect><circle cx=\"{in_x}\" cy=\"{port_y}\" r=\"4\" class=\"port in\"></circle><circle cx=\"{out_x}\" cy=\"{port_y}\" r=\"4\" class=\"port out\"></circle><text x=\"{label_x}\" y=\"{label_y}\">{label}</text><text x=\"{kind_x}\" y=\"{kind_y}\" class=\"node-kind\">{kind}</text></g>",
                x = node.x,
                y = node.y,
                in_x = node.x + 8,
                out_x = node.x + 122,
                port_y = node.y + 28,
                label_x = node.x + 16,
                label_y = node.y + 24,
                kind_x = node.x + 16,
                kind_y = node.y + 43,
                label = escape(&node.label),
                kind = escape(&node.kind)
            )
        })
        .collect::<String>();

    format!(
        "<svg viewBox=\"0 0 820 310\" aria-hidden=\"true\"><line x1=\"140\" y1=\"120\" x2=\"690\" y2=\"120\" class=\"workflow-edge\"></line>{svg_nodes}</svg>"
    )
}
