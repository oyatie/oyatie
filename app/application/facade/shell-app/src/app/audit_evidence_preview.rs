use super::*;

pub(super) const AE_UNAVAILABLE: &str = "Live Audit query, review, export, attachment, sealing and verification are unavailable. No receipt, draft, message or file is created.";

pub(super) fn ae_available(available: Signal<bool>) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        available.try_get().unwrap_or(false)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        available.try_get_untracked().unwrap_or(false)
    }
}

pub(super) fn ae_records() -> Vec<(String, Vec<(&'static str, &'static str)>)> {
    let mut records = Vec::new();
    for (state, receipt, title, body, source, owner, sla) in evidence_events() {
        records.push((
            format!("event:{receipt}"),
            vec![
                ("State", state),
                ("Receipt", receipt),
                ("Title", title),
                ("Body", body),
                ("Source", source),
                ("Owner", owner),
                ("SLA", sla),
            ],
        ));
    }
    for (key, label, value, description) in object_nodes() {
        records.push((
            format!("node:{key}"),
            vec![
                ("Key", key),
                ("Label", label),
                ("Value", value),
                ("Description", description),
            ],
        ));
    }
    for row in audit_receipts() {
        records.push((
            format!("audit:{}", row.receipt),
            vec![
                ("Time", row.time),
                ("Event", row.event),
                ("Actor", row.actor),
                ("Receipt", row.receipt),
                ("State", row.severity),
            ],
        ));
    }
    for (key, title, receipt, route, owner, state) in receipt_sources() {
        records.push((
            format!("source:{key}"),
            vec![
                ("Key", key),
                ("Title", title),
                ("Receipt", receipt),
                ("Route", route),
                ("Owner", owner),
                ("State", state),
            ],
        ));
    }
    records
}

pub(super) fn ae_record(token: &str) -> String {
    let fields = ae_records()
        .into_iter()
        .find(|(id, _)| id == token)
        .expect("native sample record exists")
        .1;
    serde_json::to_string(&fields).expect("native sample fields serialize")
}

fn ae_route_target(family: &str, action: &str) -> Option<(&'static str, Option<&'static str>)> {
    let route = match family {
        "trust" => action.strip_prefix("route-")?,
        "audit" if action == "open-evidence" => "evidence",
        "audit" => action.strip_prefix("route-")?,
        "intel" | "stitch" => action,
        _ => return None,
    };
    Some(match route {
        "workflow" => ("workflow-studio", None),
        "audit" => ("resource-panel-audit", None),
        "evidence" => ("evidence-spine", None),
        "graph" => ("object-graph", None),
        "catalog" => ("service-catalog", None),
        "cloud" => ("cloud-ops-cockpit", None),
        "policy" => ("cockpit-panel-policy", None),
        "inventory" => ("resource-panel-inventory", None),
        "gates" => ("resource-panel-gates", None),
        "finops" => ("cockpit-panel-finops", None),
        "mail" => ("work-hub", Some("Mail")),
        "community" => ("work-hub", Some("Community")),
        "messenger" => ("work-hub", Some("Messenger")),
        _ => return None,
    })
}

pub(super) fn ae_routes(
    context: OperatorContext,
) -> Vec<(String, &'static str, Option<&'static str>)> {
    let families: &[(&str, &[&str])] = &[
        (
            "trust",
            &[
                "route-workflow",
                "route-cloud",
                "route-gates",
                "route-mail",
                "route-community",
                "route-finops",
                "route-inventory",
                "route-graph",
                "route-policy",
                "route-catalog",
                "route-evidence",
            ],
        ),
        (
            "audit",
            &[
                "open-evidence",
                "route-mail",
                "route-cloud",
                "route-gates",
                "route-policy",
                "route-workflow",
                "route-community",
            ],
        ),
        (
            "intel",
            &["audit", "workflow", "mail", "community", "messenger"],
        ),
        (
            "stitch",
            &["workflow", "cloud", "mail", "community", "graph", "gates"],
        ),
    ];
    let mut routes: Vec<_> = families
        .iter()
        .flat_map(|(family, actions)| {
            actions.iter().map(move |action| {
                let (target, channel) =
                    ae_route_target(family, action).expect("native sample route is admitted");
                (format!("{family}:{action}"), target, channel)
            })
        })
        .collect();
    for (family, action, occurrence) in [
        ("trust", "route-cloud", "graph"),
        ("trust", "route-policy", "graph"),
        ("trust", "route-mail", "graph"),
        ("trust", "route-workflow", "graph"),
        ("trust", "route-inventory", "graph"),
        ("trust", "route-community", "graph"),
        ("trust", "route-finops", "graph"),
        ("audit", "open-evidence", "footer"),
        ("intel", "mail", "lineage"),
        ("intel", "community", "lineage"),
        ("intel", "audit", "lineage"),
    ] {
        let (target, channel) =
            ae_route_target(family, action).expect("native route occurrence is admitted");
        routes.push((format!("{family}:{action}:{occurrence}"), target, channel));
    }
    routes.retain(|(_, target, _)| {
        context != OperatorContext::HealthcareClinician || *target != "cockpit-panel-finops"
    });
    routes
}

