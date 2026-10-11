use super::*;

pub fn shell_scope_notice_text() -> &'static str {
    "Anonymous preview only: sample tenant and role; no PHI/PII. No tenant or role was verified, and live service status was not queried. Live Ontology status requires sign-in."
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
pub(super) fn ShellRail() -> impl IntoView {
    view! {
        <aside class="app-rail" aria-label="Product navigation">
            <div class="rail-brand">
                <span class="rail-mark" aria-hidden="true">"O"</span>
                <div>
                    <strong>"Oyatie"</strong>
                    <span>"Control Center"</span>
                </div>
                <code>"v0.1"</code>
            </div>
            <section class="rail-proof-card" aria-label="FD-001 and Oyatie Cloud shell proof">
                <p>"FD-001 TENANT WORKLOADS"</p>
                <strong>"Service graph on Oyatie Cloud"</strong>
                <span>"Messenger · Mail · Community dogfood the substrate."</span>
                <small data-rail-status="true">"REC-WF-7741 · cell-us-east-2 · local visual routes"</small>
                <div class="rail-proof-actions" aria-label="Persistent shell proof routes">
                    <button type="button" class="is-selected" data-rail-proof-action="service-graph">"Service graph"</button>
                    <button type="button" data-rail-proof-action="cloud">"Cloud"</button>
                    <button type="button" data-rail-proof-action="evidence">"Evidence"</button>
                    <button type="button" data-rail-proof-action="work-hub">"Work hub"</button>
                </div>
                <div class="rail-comms-switcher" aria-label="Built-in Work Hub surface routes">
                    <button type="button" class="is-selected" data-rail-comms-surface="Messenger">"Messenger"</button>
                    <button type="button" data-rail-comms-surface="Mail">"Mail"</button>
                    <button type="button" data-rail-comms-surface="Community">"Community"</button>
                </div>
            </section>
            <p class="rail-group">"Run the company"</p>
            <a class="rail-nav active" href="#console-shell"><span aria-hidden="true">"⌂"</span>"Command center"</a>
            <a class="rail-nav" href="#command-center-workbench"><span aria-hidden="true">"▥"</span>"Action Inbox"<em>"8"</em></a>
            <a class="rail-nav" href="#governance-analytics"><span aria-hidden="true">"↟"</span>"Governance analytics"</a>
            <p class="rail-group">"Operate"</p>
            <a class="rail-nav" href="#business-logics"><span aria-hidden="true">"⌬"</span>"Business Logics"<em>"17"</em></a>
            <a class="rail-nav" href="#tasks-title"><span aria-hidden="true">"☑"</span>"Tasks"<em>"73"</em></a>
            <a class="rail-nav" href="#schedule-title"><span aria-hidden="true">"◷"</span>"Schedule"</a>
            <a class="rail-nav" href="#workflow-studio"><span aria-hidden="true">"⌘"</span>"Workflow Studio"</a>
            <a class="rail-nav" href="#work-hub"><span aria-hidden="true">"✉"</span>"Messenger · Mail · Community"<em>"18"</em></a>
            <a class="rail-nav" href="#cloud-ops-cockpit"><span aria-hidden="true">"◫"</span>"Cloud Ops"</a>
            <p class="rail-group">"Money"</p>
            <a class="rail-nav" href="#payroll-cockpit"><span aria-hidden="true">"₩"</span>"Payroll"</a>
            <a class="rail-nav" href="#finance-panel-ledger"><span aria-hidden="true">"▤"</span>"Ledger"</a>
            <a class="rail-nav" href="#finance-panel-vendors"><span aria-hidden="true">"◇"</span>"Vendors & spend"</a>
            <a class="rail-nav" href="#finance-panel-billing"><span aria-hidden="true">"▧"</span>"Billing & tax"</a>
            <a class="rail-nav" href="#cockpit-panel-finops"><span aria-hidden="true">"₩"</span>"FinOps"</a>
            <p class="rail-group">"Compliance"</p>
            <a class="rail-nav" href="#filing-readiness"><span aria-hidden="true">"□"</span>"Filing readiness"<em>"2"</em></a>
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
            <div class="workspace-switch">
                <span class="workspace-avatar" aria-hidden="true">"N"</span>
                <div>
                    <strong>"Northwind"</strong>
                    <span>"Enterprise · US/EU/KR"</span>
                </div>
            </div>
        </aside>
    }
}

