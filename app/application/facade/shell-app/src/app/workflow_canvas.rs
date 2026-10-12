use super::*;

pub(super) fn workflow_canvas(sample: RwSignal<WorkflowSample>, editable: bool) -> impl IntoView {
    view! {
        <div class="workflow-canvas island-frame" role="region" aria-label="Sample workflow canvas; select a node to inspect">
            <div class="workflow-toolbar" aria-label="Sample canvas tools">
                <span>"Select nodes to inspect"</span>
                <button type="button" disabled aria-describedby="workflow-live-unavailable">"Connect unavailable"</button>
                <button type="button" data-workflow-action="path" data-workflow-focus="canvas-path" disabled=!editable on:click=move |_| edit_workflow_sample(sample, WorkflowSample::preview_path)>"Preview sample path"</button>
            </div>
            <div class="workflow-board selectable" data-workflow-board="true" style="overflow: auto;">
                <div style=move || { let state = render_signal(sample.read_only()); format!("position: relative; width: {}px; height: {}px", 860 * state.zoom / 100, workflow_board_height(&state) * state.zoom / 100) }>
                    <div style=move || { let state = render_signal(sample.read_only()); format!("position: absolute; width: 860px; height: {}px; transform: scale({}); transform-origin: 0 0", workflow_board_height(&state), f64::from(state.zoom) / 100.0) }>
                        <svg class="workflow-board-edges" viewBox=move || format!("0 0 860 {}", workflow_board_height(&render_signal(sample.read_only()))) aria-hidden="true" focusable="false">
                            <defs><marker id="workflow-board-arrow" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth"><path d="M0,0 L0,6 L9,3 z" class="workflow-arrow" /></marker></defs>
                            {move || { let state = render_signal(sample.read_only()); board_edges(&state.nodes, &state.edges).into_iter().map(|(from, to, path)| view! {
                                <path class="workflow-edge workflow-board-edge" data-edge-from=from data-edge-to=to d=path marker-end="url(#workflow-board-arrow)" />
                            }).collect_view() }}
                        </svg>
                        <For each=move || render_signal(sample.read_only()).nodes key=|node| node.id.clone() children=move |node| {
                            let id = StoredValue::new(node.id);
                            view! {
                                <button type="button" disabled=!editable
                                    class=move || if render_signal(sample.read_only()).selected_id == id.get_value() { "workflow-card active selectable" } else { "workflow-card selectable" }
                                    aria-pressed=move || (render_signal(sample.read_only()).selected_id == id.get_value()).to_string()
                                    style=move || { let state = render_signal(sample.read_only()); let index = state.nodes.iter().position(|node| node.id == id.get_value()).unwrap_or(0); format!("left: {}px; top: {}px; cursor: pointer", workflow_board_x(index), state.nodes.get(index).map(|node| workflow_board_y(index, node)).unwrap_or(0)) }
                                    data-workflow-card="true" data-node-id=id.get_value()
                                    data-workflow-focus=format!("card-{}", id.get_value())
                                    data-node-label=move || current_workflow_node(sample, id).map(|node| node.label).unwrap_or_default()
                                    data-node-kind=move || current_workflow_node(sample, id).map(|node| node.kind).unwrap_or_default()
                                    data-node-desc=move || current_workflow_node(sample, id).map(|node| node.explanation).unwrap_or_default()
                                    on:click=move |_| edit_workflow_sample(sample, |state| { if state.nodes.iter().any(|node| node.id == id.get_value()) { state.selected_id = id.get_value(); } })>
                                    <span class="workflow-card-type">{move || current_workflow_node(sample, id).map(|node| node.kind).unwrap_or_default()}</span>
                                    <strong>{move || current_workflow_node(sample, id).map(|node| node.label).unwrap_or_default()}</strong>
                                    <small>{move || current_workflow_node(sample, id).map(|node| node.explanation).unwrap_or_default()}</small>
                                </button>
                            }
                        } />
                    </div>
                </div>
            </div>
            <div class="canvas-footer">
                <div class="zoom-controls" aria-label="Sample canvas zoom">
                    <button type="button" data-workflow-action="zoom-out" data-workflow-focus="zoom-out" aria-label="Zoom sample canvas out" disabled=move || !editable || render_signal(sample.read_only()).zoom <= 25 on:click=move |_| edit_workflow_sample(sample, |state| state.zoom = (state.zoom - 10).clamp(25, 150))>"−"</button>
                    <span data-workflow-zoom="true">{move || format!("{}%", render_signal(sample.read_only()).zoom)}</span>
                    <button type="button" data-workflow-action="zoom-in" data-workflow-focus="zoom-in" aria-label="Zoom sample canvas in" disabled={move || !editable || render_signal(sample.read_only()).zoom >= 150} on:click=move |_| edit_workflow_sample(sample, |state| state.zoom = (state.zoom + 10).clamp(25, 150))>"+"</button>
                </div>
                <span>"Sample structure only; no run telemetry"</span>
            </div>
            <div class="node-toolbar" aria-label="Select sample workflow node">
                <For each=move || render_signal(sample.read_only()).nodes key=|node| node.id.clone() children=move |node| {
                    let id = StoredValue::new(node.id);
                    view! {
                        <button type="button" disabled=!editable data-workflow-focus=format!("list-{}", id.get_value())
                            aria-pressed=move || (render_signal(sample.read_only()).selected_id == id.get_value()).to_string()
                            on:click=move |_| edit_workflow_sample(sample, |state| state.selected_id = id.get_value())>
                            {move || current_workflow_node(sample, id).map(|node| node.label).unwrap_or_default()}
                        </button>
                    }
                } />
            </div>
            <aside class="workflow-ai-suggestion" style="position: static; width: auto;" aria-label="Illustrative sample idea" hidden=move || render_signal(sample.read_only()).suggestion_dismissed>
                <p>"SAMPLE IDEA · no AI generation or provenance"</p>
                <strong>"Illustrative idea: request human review before an external example step."</strong>
                <p hidden=move || !render_signal(sample.read_only()).suggestion_open>"This fixed example is descriptive only. No rule, model, owner, or SLA has been evaluated."</p>
                <div>
                    <button type="button" data-workflow-suggestion="dismiss" data-workflow-focus="suggestion-dismiss" disabled=!editable on:click=move |_| edit_workflow_sample(sample, |state| state.suggestion_dismissed = true)>"Dismiss sample idea"</button>
                    <button type="button" data-workflow-suggestion="preview" data-workflow-focus="suggestion-preview" disabled=!editable on:click=move |_| edit_workflow_sample(sample, |state| state.suggestion_open = !state.suggestion_open)>"Inspect sample idea"</button>
                    <button type="button" data-workflow-suggestion="apply" disabled aria-describedby="workflow-live-unavailable">"Apply unavailable"</button>
                </div>
            </aside>
        </div>
    }
}

