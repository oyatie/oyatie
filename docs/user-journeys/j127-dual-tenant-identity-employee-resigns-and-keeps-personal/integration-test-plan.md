---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j127-dual-tenant-identity-employee-resigns-and-keeps-personal
status: draft
date: 2026-05-20
authority_tier: 3
related_adrs:
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0276-backup-portability-gdpr-art-20
  - ADR-0188-passkey-webauthn-as-canonical-auth
ci_lane: oya-governance-journey-j127-integration
---

# j127 — Integration test plan: resignation + offboarding cascade + dual-tenant boundary preservation

## 0. Test environment

| Component | Configuration |
|---|---|
| Work tenant | `chen-aerospace.federal-contractor.us` in cell `us-east-1-fedramp-test` |
| Personal tenant | `nadia-petrov-personal-44721` in cell `us-east-1-test` |
| Bristlecone tenant (Monday) | `bristlecone-robotics.us` in cell `us-east-1-test` |
| Hardware-key | Software-emulated YubiKey with 1 handle at start, 2 by Monday |
| Synthetic test data | 47 messenger threads, 412 mails, 23 drive folders, 18 calendar events under Nadia in work tenant |

## 1. Test class A — Happy path: cascade completes

### A.1 Revocation transitions membership to REVOKED

```rust
#[tokio::test]
async fn test_a1_revocation_transitions_membership() {
    let env = TestEnv::new().await;
    let nadia = env.enroll_employee("nadia.petrov@chen-aerospace.us", "chen-aerospace.federal-contractor.us").await;

    env.workflow_engine().trigger_offboarding(&nadia.principal_id).await.unwrap();
    env.await_workflow_complete("offboarding", Duration::from_secs(60)).await;

    let membership = env.identity()
        .get_tenant_membership(&nadia.principal_id, "chen-aerospace.federal-contractor.us")
        .await.unwrap();
    assert_eq!(membership.status, "REVOKED");
    assert!(membership.revoked_at.is_some());
}
```

### A.2 Personal-tenant membership UNCHANGED

```rust
#[tokio::test]
async fn test_a2_personal_membership_unchanged() {
    let env = TestEnv::new().await;
    let nadia = env.enroll_dual_tenant_employee().await;

    let before = env.identity()
        .get_tenant_membership(&nadia.principal_id, "nadia-petrov-personal-44721")
        .await.unwrap();

    env.workflow_engine().trigger_offboarding_for_work_tenant(&nadia.principal_id).await.unwrap();
    env.await_workflow_complete("offboarding", Duration::from_secs(60)).await;

    let after = env.identity()
        .get_tenant_membership(&nadia.principal_id, "nadia-petrov-personal-44721")
        .await.unwrap();

    assert_eq!(before.status, after.status, "personal membership status must not change");
    assert_eq!(before.last_modified_at, after.last_modified_at, "personal membership row must not be modified");
}
```

### A.3 All work surfaces archived

```rust
#[tokio::test]
async fn test_a3_all_work_surfaces_archived() {
    let env = TestEnv::new().await;
    env.seed_work_surfaces_for_nadia().await; // 47 threads, 412 mails, 23 folders, 18 events

    env.workflow_engine().trigger_offboarding_for_work_tenant("nadia.petrov@chen-aerospace.us").await.unwrap();
    env.await_workflow_complete("offboarding", Duration::from_secs(60)).await;

    let threads = env.messenger().list_archived_threads("chen-aerospace.federal-contractor.us", "nadia.petrov@chen-aerospace.us").await.unwrap();
    assert_eq!(threads.len(), 47);

    let mails = env.mail().count_archived("chen-aerospace.federal-contractor.us", "nadia.petrov@chen-aerospace.us").await.unwrap();
    assert_eq!(mails, 412);

    let folders = env.drive().count_transferred("chen-aerospace.federal-contractor.us", "aleksandr.volkov@chen-aerospace.us").await.unwrap();
    assert_eq!(folders, 23);

    let events = env.calendar().count_cancelled_future("chen-aerospace.federal-contractor.us", "nadia.petrov@chen-aerospace.us").await.unwrap();
    assert_eq!(events, 18);
}
```

## 2. Test class B — Boundary invariants (THE doctrine)

### B.1 Personal-tenant Mail inbox unaffected by revocation

