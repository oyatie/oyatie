use super::*;

pub(super) fn workflow_destination_button(
    route: &'static str,
    label: &'static str,
    focus: &'static str,
) -> impl IntoView {
    view! {
        <button type="button" data-workflow-output-route=route data-workflow-focus=focus>
            <strong>{label}</strong><em>"Open existing sample destination"</em>
        </button>
    }
}

pub(super) fn workflow_output_bus() -> impl IntoView {
    view! {
        <div class="workflow-output-bus" data-workflow-output-bus="true" aria-label="Existing sample destinations">
            <div class="workflow-output-head"><p class="screen-anchor">"SAMPLE DESTINATIONS"</p><strong>"Inspect existing views"</strong>
                <span role="status" data-workflow-output-status="true">"No draft, message, publication, or receipt created."</span>
            </div>
            <div class="workflow-output-flow" aria-label="Open existing sample destinations">
                {workflow_destination_button("messenger", "Messenger", "output-messenger")}
                {workflow_destination_button("mail", "Mail", "output-mail")}
                {workflow_destination_button("community", "Community", "output-community")}
                {workflow_destination_button("evidence", "Evidence", "output-evidence")}
            </div>
            <aside class="workflow-output-proof"><p data-workflow-output-receipt="true">"No receipt created or verified; live Workflow authority is unavailable."</p></aside>
        </div>
    }
}
