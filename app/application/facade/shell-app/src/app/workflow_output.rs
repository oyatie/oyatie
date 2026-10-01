use super::*;

pub(super) fn workflow_output_bus() -> impl IntoView {
    view! {
        <div
            class="workflow-output-bus"
            data-workflow-output-bus="true"
            aria-label="Workflow output bus for FD-001 tenant workload routes"
        >
            <div class="workflow-output-head">
                <p class="screen-anchor">"FD-001 OUTPUT BUS"</p>
                <strong>"Run preview emits tenant workload drafts"</strong>
                <span data-workflow-output-status="true">
                    "Idle · run/validate/publish stays local until a route is selected"
                </span>
            </div>
            <div class="workflow-output-flow" aria-label="Workflow output routes">
                <button type="button" class="selected" data-workflow-output-route="messenger">
                    <span>"01"</span>
                    <strong>"Messenger"</strong>
                    <em>"Ops room run note"</em>
                </button>
                <button type="button" data-workflow-output-route="mail">
                    <span>"02"</span>
                    <strong>"Mail"</strong>
                    <em>"Approval brief"</em>
                </button>
                <button type="button" data-workflow-output-route="community">
                    <span>"03"</span>
                    <strong>"Community"</strong>
                    <em>"Council digest"</em>
                </button>
                <button type="button" data-workflow-output-route="evidence">
                    <span>"04"</span>
                    <strong>"Evidence"</strong>
                    <em>"Receipt spine"</em>
                </button>
            </div>
            <aside class="workflow-output-proof" aria-label="FD-001 and Oyatie Cloud proof context">
                <dl>
                    <div>
                        <dt>"Product goal"</dt>
                        <dd>"FD-001 delivery"</dd>
                    </div>
                    <div>
                        <dt>"Substrate"</dt>
                        <dd>"Oyatie Cloud · cell-us-east-2"</dd>
                    </div>
                    <div>
                        <dt>"Receipt"</dt>
                        <dd data-workflow-output-receipt="true">"REC-FD001-WF-018 · draft"</dd>
                    </div>
                </dl>
            </aside>
        </div>
    }
}
