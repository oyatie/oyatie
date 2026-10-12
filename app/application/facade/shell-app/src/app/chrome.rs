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
                    <span>"Control Center"</span>
                </div>
                <code>"v0.1"</code>
            </div>
            <section class="rail-proof-card" aria-label="Anonymous sample destinations">
                <p>"ANONYMOUS SAMPLE VIEWS"</p>
                <strong>"Explore shell samples"</strong>
                <span>"Catalog, cloud, evidence and Work Hub samples. Live operations are unavailable."</span>
                <small data-rail-status="true">"Navigation only · no live operation"</small>
                <div class="rail-proof-actions" aria-label="Sample destinations">
                    <a href="#service-catalog">"Catalog samples"</a>
                    <a href="#cloud-ops-cockpit">"Cloud samples"</a>
                    <a href="#evidence-spine">"Evidence samples"</a>
                    <a href="#work-hub">"Work hub samples"</a>
                </div>
            </section>
            <ShellNavigation />
            <div class="workspace-switch">
                <span class="workspace-avatar" aria-hidden="true">"S"</span>
                <div>
                    <strong>"Sample workspace"</strong>
                    <span>"Anonymous preview"</span>
                </div>
            </div>
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
                <span>"Sample views"</span>
                <span class="sep" aria-hidden="true">"/"</span>
                <strong aria-current="page">"Control Center"</strong>
            </nav>
            <div class="header-route-strip" aria-label="Sample quick routes">
                <a href="#service-catalog"><span>"Catalog"</span>" samples"</a>
                <a href="#cloud-ops-cockpit"><span>"Cloud"</span>" samples"</a>
                <a href="#work-hub"><span>"Work hub"</span>" samples"</a>
                <a href="#evidence-spine"><span>"Evidence"</span>" samples"</a>
                <small data-header-route-status="true">"Navigation only · no live operation"</small>
            </div>
            <a class="command-trigger" href="#catalog-sample-search">
                <span aria-hidden="true">"⌕"</span>
                <span>"Search Catalog samples"</span>
            </a>
            <div class="header-actions" aria-label="Sample tools">
                <a class="header-sign-in" href="#oyatie-sign-in">"Sign in"</a>
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
mod tests {
    use super::*;

    #[test]
    fn persistent_chrome_is_anonymous_and_keeps_owned_sample_destinations() {
        for context in OperatorContext::ALL {
            for available in [true, false] {
                let owner = Owner::new();
                let (chrome, dashboard, tools) = owner.with(|| {
                    let envelope = server_derived_envelope(context);
                    let workflow = RwSignal::new(WorkflowSample::from_envelope(&envelope));
                    let hub = RwSignal::new(HubSample::from_envelope(&envelope));
                    let (active, set_active) = signal(ProductSurface::Workflow);
                    (
                        view! { <ShellRail /> <ShellHeader /> <HeroPanel /> }.to_html(),
                        dashboard_view(
                            envelope,
                            workflow,
                            active,
                            set_active,
                            hub,
                            Signal::derive(move || available),
                        )
                        .into_view()
                        .to_html(),
                        view! { <SignInPanel /> <UtilityPanels /> }.to_html(),
                    )
                });
                owner.cleanup();
                assert!(chrome.contains("<strong>Sample workspace</strong>"));
                assert!(chrome.contains("id=\"console-title\">Operations samples</h1>"));
                assert!(chrome.contains("Anonymous preview"));
                assert!(chrome.contains(shell_scope_notice_text()));
                assert_eq!(chrome.matches("<button ").count(), 2);
                assert_eq!(chrome.matches("data-activity-badge=").count(), 1);
                for removed in [
                    "data-rail-comms-surface=",
                    "data-header-comms-surface=",
                    "data-hero-action=",
                    "data-render-arch-action=",
                    "data-render-arch-card=",
                    "data-sidepeek-trigger=\"new-action\"",
                    "data-hero-status=",
                    "Northwind",
                    "1,000 ppl",
                    "2026 May",
                    "REC-CLOSE-2026-04",
                    "FD-001 CLOSE COMMAND",
                    "Current operator session",
                    "SSR shell",
                    "Selective WASM",
                    "<em>73</em>",
                    "<em>18</em>",
                ] {
                    assert!(
                        !chrome.contains(removed),
                        "{context:?}/{available}: {removed}"
                    );
                }
                for target in [
                    "service-catalog",
                    "cloud-ops-cockpit",
                    "evidence-spine",
                    "work-hub",
                ] {
                    assert!(chrome.contains(&format!("href=\"#{target}\"")));
                    assert_eq!(dashboard.matches(&format!("id=\"{target}\"")).count(), 1);
                }
                assert_eq!(chrome.matches("href=\"#catalog-sample-search\"").count(), 2);
                assert_eq!(dashboard.matches("id=\"catalog-sample-search\"").count(), 1);
                for dialog in ["notification-center", "settings-center"] {
                    assert!(
                        chrome.contains(&format!("commandfor=\"{dialog}\" command=\"show-modal\""))
                    );
                    assert_eq!(tools.matches(&format!("id=\"{dialog}\"")).count(), 1);
                }
                assert!(chrome.contains(&format!(
                    "aria-label=\"Open sample notifications, {} unread\"",
                    notification_unread_count()
                )));
                assert!(chrome.contains("href=\"#oyatie-sign-in\""));
                assert_eq!(tools.matches("id=\"oyatie-sign-in\"").count(), 1);
            }
        }
    }

