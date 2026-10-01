use super::*;

pub(super) const COMMAND_SHELL_ROUTES: [(&str, &str, &str, &str); 8] = [
    ("fd001", "FD-001", "Service graph", "#service-catalog"),
    (
        "daily",
        "Action Inbox",
        "Priority work",
        "#command-center-workbench",
    ),
    ("workflow", "Workflow", "Studio IDE", "#workflow-studio"),
    (
        "messenger",
        "Comms",
        "Messenger/Mail/Community",
        "#work-hub",
    ),
    (
        "finance",
        "Finance",
        "Close + ledger",
        "#finance-commercial-service",
    ),
    (
        "identity",
        "Identity",
        "Org + access",
        "#identity-workforce-service",
    ),
    ("cloud", "Cloud", "Substrate cells", "#cloud-ops-cockpit"),
    (
        "evidence",
        "Evidence",
        "Audit receipts",
        "#resource-audit-console",
    ),
];

pub(super) fn command_shell_substrate() -> impl IntoView {
    view! {
        <section
            id="command-shell-substrate"
            class="command-shell-substrate panel"
            aria-labelledby="command-shell-title"
            data-command-shell-substrate="true"
        >
            <div class="command-shell-copy">
                <p class="screen-anchor">"COMMAND SHELL SUBSTRATE"</p>
                <h3 id="command-shell-title">"Every lower panel inherits the same active route, tenant lens, and local boundary"</h3>
                <span data-command-shell-status="true">"FD-001 graph is active · lower surfaces will keep the route/status/inspector spine synchronized."</span>
            </div>
            <div class="command-shell-context" aria-live="polite">
                <span><small>"Active route"</small><strong data-command-shell-route="true">"FD-001 graph"</strong></span>
                <span><small>"Target"</small><strong data-command-shell-target="true">"#service-catalog"</strong></span>
                <span><small>"Updated"</small><strong data-command-shell-updated="true">"SSR render"</strong></span>
            </div>
            <div class="command-shell-routes" role="toolbar" aria-label="Lower dashboard product routes">
                {COMMAND_SHELL_ROUTES.into_iter().map(|(route, label, detail, target)| view! {
                    <button
                        type="button"
                        class=if route == "fd001" { "selected" } else { "" }
                        data-shell-context-route=route
                        data-shell-context-target=target
                    >
                        <strong>{label}</strong>
                        <span>{detail}</span>
                    </button>
                }).collect_view()}
            </div>
        </section>
    }
}

pub(super) fn substrate_proof_command(envelope: TenantRenderEnvelope) -> impl IntoView {
    view! {
        <section id="substrate-proof" class="substrate-proof-command panel" data-substrate-proof="true" aria-labelledby="substrate-proof-title">
            <div class="substrate-proof-head">
                <div>
                    <p class="screen-anchor">"OYATIE CLOUD · FD-001 DOGFOOD SUBSTRATE"</p>
                    <h3 id="substrate-proof-title">"Prove production tenancy by running FD-001 as real tenant workloads"</h3>
                    <p>"FD-001 remains the product delivery goal. Oyatie Cloud is the hyperscaler-grade substrate proving those microservices can host production tenants before any external claim."</p>
                </div>
                <div class="substrate-proof-actions">
                    <span class="status-chip success" data-substrate-status="true">"12 workloads · 3 cells · 0 external writes"</span>
                    <button type="button" data-substrate-action="cloud">"Cloud cells"</button>
                    <button type="button" data-substrate-action="workflow">"Workflow proof"</button>
                    <button type="button" data-substrate-action="evidence">"Evidence"</button>
                </div>
            </div>

            <div class="substrate-proof-grid" aria-label="Substrate proof metrics">
                <article class="substrate-proof-card primary">
                    <p class="screen-anchor">"PRODUCT GOAL"</p>
                    <strong>"FD-001 delivery"</strong>
                    <span>"Core, workflow, messenger, mail, community, finance, identity, intelligence, and ontology run as tenant workload previews."</span>
                </article>
                <article class="substrate-proof-card">
                    <p class="screen-anchor">"SUBSTRATE"</p>
                    <strong>"Oyatie Cloud"</strong>
                    <span>"Cellular runtime, policy, FinOps, resource inventory, deployment gates, and rollback evidence."</span>
                </article>
                <article class="substrate-proof-card">
                    <p class="screen-anchor">"TENANT LENS"</p>
                    <strong>{envelope.tenant_name.clone()}</strong>
                    <span>{format!("{} · server-derived envelope · local dogfood only", envelope.role_name)}</span>
                </article>
                <article class="substrate-proof-card warning">
                    <p class="screen-anchor">"READINESS"</p>
                    <strong>"84% proof"</strong>
                    <span>"3 blockers: payroll delta, cloud rollback receipt, PIPA review."</span>
                </article>
            </div>

            <div class="substrate-workload-map" aria-label="FD-001 tenant workload deployment map">
                <div class="substrate-map-column substrate-product-column">
                    <p class="screen-anchor">"FD-001 WORKLOADS"</p>
                    <button type="button" data-substrate-action="workflow"><strong>"Workflow"</strong><span>"approval engine · no-code studio"</span></button>
                    <button type="button" data-substrate-action="messenger"><strong>"Messenger"</strong><span>"ops room thread · evidence extraction"</span></button>
                    <button type="button" data-substrate-action="mail"><strong>"Mail"</strong><span>"formal approval brief"</span></button>
                    <button type="button" data-substrate-action="community"><strong>"Community"</strong><span>"governance council post"</span></button>
                </div>
                <div class="substrate-map-spine" aria-hidden="true">
                    <span>"tenant workload"</span>
                    <i></i>
                    <span>"cell runtime"</span>
                    <i></i>
                    <span>"evidence receipt"</span>
                </div>
                <div class="substrate-map-column substrate-cloud-column">
                    <p class="screen-anchor">"OYATIE CLOUD CELLS"</p>
                    <button type="button" data-substrate-action="cloud"><strong>"cell-us-east-2"</strong><span>"primary · workload dogfood"</span></button>
                    <button type="button" data-substrate-action="finops"><strong>"kr-seoul-1"</strong><span>"localization pack · FinOps watch"</span></button>
                    <button type="button" data-substrate-action="deployment"><strong>"gitops promotion"</strong><span>"Jenkins · ArgoCD · cosign · audit"</span></button>
                    <button type="button" data-substrate-action="evidence"><strong>"evidence spine"</strong><span>"REC-FD001-CLOUD-009"</span></button>
                </div>
            </div>

            <div class="substrate-proof-footer" aria-label="Dogfood proof routes">
                <span>"Proof loop: tenant workload → Oyatie Cloud cell → policy gate → human route → evidence receipt"</span>
                <button type="button" data-substrate-action="finance">"Finance close"</button>
                <button type="button" data-substrate-action="identity">"Identity policy"</button>
                <button type="button" data-substrate-action="catalog">"Service catalog"</button>
            </div>
        </section>
    }
}
