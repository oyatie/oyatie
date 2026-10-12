use super::*;

const WORKFLOW_BLOCKS: &[(&str, &str, &str, &str)] = &[
    (
        "Primitives",
        "System task",
        "System",
        "Deterministic step example",
    ),
    ("Primitives", "Approval", "Human", "Human approval example"),
    (
        "Primitives",
        "Validation",
        "Validation",
        "Rule check example",
    ),
    (
        "Primitives",
        "External call",
        "External",
        "HTTP, RPC, or connector example",
    ),
    (
        "Primitives",
        "Branch / Switch",
        "Branch",
        "Condition split example",
    ),
    (
        "Primitives",
        "Wait / Timer",
        "Wait",
        "Time or duration example",
    ),
    (
        "Primitives",
        "Loop / For-each",
        "Loop",
        "Collection example",
    ),
    (
        "Primitives",
        "AI step",
        "AI example",
        "Illustrative suggestion; no model connected",
    ),
    (
        "Primitives",
        "중단 / 에스컬레이트",
        "Escalation",
        "Escalation example",
    ),
    ("Primitives", "Form / Input", "Input", "Human input example"),
    (
        "Primitives",
        "Webhook trigger",
        "Trigger",
        "Event start example",
    ),
    (
        "Primitives",
        "End / Receipt",
        "Audit",
        "Evidence reference example; no receipt issued",
    ),
    ("Actions", "Task", "Action", "Work item example"),
    ("Actions", "HTTP request", "Action", "External HTTP example"),
    (
        "Actions",
        "Database",
        "Action",
        "Record example; no read or write",
    ),
    (
        "Actions",
        "Transform",
        "Action",
        "Payload transformation example",
    ),
    ("Actions", "Filter", "Action", "Item filter example"),
    (
        "Actions",
        "Write to doc",
        "Action",
        "Document example; no document changed",
    ),
    ("Logic", "If / Branch", "Logic", "Two-way condition example"),
    ("Logic", "Switch", "Logic", "Routing example"),
    ("Logic", "Loop / For-each", "Logic", "Iteration example"),
    ("Logic", "Wait", "Logic", "Delay example"),
    ("Logic", "Merge", "Logic", "Branch join example"),
    (
        "Connectors",
        "HomeTax",
        "Connector",
        "Tax connector example; disconnected",
    ),
    (
        "Connectors",
        "NPS / 4대",
        "Connector",
        "Insurance connector example; disconnected",
    ),
    (
        "Connectors",
        "Shinhan",
        "Connector",
        "Bank connector example; disconnected",
    ),
    (
        "Connectors",
        "Toss",
        "Connector",
        "Payment connector example; disconnected",
    ),
    (
        "Connectors",
        "Kakao Work",
        "Connector",
        "Messaging connector example; disconnected",
    ),
    (
        "Connectors",
        "Slack",
        "Connector",
        "Messaging connector example; disconnected",
    ),
    (
        "Connectors",
        "Workspace",
        "Connector",
        "Workspace connector example; disconnected",
    ),
    (
        "Connectors",
        "QuickBooks",
        "Connector",
        "Accounting connector example; disconnected",
    ),
    (
        "Connectors",
        "Notion",
        "Connector",
        "Document connector example; disconnected",
    ),
];

fn workflow_block_matches(query: &str, label: &str, detail: &str) -> bool {
    format!("{label} {detail}")
        .to_lowercase()
        .contains(&query.trim().to_lowercase())
}

