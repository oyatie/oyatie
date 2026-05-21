---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j126-government-auditor-3pao-conducts-fedramp-audit
status: draft
date: 2026-05-20
authority_tier: 3
related_adrs:
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0028-audit-chain-merkle-sealed
  - ADR-0263-observability-emission-contract
  - ADR-0246-policy-engine-library-first
companion_docs:
  - story.md
  - ux-flow.md
  - handshake.md
test_classes:
  - integration-cross-microservice
  - cross-tenant-boundary-preservation
  - cedar-default-deny-invariants
  - audit-chain-dual-tenant-emission
  - observability-cross-tenant-metric-export
ci_lane: oya-governance-journey-j126-integration
---

# j126 — Integration test plan: FedRAMP 3PAO audit pull with dual-tenant boundary preservation

This test plan verifies that the j126 handshake (per `handshake.md`)
holds end-to-end, with particular attention to the load-bearing
ADR-0311 invariants: the dual-tenant identity boundary is preserved
under nominal operation and under each failure mode.

## 0. Test environment

| Component | Configuration |
|---|---|
| Tenant under test 1 | `gao.audit.fedramp-3pao` in cell `us-gov-east-1-test` |
| Tenant under test 2 | `chen-aerospace.federal-contractor.us` in cell `us-east-1-fedramp-test` |
| Tenant under test 3 | `diana-reyes-personal-92381-test` in cell `us-east-1-test` |
| Cedar fragments | Loaded from `microservices/*/policy/*.cedar` per per-tenant fragment-set |
| audit-chain | Three independent Merkle chains, one per tenant, sealed by audit-chain µservice |
| identity | Single µservice, multi-tenant; Diana's test-user enrolled in all three tenants |
| Hardware-key | Software-emulated YubiKey via `oya-test-webauthn-emulator` |
| Test data | 4,127,841 synthetic audit events in Marcus's tenant for the audit period |

## 1. Test class A — happy path (handshake correctness)

### A.1 Session establish with two-tenants context picker

```rust
#[tokio::test]
async fn test_a1_two_tenants_detected_at_session_init() {
    let env = TestEnv::new().await;
    let diana = env.enroll_user_in_tenants(vec![
        "gao.audit.fedramp-3pao",
        "diana-reyes-personal-92381",
    ]).await;
    let webauthn = env.webauthn_emulator(diana.credential_id);

    let resp = env.api_gateway()
        .post("/webauthn/verify")
        .json(&webauthn.assertion())
        .send().await.unwrap();

    assert_eq!(resp.status(), 200);
    let body: TwoTenantsResponse = resp.json().await.unwrap();

    assert_eq!(body.tenants.len(), 2);
    assert!(body.tenants.iter().any(|t| t.tenant_id == "gao.audit.fedramp-3pao"));
    assert!(body.tenants.iter().any(|t| t.tenant_id == "diana-reyes-personal-92381"));

    // Critical: no auto-selection
    assert_eq!(body.preselected, None);
}
```

### A.2 Session init for selected tenant; audience-type set

```rust
#[tokio::test]
async fn test_a2_session_init_sets_audience_type() {
    let env = TestEnv::new().await;
    let session = env.session_init_for(
        "diana.reyes@gao.gov",
        "gao.audit.fedramp-3pao",
    ).await.unwrap();

    assert_eq!(session.audience_type, "INTERNAL_AUDITOR_3PAO");
    assert_eq!(session.cell_id, "us-gov-east-1-test");
    assert!(session.packs_active.contains(&"pack-us-fedramp-mod".to_string()));

    // Critical: 3PAO accreditation status snapshot
    assert_eq!(session.fedramp_3pao_accreditation_active, true);
}
```

### A.3 Active docket list returned

```rust
#[tokio::test]
async fn test_a3_active_docket_list() {
    let env = TestEnv::new().await;
    env.seed_docket("3PAO-2026-MAY-CHEN-AERO-001", &"diana.reyes@gao.gov").await;

    let session = env.session_for("diana.reyes@gao.gov", "gao.audit.fedramp-3pao").await;
    let dockets = env.ops_dashboard().list_active_dockets(&session).await.unwrap();

    assert_eq!(dockets.len(), 1);
    assert_eq!(dockets[0].docket_id, "3PAO-2026-MAY-CHEN-AERO-001");
    assert_eq!(dockets[0].csp_tenant_id, "chen-aerospace.federal-contractor.us");
}
```