```rust
#[tokio::test]
async fn test_b1_personal_mail_unaffected() {
    let env = TestEnv::new().await;
    let _ = env.seed_personal_mail("nadia@nadia-petrov.me", 23).await;

    env.workflow_engine().trigger_offboarding_for_work_tenant("nadia.petrov@chen-aerospace.us").await.unwrap();
    env.await_workflow_complete("offboarding", Duration::from_secs(60)).await;

    let personal_session = env.session_for("nadia@nadia-petrov.me", "nadia-petrov-personal-44721").await;
    let inbox = env.mail().list_inbox(&personal_session).await.unwrap();
    assert_eq!(inbox.len(), 23);
}
```

### B.2 Work-tenant principal cannot read personal-tenant resources (the inverse direction)

```rust
#[tokio::test]
async fn test_b2_no_cross_tenant_personal_read_remains() {
    let env = TestEnv::new().await;
    // Nadia's former colleague Aleksandr tries to access her personal Drive
    let aleksandr_session = env.session_for("aleksandr.volkov@chen-aerospace.us", "chen-aerospace.federal-contractor.us").await;
    let result = env.drive().try_list_files(
        &aleksandr_session,
        Resource::tenant_drive_root("nadia-petrov-personal-44721"),
    ).await;

    assert!(matches!(result, Err(DriveError::CedarDeny { .. })));
}
```

### B.3 Personal-tenant audit-chain has ZERO emissions during offboarding cascade

```rust
#[tokio::test]
async fn test_b3_personal_chain_zero_emissions_during_cascade() {
    let env = TestEnv::new().await;
    let cascade_start = env.now();

    env.workflow_engine().trigger_offboarding_for_work_tenant("nadia.petrov@chen-aerospace.us").await.unwrap();
    env.await_workflow_complete("offboarding", Duration::from_secs(60)).await;
    let cascade_end = env.now();

    let personal_emissions = env.audit_chain().query(AuditQuery {
        tenant_id: "nadia-petrov-personal-44721",
        time_window: cascade_start..cascade_end,
    }).await.unwrap();

    // Only events FROM Nadia's own personal actions count; offboarding cascade itself
    // must NOT emit to personal chain.
    let offboarding_class_events: Vec<_> = personal_emissions.iter()
        .filter(|e| e.audit_class.contains("Offboarding") || e.audit_class.contains("Revoke"))
        .collect();
    assert_eq!(offboarding_class_events.len(), 0);
}
```

### B.4 Personal passkey credential handle remains ACTIVE

```rust
#[tokio::test]
async fn test_b4_personal_credential_handle_active() {
    let env = TestEnv::new().await;
    let personal_handle = env.identity().get_credential_handle_for_tenant(
        "nadia.petrov@chen-aerospace.us",  // principal-key
        "nadia-petrov-personal-44721",     // tenant
    ).await.unwrap();
    let before_active = personal_handle.active;

    env.workflow_engine().trigger_offboarding_for_work_tenant("nadia.petrov@chen-aerospace.us").await.unwrap();
    env.await_workflow_complete("offboarding", Duration::from_secs(60)).await;

    let personal_handle_after = env.identity().get_credential_handle_for_tenant(
        "nadia.petrov@chen-aerospace.us",
        "nadia-petrov-personal-44721",
    ).await.unwrap();

    assert_eq!(before_active, true);
    assert_eq!(personal_handle_after.active, true);
}
```

### B.5 Work passkey credential handle REVOKED

```rust
#[tokio::test]
async fn test_b5_work_credential_handle_revoked() {
    let env = TestEnv::new().await;

    env.workflow_engine().trigger_offboarding_for_work_tenant("nadia.petrov@chen-aerospace.us").await.unwrap();
    env.await_workflow_complete("offboarding", Duration::from_secs(60)).await;

    let work_handle = env.identity().get_credential_handle_for_tenant(
        "nadia.petrov@chen-aerospace.us",
        "chen-aerospace.federal-contractor.us",
    ).await.unwrap();
    assert_eq!(work_handle.active, false);
    assert!(work_handle.revoked_at.is_some());
}
```

### B.6 Context-picker hides revoked tenant

