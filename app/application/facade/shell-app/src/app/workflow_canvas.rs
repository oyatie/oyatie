use super::*;

pub(super) fn workflow_canvas(
    nodes: Vec<WorkflowNode>,
    set_selected_node_id: WriteSignal<String>,
    workflow_tool: ReadSignal<WorkflowTool>,
    set_workflow_tool: WriteSignal<WorkflowTool>,
) -> impl IntoView {
    let board_nodes = nodes.clone();
    let toolbar_nodes = nodes.clone();

    view! {
        <div class="workflow-canvas island-frame" role="img" aria-label="Interactive workflow canvas preview">
            <div class="workflow-toolbar" aria-label="Workflow studio tools">
                <button type="button" on:click=move |_| set_workflow_tool.set(WorkflowTool::Select)>"Select"</button>
                <button type="button" on:click=move |_| set_workflow_tool.set(WorkflowTool::Connect)>"Connect"</button>
                <button type="button" on:click=move |_| set_workflow_tool.set(WorkflowTool::Simulate)>"Simulate"</button>
                <span class="island-label">"interactive island"</span>
            </div>
            <div
                class=move || match render_signal(workflow_tool) {
                    WorkflowTool::Select => "workflow-board selectable",
                    WorkflowTool::Connect => "workflow-board connectable",
                    WorkflowTool::Simulate => "workflow-board simulating",
                }
                data-workflow-board="true"
            >
                <svg class="workflow-board-edges" viewBox="0 0 860 430" aria-hidden="true" focusable="false">
                    <defs>
                        <marker id="workflow-board-arrow" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth">
                            <path d="M0,0 L0,6 L9,3 z" class="workflow-arrow" />
                        </marker>
                    </defs>
                    {board_edges(&board_nodes).into_iter().map(|(from, to, path)| view! {
                        <path
                            class="workflow-edge workflow-board-edge"
                            data-edge-from=from
                            data-edge-to=to
                            d=path
                            marker-end="url(#workflow-board-arrow)"
                        />
                    }).collect_view()}
                </svg>
                {workflow_canvas_metrics()}
                {board_nodes.into_iter().enumerate().map(|(index, node)| {
                    let id = node.id.clone();
                    let id_attr = id.clone();
                    let label_attr = node.label.clone();
                    let label_text = node.label.clone();
                    let kind_attr = node.kind.clone();
                    let kind_text = node.kind.clone();
                    let desc_attr = node.explanation.clone();
                    let desc_text = node.explanation.clone();
                    view! {
                        <button
                            type="button"
                            class=move || match render_signal(workflow_tool) {
                                WorkflowTool::Select => if index == 0 { "workflow-card active selectable" } else { "workflow-card selectable" },
                                WorkflowTool::Connect => "workflow-card connectable",
                                WorkflowTool::Simulate => "workflow-card simulating",
                            }
                            style=format!(
                                "left: {}px; top: {}px",
                                workflow_board_x(index),
                                workflow_board_y(index, &node)
                            )
                            data-workflow-card="true"
                            data-node-id=id_attr
                            data-node-label=label_attr
                            data-node-kind=kind_attr
                            data-node-desc=desc_attr
                            on:click=move |_| set_selected_node_id.set(id.clone())
                        >
                            <span class="board-port in" aria-hidden="true"></span>
                            <span class="board-port out" aria-hidden="true"></span>
                            <span class="workflow-card-type">{kind_text}</span>
                            <strong>{label_text}</strong>
                            <small>{desc_text}</small>
                        </button>
                    }
                }).collect_view()}
                <div class="workflow-ai-suggestion" aria-label="AI workflow suggestion">
                    <p>"AI · WORKFLOW SUGGESTION"</p>
                    <strong>"CFO 승인이 SLA를 초과할 때 자동 위임 조건을 추가"</strong>
                    <span>"conf 0.86 · model oyatie-flow-sense-1.4 · why →"</span>
                    <div>
                        <button type="button" data-workflow-suggestion="dismiss">"Dismiss"</button>
                        <button type="button" data-workflow-suggestion="preview">"Preview"</button>
                        <button type="button" data-workflow-suggestion="apply">"Apply"</button>
                    </div>
                </div>
                <div class="canvas-drop-hint" aria-hidden="true">
                    "Drag blocks here · connect ports visually · local only"
                </div>
            </div>
            <div class="canvas-footer">
                <div class="zoom-controls" aria-label="Visual zoom controls">
                    <button type="button">"−"</button>
                    <span>"82%"</span>
                    <button type="button">"+"</button>
                </div>
                <div class="mini-map" aria-hidden="true">
                    <span></span><span></span><span></span><span></span>
                </div>
            </div>
            <div class="node-toolbar" aria-label="Select workflow node to inspect">
                {toolbar_nodes.into_iter().map(|node| {
                    let id = node.id.clone();
                    view! {
                        <button type="button" on:click=move |_| set_selected_node_id.set(id.clone())>
                            {node.label}
                        </button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

pub(super) fn workflow_canvas_metrics() -> impl IntoView {
    view! {
        <div class="workflow-canvas-metrics" aria-label="Workflow simulation metrics overlay">
            <span><small>"CYCLE"</small><strong>"5.4d"</strong><em>"+1.4 vs target"</em></span>
            <span><small>"TARGET"</small><strong>"4.0d"</strong><em>"SLA limit"</em></span>
            <span><small>"COST"</small><strong>"₩2.18M"</strong><em>"delay cost"</em></span>
            <span><small>"REWORK"</small><strong>"8%"</strong><em>"2 loops"</em></span>
        </div>
    }
}

pub(super) fn workflow_property_form() -> impl IntoView {
    view! {
        <form class="workflow-property-form" aria-label="Selected workflow node properties">
            <label>
                <span>"LABEL · KO"</span>
                <input data-workflow-prop="label-ko" value="재무 검토 · 사인오프" />
            </label>
            <label>
                <span>"TYPE"</span>
                <select data-workflow-prop="type">
                    <option>"Single · auto-delegate"</option>
                    <option>"Parallel quorum"</option>
                    <option>"Human review stop"</option>
                </select>
            </label>
            <label>
                <span>"OWNER"</span>
                <select data-workflow-prop="owner">
                    <option>"Sarah Kim · EMP-188 · HR Manager"</option>
                    <option>"Choi Yu-na · CFO"</option>
                    <option>"David Chen · Delegate"</option>
                </select>
            </label>
            <div class="workflow-form-row">
                <label><span>"SLA TARGET"</span><input data-workflow-prop="sla" value="1.2d" /></label>
                <label><span>"ESCALATE AFTER"</span><input data-workflow-prop="escalate" value="0.8d" /></label>
            </div>
            <fieldset class="workflow-rule-stack">
                <legend>"승인 조건"</legend>
                <label><span>"1"</span><input data-workflow-prop="rule-1" value="payroll.gross > ₩500,000,000" /></label>
                <label><span>"2"</span><input data-workflow-prop="rule-2" value="policy.P0 == active" /></label>
                <button type="button" data-workflow-process-action="add-condition">"+ Add condition"</button>
            </fieldset>
        </form>
    }
}

pub(super) fn selected_node_view(node: Option<WorkflowNode>) -> impl IntoView {
    match node {
        Some(node) => view! {
            <aside class="node-inspector" aria-live="polite">
                <p class="eyebrow">"Selected node"</p>
                <h4>{node.label}</h4>
                <p><strong>{node.kind}</strong>" · "{node.explanation}</p>
            </aside>
        }
        .into_any(),
        None => view! {
            <aside class="node-inspector" aria-live="polite">
                <p>"Select a node to inspect its workflow, ontology, and guardrail meaning."</p>
            </aside>
        }
        .into_any(),
    }
}