### A.4 Cross-tenant permit evaluation succeeds

```rust
#[tokio::test]
async fn test_a4_cross_tenant_permit_allow() {
    let env = TestEnv::new().await;
    env.load_fragment_into_tenant(
        "chen-aerospace.federal-contractor.us",
        include_str!("../fixtures/cross-tenant-fedramp-3pao-audit-evidence.cedar"),
    ).await;

    let session = env.session_for("diana.reyes@gao.gov", "gao.audit.fedramp-3pao").await;

    let result = env.policy_engine().evaluate(
        &session.as_principal(),
        "audit_chain.read_sealed_evidence",
        Resource::tenant("chen-aerospace.federal-contractor.us"),
        Context::new()
            .with("audit_docket_id", "3PAO-2026-MAY-CHEN-AERO-001")
            .with("audit_period_start", "2025-05-01T00:00:00Z")
            .with("audit_period_end", "2026-04-30T23:59:59Z"),
    ).await;

    assert_eq!(result.decision, Decision::Allow);
    assert!(result.evaluation_ms <= 25); // per ADR-0246 amendment §D-latency
}
```

### A.5 Evidence pull workflow completes end-to-end

```rust
#[tokio::test]
async fn test_a5_evidence_pull_end_to_end() {
    let env = TestEnv::new().await;
    env.seed_marcus_audit_events(4_127_841).await; // synthetic
    let session = env.session_for("diana.reyes@gao.gov", "gao.audit.fedramp-3pao").await;

    let bundle = env.workflow_engine().start_evidence_pull(
        &session,
        "3PAO-2026-MAY-CHEN-AERO-001",
        vec!["AU-2", "AU-12", "AC-3", "IA-2", "CM-3"],
    ).await.unwrap();

    // Verify bundle Merkle-sealed
    assert!(bundle.merkle_root.is_some());
    assert_eq!(bundle.controls.len(), 5);

    // Verify pull-latency within budget
    assert!(bundle.total_pull_ms <= 25_000); // 25s p99 per handshake §4
}
```

### A.6 Finding filed routes to CSP

```rust
#[tokio::test]
async fn test_a6_finding_routes_cross_tenant() {
    let env = TestEnv::new().await;
    let session = env.session_for("diana.reyes@gao.gov", "gao.audit.fedramp-3pao").await;

    let finding_id = env.workflow_engine().file_finding(
        &session,
        AuditFinding {
            docket_id: "3PAO-2026-MAY-CHEN-AERO-001",
            control: "AU-2",
            severity: "APPROVE_WITH_FINDINGS",
            description: "test cardinality anomaly".into(),
            response_due: "30 days",
        },
    ).await.unwrap();

    // Verify finding in Marcus's tenant queue
    let marcus_session = env.tenant_admin_session("chen-aerospace.federal-contractor.us").await;
    let pending = env.ops_dashboard().list_pending_findings(&marcus_session).await.unwrap();

    assert!(pending.iter().any(|f| f.finding_id == finding_id));

    // Verify tenant-admin email notification dispatched
    let emails = env.comms_email().outbox_for("marcus.chen@chen-aerospace.us").await;
    assert!(emails.iter().any(|e| e.subject.contains(&finding_id)));
}
```

## 2. Test class B — Cedar default-deny invariants (THE boundary)

These are the load-bearing tests. If any of these fail, ADR-0311 is
violated and the platform's dual-tenant claim is false.

### B.1 Work session cannot read personal-tenant messenger