```rust
#[tokio::test]
async fn test_b6_context_picker_hides_revoked() {
    let env = TestEnv::new().await;

    env.workflow_engine().trigger_offboarding_for_work_tenant("nadia.petrov@chen-aerospace.us").await.unwrap();
    env.await_workflow_complete("offboarding", Duration::from_secs(60)).await;

    let webauthn = env.webauthn_emulator_for_principal("nadia.petrov@chen-aerospace.us");
    let resp: TwoTenantsOrSingleResponse = env.api_gateway()
        .post("/webauthn/verify")
        .json(&webauthn.assertion())
        .send().await.unwrap()
        .json().await.unwrap();

    // Should be SingleTenant response since only personal tenant active
    match resp {
        TwoTenantsOrSingleResponse::Single(token) => {
            assert_eq!(token.tenant_id, "nadia-petrov-personal-44721");
        }
        _ => panic!("expected SingleTenant; got multi"),
    }
}
```

## 3. Test class C — Cross-tenant permit revocation

### C.1 Cross-tenant resignation-share permit revoked

```rust
#[tokio::test]
async fn test_c1_cross_tenant_permit_revoked() {
    let env = TestEnv::new().await;
    let permit_id = env.seed_cross_tenant_resignation_share().await;

    env.workflow_engine().trigger_offboarding_for_work_tenant("nadia.petrov@chen-aerospace.us").await.unwrap();
    env.await_workflow_complete("offboarding", Duration::from_secs(60)).await;

    let permit = env.tenancy().get_cross_tenant_permit(&permit_id).await.unwrap();
    assert_eq!(permit.active, false);
    assert!(permit.revoked_at.is_some());
}
```

## 4. Test class D — Monday onboarding at new employer

### D.1 New credential handle enrolled

```rust
#[tokio::test]
async fn test_d1_bristlecone_enrollment_adds_handle() {
    let env = TestEnv::new().await;

    env.workflow_engine().trigger_offboarding_for_work_tenant("nadia.petrov@chen-aerospace.us").await.unwrap();
    env.await_workflow_complete("offboarding", Duration::from_secs(60)).await;

    let bristlecone_handle = env.identity().enroll_credential_for_tenant(
        "nadia.petrov@chen-aerospace.us",  // same principal-key (same human)
        "bristlecone-robotics.us",
        env.webauthn_emulator_for_new_handle(),
    ).await.unwrap();

    assert!(bristlecone_handle.active);

    // Personal handle still active
    let personal = env.identity().get_credential_handle_for_tenant(
        "nadia.petrov@chen-aerospace.us",
        "nadia-petrov-personal-44721",
    ).await.unwrap();
    assert!(personal.active);

    // Chen Aerospace handle still REVOKED
    let chen = env.identity().get_credential_handle_for_tenant(
        "nadia.petrov@chen-aerospace.us",
        "chen-aerospace.federal-contractor.us",
    ).await.unwrap();
    assert!(!chen.active);
}
```

### D.2 Context picker on Monday shows 2 tenants

```rust
#[tokio::test]
async fn test_d2_monday_picker_shows_two() {
    let env = TestEnv::new().await;

    env.simulate_offboarding_and_monday_onboarding().await;

    let resp: TwoTenantsOrSingleResponse = env.api_gateway()
        .post("/webauthn/verify")
        .json(&env.webauthn_emulator_for_principal("nadia.petrov@chen-aerospace.us").assertion())
        .send().await.unwrap()
        .json().await.unwrap();

    match resp {
        TwoTenantsOrSingleResponse::Multi(picker) => {
            assert_eq!(picker.tenants.len(), 2);
            let ids: Vec<_> = picker.tenants.iter().map(|t| &t.tenant_id).collect();
            assert!(ids.contains(&&"bristlecone-robotics.us".to_string()));
            assert!(ids.contains(&&"nadia-petrov-personal-44721".to_string()));
            assert!(!ids.contains(&&"chen-aerospace.federal-contractor.us".to_string()));
        }
        _ => panic!("expected multi"),
    }
}
```

## 5. Test class E — Failure modes

### E.1 Identity revoke retry on transient failure

```rust
#[tokio::test]
async fn test_e1_identity_revoke_retries() {
    let env = TestEnv::new().await;
    env.fault_inject().identity_revoke_fails(2).await; // first 2 attempts fail

    env.workflow_engine().trigger_offboarding_for_work_tenant("nadia.petrov@chen-aerospace.us").await.unwrap();
    env.await_workflow_complete("offboarding", Duration::from_secs(120)).await;

    let membership = env.identity().get_tenant_membership(
        "nadia.petrov@chen-aerospace.us", "chen-aerospace.federal-contractor.us"
    ).await.unwrap();
    assert_eq!(membership.status, "REVOKED");
}
```