    #[test]
    fn ordinary_shell_routes_resolve_owned_sample_views_across_contexts() {
        let destinations = [
            ("service-catalog", "Catalog samples"),
            ("cloud-ops-cockpit", "Cloud samples"),
            ("evidence-spine", "Evidence samples"),
            ("work-hub", "Work hub samples"),
        ];
        for context in OperatorContext::ALL {
            for available in [true, false] {
                let owner = Owner::new();
                let (rail, header, dashboard) = owner.with(|| {
                    let envelope = server_derived_envelope(context);
                    let workflow = RwSignal::new(WorkflowSample::from_envelope(&envelope));
                    let hub = RwSignal::new(HubSample::from_envelope(&envelope));
                    let (active, set_active) = signal(ProductSurface::Workflow);
                    (
                        ShellRail().into_view().to_html(),
                        ShellHeader().into_view().to_html(),
                        dashboard_view(
                            envelope,
                            workflow,
                            active,
                            set_active,
                            hub,
                            Signal::derive(move || available),
                        )
                        .into_view()
                        .to_html(),
                    )
                });
                owner.cleanup();
                let rail_routes = rail
                    .split_once("class=\"rail-proof-actions\"")
                    .unwrap()
                    .1
                    .split_once("</div>")
                    .unwrap()
                    .0;
                let header_routes = header
                    .split_once("class=\"header-route-strip\"")
                    .unwrap()
                    .1
                    .split_once("<a class=\"command-trigger\"")
                    .unwrap()
                    .0;
                for routes in [rail_routes, header_routes] {
                    assert_eq!(
                        routes.matches("<a ").count(),
                        4,
                        "{context:?}/{available}: four ordinary native routes"
                    );
                    for (target, label) in destinations {
                        let marker = format!("href=\"#{target}\"");
                        assert_eq!(routes.matches(&marker).count(), 1);
                        let start = routes[..routes.find(&marker).unwrap()]
                            .rfind("<a ")
                            .unwrap();
                        let anchor = routes[start..].split_once("</a>").unwrap().0;
                        let body = anchor.split_once('>').unwrap().1;
                        assert_eq!(
                            body.replace("<span>", "")
                                .replace("</span>", "")
                                .replace("<!---->", "")
                                .trim(),
                            label
                        );
                        for claim in [
                            "is-selected",
                            "aria-current",
                            "data-rail-proof-action",
                            "data-header-route",
                            "FD-001",
                            "Audit",
                            "proof",
                            "REC-",
                        ] {
                            assert!(
                                !anchor.contains(claim),
                                "ordinary route does not claim {claim}"
                            );
                        }
                    }
                    assert!(
                        !routes.contains("data-rail-proof-action=")
                            && !routes.contains("data-header-route=")
                    );
                }
                for (target, _) in destinations {
                    assert_eq!(dashboard.matches(&format!("id=\"{target}\"")).count(), 1);
                }
                assert!(dashboard.contains(&format!("data-catalog-available=\"{available}\"")));
                assert!(dashboard.contains(&format!("data-ae-source-available=\"{available}\"")));
                assert!(dashboard.contains("Live Audit query, review, export, attachment, sealing and verification are unavailable."));
                assert!(dashboard.contains(
                    "Server sample preview; local controls require the matching WebAssembly island."
                ));
                assert_eq!(rail.matches("data-rail-comms-surface=").count(), 0);
                assert_eq!(header.matches("data-header-comms-surface=").count(), 0);
                for loading in [true, false] {
                    let owner = Owner::new();
                    let missing = owner.with(|| loading_state(loading).into_view().to_html());
                    owner.cleanup();
                    for (target, _) in destinations {
                        assert!(!missing.contains(&format!("id=\"{target}\"")));
                    }
                    assert!(missing.contains(sample_context_notice_text()));
                }
            }
        }
    }

    #[test]
    fn collapsed_navigation_reuses_the_complete_rail_inventory() {
        let owner = Owner::new();
        let (inventory, rail, mobile) = owner.with(|| {
            (
                ShellNavigation().into_view().to_html(),
                ShellRail().into_view().to_html(),
                MobileNavBar().into_view().to_html(),
            )
        });
        owner.cleanup();
        assert_eq!(inventory.matches("href=\"#").count(), 21);
        assert!(rail.contains(&inventory));
        assert!(mobile.contains(&inventory));
        for group in ["commercial", "finops", "filing"] {
            assert_eq!(
                inventory
                    .matches(&format!("data-finance-rail-slot=\"{group}\""))
                    .count(),
                1
            );
        }
        assert_eq!(
            inventory.matches("data-governance-rail=\"true\"").count(),
            2
        );
        let quick = mobile
            .split("<details class=\"mobile-nav-more\">")
            .next()
            .unwrap();
        assert!(quick.contains("data-governance-rail=\"true\" href=\"#business-logics\""));
        assert!(mobile.contains("<details class=\"mobile-nav-more\"><summary>"));
        assert!(!mobile.contains("<details class=\"mobile-nav-more\" open"));
    }

    #[test]
    fn shell_destination_links_do_not_claim_a_permanent_current_location() {
        let owner = Owner::new();
        let (rail, mobile) = owner.with(|| {
            (
                ShellRail().into_view().to_html(),
                MobileNavBar().into_view().to_html(),
            )
        });
        owner.cleanup();
        for html in [rail, mobile] {
            assert!(!html.contains("class=\"active\""));
            assert!(!html.contains("class=\"rail-nav active\""));
            assert!(!html.contains("aria-current="));
        }
        assert!(!static_dashboard_html().contains("class=\"rail-nav active\""));
    }
}
