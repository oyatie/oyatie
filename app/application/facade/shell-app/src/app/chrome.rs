use super::*;

pub fn shell_scope_notice_text() -> &'static str {
    "Sample dashboard · anonymous preview only: sample tenant and role; no PHI/PII. This preview does not verify a tenant or role and does not query live service status. The separate live Ontology card requires sign-in."
}

pub fn shell_landmark_label() -> &'static str {
    "Oyatie Operations · Cloud/Tenant Control Center"
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <div class="console-app">
            <a class="skip-link" href="#console-shell">"Skip to dashboard"</a>
            <ShellRail />
            // R-1: bottom tab bar replaces rail navigation at ≤72rem breakpoint
            <MobileNavBar />
            <ShellHeader />
            <main
                id="console-shell"
                class="control-center"
                aria-labelledby="console-title"
                aria-describedby="console-notice"
            >
                <HeroPanel />
                <SignInPanel />
                <super::live_ontology::FoundryWorkspace />
                <div id=crate::DASHBOARD_MOUNT_HOST_ID>
                    <DashboardIsland />
                </div>
            </main>
            <UtilityPanels />
            <SidePeek />
        </div>
    }
}

#[component]
fn ShellNavigation() -> impl IntoView {
    view! {
                <p class="rail-group">"Foundry"</p>
                <a class="rail-nav" href="#foundry-workspace"><span aria-hidden="true">"◎"</span>"Ontology workspace"</a>
                <p class="rail-group">"Workspace samples"</p>
                <a class="rail-nav" href="#console-shell"><span aria-hidden="true">"⌂"</span>"Command center"</a>
                <a class="rail-nav" href="#command-center-workbench"><span aria-hidden="true">"▥"</span>"Action Inbox"</a>
                <div class="governance-rail-links" inner_html=governance_rail_html(OperatorContext::TenantAdmin, false)></div>
                <p class="rail-group">"Operate"</p>
                <div class="governance-rail-links" inner_html=governance_rail_html(OperatorContext::TenantAdmin, true)></div>
                <a class="rail-nav" href="#tasks-title"><span aria-hidden="true">"☑"</span>"Tasks"</a>
                <a class="rail-nav" href="#schedule-title"><span aria-hidden="true">"◷"</span>"Schedule"</a>
                <a class="rail-nav" href="#workflow-studio"><span aria-hidden="true">"⌘"</span>"Workflow Studio"</a>
                <a class="rail-nav" href="#work-hub"><span aria-hidden="true">"✉"</span>"Messenger · Mail · Community"</a>
                <a class="rail-nav" href="#cloud-ops-cockpit"><span aria-hidden="true">"◫"</span>"Cloud Ops"</a>
                <p class="rail-group">"Money"</p>
                <a class="rail-nav" href="#payroll-cockpit"><span aria-hidden="true">"₩"</span>"Payroll"</a>
    <span data-finance-rail-slot="commercial"></span>
    <span data-finance-rail-slot="finops"></span>
                <p class="rail-group">"Compliance"</p>
    <span data-finance-rail-slot="filing"></span>
                <a class="rail-nav" href="#resource-panel-audit"><span aria-hidden="true">"◱"</span>"Audit ledger"</a>
                <a class="rail-nav" href="#cockpit-panel-policy"><span aria-hidden="true">"⚿"</span>"Policy & access"</a>
                <p class="rail-group">"People"</p>
                <a class="rail-nav" href="#identity-panel-employees"><span aria-hidden="true">"◎"</span>"Employees"</a>
                <a class="rail-nav" href="#finance-panel-leave"><span aria-hidden="true">"◫"</span>"Leave & time"</a>
                <a class="rail-nav" href="#identity-workforce-service"><span aria-hidden="true">"⚿"</span>"Auth · Org"</a>
                <p class="rail-group">"Trust"</p>
                <a class="rail-nav" href="#resource-panel-inventory"><span aria-hidden="true">"▤"</span>"Resource inventory"</a>
                <a class="rail-nav" href="#modules-title"><span aria-hidden="true">"▦"</span>"Service catalog"</a>
                <a class="rail-nav" href="#evidence-spine"><span aria-hidden="true">"▥"</span>"Evidence spine"</a>
                <a class="rail-nav" href="#resource-panel-gates"><span aria-hidden="true">"✓"</span>"Deployment gates"</a>
                <a class="rail-nav" href="#ontology-command-console"><span aria-hidden="true">"◎"</span>"Object graph"</a>
                <a class="rail-nav" href="#intelligence-command-console"><span aria-hidden="true">"✦"</span>"Copilot rail"</a>
    }
}

#[component]
pub(super) fn ShellRail() -> impl IntoView {
    view! {
        <aside class="app-rail" aria-label="Product navigation">
            <div class="rail-brand">
                <span class="rail-mark" aria-hidden="true">"O"</span>
                <div>
                    <strong>"Oyatie"</strong>
                    <span id="shell-rail-product">"Control Center"</span>
                </div>
                <code>"v0.1"</code>
            </div>
            <section id="shell-rail-context" class="rail-proof-card" aria-label="Anonymous sample destinations">
                <p id="shell-rail-scope">"ANONYMOUS SAMPLE VIEWS"</p>
                <strong id="shell-rail-title">"Explore shell samples"</strong>
                <span id="shell-rail-description">"Catalog, cloud, evidence and Work Hub samples. Live operations are unavailable."</span>
                <small id="shell-rail-status" data-rail-status="true">"Navigation only · no live operation"</small>
                <div class="rail-proof-actions" aria-label="Sample destinations">
                    <a href="#service-catalog">"Catalog samples"</a>
                    <a href="#cloud-ops-cockpit">"Cloud samples"</a>
                    <a href="#evidence-spine">"Evidence samples"</a>
                    <a href="#work-hub">"Work hub samples"</a>
                </div>
            </section>
            <ShellNavigation />
            <a class="workspace-switch" href="#oyatie-sign-in" aria-label="Account and session">
                <span id="shell-account-mark" class="workspace-avatar" aria-hidden="true">"S"</span>
                <div>
                    <strong id="shell-account-title">"Sample workspace"</strong>
                    <span id="shell-account-detail">"Anonymous preview"</span>
                </div>
            </a>
        </aside>
    }
}

