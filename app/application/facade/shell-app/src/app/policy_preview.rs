use super::*;

/// One preview renderer for the native shell, static export, and browser island.
pub(super) fn policy_preview_panel(envelope: TenantRenderEnvelope) -> impl IntoView {
    let healthcare_gate = if envelope.accreditation.healthcare_enabled {
        "Healthcare enabled in sample"
    } else {
        "Healthcare gated in sample"
    };
    view! {
    <article id="cockpit-panel-policy" class="cockpit-panel" data-cockpit-panel="policy" data-policy-context=envelope.context.id() data-policy-mode=if cfg!(target_arch = "wasm32") { "island" } else { "ssr" } role="tabpanel" aria-labelledby="cockpit-tab-policy">
        <div class="cockpit-column-head">
            <p class="screen-anchor">"POLICY & ACCESS"</p>
            <h4 id="policy-access-title">"Sample policy preview"</h4>
        </div>
        <p data-policy-preview-source="true">"Sample tenant and role data. Live policy evaluation and verified grants are unavailable; these controls change local preview state only."</p>
        <div class="policy-command-grid" aria-label="Sample tenant policy routes">
            <article class="policy-command-card selected" data-policy-card="fd001">
                <div>
                    <p class="screen-anchor">"FD-001 TENANT"</p>
                    <h5>"Product delivery stays the goal"</h5>
                    <p>
                        "Messenger, Mail, Community, Workflow, Ontology, and Intelligence run as tenant workloads; "
                        "Oyatie Cloud proves they can be hosted without moving the tenant workload north star."
                    </p>
                </div>
                <div class="policy-command-actions" aria-label="FD-001 policy routes">
                    <button type="button" data-policy-anchor-action="role-review" data-policy-target="identity-panel-roles">"Review role grants"</button>
                    <button type="button" data-policy-anchor-action="open-identity" data-policy-target="identity-panel-auth">"Open identity"</button>
                    <button type="button" data-policy-anchor-action="route-evidence" data-policy-target="evidence-spine">"Evidence spine"</button>
                </div>
            </article>
            <article class="policy-command-card" data-policy-card="substrate">
                <div>
                    <p class="screen-anchor">"OYATIE CLOUD"</p>
                    <h5>"Dogfood substrate boundary"</h5>
                    <p>
                        "Cloud controls stay tenant-scoped, PIPA-aware, auditable, and local-only until "
                        "real FD-001 services are admitted through release gates."
                    </p>
                </div>
                <div class="policy-command-actions" aria-label="Oyatie Cloud policy routes">
                    <button type="button" data-policy-anchor-action="route-cloud" data-policy-target="cockpit-panel-topology">"Cloud topology"</button>
                    <button type="button" data-policy-anchor-action="pipa-boundary" data-policy-target="policy-pipa-boundary">"PIPA boundary"</button>
                    <button type="button" data-policy-anchor-action="open-audit" data-policy-target="resource-panel-audit">"Audit trail"</button>
                </div>
            </article>
            <article class="policy-command-card" data-policy-card="autonomy">
                <div>
                    <p class="screen-anchor">"AUTONOMY CEILING"</p>
                    <h5>"Local explanations"</h5>
                    <p>
                        "The examples illustrate allow, gate, deny, and reviewer paths. Every action is "
                        "visual state with no cloud, billing, DNS, or workflow mutation."
                    </p>
                </div>
                <div class="policy-command-actions" aria-label="Autonomy policy routes">
                    <button type="button" data-policy-anchor-action="autonomy-ceiling" data-policy-target="policy-autonomy-ceiling">"Show ceiling"</button>
                    <button type="button" data-policy-anchor-action="residency" data-policy-target="policy-residency">"Residency pack"</button>
                    <button type="button" data-policy-anchor-action="route-mail" data-policy-target="work-hub" data-policy-channel="Mail">"Mail brief"</button>
                </div>
            </article>
        </div>
        <table class="policy-table"><caption>"Example decisions; no native policy check was performed"</caption>
            <thead>
                <tr><th>"Subject"</th><th>"Scope"</th><th>"Example outcome"</th><th>"Reason"</th></tr>
            </thead>
            <tbody>
                <tr><td>"Tenant admin"</td><td>"Cloud controls"</td><td><span class="status-chip success">"Allow"</span></td><td>"Owner role"</td></tr>
                <tr><td>{envelope.role_name.clone()}</td><td>"Healthcare"</td><td><span class="status-chip warning">{healthcare_gate}</span></td><td>"Sample accreditation flag"</td></tr>
                <tr><td>"Workflow builder"</td><td>"Execution"</td><td><span class="status-chip danger">"Deny"</span></td><td>"Autonomy ceiling"</td></tr>
            </tbody>
        </table>
        <div class="policy-evidence-grid">
            <span><strong>"3"</strong><small>"example decision rows"</small></span>
            <span><strong>{envelope.modules.len()}</strong><small>"sample module cards"</small></span>
            <span><strong>"0"</strong><small>"live policy checks"</small></span>
        </div>
        <div class="policy-decision-strip" aria-label="Example policy decisions">
            <article class="policy-decision-card" data-policy-card="allow">
                <span class="status-chip success">"Allow"</span>
                <strong>"Tenant admin → Cloud controls"</strong>
                <p>"Example only: an owner may inspect controls. No verified owner grant or persisted receipt is established here."</p>
                <button type="button" data-policy-anchor-action="route-cloud" data-policy-target="cockpit-panel-topology">"Inspect controls"</button>
            </article>
            <article class="policy-decision-card" data-policy-card="gate">
                <span class="status-chip warning">"Gate"</span>
                <strong>{format!("{} → regulated data", envelope.role_name.clone())}</strong>
                <p>{format!("{healthcare_gate} · reviewer evidence and residency pack required before any FD-001 workload placement.")}</p>
                <button type="button" data-policy-anchor-action="residency" data-policy-target="policy-residency">"Review gate"</button>
            </article>
            <article class="policy-decision-card" data-policy-card="deny">
                <span class="status-chip danger">"Deny"</span>
                <strong>"Workflow builder → execution"</strong>
                <p>"Example only: execution is unavailable in this preview. This is not an evaluated authorization denial."</p>
                <button type="button" data-policy-anchor-action="autonomy-ceiling" data-policy-target="policy-autonomy-ceiling">"Trace denial"</button>
            </article>
        </div>
        <aside class="policy-command-card" aria-label="Local policy explanations">
            <details><summary id="policy-pipa-boundary" tabindex="0">"PIPA boundary example"</summary><p>"This sample illustrates a tenant privacy boundary. It does not verify consent, retention, employee data access, or a live PIPA control."</p></details>
            <details><summary id="policy-autonomy-ceiling" tabindex="0">"Execution ceiling example"</summary><p>"This preview cannot execute workflows or change cloud, billing, DNS, or policy. The example denial is not a native policy decision."</p></details>
            <details><summary id="policy-residency" tabindex="0">"Residency and accreditation example"</summary><p>{format!("{healthcare_gate}. This is a sample render-envelope flag; no residency pack, regulatory admission, or reviewer evidence was verified.")}</p></details>
        </aside>
        <template data-policy-hub-template="Mail" inner_html=policy_hub_html(&envelope, ProductSurface::Mail)></template>
        <template data-policy-hub-template="Community" inner_html=policy_hub_html(&envelope, ProductSurface::Community)></template>
        <div class="policy-anchor-footer">
            <span class="cockpit-status" data-policy-anchor-status="true" role="status" aria-live="polite">
                "Sample policy preview ready · live evaluation unavailable."
            </span>
            <div class="policy-anchor-routes" aria-label="Connected policy routes">
                <button type="button" data-policy-anchor-action="route-community" data-policy-target="work-hub" data-policy-channel="Community">"Community review"</button>
                <button type="button" data-policy-anchor-action="open-audit" data-policy-target="resource-panel-audit">"Audit ledger"</button>
                <button type="button" data-policy-anchor-action="route-evidence" data-policy-target="evidence-spine">"Evidence graph"</button>
            </div>
        </div>
    </article>
    }
}

// Server snapshots reuse the whole hub so list, detail and route status agree.
#[cfg(not(target_arch = "wasm32"))]
fn policy_hub_html(envelope: &TenantRenderEnvelope, surface: ProductSurface) -> String {
    let Some(channel) = surface.hub_channel() else {
        return String::new();
    };
    communication_hub_html(envelope, channel)
}

#[cfg(target_arch = "wasm32")]
fn policy_hub_html(_envelope: &TenantRenderEnvelope, _surface: ProductSurface) -> String {
    // The mounted island uses its existing channel signals; no SSR snapshot is needed.
    String::new()
}

#[cfg(test)]
#[path = "policy_preview_tests.rs"]
mod tests;
