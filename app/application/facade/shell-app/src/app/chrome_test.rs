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
