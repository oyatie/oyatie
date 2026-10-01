use super::*;

pub(super) fn product_activity_spine(spine: ProductActivitySpine) -> impl IntoView {
    let active_route = spine.active_route.clone();
    let active_label = spine
        .steps
        .iter()
        .find(|step| step.route_key == active_route)
        .map(|step| step.label.clone())
        .unwrap_or_else(|| active_route.clone());
    let route_steps = spine.steps.clone();
    let lane_steps = spine.steps.clone();

    view! {
        <section
            id="product-activity-spine"
            class="product-activity-spine panel"
            aria-labelledby="product-activity-title"
            data-product-activity-spine="true"
        >
            <div class="activity-spine-head">
                <div>
                    <p class="screen-anchor">"PRODUCT ACTIVITY SPINE"</p>
                    <h3 id="product-activity-title">"One operating model for FD-001 tenant workloads on Oyatie Cloud"</h3>
                    <span data-spine-active-context="true">{spine.active_context.clone()}</span>
                </div>
                <div class="activity-spine-proof">
                    <span data-spine-active-route="true">{active_label}</span>
                    <code data-spine-evidence-id="true">{spine.evidence_id.clone()}</code>
                    <strong data-global-activity-status="true">{spine.status_label.clone()}</strong>
                </div>
            </div>
            <div class="activity-spine-grid">
                <aside class="activity-route-column" aria-label="Cross-surface routes">
                    <p class="screen-anchor">"ROUTES"</p>
                    {route_steps.into_iter().map(|step| product_activity_route_button(step, active_route.clone())).collect_view()}
                </aside>
                <div class="activity-flow-lane" aria-label="FD-001 workload path">
                    {lane_steps.into_iter().map(|step| product_activity_step_card(step, active_route.clone())).collect_view()}
                </div>
                <aside class="activity-inspector-card" aria-label="Selected route inspector">
                    <p class="screen-anchor">"INSPECTOR"</p>
                    <h4 data-spine-inspector-title="true">"FD-001 graph · product substrate"</h4>
                    <p data-spine-inspector-body="true">"Service catalog, workflow, Messenger, Mail, Community, cloud posture, and evidence receipts are one cohesive local operating graph."</p>
                    <dl>
                        <div><dt>"Tenant"</dt><dd data-spine-inspector-tenant="true">{spine.active_context.clone()}</dd></div>
                        <div><dt>"Boundary"</dt><dd>"Visual-only · no backend write"</dd></div>
                        <div><dt>"Receipt"</dt><dd>{spine.evidence_id.clone()}</dd></div>
                    </dl>
                    <div class="activity-inspector-actions">
                        <button type="button" data-activity-route="workflow">"Open Workflow"</button>
                        <button type="button" data-activity-route="mail">"Mail brief"</button>
                        <button type="button" data-activity-route="evidence">"Evidence"</button>
                    </div>
                </aside>
            </div>
            <div class="activity-spine-statusbar" aria-label="Current local shell state">
                <span>"SSR shell"</span>
                <span>"Selective WASM islands"</span>
                <span>"Local-only actions"</span>
                <span data-spine-last-action="true">"Ready · route and inspector state will update visually"</span>
            </div>
        </section>
    }
}

pub(super) fn product_activity_route_button(
    step: ProductActivityStep,
    active_route: String,
) -> impl IntoView {
    let selected = step.route_key == active_route;
    let route_key = step.route_key.clone();
    let target = step.target.clone();
    let label = step.label.clone();
    let label_attr = label.clone();
    let detail = step.detail.clone();
    let state = step.state.clone();
    let surface = step.surface.clone();

    view! {
        <button
            type="button"
            class=if selected { "selected" } else { "" }
            data-activity-route=route_key
            data-activity-target=target
            data-activity-label=label_attr
            data-activity-detail=detail
            data-activity-state=state
        >
            <strong>{label}</strong>
            <span>{surface}</span>
        </button>
    }
}

pub(super) fn product_activity_step_card(
    step: ProductActivityStep,
    active_route: String,
) -> impl IntoView {
    let selected = step.route_key == active_route;
    let route_key = step.route_key.clone();
    let target = step.target.clone();
    let label = step.label.clone();
    let label_attr = label.clone();
    let detail = step.detail.clone();
    let detail_attr = detail.clone();
    let state = step.state.clone();
    let state_attr = state.clone();
    let surface = step.surface.clone();

    view! {
        <button
            type="button"
            class=if selected { "activity-step-card selected" } else { "activity-step-card" }
            data-activity-route=route_key.clone()
            data-activity-target=target
            data-activity-label=label_attr
            data-activity-detail=detail_attr
            data-activity-state=state_attr
            data-spine-step=route_key
        >
            <span>{surface}</span>
            <strong>{label}</strong>
            <small>{detail}</small>
            <em>{state}</em>
        </button>
    }
}