pub(super) fn ae_source(envelope: &TenantRenderEnvelope) -> String {
    serde_json::to_string(&(
        envelope.context.id(),
        daily_sample_label(envelope),
        HubSample::envelope_source(envelope),
        &envelope.workflow.name,
        &envelope.workflow.goal,
        ae_records(),
        ae_routes(envelope.context),
    ))
    .expect("complete Audit and Evidence sample identity serializes")
}

pub(super) fn ae_action(
    family: &'static str,
    action: &'static str,
    label: &'static str,
) -> impl IntoView {
    ae_action_at(family, action, label, "")
}

pub(super) fn ae_action_at(
    family: &'static str,
    action: &'static str,
    label: &'static str,
    occurrence: &'static str,
) -> impl IntoView {
    let key = if occurrence.is_empty() {
        format!("{family}:{action}")
    } else {
        format!("{family}:{action}:{occurrence}")
    };
    let evidence = (family == "evidence").then_some(action);
    let trust = (family == "trust").then_some(action);
    let intel = (family == "intel").then_some(action);
    let audit = (family == "audit").then_some(action);
    let stitch = (family == "stitch").then_some(action);
    if let Some((target, channel)) = ae_route_target(family, action) {
        view! { <a class="ae-route" href=format!("#{target}") data-ae-route=target data-ae-route-key=key.clone() data-ae-channel=channel
            data-ae-focus=key data-evidence-action=evidence data-trust-proof-action=trust data-intel-action=intel
            data-audit-anchor-action=audit data-receipt-stitching-action=stitch>{label}</a> }.into_any()
    } else {
        view! { <button type="button" disabled data-ae-unavailable="true" data-ae-effect=key title=AE_UNAVAILABLE
            data-evidence-action=evidence data-trust-proof-action=trust data-intel-action=intel data-audit-anchor-action=audit
            data-receipt-stitching-action=stitch>{label}</button> }.into_any()
    }
}

pub(super) fn ae_finops(context: OperatorContext, occurrence: &'static str) -> impl IntoView {
    if context == OperatorContext::HealthcareClinician {
        let key = if occurrence.is_empty() {
            "trust:route-finops"
        } else {
            "trust:route-finops:graph"
        };
        view! { <button type="button" disabled data-ae-unavailable="true" data-ae-effect=key
            title="FinOps is unavailable in this sample context; no output created.">"FinOps unavailable in this sample"</button> }.into_any()
    } else {
        ae_action_at("trust", "route-finops", "FinOps", occurrence).into_any()
    }
}

pub(super) fn ae_notice() -> impl IntoView {
    view! {
        <p data-ae-availability="true" role="status">"Anonymous sample only. No tenant or role is verified; live Audit readiness is unknown."</p>
        <p data-ae-fixture="true">"These are illustrative Northwind and April examples, including their original sample times. They are not facts about the selected sample tenant. State names such as sealed are fixture labels, not verified or immutable proof."</p>
        <p data-ae-unavailable-reason="true">{AE_UNAVAILABLE}</p>
        <p data-ae-route-status="true" role="status">"Existing sample routes only; no action or receipt performed."</p>
        <p data-ae-reset="true" role="status" hidden>"Sample source changed; prior local query, selection and inspection cleared."</p>
    }
}

pub(super) fn ae_inspection() -> impl IntoView {
    view! {
        <section class="ae-inspection" data-ae-inspection="true" aria-label="Selected illustrative record" hidden>
            <h4 data-ae-inspect-title="true">"Sample record"</h4>
            <p>"Illustrative fields only; no live verification or operation."</p>
            <dl>{(0..7).map(|index| view! { <div data-ae-inspect-field=index hidden><dt data-ae-inspect-label="true"></dt><dd data-ae-inspect-value="true"></dd></div> }).collect_view()}</dl>
            <button type="button" data-ae-close="true" data-ae-focus="inspection-close">"Close sample inspection"</button>
        </section>
    }
}

pub(super) fn ae_hub_templates(envelope: &TenantRenderEnvelope) -> impl IntoView + use<> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        view! { {HubChannel::ALL.into_iter().map(|channel| view! {
            <template data-ae-hub-template=channel.label() inner_html=communication_hub_html(envelope, channel)></template>
        }).collect_view()} }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = envelope;
        view! {}
    }
}