```rust
#[tokio::test]
async fn test_b1_work_session_cannot_read_personal_messenger() {
    let env = TestEnv::new().await;

    // Seed Diana's personal messenger thread
    let personal_session = env.session_for(
        "diana@diana-reyes.me",
        "diana-reyes-personal-92381",
    ).await;
    let thread = env.messenger().create_thread(&personal_session, "Reyes Family").await;
    env.messenger().send_message(&personal_session, thread.id, "test message").await;

    // Switch to work session
    let work_session = env.session_for(
        "diana.reyes@gao.gov",
        "gao.audit.fedramp-3pao",
    ).await;

    // Attempt to read personal thread from work session
    let result = env.messenger()
        .try_read_thread(&work_session, thread.id)
        .await;

    // MUST fail with Cedar Deny
    match result {
        Err(MessengerError::CedarDeny { reason, .. }) => {
            assert!(reason.contains("principal.tenant != resource.tenant"));
        }
        _ => panic!("expected CedarDeny, got {:?}", result),
    }

    // Verify no audit event was emitted to GAO tenant audit-chain
    let gao_audit = env.audit_chain().query(
        AuditQuery {
            tenant_id: "gao.audit.fedramp-3pao",
            action: Some("messenger.read_thread"),
            time_window: env.now() - 60.seconds()..env.now(),
        }
    ).await.unwrap();
    assert!(gao_audit.is_empty());
}
```

### B.2 Cross-tenant permit cannot be exercised by non-3PAO principals

```rust
#[tokio::test]
async fn test_b2_non_3pao_in_gao_tenant_cannot_pull_evidence() {
    let env = TestEnv::new().await;
    let hr_employee = env.enroll_user_in_tenants(vec!["gao.audit.fedramp-3pao"]).await;
    env.set_user_audience_type(&hr_employee.id, "B2B_HR_ADMIN").await;

    let session = env.session_for(&hr_employee.email, "gao.audit.fedramp-3pao").await;

    let result = env.policy_engine().evaluate(
        &session.as_principal(),
        "audit_chain.read_sealed_evidence",
        Resource::tenant("chen-aerospace.federal-contractor.us"),
        Context::new()
            .with("audit_docket_id", "3PAO-2026-MAY-CHEN-AERO-001"),
    ).await;

    assert_eq!(result.decision, Decision::Deny);
    assert!(result.deny_reason.contains("audience_type"));
}
```

### B.3 Expired Cedar permit cannot be exercised

```rust
#[tokio::test]
async fn test_b3_expired_permit_denied() {
    let env = TestEnv::new().await;
    let session = env.session_for("diana.reyes@gao.gov", "gao.audit.fedramp-3pao").await;

    let result = env.policy_engine().evaluate(
        &session.as_principal(),
        "audit_chain.read_sealed_evidence",
        Resource::tenant("chen-aerospace.federal-contractor.us"),
        Context::new()
            .with("audit_docket_id", "3PAO-2026-MAY-CHEN-AERO-001")
            // Period outside the permit's window
            .with("audit_period_start", "2024-01-01T00:00:00Z")
            .with("audit_period_end", "2024-12-31T23:59:59Z"),
    ).await;

    assert_eq!(result.decision, Decision::Deny);
    assert!(result.deny_reason.contains("audit_period_start"));
}
```

### B.4 Lapsed 3PAO accreditation flips permit to Deny

```rust
#[tokio::test]
async fn test_b4_lapsed_accreditation_denies() {
    let env = TestEnv::new().await;

    // Diana starts as accredited
    let session1 = env.session_for("diana.reyes@gao.gov", "gao.audit.fedramp-3pao").await;
    let r1 = env.policy_engine().evaluate_cross_tenant_pull(&session1, "3PAO-2026-MAY-CHEN-AERO-001").await;
    assert_eq!(r1.decision, Decision::Allow);

    // Lapse her accreditation
    env.identity().set_fedramp_3pao_accreditation_active(&"diana.reyes@gao.gov", false).await;

    // Subsequent eval must deny (live lookup per story §19 invariant 8)
    let session2 = env.refresh_session(&session1).await;
    let r2 = env.policy_engine().evaluate_cross_tenant_pull(&session2, "3PAO-2026-MAY-CHEN-AERO-001").await;
    assert_eq!(r2.decision, Decision::Deny);
}
```

### B.5 Personal-tenant principal cannot exercise work-tenant permits

