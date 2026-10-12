use super::*;

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_workforce_anchor_board() -> &'static str {
    r#"<div class="workforce-anchor-grid" aria-label="FD-001 and Oyatie Cloud workforce proof"><article class="workforce-anchor-card selected" data-workforce-card="fd001"><p class="screen-anchor">FD-001 WORKFORCE</p><h5>People data powers product delivery</h5><p>Employee directory, payroll eligibility, reviewer mail, community announcements, and onboarding are FD-001 tenant workloads, not separate HR widgets.</p><div class="workforce-anchor-actions"><button type="button" data-workforce-anchor-action="route-payroll">Payroll impact</button><button type="button" data-workforce-anchor-action="route-workflow">Workflow path</button></div></article><article class="workforce-anchor-card" data-workforce-card="cloud"><p class="screen-anchor">OYATIE CLOUD</p><h5>Hosted as a governed tenant surface</h5><p>PIPA boundaries, regional pack gates, role envelopes, audit receipts, and evidence routes prove the substrate can host real workforce tenants.</p><div class="workforce-anchor-actions"><button type="button" data-workforce-anchor-action="route-policy">Policy envelope</button><button type="button" data-workforce-anchor-action="route-audit">Audit trail</button></div></article><article class="workforce-anchor-card" data-workforce-card="lifecycle"><p class="screen-anchor">LIFECYCLE OPS</p><h5>Interactive, local-only employee command</h5><p>Operators can inspect people, stage invites, route leave/time, and brief reviewers while HRIS, auth, payroll, and cloud mutations remain disconnected.</p><div class="workforce-anchor-actions"><button type="button" data-workforce-anchor-action="stage-invite">Stage invite</button><button type="button" data-workforce-anchor-action="route-leave">Leave &amp; time</button></div></article></div><div class="workforce-anchor-footer"><span data-workforce-anchor-status="true">Sample workforce preview · employee readiness unavailable.</span><div class="workforce-anchor-routes" aria-label="Workforce connected routes"><button type="button" data-workforce-anchor-action="route-mail">Reviewer Mail</button><button type="button" data-workforce-anchor-action="route-community">Community update</button><button type="button" data-workforce-anchor-action="route-evidence">Evidence graph</button><button type="button" data-workforce-anchor-action="route-cloud">Cloud cells</button></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_identity_workforce_service(envelope: &TenantRenderEnvelope) -> String {
    let kinds = [
        "auth",
        "sessions",
        "roles",
        "org",
        "employees",
        "onboarding",
    ];
    let tabs: String = kinds.iter().map(|panel| format!(r#"<button type="button" id="identity-tab-{panel}" data-identity-tab="{panel}" role="tab" aria-selected="{selected}" aria-controls="identity-{panel}" class="{class}">{panel} sample</button>"#,
        selected = panel == &"auth", class = if panel == &"auth" { "active" } else { "" })).collect();
    let rail: String = kinds.iter().map(|panel| format!(r#"<button type="button" data-identity-tab="{panel}" class="{class}">{panel} sample</button>"#,
        class = if panel == &"auth" { "active" } else { "" })).collect();
    let panels: String = kinds.iter().map(|panel| {
        let body = if panel == &"employees" { format!("{}{}", static_workforce_anchor_board(),
            super::super::employee_directory::sample_employee_directory().into_view().to_html()) }
        else { account_panel_html(panel) };
        format!(r#"<article id="identity-{panel}" class="identity-panel {class}" data-identity-panel="{panel}" {focus} role="tabpanel" aria-labelledby="identity-tab-{panel}">{body}</article>"#,
            class = if panel == &"auth" { "active" } else { "" },
            focus = if panel == &"employees" { String::new() } else { format!(r#"data-account-focus="panel:{panel}" tabindex="-1""#) })
    }).collect();
    format!(
        r#"<section id="identity-workforce-service" class="identity-workforce-service panel" data-identity-service="true" data-account-source="{source}" data-account-context="{context}" data-account-label="{label}" data-account-hub-source="{hub}" data-account-mode="compatibility" data-account-available="true" aria-labelledby="identity-service-title"><div class="identity-service-head"><div><p class="screen-anchor">SAMPLE ACCOUNT · SETTINGS</p><h3 id="identity-service-title" tabindex="-1" data-account-focus="heading">Sample account, organization and workspace setup</h3></div><div class="identity-service-actions"><span class="status-chip" data-identity-status="true" role="status">Sample examples only · live authority Unknown / unavailable</span><button type="button" data-identity-action="open-audit" data-account-focus="audit">Open existing audit sample</button></div></div><p data-account-reset="true" role="status" hidden>Sample source changed; previous local setup and inspection cleared.</p><div class="identity-service-shell"><aside class="identity-settings-rail" aria-label="Sample account and organization sections"><div data-account-person="true">{person}</div>{rail}</aside><div class="identity-service-main"><div class="identity-tabs" role="tablist" aria-label="Sample account views" aria-orientation="horizontal">{tabs}</div>{panels}</div><aside class="identity-context-rail" aria-label="Sample account provenance" data-account-context-facts="true">{provenance}</aside></div>{templates}</section>"#,
        source = escape(&account_sample_source(envelope)),
        context = envelope.context.id(),
        label = escape(&daily_sample_label(envelope)),
        hub = escape(&HubSample::envelope_source(envelope)),
        person = account_person_html(),
        provenance = account_context_html(),
        templates = account_channel_templates(envelope)
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_finance_commercial_service(envelope: &TenantRenderEnvelope) -> String {
    static_finance_preview(envelope)
}
