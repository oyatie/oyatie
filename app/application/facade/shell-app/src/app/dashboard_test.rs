use super::*;

#[test]
fn sample_status_distinguishes_requested_and_displayed_sources() {
    let source = server_derived_envelope(OperatorContext::TenantAdmin);
    for (loading, failed, expected) in [
        (true, false, "Loading the requested sample."),
        (false, true, "Could not load the requested sample."),
        (false, false, "The returned sample differs from the request"),
    ] {
        let status = sample_context_status(
            OperatorContext::CorporateOffice,
            Some(&source),
            loading,
            failed,
            true,
        );
        assert!(status.contains("Requested sample: Corporate office."));
        assert!(status.contains(&source.tenant_name) && status.contains(&source.role_name));
        assert!(status.contains("(Tenant admin).") && status.contains(expected));
        assert_eq!(
            status.contains("Retained sample displayed:"),
            loading || failed
        );
        assert!(!status.contains("Select the current sample again to refresh."));
    }
    let ready = sample_context_status(source.context, Some(&source), false, false, true);
    assert!(ready.contains("Select the current sample again to refresh."));
    for (loading, failed, expected) in [
        (true, false, "Loading the requested sample."),
        (false, true, "Could not load the requested sample."),
        (false, false, "The requested sample is unavailable."),
    ] {
        let status = sample_context_status(source.context, None, loading, failed, true);
        assert!(status.contains("No sample is displayed.") && status.contains(expected));
    }
}

#[test]
fn ssr_sample_selector_does_not_advertise_an_unbound_action() {
    Owner::new().with(|| {
        let html = DashboardIsland().into_view().to_html();
        let selector = html
            .split_once("id=\"sample-context-notice\"")
            .unwrap()
            .1
            .split_once("</section>")
            .unwrap()
            .0;
        assert!(selector.contains(sample_context_notice_text()));
        assert!(
            selector
                .contains("Sample switching becomes available when the interactive preview loads.")
        );
        assert!(!selector.contains("again to refresh") && !selector.contains("again to retry"));
        assert_eq!(
            selector.matches("data-sample-context=\"").count(),
            OperatorContext::ALL.len()
        );
        assert_eq!(
            selector.matches(" disabled").count(),
            OperatorContext::ALL.len()
        );
        assert_eq!(selector.matches("aria-pressed=\"true\"").count(), 1);
        assert!(!html.contains("Loading permitted dashboard"));
        assert!(
            !loading_state(true)
                .into_view()
                .to_html()
                .contains("permitted")
        );
        assert!(
            loading_state(false)
                .into_view()
                .to_html()
                .contains("Anonymous sample unavailable")
        );
    });
}