```rust
#[tokio::test]
async fn test_b5_personal_tenant_cannot_use_work_permits() {
    let env = TestEnv::new().await;
    let session = env.session_for("diana@diana-reyes.me", "diana-reyes-personal-92381").await;

    let result = env.policy_engine().evaluate(
        &session.as_principal(),
        "audit_chain.read_sealed_evidence",
        Resource::tenant("chen-aerospace.federal-contractor.us"),
        Context::new()
            .with("audit_docket_id", "3PAO-2026-MAY-CHEN-AERO-001"),
    ).await;

    assert_eq!(result.decision, Decision::Deny);
    // Permit requires principal.tenant == "gao.audit.fedramp-3pao"
    assert!(result.deny_reason.contains("principal in Tenant"));
}
```

### B.6 No permit exists permitting agency → personal-tenant access

```rust
#[tokio::test]
async fn test_b6_no_permit_grants_cross_tenant_personal_read() {
    let env = TestEnv::new().await;

    // Exhaustive enumeration of all Cedar fragments loaded in test env
    let fragments = env.policy_engine().enumerate_fragments().await.unwrap();

    for fragment in fragments {
        for permit in fragment.permits {
            // Disallowed shape: principal-tenant in GAO, action on personal tenant
            if let (Some(principal_tenant), Some(resource_tenant)) =
                (&permit.principal_tenant, &permit.resource_tenant) {
                let permits_agency_to_personal = principal_tenant.starts_with("gao.") &&
                    resource_tenant.starts_with("diana-reyes-personal-");
                assert!(
                    !permits_agency_to_personal,
                    "Forbidden permit in fragment {}: {:?}", fragment.id, permit
                );
            }
        }
    }
}
```

## 3. Test class C — Audit-chain dual-tenant emission

### C.1 Cross-tenant pull emits to BOTH tenants

```rust
#[tokio::test]
async fn test_c1_dual_tenant_emission_on_cross_tenant_pull() {
    let env = TestEnv::new().await;
    let session = env.session_for("diana.reyes@gao.gov", "gao.audit.fedramp-3pao").await;
    let _bundle = env.workflow_engine().start_evidence_pull(
        &session, "3PAO-2026-MAY-CHEN-AERO-001", vec!["AU-2"],
    ).await.unwrap();

    let gao_events = env.audit_chain().query(AuditQuery {
        tenant_id: "gao.audit.fedramp-3pao",
        action: Some("audit.cross_tenant_pull"),
        time_window: env.now() - 60.seconds()..env.now(),
    }).await.unwrap();

    let marcus_events = env.audit_chain().query(AuditQuery {
        tenant_id: "chen-aerospace.federal-contractor.us",
        action: Some("audit.cross_tenant_export"),
        time_window: env.now() - 60.seconds()..env.now(),
    }).await.unwrap();

    assert_eq!(gao_events.len(), 1);
    assert_eq!(marcus_events.len(), 1);
    assert_eq!(gao_events[0].docket_id, marcus_events[0].docket_id);
}
```

### C.2 Personal-tenant messenger emits to personal-tenant ONLY

```rust
#[tokio::test]
async fn test_c2_personal_messenger_personal_chain_only() {
    let env = TestEnv::new().await;
    let personal_session = env.session_for(
        "diana@diana-reyes.me",
        "diana-reyes-personal-92381",
    ).await;
    let thread = env.messenger().create_thread(&personal_session, "Reyes Family").await;
    env.messenger().send_message(&personal_session, thread.id, "easter plans").await;

    // Personal-tenant audit-chain must have the event
    let personal_events = env.audit_chain().query(AuditQuery {
        tenant_id: "diana-reyes-personal-92381",
        action: Some("messenger.send_message"),
        time_window: env.now() - 60.seconds()..env.now(),
    }).await.unwrap();
    assert_eq!(personal_events.len(), 1);

    // GAO tenant audit-chain must NOT have the event
    let gao_events = env.audit_chain().query(AuditQuery {
        tenant_id: "gao.audit.fedramp-3pao",
        action: Some("messenger.send_message"),
        time_window: env.now() - 60.seconds()..env.now(),
    }).await.unwrap();
    assert_eq!(gao_events.len(), 0);
}
```

### C.3 Audit-chain rejects emission with mismatched tenant_id

