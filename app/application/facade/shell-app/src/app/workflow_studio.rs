use super::*;

#[expect(
    clippy::too_many_arguments,
    reason = "Workflow panel keeps each reactive control explicit pending a production component boundary."
)]
pub(super) fn workflow_studio_panel(
    name: String,
    goal: String,
    nodes: Vec<WorkflowNode>,
    selected_node: Option<WorkflowNode>,
    set_selected_node_id: WriteSignal<String>,
    workflow_tool: ReadSignal<WorkflowTool>,
    set_workflow_tool: WriteSignal<WorkflowTool>,
    draft_node_count: ReadSignal<usize>,
    set_draft_node_count: WriteSignal<usize>,
    set_active_surface: WriteSignal<ProductSurface>,
) -> impl IntoView {
    view! {
        <section id="workflow-studio" class="panel workflow-panel cohesive-workflow" aria-labelledby="workflow-title">
            <div class="workflow-topbar">
                <div>
                    <p class="eyebrow">"Workflow Studio"</p>
                    <h3 id="workflow-title">{name}</h3>
                    <div class="workflow-doc-meta">
                        <span>"v18 · draft"</span>
                        <span>"Owner · tenant admin"</span>
                        <span>"SLA · 4.0h"</span>
                    </div>
                </div>
                <div class="workflow-run-chip" aria-label="Run state preview">
                    <span></span>
                    {move || match render_signal(workflow_tool) {
                        WorkflowTool::Select => "draft · select mode",
                        WorkflowTool::Connect => "draft · connect mode",
                        WorkflowTool::Simulate => "simulation preview",
                    }}
                </div>
                <div class="workflow-actions" aria-label="Workflow actions preview">
                    <button type="button">"Fit"</button>
                    <button type="button" on:click=move |_| set_workflow_tool.set(WorkflowTool::Select)>"Clear run"</button>
                    <button type="button" on:click=move |_| set_workflow_tool.set(WorkflowTool::Select)>"Validate"</button>
                    <button class="primary-action" type="button" on:click=move |_| set_workflow_tool.set(WorkflowTool::Simulate)>"Run"</button>
                    <button type="button" on:click=move |_| set_draft_node_count.set(render_signal(draft_node_count) + 1)>"Add block"</button>
                    <button class="dark-action" type="button">"Publish"</button>
                </div>
            </div>
            {workflow_process_chrome()}
            {workflow_output_bus()}
            <p class="panel-intro">{goal}</p>

            <div class="workflow-modebar" role="toolbar" aria-label="Workflow editor modes">
                {WorkflowTool::ALL.into_iter().map(|tool| view! {
                    <button
                        type="button"
                        class=move || if render_signal(workflow_tool) == tool { "active" } else { "" }
                        aria-pressed=move || (render_signal(workflow_tool) == tool).to_string()
                        on:click=move |_| set_workflow_tool.set(tool)
                    >
                        {tool.label()}
                    </button>
                }).collect_view()}
            </div>
            {workflow_lens_toolbar()}

            <div class="workflow-ide">
                <aside class="workflow-palette" aria-label="Workflow building blocks">
                    <div class="palette-search">
                        <span aria-hidden="true">"⌕"</span>
                        <input data-workflow-palette-search="true" aria-label="Search workflow blocks" placeholder="Search nodes..." />
                        <kbd>"⌘K"</kbd>
                    </div>
                    <div class="palette-heading"><span>"Primitives"</span><em>"12"</em></div>
                    {[
                        ("System task", "Deterministic step · 0 ms", "S"),
                        ("Approval", "Single, parallel, or quorum", "A"),
                        ("Validation", "Rule check · halts on fail", "V"),
                        ("External call", "HTTP, RPC, or connector", "E"),
                        ("Branch / Switch", "Multi-way condition split", "B"),
                        ("Wait / Timer", "Until time · or duration", "W"),
                        ("Loop / For-each", "Iterate over collection", "L"),
                        ("AI step", "Suggest · classify · extract", "⌥A"),
                        ("중단 / 에스컬레이트", "CFO 알림 · 실행 중단", "H"),
                        ("Form / Input", "Collect data from human", "F"),
                        ("Webhook trigger", "Inbound event start", "T"),
                        ("End / Receipt", "Emit immutable event", "⌘E"),
                    ].into_iter().map(|(label, detail, key)| view! {
                        <button
                            type="button"
                            data-palette-item="primitive"
                            on:click=move |_| set_draft_node_count.set(render_signal(draft_node_count) + 1)
                        >
                            <span>{label}</span><small>{detail}</small><kbd>{key}</kbd>
                        </button>
                    }).collect_view()}
                    <div class="palette-heading"><span>"Actions"</span><em>"6"</em></div>
                    {[
                        ("Task", "Create a governed work item", "T"),
                        ("HTTP request", "Call external REST/HTTP", "H"),
                        ("Database", "Read/write a record", "D"),
                        ("Transform", "Reshape the payload", "X"),
                        ("Filter", "Drop failed items", "F"),
                        ("Write to doc", "Append a row / line", "W"),
                    ].into_iter().map(|(label, detail, key)| view! {
                        <button
                            type="button"
                            data-palette-item="action"
                            on:click=move |_| set_draft_node_count.set(render_signal(draft_node_count) + 1)
                        >
                            <span>{label}</span><small>{detail}</small><kbd>{key}</kbd>
                        </button>
                    }).collect_view()}
                    <div class="palette-heading"><span>"Logic"</span><em>"5"</em></div>
                    {[
                        ("If / Branch", "Two-way condition split", "I"),
                        ("Switch", "Multi-way routing", "S"),
                        ("Loop / For-each", "Iterate collection", "L"),
                        ("Wait", "Delay or duration", "W"),
                        ("Merge", "Wait for branches", "M"),
                    ].into_iter().map(|(label, detail, key)| view! {
                        <button
                            type="button"
                            data-palette-item="logic"
                            on:click=move |_| set_draft_node_count.set(render_signal(draft_node_count) + 1)
                        >
                            <span>{label}</span><small>{detail}</small><kbd>{key}</kbd>
                        </button>
                    }).collect_view()}
                    <div class="palette-heading"><span>"Built-in surfaces"</span><em>"3"</em></div>
                    <button type="button" data-palette-item="surface" on:click=move |_| set_active_surface.set(ProductSurface::Messenger)><span>"Messenger post"</span><small>"Route run summary to Ops room"</small><kbd>"M"</kbd></button>
                    <button type="button" data-palette-item="surface" on:click=move |_| set_active_surface.set(ProductSurface::Mail)><span>"Mail draft"</span><small>"Formal approval note"</small><kbd>"⌘M"</kbd></button>
                    <button type="button" data-palette-item="surface" on:click=move |_| set_active_surface.set(ProductSurface::Community)><span>"Community note"</span><small>"Publish governed update"</small><kbd>"C"</kbd></button>
                    <div class="palette-heading"><span>"Connectors"</span><em>"9"</em></div>
                    <div class="workflow-connector-grid" aria-label="Workflow connector shortcuts">
                        {[
                            ("국세", "HomeTax"),
                            ("국민", "NPS / 4대"),
                            ("신한", "Shinhan"),
                            ("T", "Toss"),
                            ("K", "Kakao Work"),
                            ("#", "Slack"),
                            ("G", "Workspace"),
                            ("Q", "QuickBooks"),
                            ("N", "Notion"),
                        ].into_iter().map(|(mark, label)| view! {
                            <button type="button" data-palette-item="connector" on:click=move |_| set_draft_node_count.set(render_signal(draft_node_count) + 1)>
                                <strong>{mark}</strong><span>{label}</span>
                            </button>
                        }).collect_view()}
                    </div>
                </aside>

                {workflow_canvas(nodes.clone(), set_selected_node_id, workflow_tool, set_workflow_tool)}

                <aside class="workflow-inspector" aria-label="Selected workflow node inspector">
                    <div class="inspector-tabs" aria-hidden="true">
                        <span class="active">"Inspector"</span>
                        <span>"Run log"</span>
                    </div>
                    {selected_node_view(selected_node)}
                    {workflow_property_form()}
                    <dl class="inspector-fields">
                        <div><dt>"Guardrail"</dt><dd>"Human review before action"</dd></div>
                        <div><dt>"Output"</dt><dd>"Task · message · evidence draft"</dd></div>
                        <div><dt>"Execution"</dt><dd>"Disabled until live integration"</dd></div>
                    </dl>
                    <div class="inspector-stat-grid" aria-label="Selected node run statistics">
                        <div><span>"Avg"</span><strong>"0.8s"</strong></div>
                        <div><span>"P95"</span><strong>"2.1s"</strong></div>
                        <div><span>"Errors"</span><strong>"0"</strong></div>
                        <div><span>"Cost"</span><strong>"$0.03"</strong></div>
                    </div>
                    <div class="run-log-preview">
                        <p class="eyebrow">"Run log"</p>
                        <ol>
                            <li><time>"10:31"</time><span>"Validation preview passed"</span></li>
                            <li><time>"10:32"</time><span>"Messenger/Mail/Community drafts generated"</span></li>
                            <li><time>"10:33"</time><span>"Audit receipt staged locally"</span></li>
                        </ol>
                    </div>
                </aside>
            </div>

            <div class="workflow-statusbar" aria-label="Workflow editor status">
                <span>{move || format!("Nodes: {}", nodes.len())}</span>
                <span>{move || format!("Local blocks: {}", render_signal(draft_node_count))}</span>
                <span>"Messenger/Mail/Community outputs are drafts"</span>
                <span>{move || match render_signal(workflow_tool) {
                    WorkflowTool::Select => "Ready · staged",
                    WorkflowTool::Connect => "Click nodes to visualize links",
                    WorkflowTool::Simulate => "Previewing run path only",
                }}</span>
            </div>
        </section>
    }
}