### E.2 Drive transfer failure escalates but cascade continues

```rust
#[tokio::test]
async fn test_e2_drive_transfer_failure_does_not_block() {
    let env = TestEnv::new().await;
    env.fault_inject().drive_transfer_fails_permanently().await;

    env.workflow_engine().trigger_offboarding_for_work_tenant("nadia.petrov@chen-aerospace.us").await.unwrap();
    env.await_workflow_complete("offboarding", Duration::from_secs(120)).await;

    // Membership IS revoked despite drive failure
    let membership = env.identity().get_tenant_membership(
        "nadia.petrov@chen-aerospace.us", "chen-aerospace.federal-contractor.us"
    ).await.unwrap();
    assert_eq!(membership.status, "REVOKED");

    // Drive failure recorded for HR triage
    let triage_queue = env.workflow_engine().get_triage_queue("offboarding-drive-transfer-failure").await;
    assert!(!triage_queue.is_empty());
}
```

## 6. Acceptance criteria

j127 ships when:
- All A-tests pass
- All B-tests pass (boundary invariant)
- All C-tests pass
- All D-tests pass
- All E-tests pass
- ZERO emissions to personal-tenant audit-chain during cascade (test B.3)
- Cascade completes ≤30s p99

## 7. Cross-references

- `story.md`
- `handshake.md`
- ADR-0311 §B-3 + §B-9
- ADR-0276 portability
- documentation-rigor.md §3.2.5 row 18 + row 26

## Completion expansion — j127 integration rigor pass

Scope: employee resignation where work access is revoked and personal tenant survives.
Persona: Marcus tenant engineer.
Services: identity + tenancy + messenger + mail + drive + workflow-engine.
Applicable ADRs: ADR-0244, ADR-0299, ADR-0311, ADR-0313, ADR-0317, ADR-0320.
The expansion below is intentionally explicit so an intern can build and verify the journey without oral context.