```rust
#[tokio::test]
async fn test_c3_audit_chain_rejects_mismatched_tenant_emission() {
    let env = TestEnv::new().await;
    let work_caller = env.spiffe_identity("messenger-us-gov-east-1").await;

    // Attempt: messenger caller from GAO tenant emits to personal tenant
    // (This would happen if messenger had a bug.)
    let result = env.audit_chain().try_emit(
        &work_caller,
        SealedAuditEvent {
            tenant_id: "diana-reyes-personal-92381", // mismatched
            class: "messenger.send_message",
            principal_id: "diana@diana-reyes.me",
            // ...
        },
    ).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, AuditChainError::TenantMismatch { .. }));
}
```

### C.4 Merkle-seal end-to-end verification

```rust
#[tokio::test]
async fn test_c4_merkle_seal_end_to_end() {
    let env = TestEnv::new().await;
    let session = env.session_for("diana.reyes@gao.gov", "gao.audit.fedramp-3pao").await;
    let bundle = env.workflow_engine().start_evidence_pull(
        &session, "3PAO-2026-MAY-CHEN-AERO-001", vec!["AU-2"],
    ).await.unwrap();

    let root = bundle.merkle_root.unwrap();
    for evidence in bundle.evidence {
        let proof = env.audit_chain().get_proof(evidence.id).await.unwrap();
        let valid = audit_chain_verify::verify(&evidence, &proof, &root);
        assert!(valid, "Merkle proof failed for {}", evidence.id);
    }
}
```

## 4. Test class D — Cross-tenant transparency

### D.1 Marcus's tenant-admin receives notification within 15min

```rust
#[tokio::test]
async fn test_d1_marcus_notified_within_15min() {
    let env = TestEnv::new().await;
    let pull_start = env.now();

    let session = env.session_for("diana.reyes@gao.gov", "gao.audit.fedramp-3pao").await;
    let _bundle = env.workflow_engine().start_evidence_pull(
        &session, "3PAO-2026-MAY-CHEN-AERO-001", vec!["AU-2"],
    ).await.unwrap();

    let email = env.comms_email().wait_for_email(
        "marcus.chen@chen-aerospace.us",
        Duration::from_secs(900), // 15 min
    ).await.expect("Marcus must receive notification within 15min");

    let delta = email.sent_at - pull_start;
    assert!(delta <= Duration::from_secs(900));
    assert!(email.subject.contains("3PAO-2026-MAY-CHEN-AERO-001"));
    assert!(email.body.contains("Diana Reyes"));
    assert!(email.body.contains("audit-evidence pull"));
}
```

### D.2 Tenant-admin dashboard shows cross-tenant access events

```rust
#[tokio::test]
async fn test_d2_tenant_admin_visibility() {
    let env = TestEnv::new().await;
    let _bundle = env.session_pull_evidence("diana.reyes@gao.gov", "3PAO-2026-MAY-CHEN-AERO-001").await;

    let marcus_session = env.tenant_admin_session("chen-aerospace.federal-contractor.us").await;
    let access_events = env.ops_dashboard()
        .list_cross_tenant_access_events(&marcus_session, 30.days())
        .await.unwrap();

    assert!(access_events.iter().any(|e| {
        e.cross_tenant_principal == "diana.reyes@gao.gov" &&
        e.docket_id == "3PAO-2026-MAY-CHEN-AERO-001"
    }));
}
```

## 5. Test class E — Observability / metric emission

### E.1 Cross-tenant audit pull emits metric labeled with both tenants

```rust
#[tokio::test]
async fn test_e1_cross_tenant_metric_labels() {
    let env = TestEnv::new().await;
    let session = env.session_for("diana.reyes@gao.gov", "gao.audit.fedramp-3pao").await;
    let _bundle = env.workflow_engine().start_evidence_pull(
        &session, "3PAO-2026-MAY-CHEN-AERO-001", vec!["AU-2"],
    ).await.unwrap();

    let metrics = env.observability().scrape_metrics().await;
    let cross_tenant_metric = metrics.iter()
        .find(|m| m.name == "oya_cross_tenant_audit_evidence_pulled_total")
        .expect("metric must be emitted");

    assert_eq!(cross_tenant_metric.labels["principal_tenant"], "gao.audit.fedramp-3pao");
    assert_eq!(cross_tenant_metric.labels["resource_tenant"], "chen-aerospace.federal-contractor.us");
    assert_eq!(cross_tenant_metric.labels["docket_id"], "3PAO-2026-MAY-CHEN-AERO-001");
}
```