fn current_workflow_node(
    sample: RwSignal<WorkflowSample>,
    id: StoredValue<String>,
) -> Option<WorkflowNode> {
    render_signal(sample.read_only())
        .nodes
        .into_iter()
        .find(|node| node.id == id.get_value())
}

pub(super) fn workflow_property_form(
    sample: RwSignal<WorkflowSample>,
    editable: bool,
) -> impl IntoView {
    view! {
        <form class="workflow-property-form" aria-label="Selected sample node properties" on:submit=move |event| event.prevent_default()>
            <label><span>"Sample label"</span>
                <input id="workflow-selected-label" data-workflow-prop="label-ko" data-workflow-focus="selected-label" disabled=!editable
                    value=move || render_signal(sample.read_only()).selected().map(|node| node.label.clone()).unwrap_or_default()
                    prop:value=move || render_signal(sample.read_only()).selected().map(|node| node.label.clone()).unwrap_or_default()
                    on:input=move |event| edit_workflow_sample(sample, |state| state.rename_selected(event_target_value(&event))) />
            </label>
            <label><span>"Native sample kind · read-only"</span><input data-workflow-prop="type" data-workflow-focus="selected-kind" readonly
                value=move || render_signal(sample.read_only()).selected().map(|node| node.kind.clone()).unwrap_or_default()
                prop:value=move || render_signal(sample.read_only()).selected().map(|node| node.kind.clone()).unwrap_or_default() /></label>
            <fieldset disabled aria-describedby="workflow-property-unavailable">
                <legend>"Unsupported live properties"</legend>
                <label><span>"Owner"</span><input data-workflow-prop="owner" placeholder="Unavailable" /></label>
                <label><span>"SLA target"</span><input data-workflow-prop="sla" placeholder="Unavailable" /></label>
                <label><span>"Escalation"</span><input data-workflow-prop="escalate" placeholder="Unavailable" /></label>
                <button type="button" data-workflow-process-action="add-condition">"Approval conditions unavailable"</button>
            </fieldset>
            <p id="workflow-property-unavailable" role="note">"Owner, SLA, escalation, and approval evaluation are unavailable; no matching live properties or policy authority are connected."</p>
        </form>
    }
}