Test case 001: default-deny refusal for tenancy seeds tenant fixtures, executes the journey action, and asserts no unrelated tenant row is read.
Fixture 002: create work tenant, personal tenant, Marcus tenant engineer principal, counterparty principal, Cedar policy bundle, and audit-chain expected leaf.
Assertion 003: OpenAPI 3.2.0 response, AsyncAPI 3.1.0 emission, and proto3 message agree on case_id, tenant_id, audience_type, and purpose code.
Regression 004: run with stale credential, wrong audience type, region failover, policy timeout, and duplicate idempotency key; expected behavior remains fail-closed.
Test case 005: audit-chain seal verification for workflow-engine seeds tenant fixtures, executes the journey action, and asserts no unrelated tenant row is read.
Fixture 006: create work tenant, personal tenant, Marcus tenant engineer principal, counterparty principal, Cedar policy bundle, and audit-chain expected leaf.
Assertion 007: OpenAPI 3.2.0 response, AsyncAPI 3.1.0 emission, and proto3 message agree on case_id, tenant_id, audience_type, and purpose code.
Regression 008: run with stale credential, wrong audience type, region failover, policy timeout, and duplicate idempotency key; expected behavior remains fail-closed.
Test case 009: default-deny refusal for mail seeds tenant fixtures, executes the journey action, and asserts no unrelated tenant row is read.
Fixture 010: create work tenant, personal tenant, Marcus tenant engineer principal, counterparty principal, Cedar policy bundle, and audit-chain expected leaf.
Assertion 011: OpenAPI 3.2.0 response, AsyncAPI 3.1.0 emission, and proto3 message agree on case_id, tenant_id, audience_type, and purpose code.
Regression 012: run with stale credential, wrong audience type, region failover, policy timeout, and duplicate idempotency key; expected behavior remains fail-closed.
Test case 013: audit-chain seal verification for tenancy seeds tenant fixtures, executes the journey action, and asserts no unrelated tenant row is read.
Fixture 014: create work tenant, personal tenant, Marcus tenant engineer principal, counterparty principal, Cedar policy bundle, and audit-chain expected leaf.
Assertion 015: OpenAPI 3.2.0 response, AsyncAPI 3.1.0 emission, and proto3 message agree on case_id, tenant_id, audience_type, and purpose code.
Regression 016: run with stale credential, wrong audience type, region failover, policy timeout, and duplicate idempotency key; expected behavior remains fail-closed.
Checkpoint 01: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Test case 017: default-deny refusal for workflow-engine seeds tenant fixtures, executes the journey action, and asserts no unrelated tenant row is read.
Fixture 018: create work tenant, personal tenant, Marcus tenant engineer principal, counterparty principal, Cedar policy bundle, and audit-chain expected leaf.
Assertion 019: OpenAPI 3.2.0 response, AsyncAPI 3.1.0 emission, and proto3 message agree on case_id, tenant_id, audience_type, and purpose code.
Regression 020: run with stale credential, wrong audience type, region failover, policy timeout, and duplicate idempotency key; expected behavior remains fail-closed.
Test case 021: audit-chain seal verification for mail seeds tenant fixtures, executes the journey action, and asserts no unrelated tenant row is read.
Fixture 022: create work tenant, personal tenant, Marcus tenant engineer principal, counterparty principal, Cedar policy bundle, and audit-chain expected leaf.
Assertion 023: OpenAPI 3.2.0 response, AsyncAPI 3.1.0 emission, and proto3 message agree on case_id, tenant_id, audience_type, and purpose code.
Regression 024: run with stale credential, wrong audience type, region failover, policy timeout, and duplicate idempotency key; expected behavior remains fail-closed.
Test case 025: default-deny refusal for tenancy seeds tenant fixtures, executes the journey action, and asserts no unrelated tenant row is read.
Fixture 026: create work tenant, personal tenant, Marcus tenant engineer principal, counterparty principal, Cedar policy bundle, and audit-chain expected leaf.
Assertion 027: OpenAPI 3.2.0 response, AsyncAPI 3.1.0 emission, and proto3 message agree on case_id, tenant_id, audience_type, and purpose code.
Regression 028: run with stale credential, wrong audience type, region failover, policy timeout, and duplicate idempotency key; expected behavior remains fail-closed.
Test case 029: audit-chain seal verification for workflow-engine seeds tenant fixtures, executes the journey action, and asserts no unrelated tenant row is read.
Fixture 030: create work tenant, personal tenant, Marcus tenant engineer principal, counterparty principal, Cedar policy bundle, and audit-chain expected leaf.
Assertion 031: OpenAPI 3.2.0 response, AsyncAPI 3.1.0 emission, and proto3 message agree on case_id, tenant_id, audience_type, and purpose code.
Regression 032: run with stale credential, wrong audience type, region failover, policy timeout, and duplicate idempotency key; expected behavior remains fail-closed.
Checkpoint 02: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Test case 033: default-deny refusal for mail seeds tenant fixtures, executes the journey action, and asserts no unrelated tenant row is read.
Fixture 034: create work tenant, personal tenant, Marcus tenant engineer principal, counterparty principal, Cedar policy bundle, and audit-chain expected leaf.
Assertion 035: OpenAPI 3.2.0 response, AsyncAPI 3.1.0 emission, and proto3 message agree on case_id, tenant_id, audience_type, and purpose code.
Regression 036: run with stale credential, wrong audience type, region failover, policy timeout, and duplicate idempotency key; expected behavior remains fail-closed.
Test case 037: audit-chain seal verification for tenancy seeds tenant fixtures, executes the journey action, and asserts no unrelated tenant row is read.
Fixture 038: create work tenant, personal tenant, Marcus tenant engineer principal, counterparty principal, Cedar policy bundle, and audit-chain expected leaf.
Assertion 039: OpenAPI 3.2.0 response, AsyncAPI 3.1.0 emission, and proto3 message agree on case_id, tenant_id, audience_type, and purpose code.
Regression 040: run with stale credential, wrong audience type, region failover, policy timeout, and duplicate idempotency key; expected behavior remains fail-closed.
Test case 041: default-deny refusal for workflow-engine seeds tenant fixtures, executes the journey action, and asserts no unrelated tenant row is read.
Fixture 042: create work tenant, personal tenant, Marcus tenant engineer principal, counterparty principal, Cedar policy bundle, and audit-chain expected leaf.