### E.2 Cardinality budget respected

```rust
#[tokio::test]
async fn test_e2_metric_cardinality_budget() {
    let env = TestEnv::new().await;
    let metrics = env.observability().scrape_metrics().await;
    let cross_tenant_metric = metrics.iter()
        .find(|m| m.name == "oya_cross_tenant_audit_evidence_pulled_total")
        .unwrap();

    // Budget: 10k unique label sets per ADR-0263 §D-cardinality
    assert!(cross_tenant_metric.unique_label_sets <= 10_000);
}
```

## 6. Test class F — Cell isolation

### F.1 No L3 path between consumer cell and GovCloud cell

```rust
#[tokio::test]
async fn test_f1_no_l3_consumer_to_govcloud() {
    let env = TestEnv::new().await;
    let consumer_node = env.pick_node_in_cell("us-east-1-test").await;
    let govcloud_node = env.pick_node_in_cell("us-gov-east-1-test").await;

    let result = consumer_node.try_tcp_connect(
        govcloud_node.private_ip,
        5432, // postgres port
        Duration::from_secs(5),
    ).await;

    assert!(matches!(result, Err(ConnectError::Filtered { .. })));
}
```

### F.2 Cell-shuffle-sharding holds for FedRAMP Mod ↔ FedRAMP Mod

```rust
#[tokio::test]
async fn test_f2_govcloud_to_fedramp_mod_allowed() {
    let env = TestEnv::new().await;
    let govcloud_node = env.pick_node_in_cell("us-gov-east-1-test").await;
    let fedramp_mod_node = env.pick_node_in_cell("us-east-1-fedramp-test").await;

    let result = govcloud_node.try_mtls_connect(
        fedramp_mod_node.private_ip,
        8443, // µservice gRPC port
        env.spiffe_workload_identity("workflow-engine-us-gov-east-1"),
        Duration::from_secs(5),
    ).await;

    assert!(result.is_ok());
}
```

## 7. Test class G — Failure modes

### G.1 audit-chain seal failure → workflow-engine retry

```rust
#[tokio::test]
async fn test_g1_audit_chain_seal_failure_retries() {
    let env = TestEnv::new().await;
    env.fault_inject().audit_chain_seal_fails(2).await; // first 2 attempts fail

    let session = env.session_for("diana.reyes@gao.gov", "gao.audit.fedramp-3pao").await;
    let bundle = env.workflow_engine().start_evidence_pull(
        &session, "3PAO-2026-MAY-CHEN-AERO-001", vec!["AU-2"],
    ).await.unwrap();

    // After retry, bundle is sealed
    assert!(bundle.merkle_root.is_some());

    // Retry metric incremented
    let metric = env.observability().get_counter("oya_audit_chain_seal_retry_total").await;
    assert!(metric.value >= 2);
}
```

### G.2 policy-engine evaluation timeout fails closed

```rust
#[tokio::test]
async fn test_g2_policy_engine_timeout_fails_closed() {
    let env = TestEnv::new().await;
    env.fault_inject().policy_engine_eval_timeout(200.ms()).await;

    let session = env.session_for("diana.reyes@gao.gov", "gao.audit.fedramp-3pao").await;
    let result = env.workflow_engine().start_evidence_pull(
        &session, "3PAO-2026-MAY-CHEN-AERO-001", vec!["AU-2"],
    ).await;

    assert!(matches!(result, Err(WorkflowError::PolicyEvaluationFailed)));
}
```

### G.3 Cross-tenant notification dispatch failure does NOT block audit