pub(super) fn selected_node_view(sample: RwSignal<WorkflowSample>) -> impl IntoView {
    view! {
        <aside class="node-inspector" aria-live="polite">
            <p class="eyebrow">"Selected sample node"</p>
            <h4 data-workflow-selected-label="true">{move || render_signal(sample.read_only()).selected().map(|node| node.label.clone()).unwrap_or_else(|| "No sample node selected".into())}</h4>
            <p><strong data-workflow-selected-kind="true">{move || render_signal(sample.read_only()).selected().map(|node| node.kind.clone()).unwrap_or_default()}</strong>" · "
                {move || render_signal(sample.read_only()).selected().map(|node| node.explanation.clone()).unwrap_or_default()}</p>
        </aside>
    }
}

pub(super) fn board_edges(
    nodes: &[WorkflowNode],
    edges: &[crate::render_envelope::WorkflowEdge],
) -> Vec<(String, String, String)> {
    edges
        .iter()
        .filter_map(|edge| {
            let from_index = nodes.iter().position(|node| node.id == edge.from)?;
            let to_index = nodes.iter().position(|node| node.id == edge.to)?;
            let from = &nodes[from_index];
            let to = &nodes[to_index];
            let start_x = workflow_board_x(from_index) + 156;
            let start_y = workflow_board_y(from_index, from) + 36;
            let end_x = workflow_board_x(to_index);
            let end_y = workflow_board_y(to_index, to) + 36;
            let control_delta = ((end_x - start_x).abs() / 2).max(64);
            let path = format!(
                "M {start_x} {start_y} C {} {start_y}, {} {end_y}, {end_x} {end_y}",
                start_x + control_delta,
                end_x - control_delta,
            );
            Some((from.id.clone(), to.id.clone(), path))
        })
        .collect()
}

pub(super) fn workflow_board_x(index: usize) -> i32 {
    55 + ((index as i32 % 4) * 156)
}

pub(super) fn workflow_board_y(index: usize, node: &WorkflowNode) -> i32 {
    if node.id.starts_with("draft-block-") {
        258 + ((index as i32 / 4) * 104)
    } else {
        72 + ((index as i32 / 4) * 112)
    }
}

pub(super) fn workflow_board_height(sample: &WorkflowSample) -> i32 {
    sample
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| workflow_board_y(index, node) + 120)
        .max()
        .unwrap_or(430)
        .max(430)
}

pub(super) fn fit_workflow_sample(sample: RwSignal<WorkflowSample>) {
    #[cfg(target_arch = "wasm32")]
    if let Some(board) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| {
            document
                .query_selector("#workflow-studio [data-workflow-board]")
                .ok()
                .flatten()
        })
    {
        let zoom = ((board.client_width() - 16) * 100 / 860).clamp(25, 100);
        edit_workflow_sample(sample, |state| state.zoom = zoom);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = sample;
}
