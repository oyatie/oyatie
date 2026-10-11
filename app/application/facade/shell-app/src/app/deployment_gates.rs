use super::*;

pub(super) fn deployment_gate_command_board() -> impl IntoView {
    view! {
        <div class="deployment-proof-grid" aria-label="FD-001 and Oyatie Cloud deployment proof">
            <article class="deployment-proof-card selected" data-deployment-card="fd001">
                <p class="screen-anchor">"FD-001 RELEASE TRAIN"</p>
                <h5>"Product microservices deploy as tenants"</h5>
                <p>
                    "Messenger, Mail, Community, Workflow, Ontology, Intelligence, and core ops stay product-first; "
                    "the gates prove they can run as tenant workloads on the substrate."
                </p>
                <div class="deployment-card-actions">
                    <button type="button" data-deployment-gate-action="admit-fd001">"Admit workload"</button>
                    <button type="button" data-deployment-gate-action="route-workflow">"Workflow runbook"</button>
                </div>
            </article>
            <article class="deployment-proof-card" data-deployment-card="cloud">
                <p class="screen-anchor">"OYATIE CLOUD"</p>
                <h5>"Hyperscaler-grade substrate proof"</h5>
                <p>
                    "Cell topology, policy sidecars, cosign receipts, ArgoCD app health, rollback posture, and "
                    "audit-chain freshness must be visible before any promotion claim."
                </p>
                <div class="deployment-card-actions">
                    <button type="button" data-deployment-gate-action="route-cloud">"Inspect cells"</button>
                    <button type="button" data-deployment-gate-action="route-finops">"FinOps guard"</button>
                </div>
            </article>
            <article class="deployment-proof-card" data-deployment-card="control">
                <p class="screen-anchor">"CONTROL PLANE"</p>
                <h5>"Interactive, never wired"</h5>
                <p>
                    "Operators can simulate gate decisions, seal a release packet, and route reviewer work, while "
                    "deploy, DNS, registry, billing, and cloud mutations remain disconnected."
                </p>
                <div class="deployment-card-actions">
                    <button type="button" data-deployment-gate-action="seal-release">"Seal packet"</button>
                    <button type="button" data-deployment-gate-action="route-mail">"Reviewer mail"</button>
                </div>
            </article>
        </div>
        <div class="deployment-promotion-lane" aria-label="Tenant workload promotion lane">
            <button type="button" class="active" data-deployment-gate-action="ci-lane">
                <span>"01"</span><strong>"CI mirror"</strong><em>"Jenkins parity · 92%"</em>
            </button>
            <button type="button" data-deployment-gate-action="attest-lane">
                <span>"02"</span><strong>"Attest"</strong><em>"cosign + SBOM · 61%"</em>
            </button>
            <button type="button" data-deployment-gate-action="admit-lane">
                <span>"03"</span><strong>"Admit tenant"</strong><em>"policy + PIPA · review"</em>
            </button>
            <button type="button" data-deployment-gate-action="observe-lane">
                <span>"04"</span><strong>"Observe"</strong><em>"SLO + audit emit · 48%"</em>
            </button>
        </div>
    }
}