pub(super) fn workflow_process_chrome() -> impl IntoView {
    view! {
        <div class="workflow-process-chrome" aria-label="Workflow process command chrome">
            <div class="workflow-process-meta">
                <span>"PROCESS"</span>
                <strong>"PROC-PAYROLL-CLOSE"</strong>
                <span>"OWNER"</span>
                <strong>"Hyo-jin Park · #274"</strong>
                <span>"SLA"</span>
                <strong>"4.0d"</strong>
            </div>
            <div class="workflow-process-actions">
                <button type="button" data-workflow-process-action="validate">"✓ Validate"</button>
                <button type="button" data-workflow-process-action="simulate">"◷ Simulate"</button>
                <button type="button" data-workflow-process-action="diff">"↯ Diff v17 → v18"</button>
                <button type="button" class="dark-action" data-workflow-process-action="publish">"게시 v18"</button>
            </div>
            <span class="workflow-process-status" data-workflow-process-status="true">"autosaved · local visual IDE"</span>
        </div>
    }
}

pub(super) fn workflow_lens_toolbar() -> impl IntoView {
    view! {
        <div class="workflow-lens-toolbar" aria-label="Workflow layout, overlay, and filter controls">
            <div class="workflow-lens-group">
                <span>"LAYOUT"</span>
                <button type="button" class="active" data-workflow-lens="Graph">"Graph"</button>
                <button type="button" data-workflow-lens="Swimlanes">"Swimlanes"</button>
                <button type="button" data-workflow-lens="Timeline">"Timeline"</button>
                <button type="button" data-workflow-lens="Tree">"Tree"</button>
            </div>
            <div class="workflow-lens-group">
                <span>"OVERLAY"</span>
                <button type="button" data-workflow-overlay="Cycle">"Cycle"</button>
                <button type="button" class="active" data-workflow-overlay="Cost">"Cost"</button>
                <button type="button" data-workflow-overlay="Owner">"Owner"</button>
                <button type="button" data-workflow-overlay="Risk">"Risk"</button>
                <button type="button" data-workflow-overlay="Off">"Off"</button>
            </div>
            <div class="workflow-lens-group">
                <span>"FILTER"</span>
                <button type="button" data-workflow-filter="All">"All"</button>
                <button type="button" class="active" data-workflow-filter="Critical path">"Critical path"</button>
                <button type="button" data-workflow-filter="Bottlenecks">"Bottlenecks"</button>
                <button type="button" data-workflow-filter="AI suggestions">"AI suggestions"</button>
            </div>
        </div>
    }
}