/// R-1: Mobile bottom tab bar — shown at ≤72rem when .app-rail is hidden.
/// Provides keyboard-accessible navigation equivalent to the rail.
#[component]
pub(super) fn MobileNavBar() -> impl IntoView {
    view! {
        <nav class="mobile-nav-bar" aria-label="Primary navigation">
            <div class="mobile-nav-bar-inner">
                <a href="#console-shell" class="active">
                    <span aria-hidden="true">"⌂"</span>
                    "Command"
                </a>
                <a href="#business-logics">
                    <span aria-hidden="true">"⌬"</span>
                    "Operate"
                </a>
                <a href="#workflow-studio">
                    <span aria-hidden="true">"⌘"</span>
                    "Workflow"
                </a>
                <a href="#work-hub">
                    <span aria-hidden="true">"✉"</span>
                    "Comms"
                </a>
                <a href="#identity-workforce-service">
                    <span aria-hidden="true">"⚿"</span>
                    "Identity"
                </a>
                <a href="#cloud-ops-cockpit">
                    <span aria-hidden="true">"◫"</span>
                    "Cloud"
                </a>
                <a href="#resource-panel-audit">
                    <span aria-hidden="true">"▤"</span>
                    "Audit"
                </a>
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
                <span>"Operations"</span>
                <span class="sep" aria-hidden="true">"/"</span>
                <strong aria-current="page">"Control Center"</strong>
            </nav>
            <div class="header-route-strip" aria-label="FD-001 and Oyatie Cloud quick routes">
                <button type="button" class="is-selected" data-header-route="fd001"><span>"FD-001"</span>"Service graph"</button>
                <button type="button" data-header-route="cloud"><span>"Cloud"</span>"Substrate"</button>
                <button type="button" data-header-route="work-hub"><span>"Comms"</span>"Work hub"</button>
                <div class="header-comms-switcher" aria-label="Built-in communications quick routes">
                    <button type="button" class="is-selected" data-header-comms-surface="Messenger">"Messenger"</button>
                    <button type="button" data-header-comms-surface="Mail">"Mail"</button>
                    <button type="button" data-header-comms-surface="Community">"Community"</button>
                </div>
                <button type="button" data-header-route="evidence"><span>"Audit"</span>"Evidence"</button>
                <small data-header-route-status="true">"REC-WF-7741 · local quick routes"</small>
            </div>
            <button class="command-trigger" type="button" data-command-trigger="true" aria-haspopup="dialog">
                <span aria-hidden="true">"⌕"</span>
                <span>"Search actions, objects, workflows"</span>
                <kbd>"⌘K"</kbd>
            </button>
            <div class="header-actions" aria-label="Shell render status">
                // A-10: status badges convey information, not action — use span[role=status]
                <span role="status" class="header-status" aria-label="Render mode: SSR shell">"SSR shell"</span>
                <span role="status" class="header-status muted" aria-label="Hydration mode: Selective WASM islands">"Selective WASM islands"</span>
                <button type="button" class="header-icon" data-header-action="notifications" aria-label="Open notifications">
                    "◔"
                    <span class="header-badge" data-activity-badge="true">"3"</span>
                </button>
                <button type="button" class="header-icon" data-header-action="settings" aria-label="Open settings">"⚙"</button>
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
                        <h1 id="console-title">"Operations · 2026 May, week 19"</h1>
                        <span class="hero-lens-chip">"● Lens: tenant admin · Finance · 1,000 ppl"</span>
                    </div>
                    <p id="console-notice" class="scope-notice" role="note">
                        {shell_scope_notice_text()}
                    </p>
                </div>
                <section class="hero-close-strip" aria-label="FD-001 close command proof">
                    <div>
                        <p class="screen-anchor">"FD-001 CLOSE COMMAND"</p>
                        <strong>"April close proves the product workload on Oyatie Cloud"</strong>
                        <span data-hero-status="true">"Ready · REC-CLOSE-2026-04 · cell-us-east-2 · local command only"</span>
                    </div>
                    <div class="hero-close-actions" aria-label="Close package routes">
                        <button type="button" data-hero-action="close-april">"Stage close"</button>
                        <button type="button" data-hero-action="route-ledger">"Ledger"</button>
                        <button type="button" data-hero-action="route-cloud">"Cloud proof"</button>
                        <button type="button" data-hero-action="route-evidence">"Evidence"</button>
                    </div>
                </section>
                <section class="render-architecture-strip" aria-label="SSR shell and selective WASM hydration model">
                    <article class="selected" data-render-arch-card="ssr">
                        <p class="screen-anchor">"SSR SHELL"</p>
                        <strong>"Fast baseline, service graph visible first"</strong>
                        <span>"Navigation, proof copy, tenant posture, and core dashboards render before island hydration."</span>
                        <button type="button" class="is-selected" data-render-arch-action="ssr">"Show shell"</button>
                    </article>
                    <article data-render-arch-card="islands">
                        <p class="screen-anchor">"SELECTIVE WASM"</p>
                        <strong>"Only interactive product surfaces hydrate"</strong>
                        <span>"Workflow Studio, Work Hub, filters, canvas state, and local drafts become browser-only islands."</span>
                        <button type="button" data-render-arch-action="islands">"Show islands"</button>
                    </article>
                    <article data-render-arch-card="boundary">
                        <p class="screen-anchor">"LOCAL BOUNDARY"</p>
                        <strong>"Visually functional, deliberately unwired"</strong>
                        <span data-render-arch-status="true">"No workflow execution, external send, IAM, billing, deploy, or cloud mutation."</span>
                        <button type="button" data-render-arch-action="boundary">"Show evidence"</button>
                    </article>
                </section>
            </div>
            <div class="hero-side">
                <div class="hero-copy page-actions">
                    <button
                        type="button"
                        data-sidepeek-trigger="new-action"
                        data-sidepeek-title="Create governed action"
                        data-sidepeek-id="ACT-LOCAL-DRAFT"
                        data-sidepeek-desc="Local visual-only action draft. Nothing is persisted or sent."
                        data-sidepeek-owner="Current operator session"
                        data-sidepeek-risk="Draft"
                        data-sidepeek-sla="No live SLA"
                    >
                        "New action"
                    </button>
                    <button type="button" data-command-trigger="true">"Search ⌘K"</button>
                    <button type="button" class="primary" data-hero-action="close-april">"Close April →"</button>
                </div>
            </div>
        </section>
    }
}