pub(super) fn workflow_studio_panel(
    envelope: TenantRenderEnvelope,
    sample: RwSignal<WorkflowSample>,
) -> impl IntoView {
    let editable = cfg!(target_arch = "wasm32");
    view! {
        <section id="workflow-studio" class="panel workflow-panel cohesive-workflow" aria-labelledby="workflow-title" tabindex="-1" data-workflow-focus="panel"
            data-workflow-source=sample.get_untracked().source data-workflow-context=envelope.context.id()
            data-workflow-mode=if editable { "island" } else { "ssr" }>
            <div class="workflow-topbar">
                <div><p class="eyebrow">"Workflow Studio · sample editor"</p><h3 id="workflow-title">{envelope.workflow.name}</h3>
                    <div class="workflow-doc-meta"><span>"Local page sample"</span><span>"No version, owner, or SLA verified"</span></div>
                </div>
                <div class="workflow-run-chip"><span></span>"No workflow run connected"</div>
                <div class="workflow-actions" aria-label="Sample workflow actions">
                    <button type="button" data-workflow-action="fit" data-workflow-focus="fit" disabled=!editable on:click=move |_| fit_workflow_sample(sample)>"Fit"</button>
                    <button type="button" data-workflow-action="clear" data-workflow-focus="clear" disabled=!editable on:click=move |_| edit_workflow_sample(sample, |state| state.outcome = "Local preview cleared; nothing saved or executed.".into())>"Clear preview"</button>
                    <button type="button" data-workflow-action="validate" data-workflow-focus="validate" disabled=!editable on:click=move |_| edit_workflow_sample(sample, WorkflowSample::validate)>"Validate sample"</button>
                    <button class="primary-action" type="button" disabled aria-describedby="workflow-live-unavailable">"Run unavailable"</button>
                    <button type="button" data-workflow-action="add" data-workflow-focus="add" disabled=!editable on:click=move |_| edit_workflow_sample(sample, |state| state.add("New sample block", "Sample", "Locally added example"))>"Add block"</button>
                    <button class="dark-action" type="button" disabled aria-describedby="workflow-live-unavailable">"Publish unavailable"</button>
                </div>
            </div>
            <p id="workflow-live-unavailable" role="note">"Run, publication, AI generation, autosave, and durable persistence are unavailable: no live Workflow authority is connected. Local edits remain only on this page while context and sample source match."</p>
            <p id="workflow-edit-availability" role="note">{if editable { "Local sample editing is available; no live effect." } else { "Sample editing is unavailable until the native browser island loads. Native inspection and destination routes remain available." }}</p>
            {workflow_process_chrome(sample, editable)}
            {workflow_output_bus()}
            {payroll_preview_detail(PayrollPreview::Workflow)}
            <p class="panel-intro">{envelope.workflow.goal}</p>
            {workflow_lens_toolbar()}
            <div class="workflow-ide">
                <aside class="workflow-palette" aria-label="Sample workflow building blocks">
                    <div class="palette-search"><span aria-hidden="true">"⌕"</span>
                        <input id="workflow-palette-search" data-workflow-palette-search="true" data-workflow-focus="palette-search" aria-label="Search sample workflow blocks" placeholder="Search sample blocks" disabled=!editable
                            prop:value=move || render_signal(sample.read_only()).query
                            on:input=move |event| edit_workflow_sample(sample, |state| state.query = event_target_value(&event)) />
                    </div>
                    {["Primitives", "Actions", "Logic", "Connectors"].into_iter().map(|group| {
                        let entries: Vec<_> = WORKFLOW_BLOCKS.iter().copied().filter(|entry| entry.0 == group).collect();
                        view! {
                            <div class="palette-heading"><span>{group}</span><em>{move || WORKFLOW_BLOCKS.iter().filter(|entry| entry.0 == group && workflow_block_matches(&render_signal(sample.read_only()).query, entry.1, entry.3)).count()}</em></div>
                            {entries.into_iter().map(|(_, label, kind, detail)| view! {
                                <button type="button" data-palette-item=kind data-workflow-focus=format!("palette-{group}-{label}") disabled=!editable
                                    hidden=move || !workflow_block_matches(&render_signal(sample.read_only()).query, label, detail)
                                    on:click=move |_| edit_workflow_sample(sample, |state| state.add(label, kind, detail))>
                                    <span>{label}</span><small>{detail}</small>
                                </button>
                            }).collect_view()}
                        }
                    }).collect_view()}
                    <p data-workflow-palette-empty="true" hidden=move || WORKFLOW_BLOCKS.iter().any(|entry| workflow_block_matches(&render_signal(sample.read_only()).query, entry.1, entry.3))>"No sample blocks match your search."</p>
                    <div class="palette-heading"><span>"Existing destinations"</span><em>"3"</em></div>
                    {workflow_destination_button("messenger", "Messenger", "palette-messenger")}
                    {workflow_destination_button("mail", "Mail", "palette-mail")}
                    {workflow_destination_button("community", "Community", "palette-community")}
                </aside>
                {workflow_canvas(sample, editable)}
                <aside class="workflow-inspector" aria-label="Selected sample workflow node inspector">
                    {selected_node_view(sample)}
                    {workflow_property_form(sample, editable)}
                    <dl class="inspector-fields"><div><dt>"Execution"</dt><dd>"Unavailable; no engine connected"</dd></div><div><dt>"Metrics"</dt><dd>"Unavailable; no run telemetry connected"</dd></div></dl>
                    <div class="run-log-preview"><p class="eyebrow">"Performed local sample action"</p><p role="status" data-workflow-outcome="true">{move || render_signal(sample.read_only()).outcome}</p></div>
                </aside>
            </div>
            <div class="workflow-statusbar" aria-label="Current sample editor status">
                <span data-workflow-node-count="true">{move || format!("Nodes: {}", render_signal(sample.read_only()).nodes.len())}</span>
                <span data-workflow-draft-count="true">{move || { let state = render_signal(sample.read_only()); format!("Local blocks: {}", state.nodes.len().saturating_sub(state.base_count)) }}</span>
                <span>"No output, receipt, or durable save created"</span>
            </div>
        </section>
    }
}

fn workflow_process_chrome(sample: RwSignal<WorkflowSample>, editable: bool) -> impl IntoView {
    view! {
        <div class="workflow-process-chrome" aria-label="Local sample commands">
            <div class="workflow-process-meta"><span>"CURRENT SAMPLE"</span><strong>"Structure and list order only"</strong></div>
            <div class="workflow-process-actions">
                <button type="button" data-workflow-process-action="validate" data-workflow-focus="process-validate" disabled=!editable on:click=move |_| edit_workflow_sample(sample, WorkflowSample::validate)>"Validate sample"</button>
                <button type="button" data-workflow-process-action="simulate" data-workflow-focus="path-preview" disabled=!editable on:click=move |_| edit_workflow_sample(sample, WorkflowSample::preview_path)>"Preview sample path"</button>
                <button type="button" data-workflow-process-action="diff" disabled aria-describedby="workflow-live-unavailable">"Version diff unavailable"</button>
                <button type="button" data-workflow-process-action="publish" disabled aria-describedby="workflow-live-unavailable">"Publish unavailable"</button>
            </div>
            <span class="workflow-process-status" data-workflow-process-status="true">"Local page sample · autosave unavailable"</span>
        </div>
    }
}

fn workflow_lens_toolbar() -> impl IntoView {
    view! {
        <div class="workflow-lens-toolbar" aria-label="Sample layout limitations">
            <span>"Graph · all sample nodes"</span>
            <button type="button" data-workflow-lens="Swimlanes" disabled>"Swimlanes unavailable"</button>
            <button type="button" data-workflow-lens="Timeline" disabled>"Timeline unavailable"</button>
            <button type="button" data-workflow-lens="Tree" disabled>"Tree unavailable"</button>
            <span role="note">"Evaluated overlays, critical path, and bottleneck filters are unavailable; no execution or telemetry is connected."</span>
        </div>
    }
}