```rust
#[tokio::test]
async fn test_g3_notification_failure_does_not_block_audit() {
    let env = TestEnv::new().await;
    env.fault_inject().comms_email_dispatch_fails(99).await; // many failures

    let session = env.session_for("diana.reyes@gao.gov", "gao.audit.fedramp-3pao").await;
    let bundle = env.workflow_engine().start_evidence_pull(
        &session, "3PAO-2026-MAY-CHEN-AERO-001", vec!["AU-2"],
    ).await.unwrap();

    // Audit succeeds
    assert!(bundle.merkle_root.is_some());

    // But notification retry queue is non-empty (will be retried)
    let queue_depth = env.workflow_engine()
        .get_retry_queue_depth("cross-tenant-notification").await;
    assert!(queue_depth >= 1);

    // Audit-chain has a `TenantAdminNotificationDispatchFailed` event for SRE
    let failures = env.audit_chain().query(AuditQuery {
        tenant_id: "gao.audit.fedramp-3pao",
        action: Some("comms_email.dispatch_failed"),
        time_window: env.now() - 60.seconds()..env.now(),
    }).await.unwrap();
    assert!(failures.len() >= 1);
}
```

## 8. Test class H — Property-based / fuzz

### H.1 Every cross-tenant Cedar fragment in the test corpus enforces tenant-scoping

```rust
proptest! {
    #[test]
    fn property_cross_tenant_fragments_enforce_scope(
        fragment in cross_tenant_fragment_strategy(),
        principal in principal_strategy(),
        resource in resource_strategy(),
    ) {
        // For any cross-tenant fragment, evaluation MUST require principal.tenant
        // to be in the fragment's permitted-principal set.
        let result = evaluate(&fragment, &principal, &resource);
        if result == Decision::Allow {
            assert!(fragment.principal_tenants.contains(&principal.tenant));
            assert!(fragment.resource_tenants.contains(&resource.tenant));
        }
    }
}
```

## 9. Acceptance criteria for j126

j126 is intern-buildable and ADR-0311-conformant when:

- All A-tests pass (happy path is correct).
- All B-tests pass (boundary holds under every Cedar evaluation).
- All C-tests pass (audit-chain emits to right tenants only).
- All D-tests pass (transparency to counterparty tenant within timing).
- All E-tests pass (observability metrics emit correctly with bounded
  cardinality).
- All F-tests pass (cell isolation holds at L3).
- All G-tests pass (failure modes don't break boundary).
- All H property tests hold (no cross-tenant fragment violates the
  scope invariant).
- Static analysis (CI lane `oya-governance-cedar-fragment-shape`)
  finds no cross-tenant fragment with the agency→personal shape
  (test B.6).
- Latency budgets per `handshake.md` §Latency budget hold p99.

## 10. CI lane wiring

```yaml
# .github/workflows/oya-governance-journey-j126.yml
name: oya-governance-journey-j126-integration
on:
  pull_request:
    paths:
      - "docs/user-journeys/j126-*/**"
      - "microservices/identity/**"
      - "microservices/tenancy/**"
      - "microservices/audit-chain/**"
      - "microservices/compliance/**"
      - "microservices/ops-dashboard-control-center/**"
      - "microservices/observability/**"
jobs:
  j126-integration:
    runs-on: ubuntu-24.04
    steps:
      - run: cargo test -p oya-journey-j126-integration -- --include-ignored
      - run: ./scripts/oya-governance-cedar-fragment-shape.sh
      - run: ./scripts/oya-governance-cross-tenant-permit-audit.sh
```

## 11. Cross-references

- `story.md` — narrative
- `ux-flow.md` — UX screens
- `handshake.md` — µservice sequences
- `microservices/identity/IP-journey-j126-fedramp-3pao-cross-tenant-resolver.md`
- `microservices/tenancy/IP-journey-j126-cross-tenant-permit-grant.md`
- `microservices/audit-chain/IP-journey-j126-dual-tenant-emission-classes.md`
- `microservices/compliance/IP-journey-j126-fedramp-conmon-pack-overlay.md`
- `microservices/ops-dashboard-control-center/IP-journey-j126-3pao-docket-dashboard.md`
- `microservices/observability/IP-journey-j126-cross-tenant-audit-metrics.md`
- documentation-rigor.md §3.2.1 row 28 (abuse-defence) + §3.2.5 row 18 (audit/regulator access)
