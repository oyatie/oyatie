use super::*;

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_command_center_workbench(envelope: &TenantRenderEnvelope) -> String {
    format!(
        r#"<section id="command-center-workbench" class="command-center-workbench" aria-labelledby="command-workbench-title" data-inbox-source="{}" data-inbox-context="{}" data-inbox-label="{}" data-inbox-hub-source="{}" data-inbox-mode="ssr" data-inbox-available="{}">{}</section>"#,
        escape(&inbox_sample_source(envelope)),
        envelope.context.id(),
        escape(&daily_sample_label(envelope)),
        escape(&HubSample::envelope_source(envelope)),
        inbox_sample_valid(envelope),
        inbox_body_html(envelope),
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_tenant_rbac_board(envelope: &TenantRenderEnvelope) -> String {
    format!(
        r#"<section class="tenant-rbac-board" aria-labelledby="service-board-title" id="business-logics"><div class="service-board-head"><div><p class="screen-anchor">SAMPLE SERVICE GRAPH</p><h3 id="service-board-title">Sample corporate operations graph</h3></div><span class="status-chip success">{module_count} sample service cards</span></div>{governance_preview}{payroll_card}{filing_card}<article id="employee-directory" class="service-card employee-card"><p class="screen-anchor">EMPLOYEES</p><h4>Employee directory</h4><p class="service-card-brief">Identity and workforce data remain role-visible while FD-001 tenant workloads prove policy envelopes.</p><div class="service-people-stats"><span><strong>118</strong><small>employees</small></span><span><strong>109</strong><small>active</small></span><span><strong>5</strong><small>probation watch</small></span></div><table class="employee-mini-table"><thead><tr><th>Name</th><th>Role</th><th>Team</th><th>Status</th></tr></thead><tbody><tr><td>이재현 Jaehyun Lee</td><td>CEO</td><td>Office</td><td>활성</td></tr><tr><td>최유나 Yuna Choi</td><td>CFO</td><td>Finance</td><td>활성</td></tr><tr><td>박서준 Seojun Park</td><td>VP Engineering</td><td>Infrastructure</td><td>활성</td></tr><tr><td>김지영 Jiyoung Kim</td><td>Manager</td><td>Infrastructure</td><td>수습</td></tr></tbody></table><div class="service-card-actions"><button type="button" data-service-action="employee-identity">Identity service</button><button type="button" data-service-action="employee-onboarding">Onboarding</button><button type="button" data-service-action="employee-policy">Policy</button><button type="button" data-service-action="employee-mail">Mail reviewer</button></div></article></section>"#,
        module_count = envelope.modules.len(),
        governance_preview = static_governance_preview(envelope),
        payroll_card = static_payroll_preview_card(),
        filing_card = if finance_sample_scope(envelope.context).filing_examples {
            finance_compatible_html(finance_filing_card(envelope).into_view().to_html())
        } else {
            String::new()
        },
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_operator_intelligence_strip(envelope: &TenantRenderEnvelope) -> String {
    operator_intelligence_strip(envelope.clone(), Signal::derive(|| true))
        .into_view()
        .to_html()
}

pub(super) fn static_ontology_command_console(
    facts: &[OntologyFact],
    context: OperatorContext,
) -> String {
    let fact_cards = facts
        .iter()
        .enumerate()
        .map(|(index, fact)| static_ontology_fact_card(index, fact))
        .collect::<String>();

    format!(
        r#"<div class="ontology-command-console" data-ontology-console="true" data-ontology-context="{context}"><div class="ontology-console-head"><div><p class="screen-anchor">ONTOLOGY · FD-001 TENANT WORKLOAD MAP</p><h4>What exists, who can see it, and where it runs</h4><span>Typed entities connect FD-001 tenant workload delivery to Oyatie Cloud cells, policy envelopes, workflow outputs, and evidence receipts.</span></div><div class="ontology-console-actions"><span class="status-chip success" data-ontology-status="true">{fact_count} facts · 7 workload nodes · local graph</span><button type="button" data-ontology-action="lineage">Trace lineage</button><button type="button" data-ontology-action="policy">Policy view</button><button type="button" data-ontology-action="evidence">Evidence</button></div></div><div class="ontology-topology-grid" aria-label="FD-001 tenant workload ontology graph"><button type="button" class="ontology-node root selected" data-ontology-node="Tenant" data-node-route="workload" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Tenant" data-sidepeek-id="ONT-TENANT" data-sidepeek-desc="Tenant owns the permitted FD-001 module set and runtime envelope." data-sidepeek-owner="Tenant admin" data-sidepeek-risk="Visible" data-sidepeek-sla="Local graph only"><span>TENANT</span><strong>Tenant Admin</strong><em>owns envelope</em></button><button type="button" class="ontology-node workload" data-ontology-node="FD-001 Workloads" data-node-route="workflow" data-sidepeek-trigger="ontology-node" data-sidepeek-title="FD-001 workload set" data-sidepeek-id="ONT-FD001" data-sidepeek-desc="Core FD-001 microservices are represented as tenant workloads for dogfood validation." data-sidepeek-owner="Product delivery" data-sidepeek-risk="P0" data-sidepeek-sla="Dogfood proving loop"><span>FD-001</span><strong>Microservice workloads</strong><em>product goal</em></button><button type="button" class="ontology-node cloud" data-ontology-node="Oyatie Cloud" data-node-route="cloud" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Oyatie Cloud substrate" data-sidepeek-id="ONT-CLOUD" data-sidepeek-desc="Hyperscaler-grade substrate hosts dogfood tenant workloads and exposes cell posture." data-sidepeek-owner="Cloud substrate" data-sidepeek-risk="Substrate proof" data-sidepeek-sla="99.95 target staged"><span>CLOUD</span><strong>Cell substrate</strong><em>hosts tenants</em></button><button type="button" class="ontology-node workflow" data-ontology-node="Workflow" data-node-route="workflow" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Workflow runtime" data-sidepeek-id="ONT-WORKFLOW" data-sidepeek-desc="Workflow coordinates payroll close, approval, comms outputs, and receipts." data-sidepeek-owner="Workflow Studio" data-sidepeek-risk="Governed" data-sidepeek-sla="4.0h gate"><span>FLOW</span><strong>Workflow</strong><em>orchestrates</em></button><button type="button" class="ontology-node comms" data-ontology-node="Built-in Comms" data-node-route="mail" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Built-in communications" data-sidepeek-id="ONT-COMMS" data-sidepeek-desc="Messenger, Mail, and Community receive workflow outputs without external send." data-sidepeek-owner="Work Hub" data-sidepeek-risk="Local only" data-sidepeek-sla="No backend send"><span>COMMS</span><strong>Messenger · Mail · Community</strong><em>outputs</em></button><button type="button" class="ontology-node evidence" data-ontology-node="Evidence" data-node-route="evidence" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Evidence spine" data-sidepeek-id="ONT-EVIDENCE" data-sidepeek-desc="Receipts bind workload state, approvals, messages, and deployment gates." data-sidepeek-owner="Audit spine" data-sidepeek-risk="Immutable staged" data-sidepeek-sla="Sealed draft"><span>AUDIT</span><strong>Evidence spine</strong><em>proves</em></button><button type="button" class="ontology-node policy" data-ontology-node="Policy" data-node-route="identity" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Policy envelope" data-sidepeek-id="ONT-POLICY" data-sidepeek-desc="Role, data-class, residency, and autonomy ceilings decide visibility and action eligibility." data-sidepeek-owner="Governance" data-sidepeek-risk="Guardrail" data-sidepeek-sla="Human review"><span>POLICY</span><strong>Access envelope</strong><em>permits</em></button></div><div class="ontology-fact-matrix" aria-label="Current permitted ontology facts">{fact_cards}</div><div class="ontology-proof-rail" aria-label="Substrate proof contract"><article><p class="screen-anchor">SUBSTRATE PROOF</p><strong>FD-001 runs as tenant workloads before service claim</strong><span>Cloud cells, policy envelopes, and evidence receipts prove the substrate can host real production tenants.</span></article><article><p class="screen-anchor">VISIBILITY</p><strong>Role + data-class gates</strong><span>Tenant admin can inspect posture; hidden modules remain server-derived, not client hidden.</span></article><article><p class="screen-anchor">GRAPH STATUS</p><strong data-ontology-detail="true">Tenant selected · workload lineage visible</strong><span>Click nodes or facts to stage local graph state.</span></article></div></div>"#,
        fact_count = facts.len(),
        fact_cards = fact_cards,
        context = context.id(),
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_ontology_fact_card(index: usize, fact: &OntologyFact) -> String {
    format!(
        r#"<article class="ontology-fact-card" data-ontology-fact="true"><span class="status-chip">FACT-{index:02}</span><strong>{entity}</strong><em>{relation}</em><p>{reason}</p><div><button type="button" data-ontology-action="inspect-fact">Inspect</button><button type="button" data-ontology-action="route-workflow">Workflow</button></div></article>"#,
        index = index + 1,
        entity = escape(&fact.entity),
        relation = escape(&fact.relation),
        reason = escape(&fact.access_reason),
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_ontology(fact: &OntologyFact) -> String {
    format!(
        "<li><strong>{}</strong><span>{}</span><p>{}</p></li>",
        escape(&fact.entity),
        escape(&fact.relation),
        escape(&fact.access_reason)
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_suggestion(suggestion: &IntelligenceSuggestion) -> String {
    format!(
        "<li><strong>{}</strong><p>{}</p><span>{}</span></li>",
        escape(&suggestion.title),
        escape(&suggestion.body),
        escape(&suggestion.guardrail)
    )
}

#[cfg(test)]
mod audit_evidence_tests {
    use super::*;

    #[test]
    fn audit_evidence_standalone_render_owns_cold_and_nested_lifetimes() {
        std::thread::spawn(|| {
            assert!(Owner::current().is_none());
            let cold = crate::static_dashboard_html();
            assert!(Owner::current().is_none());
            assert!(cold.contains("id=\"evidence-spine\""));
            assert!(cold.contains("id=\"resource-audit-console\""));
            assert_eq!(crate::static_dashboard_html(), cold);

            let owner = Owner::new();
            owner.with(|| {
                let caller = Owner::current().unwrap();
                let sentinel = RwSignal::new(41_u8);
                assert_eq!(crate::static_dashboard_html(), cold);
                assert_eq!(Owner::current(), Some(caller));
                assert_eq!(sentinel.get_untracked(), 41);
                sentinel.set(42);
                assert_eq!(sentinel.get_untracked(), 42);
                let after = RwSignal::new(7_u8);
                after.set(8);
                assert_eq!(after.get_untracked(), 8);
            });
            owner.cleanup();
            assert!(Owner::current().is_none());
        })
        .join()
        .expect("standalone renderer preserves cold and nested owner lifetimes");
    }

    #[test]
    fn audit_evidence_native_parity_controls_and_illustrative_scope() {
        let owner = Owner::new();
        owner.with(|| {
            for context in OperatorContext::ALL {
                let envelope = server_derived_envelope(context);
                let evidence =
                    operator_intelligence_strip(envelope.clone(), Signal::derive(|| true))
                        .into_view()
                        .to_html();
                let audit = resource_audit_console(envelope.clone(), Signal::derive(|| true))
                    .into_view()
                    .to_html();
                assert_eq!(evidence, static_operator_intelligence_strip(&envelope));
                assert_eq!(
                    audit,
                    super::super::static_resource_audit_console(&envelope)
                );
                let html = format!("{evidence}{audit}");
                assert_eq!(html.matches("data-ae-select=\"").count(), 23);
                assert_eq!(html.matches("data-ae-record=\"").count(), 23);
                assert_eq!(html.matches("data-ae-inspect-field=\"").count(), 14);
                assert_eq!(html.matches("data-ae-fixture=\"true\"").count(), 2);
                assert_eq!(
                    html.matches("data-ae-route-key=\"").count(),
                    ae_routes(context).len()
                );
                for (token, _, _) in ae_routes(context) {
                    assert_eq!(
                        html.matches(&format!("data-ae-route-key=\"{token}\""))
                            .count(),
                        1
                    );
                }
                let clinician = context == OperatorContext::HealthcareClinician;
                assert_eq!(
                    html.matches("data-ae-unavailable=\"true\"").count(),
                    if clinician { 9 } else { 7 }
                );
                assert_eq!(
                    html.contains("FinOps unavailable in this sample"),
                    clinician
                );
                assert!(
                    html.contains("Northwind Corp.")
                        && html.contains("April")
                        && html.contains("09:18")
                );
                assert!(
                    html.contains("5</strong> sample receipts") && html.contains("live readiness")
                );
                assert!(
                    html.contains("id=\"resource-tab-audit\"")
                        && html.contains("id=\"resource-panel-inventory\"")
                        && html.contains("id=\"resource-panel-gates\"")
                );
                assert!(html.contains("href=\"#payroll-cockpit\""));
                assert!(
                    !html.contains("data-sidepeek-trigger=\"evidence\"")
                        && !html.contains("data-sidepeek-trigger=\"object-graph\"")
                );
                let body = format!(
                    "{}</section>{audit}",
                    evidence
                        .split("<template data-ae-hub-template=")
                        .next()
                        .unwrap()
                );
                println!(
                    "AUDIT_EVIDENCE_BODY {}",
                    serde_json::to_string(&(context.id(), body)).unwrap()
                );
            }
            let mut envelope = server_derived_envelope(OperatorContext::TenantAdmin);
            envelope.workflow.name = "<script>unsafe & \"quoted\"</script>".into();
            let html = operator_intelligence_strip(envelope, Signal::derive(|| false))
                .into_view()
                .to_html();
            assert!(!html.contains("<script>unsafe"));
            assert!(
                html.contains("&lt;script&gt;unsafe &amp;") && html.contains("&lt;/script&gt;")
            );
            assert!(
                html.contains("data-ae-source-available=\"false\"")
                    && html.contains("data-ae-available=\"false\"")
            );
        });
        owner.cleanup();
    }

    #[test]
    fn audit_evidence_native_records_and_complete_source_identity() {
        assert_eq!(evidence_events().len(), 6);
        assert_eq!(object_nodes().len(), 8);
        assert_eq!(audit_receipts().len(), 5);
        assert_eq!(receipt_sources().len(), 4);
        let records = ae_records();
        assert_eq!(records.len(), 23);
        let ids: std::collections::HashSet<_> = records.iter().map(|(id, _)| id).collect();
        assert_eq!(ids.len(), records.len());
        for (id, fields) in &records {
            assert_eq!(
                serde_json::from_str::<Vec<(String, String)>>(&ae_record(id)).unwrap(),
                fields
                    .iter()
                    .map(|(name, value)| (name.to_string(), value.to_string()))
                    .collect::<Vec<_>>()
            );
        }
        let mut sources = std::collections::HashSet::new();
        for context in OperatorContext::ALL {
            let envelope = server_derived_envelope(context);
            let source = ae_source(&envelope);
            let value: serde_json::Value = serde_json::from_str(&source).unwrap();
            assert_eq!(value[0], context.id());
            assert_eq!(value[5].as_array().unwrap().len(), 23);
            assert_eq!(value[6].as_array().unwrap().len(), ae_routes(context).len());
            assert!(source.contains("Northwind Corp."));
            assert!(source.contains("09:18"));
            assert!(sources.insert(source.clone()));
            println!("AUDIT_EVIDENCE_SOURCE {}", source);
            let mut changed = envelope.clone();
            changed.workflow.goal.push_str(" changed");
            assert_ne!(ae_source(&changed), source);
        }
    }
}
