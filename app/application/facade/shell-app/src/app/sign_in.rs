use leptos::prelude::*;

/// Kept outside DashboardIsland so context remounts cannot erase a ceremony.
#[component]
pub(super) fn SignInPanel() -> impl IntoView {
    view! {
        <section id="oyatie-sign-in" class="sign-in-panel" aria-labelledby="sign-in-title" data-sign-in-panel="true">
            <div class="sign-in-copy">
                <p class="screen-anchor">"OYATIE IAM"</p>
                <h2 id="sign-in-title">"Sign in with a passkey"</h2>
                <p>"Enter your tenant and account. Your administrator provides these IDs and a one-use invitation to add your first passkey."</p>
                <p>"The dashboard below uses sample data. The separate live Ontology card requires verified access."</p>
            </div>
            <form id="passkey-form" class="sign-in-form" aria-describedby="sign-in-status">
                <label for="sign-in-tenant">"Tenant ID"</label>
                <input id="sign-in-tenant" name="tenant" type="text" placeholder="ten_acme" autocomplete="off" required maxlength="256" spellcheck="false" />
                <label for="sign-in-account">"Account ID"</label>
                <input id="sign-in-account" name="account" type="text" placeholder="usr_alice" autocomplete="username" required maxlength="256" spellcheck="false" />
                <div class="sign-in-actions">
                    <button type="submit" id="passkey-sign-in" disabled>"Use passkey"</button>
                    <button type="button" id="session-restore" disabled>"Check session"</button>
                    <button type="button" id="passkey-cancel" hidden>"Cancel"</button>
                    <button type="button" id="session-logout" disabled>"Sign out"</button>
                </div>
                <details class="sign-in-invitation">
                    <summary>"Add your first passkey"</summary>
                    <p>"Choose the invitation file provided by your administrator. It is used once, only on this page."</p>
                    <label for="passkey-invitation">"Passkey invitation (JSON file)"</label>
                    <input type="file" id="passkey-invitation" accept="application/json,.json" disabled />
                    <button type="button" id="passkey-enroll" disabled>"Add passkey"</button>
                </details>
            </form>
            <p id="sign-in-status" class="sign-in-status" role="status" aria-live="polite">"Checking sign-in availability. Anonymous preview remains available."</p>
            <section id="verified-ontology-card" class="verified-ontology-card" aria-labelledby="verified-ontology-title" hidden>
                <h3 id="verified-ontology-title">"Live Ontology · verified access"</h3>
                <p id="verified-ontology-identity"></p>
                <dl class="verified-ontology-facts">
                    <div><dt>"Policy version"</dt><dd data-live-fact="policyVersion"></dd></div>
                    <div><dt>"Served tenants"</dt><dd data-live-fact="servedTenants"></dd></div>
                    <div><dt>"Projection lag"</dt><dd data-live-fact="projectionLag"></dd></div>
                    <div><dt>"Poisoned entries"</dt><dd data-live-fact="poisonedEntries"></dd></div>
                </dl>
                <p id="verified-ontology-updated"></p>
                <button type="button" id="ontology-refresh">"Refresh live status"</button>
            </section>
        </section>
    }
}
