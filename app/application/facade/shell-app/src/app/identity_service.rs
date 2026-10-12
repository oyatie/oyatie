use super::*;

pub(super) fn identity_workforce_service(
    envelope: TenantRenderEnvelope,
    available: Signal<bool>,
) -> impl IntoView {
    let source = account_sample_source(&envelope);
    view! {
        <For each=move || [(source.clone(), envelope.clone())]
            key=|(source, _)| source.clone()
            children=move |(source, envelope)| identity_workforce_panel(envelope, available, source) />
    }
}

fn identity_workforce_panel(
    envelope: TenantRenderEnvelope,
    available: Signal<bool>,
    source: String,
) -> impl IntoView {
    view! {
        <section id="identity-workforce-service" class="identity-workforce-service panel" data-identity-service="true"
            data-account-source=source data-account-context=envelope.context.id()
            data-account-label=daily_sample_label(&envelope) data-account-hub-source=HubSample::envelope_source(&envelope)
            data-account-mode=if cfg!(target_arch = "wasm32") { "island" } else { "ssr" }
            data-account-available=move || account_available(available).to_string() aria-labelledby="identity-service-title">
            <div class="identity-service-head"><div><p class="screen-anchor">"SAMPLE ACCOUNT · SETTINGS"</p>
                <h3 id="identity-service-title" tabindex="-1" data-account-focus="heading">"Sample account, organization and workspace setup"</h3></div>
                <div class="identity-service-actions"><span class="status-chip" data-identity-status="true" role="status">"Sample examples only · live authority Unknown / unavailable"</span>
                    <button type="button" data-identity-action="open-audit" data-account-focus="audit">"Open existing audit sample"</button></div></div>
            <p data-account-reset="true" role="status" hidden>"Sample source changed; previous local setup and inspection cleared."</p>
            <div class="identity-service-shell">
                <aside class="identity-settings-rail" aria-label="Sample account and organization sections">
                    <div data-account-person="true" inner_html=account_person_html()></div>
                    <button type="button" data-identity-tab="auth" class="active" aria-current="page">"Sample credentials"</button>
                    <button type="button" data-identity-tab="sessions">"Sample devices"</button>
                    <button type="button" data-identity-tab="roles">"Sample roles"</button>
                    <button type="button" data-identity-tab="org">"Sample organization"</button>
                    <button type="button" data-identity-tab="employees">"Employees"</button>
                    <button type="button" data-identity-tab="onboarding">"Sample setup"</button>
                </aside>
                <div class="identity-service-main">
                    <div class="identity-tabs" role="tablist" aria-label="Sample account views" aria-orientation="horizontal">
                        <button type="button" id="identity-tab-auth" class="active" data-identity-tab="auth" role="tab" aria-label="Sample Auth" aria-selected="true" aria-controls="identity-panel-auth">"Auth"</button>
                        <button type="button" id="identity-tab-sessions" data-identity-tab="sessions" role="tab" aria-label="Sample Sessions" aria-selected="false" aria-controls="identity-panel-sessions">"Sessions"</button>
                        <button type="button" id="identity-tab-roles" data-identity-tab="roles" role="tab" aria-label="Sample Roles" aria-selected="false" aria-controls="identity-panel-roles">"Roles"</button>
                        <button type="button" id="identity-tab-org" data-identity-tab="org" role="tab" aria-label="Sample Org profile" aria-selected="false" aria-controls="identity-panel-org">"Org profile"</button>
                        <button type="button" id="identity-tab-employees" data-identity-tab="employees" role="tab" aria-selected="false" aria-controls="identity-panel-employees">"Employees"</button>
                        <button type="button" id="identity-tab-onboarding" data-identity-tab="onboarding" role="tab" aria-label="Sample Onboarding" aria-selected="false" aria-controls="identity-panel-onboarding">"Onboarding"</button>
                    </div>

                    <article id="identity-panel-auth" class="identity-panel active" data-identity-panel="auth" data-account-focus="panel:auth" tabindex="-1" role="tabpanel" aria-labelledby="identity-tab-auth" inner_html=account_panel_html("auth")></article>
                    <article id="identity-panel-sessions" class="identity-panel" data-identity-panel="sessions" data-account-focus="panel:sessions" tabindex="-1" role="tabpanel" aria-labelledby="identity-tab-sessions" inner_html=account_panel_html("sessions")></article>
                    <article id="identity-panel-roles" class="identity-panel" data-identity-panel="roles" data-account-focus="panel:roles" tabindex="-1" role="tabpanel" aria-labelledby="identity-tab-roles" inner_html=account_panel_html("roles")></article>
                    <article id="identity-panel-org" class="identity-panel" data-identity-panel="org" data-account-focus="panel:org" tabindex="-1" role="tabpanel" aria-labelledby="identity-tab-org" inner_html=account_panel_html("org")></article>
                    <article id="identity-panel-employees" class="identity-panel" data-identity-panel="employees" role="tabpanel" aria-labelledby="identity-tab-employees">
                        <div class="identity-panel-copy">
                            <p class="screen-anchor">"SETTINGS · EMPLOYEES"</p>
                            <h4 id="employees-panel-title">"직원 디렉토리"</h4>
                            <p>
                                "FD-001 people, payroll, policy, Mail, and Community workloads stay product-first while "
                                "Oyatie Cloud proves identity data can be hosted as a governed tenant surface."
                            </p>
                        </div>
                        {workforce_anchor_board()}
                        {employee_directory::sample_employee_directory()}
                    </article>

                    <article id="identity-panel-onboarding" class="identity-panel" data-identity-panel="onboarding" data-account-focus="panel:onboarding" tabindex="-1" role="tabpanel" aria-labelledby="identity-tab-onboarding" inner_html=account_panel_html("onboarding")></article>
                </div>
                <aside class="identity-context-rail" aria-label="Sample account provenance" data-account-context-facts="true" inner_html=account_context_html()></aside>
            </div>
            <div inner_html=account_channel_templates(&envelope)></div>
        </section>
    }
}

fn account_available(available: Signal<bool>) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        available.try_get().unwrap_or(false)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        available.try_get_untracked().unwrap_or(false)
    }
}
