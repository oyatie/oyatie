use super::*;

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_resource_inventory_anchor_board() -> &'static str {
    r#"<div class="trust-anchor-board" aria-label="FD-001 resource inventory and Oyatie Cloud substrate proof"><div class="trust-anchor-grid"><article class="trust-anchor-card selected" data-trust-proof-card="resource-fd001"><p class="screen-anchor">FD-001 SERVICE FLEET</p><h5>Microservices are tenant workloads</h5><p>Tenant admin, workflow runner, audit vault, Mail, Messenger, Community, and Intelligence assets are tracked as one FD-001 product fleet.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="link-resource">Link resource</button><button type="button" data-trust-proof-action="route-catalog">Service catalog</button></div></article><article class="trust-anchor-card" data-trust-proof-card="resource-cloud"><p class="screen-anchor">OYATIE CLOUD INVENTORY</p><h5>Substrate owns residency and release posture</h5><p>Each resource shows cell, owner, cost, risk, policy, deployment gate, and audit receipt so the cloud substrate can prove real tenant hosting.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="route-finops">FinOps cost</button><button type="button" data-trust-proof-action="route-gates">Admission gates</button></div></article><article class="trust-anchor-card" data-trust-proof-card="resource-local"><p class="screen-anchor">LOCAL-ONLY RESOURCE OPS</p><h5>Inspect without provider mutation</h5><p>Operators can inspect ownership, route evidence, and preview remediation; no cloud provider, database, deploy, billing, or audit mutation executes.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="route-audit">Audit ledger</button><button type="button" data-trust-proof-action="trace-lineage">Trace lineage</button></div></article></div><div class="trust-anchor-footer"><span data-trust-proof-status="true">Resource inventory ready · FD-001 workload fleet is hosted-proof on Oyatie Cloud with local visual controls only.</span><div class="trust-anchor-routes" aria-label="Resource inventory connected routes"><button type="button" data-trust-proof-action="route-workflow">Workflow</button><button type="button" data-trust-proof-action="route-evidence">Evidence</button><button type="button" data-trust-proof-action="route-mail">Mail</button><button type="button" data-trust-proof-action="route-community">Community</button></div></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_tenant_operations_cockpit(envelope: &TenantRenderEnvelope) -> String {
    format!(
        r#"<section id="cloud-ops-cockpit" class="ops-cockpit panel" aria-labelledby="ops-cockpit-title"><div class="panel-header cockpit-header"><div><p class="eyebrow">Operate</p><h3 id="ops-cockpit-title">Cloud, policy, and FinOps cockpit</h3></div><div class="cockpit-tabs" role="tablist" aria-label="Operations cockpit views" aria-orientation="horizontal"><button type="button" id="cockpit-tab-topology" class="active" data-cockpit-tab="topology" role="tab" aria-selected="true" aria-controls="cockpit-panel-topology">Topology</button><button type="button" id="cockpit-tab-policy" data-cockpit-tab="policy" role="tab" aria-selected="false" aria-controls="cockpit-panel-policy">Policy</button>{finops_tab}</div></div><div class="cockpit-panels">{topology}{policy_preview}{finops_panel}</div></section>"#,
        policy_preview = policy_preview_panel(envelope.clone()).into_view().to_html(),
        topology = cloud_topology_panel(envelope).into_view().to_html(),
        finops_panel = if finance_sample_scope(envelope.context).finops_examples {
            finance_compatible_html(finance_finops_panel(envelope).into_view().to_html())
        } else {
            String::new()
        },
        finops_tab = if finance_sample_scope(envelope.context).finops_examples {
            r#"<button type="button" id="cockpit-tab-finops" data-cockpit-tab="finops" role="tab" aria-selected="false" aria-controls="cockpit-panel-finops">FinOps</button>"#
        } else {
            ""
        },
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_resource_audit_console(envelope: &TenantRenderEnvelope) -> String {
    resource_audit_console(envelope.clone(), Signal::derive(|| true))
        .into_view()
        .to_html()
}

pub(super) fn static_resource_row(row: &ResourceRow) -> String {
    format!(
        r#"<tr data-resource-row="true" data-resource-state="{state}"><td><span class="resource-kind">{kind}</span></td><td><strong>{name}</strong><small>{description}</small></td><td>{region}</td><td>{owner}</td><td><span class="{state_class}">{state}</span></td><td class="numeric">{monthly}</td><td><button type="button" data-sidepeek-trigger="resource" data-sidepeek-title="{name}" data-sidepeek-id="{side_id}" data-sidepeek-desc="{description}" data-sidepeek-owner="{owner}" data-sidepeek-risk="{risk}" data-sidepeek-sla="Inventory staged · no live mutation">Inspect</button></td></tr>"#,
        kind = escape(row.kind),
        name = escape(row.name),
        description = escape(row.description),
        region = escape(row.region),
        owner = escape(row.owner),
        state_class = resource_status_class(row.state),
        state = escape(row.state),
        monthly = escape(row.monthly),
        side_id = escape(row.side_id),
        risk = escape(row.risk),
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_deployment_gate_command_board() -> &'static str {
    r#"<div class="deployment-proof-grid" aria-label="FD-001 and Oyatie Cloud deployment proof"><article class="deployment-proof-card selected" data-deployment-card="fd001"><p class="screen-anchor">FD-001 RELEASE TRAIN</p><h5>Product microservices deploy as tenants</h5><p>Messenger, Mail, Community, Workflow, Ontology, Intelligence, and core ops stay product-first; the gates prove they can run as tenant workloads on the substrate.</p><div class="deployment-card-actions"><button type="button" data-deployment-gate-action="admit-fd001">Admit workload</button><button type="button" data-deployment-gate-action="route-workflow">Workflow runbook</button></div></article><article class="deployment-proof-card" data-deployment-card="cloud"><p class="screen-anchor">OYATIE CLOUD</p><h5>Hyperscaler-grade substrate proof</h5><p>Cell topology, policy sidecars, cosign receipts, ArgoCD app health, rollback posture, and audit-chain freshness must be visible before any promotion claim.</p><div class="deployment-card-actions"><button type="button" data-deployment-gate-action="route-cloud">Inspect cells</button><button type="button" data-deployment-gate-action="route-finops">FinOps guard</button></div></article><article class="deployment-proof-card" data-deployment-card="control"><p class="screen-anchor">CONTROL PLANE</p><h5>Interactive, never wired</h5><p>Operators can simulate gate decisions, seal a release packet, and route reviewer work, while deploy, DNS, registry, billing, and cloud mutations remain disconnected.</p><div class="deployment-card-actions"><button type="button" data-deployment-gate-action="seal-release">Seal packet</button><button type="button" data-deployment-gate-action="route-mail">Reviewer mail</button></div></article></div><div class="deployment-promotion-lane" aria-label="Tenant workload promotion lane"><button type="button" class="active" data-deployment-gate-action="ci-lane"><span>01</span><strong>CI mirror</strong><em>Jenkins parity · 92%</em></button><button type="button" data-deployment-gate-action="attest-lane"><span>02</span><strong>Attest</strong><em>cosign + SBOM · 61%</em></button><button type="button" data-deployment-gate-action="admit-lane"><span>03</span><strong>Admit tenant</strong><em>policy + PIPA · review</em></button><button type="button" data-deployment-gate-action="observe-lane"><span>04</span><strong>Observe</strong><em>SLO + audit emit · 48%</em></button></div>"#
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_deployment_gate(gate: &DeploymentGate) -> String {
    format!(
        r#"<article class="gate-card"><div><span class="{state_class}">{state}</span><h5>{label}</h5><p>{detail}</p></div><span class="gate-progress" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow="{progress_num}" aria-label="{label} gate progress: {progress}" style="--bar: {progress}"><em aria-hidden="true">{progress}</em></span><div class="gate-card-actions"><button type="button" data-gate-action="attach-evidence">Attach evidence</button><button type="button" data-gate-action="open-evidence">Evidence</button><button type="button" data-gate-action="route-owner">Owner route</button></div></article>"#,
        state_class = resource_status_class(gate.state),
        state = escape(gate.state),
        label = escape(gate.label),
        detail = escape(gate.detail),
        progress = escape(gate.progress),
        progress_num = escape(gate.progress.trim_end_matches('%')),
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_service_catalog(envelope: &TenantRenderEnvelope) -> String {
    service_catalog_panel(envelope.clone(), Signal::derive(|| true))
        .into_view()
        .to_html()
}

#[cfg(test)]
mod cloud_tests {
    use super::*;

    #[test]
    fn cloud_resources_native_vectors_parity_and_gate_truth() {
        let owner = Owner::new();
        owner.with(|| {
            for context in OperatorContext::ALL {
                let envelope = server_derived_envelope(context);
                let topology = cloud_topology_panel(&envelope).into_view().to_html();
                let native = tenant_operations_cockpit(envelope.clone())
                    .into_view()
                    .to_html();
                let compatible = static_tenant_operations_cockpit(&envelope);
                assert!(native.contains(&topology) && compatible.contains(&topology));
                let resources = resource_audit_console(envelope.clone(), Signal::derive(|| true))
                    .into_view()
                    .to_html();
                assert_eq!(resources, static_resource_audit_console(&envelope));
                let body = format!("{native}{resources}");
                let source: serde_json::Value =
                    serde_json::from_str(&cloud_resources_source(&envelope)).unwrap();
                assert_eq!(source[3].as_array().unwrap().len(), 17);
                for (family, count) in
                    [("region", 3), ("workload", 4), ("resource", 6), ("gate", 4)]
                {
                    assert_eq!(
                        source[3]
                            .as_array()
                            .unwrap()
                            .iter()
                            .filter(|record| record[0]
                                .as_str()
                                .unwrap()
                                .starts_with(&format!("{family}:")))
                            .count(),
                        count
                    );
                }
                assert_eq!(body.matches("data-cloud-select=\"").count(), 17);
                assert_eq!(body.matches("data-resource-search=\"").count(), 1);
                assert_eq!(body.matches("data-resource-filter=\"").count(), 4);
                assert_eq!(body.matches("data-resource-action=\"").count(), 2);
                assert_eq!(body.matches("data-cloud-inspect-field=\"").count(), 27);
                assert_eq!(
                    body.matches("data-cloud-route=\"").count(),
                    source[4].as_array().unwrap().len()
                );
                for record in source[3].as_array().unwrap() {
                    let token = record[0].as_str().unwrap();
                    assert_eq!(
                        serde_json::from_str::<serde_json::Value>(&cloud_resources_record(token))
                            .unwrap(),
                        record[1]
                    );
                    assert_eq!(
                        body.matches(&format!("data-cloud-select=\"{token}\""))
                            .count(),
                        1
                    );
                }
                for (gate, progress) in deployment_gates()
                    .into_iter()
                    .zip(["92%", "74%", "61%", "48%"])
                {
                    assert_eq!(gate.progress, progress);
                    assert!(body.contains(gate.detail));
                    assert!(body.contains(&format!(
                        "Sample {} gate progress: {}; not measured readiness",
                        gate.label, gate.progress
                    )));
                    assert!(body.contains(&format!(
                        "aria-valuenow=\"{}\"",
                        gate.progress.trim_end_matches('%')
                    )));
                }
                let clinician = context == OperatorContext::HealthcareClinician;
                assert_eq!(native.contains("id=\"cockpit-panel-finops\""), !clinician);
                assert_eq!(
                    compatible.contains("id=\"cockpit-panel-finops\""),
                    !clinician
                );
                assert_eq!(
                    source[4].as_array().unwrap().len(),
                    if clinician { 28 } else { 31 }
                );
                println!(
                    "CLOUD_RESOURCES_SOURCE {}",
                    serde_json::to_string(&(context.id(), source)).unwrap()
                );
                println!(
                    "CLOUD_RESOURCES_BODY {}",
                    serde_json::to_string(&(context.id(), body)).unwrap()
                );
            }
        });
        owner.cleanup();
    }
}