/// Native disclosure keeps the complete rail inventory available when the rail collapses.
#[component]
pub(super) fn MobileNavBar() -> impl IntoView {
    view! {
        <nav class="mobile-nav-bar" aria-label="Primary navigation">
            <div class="mobile-nav-bar-inner">
                <a href="#console-shell">
                    <span aria-hidden="true">"⌂"</span>
                    "Command"
                </a>
                <a data-governance-rail="true" href="#business-logics">
                    <span aria-hidden="true">"⌬"</span>
                    "Operate"
                </a>
                <a href="#workflow-studio">
                    <span aria-hidden="true">"⌘"</span>
                    "Workflow"
                </a>
                <a class="mobile-nav-secondary" href="#work-hub">
                    <span aria-hidden="true">"✉"</span>
                    "Comms"
                </a>
                <a class="mobile-nav-secondary" href="#identity-workforce-service">
                    <span aria-hidden="true">"⚿"</span>
                    "Identity"
                </a>
                <a class="mobile-nav-secondary" href="#cloud-ops-cockpit">
                    <span aria-hidden="true">"◫"</span>
                    "Cloud"
                </a>
                <a class="mobile-nav-secondary" href="#resource-panel-audit">
                    <span aria-hidden="true">"▤"</span>
                    "Audit"
                </a>
                <details class="mobile-nav-more">
                    <summary><span aria-hidden="true">"⋯"</span>"More"</summary>
                    <div class="mobile-nav-menu" role="group" aria-label="All product destinations">
                        <ShellNavigation />
                    </div>
                </details>
            </div>
        </nav>
    }
}

#[component]
pub(super) fn ShellHeader() -> impl IntoView {
    view! {
        <header class="app-header" role="banner">
            // A-7: breadcrumb div → nav landmark + aria-current="page" on active item
            <nav class="top-breadcrumb" aria-label="Breadcrumb">
                <span>"Oyatie Cloud"</span>
                <span class="sep" aria-hidden="true">"/"</span>
                <span id="shell-breadcrumb-scope">"Sample views"</span>
                <span class="sep" aria-hidden="true">"/"</span>
                <strong id="shell-breadcrumb-destination" aria-current="page">"Control Center"</strong>
            </nav>
            <div class="header-route-strip" aria-label="Workspace quick routes">
                <a href="#foundry-workspace"><span>"Foundry"</span>" workspace"</a>
                <a href="#service-catalog"><span>"Catalog"</span>" samples"</a>
                <a href="#cloud-ops-cockpit"><span>"Cloud"</span>" samples"</a>
                <a href="#work-hub"><span>"Work hub"</span>" samples"</a>
                <a href="#evidence-spine"><span>"Evidence"</span>" samples"</a>
                <small id="shell-route-status" data-header-route-status="true">"Navigation only · no live operation"</small>
            </div>
            <a class="command-trigger" href="#catalog-sample-search">
                <span aria-hidden="true">"⌕"</span>
                <span>"Search Catalog samples"</span>
            </a>
            <div class="header-actions" aria-label="Shell tools">
                <a id="shell-session-entry" class="header-sign-in" href="#oyatie-sign-in">"Sign in"</a>
                <button type="button" class="header-icon" data-header-action="notifications" aria-label=format!("Open sample notifications, {} unread", notification_unread_count()) aria-controls="notification-center" aria-haspopup="dialog" commandfor="notification-center" command="show-modal">
                    "◔"
                    <span class="header-badge" data-activity-badge="true" aria-hidden="true">{notification_unread_count()}</span>
                </button>
                <button type="button" class="header-icon" data-header-action="settings" aria-label="Open settings" aria-controls="settings-center" aria-haspopup="dialog" commandfor="settings-center" command="show-modal">"⚙"</button>
            </div>
        </header>
    }
}

#[component]
pub(super) fn HeroPanel() -> impl IntoView {
    view! {
        <section class="hero-panel" aria-labelledby="console-title">
            <div class="hero-main">
                <div class="page-title-copy">
                    <p class="screen-anchor">"01 / Command Center"</p>
                    <div class="hero-title-row">
                        <h1 id="console-title">"Operations samples"</h1>
                        <span class="hero-lens-chip">"Anonymous preview"</span>
                    </div>
                    <p id="console-notice" class="scope-notice" role="note">
                        {shell_scope_notice_text()}
                    </p>
                </div>
            </div>
            <div class="hero-side">
                <div class="hero-copy page-actions">
                    <a class="catalog-search-entry" href="#catalog-sample-search">"Search Catalog samples"</a>
                </div>
            </div>
        </section>
    }
}

#[cfg(test)]
#[path = "chrome_test.rs"]
mod tests;
