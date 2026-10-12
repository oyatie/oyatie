use super::*;

pub(super) fn deployment_gate_command_board(
    envelope: &TenantRenderEnvelope,
) -> impl IntoView + use<> {
    view! {
        <div class="deployment-proof-grid" aria-label="Illustrative deployment choices"><article class="deployment-proof-card"><p class="screen-anchor">"UNAVAILABLE RELEASE OPERATIONS"</p><h5>"Sample inspection only"</h5><p>"Examples cannot admit workloads, seal packets, attach evidence or assign an owner."</p><div class="deployment-card-actions">{cloud_unavailable("gates:admit","Admit workload")}{cloud_unavailable("gates:seal","Seal packet")}{cloud_route(envelope,"gates:workflow","Workflow sample")}{cloud_route(envelope,"gates:cloud","Cloud examples")}{cloud_route(envelope,"gates:finops","FinOps sample")}{cloud_route(envelope,"gates:mail","Mail sample")}</div></article></div>
        <div class="deployment-promotion-lane" role="group" aria-label="Select one of four illustrative gates">{deployment_gates().into_iter().map(|gate| view! {
            <button type="button" data-cloud-select=format!("gate:{}",gate.label) data-cloud-record=cloud_resources_record(&format!("gate:{}",gate.label)) data-cloud-focus=format!("gate:{}",gate.label) aria-pressed="false" aria-expanded="false" aria-controls="cloud-gates-inspection" aria-label=format!("Inspect illustrative gate {}",gate.label)>
                <strong data-cloud-value="Label">{gate.label}</strong><em data-cloud-value="Progress">{gate.progress}</em>
            </button>
        }).collect_view()}</div>
    }
}
