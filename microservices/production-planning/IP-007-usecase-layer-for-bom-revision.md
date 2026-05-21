---
doc_class: ImplementationPlan
microservice: production-planning
status: Accepted
date: 2026-05-20
owner_team: axis-production-planning + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0244, ADR-0252, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0316]
planned_enforcement_ref: oya-governance-production-planning-doc-suite
ip_id: IP-007
journey_ref: j101
journey_slug: j101-multi-tier-supply-chain-formation
sap_submodule: PP-BD (Basic Data BOM transactions CS01/CS02/CS03)
tenant_class: substrate
persona: design-engineer
---

# IP-007: Usecase layer for bom-revision

## A. Intent

This is the **usecase / port-in-kernel** layer that wires the pure `BomRevision` domain aggregate (IP-001) to outside-world ports: persistence, audit emission, ontology projection, Cedar evaluation, transaction control. Per ADR-0105 layer enum and Clean-Architecture inward-only flow, usecases consume ports (traits) and never reference adapter implementations directly. The adapter wiring lives in IP-013.

In SAP, the equivalent surface is the **function-module layer** above table accesses for CS01/CS02/CS03 (e.g., `CSAP_MAT_BOM_CREATE`, `CSAP_MAT_BOM_OPEN`, `CSAP_MAT_BOM_MAINTAIN`). Oyatie maps each function-module surface to one usecase method with explicit Cedar gating, tenant pinning, HLC stamping, and outbox emission.

### A.1 Why a usecase layer at all

The pure aggregate cannot persist, audit, or gate. But you also don't want persistence calls scattered through HTTP handlers (= adapters). The usecase layer is where the **transactional invariant** — "if Cedar permits AND aggregate accepts the mutation, persist + emit + audit atomically" — lives. This pattern is identical to Eric Evans's DDD `ApplicationService`.

## B. Acceptance criteria

- **AC-1:** `CreateBomRevisionUseCase::execute(input)` calls Cedar `production_planning::bom::create`, runs domain `BomRevision::new`, persists via `BomRepository::save`, dispatches outbox event `EVT-PRODUCTION_PLANNING-BOM_REVISION-CREATED`.
- **AC-2:** Failure at any step rolls back the entire unit-of-work (no partial state).
- **AC-3:** Idempotency: same `idempotency_key` produces identical output and no duplicate outbox row.
- **AC-4:** Tenant pinning: input carries `tenant_id`; all repository calls are scoped; cross-tenant denied via Cedar AND repository-level check (defense-in-depth).
- **AC-5:** HLC stamping: every persisted event carries HLC `recorded_at` from `oya-shared-time::Hlc::now()`.
- **AC-6:** Audit trail: each Cedar decision logged with `cedar_decision_id`, `policy_bundle_version`, and correlation/causation per ADR-0316.
- **AC-7:** Concurrency: optimistic concurrency via `etag` returned in responses; conflicting writes return `UseCaseError::EtagMismatch`.
- **AC-8:** Default-deny: Cedar gate fails closed; unknown principals return `UseCaseError::PermissionDenied`.

## C. Verification

```bash
cargo test -p oya-production-planning-bom-usecase -- create_happy_path
cargo test -p oya-production-planning-bom-usecase -- create_rolls_back_on_cedar_deny
cargo test -p oya-production-planning-bom-usecase -- create_rolls_back_on_repo_fail
cargo test -p oya-production-planning-bom-usecase -- create_idempotent_on_repeat_key
cargo test -p oya-production-planning-bom-usecase -- create_emits_outbox_event
cargo test -p oya-production-planning-bom-usecase -- release_rejects_when_engineering_hold
cargo test -p oya-production-planning-bom-usecase -- supersede_freezes_prior_revision
cargo test -p oya-production-planning-bom-usecase -- etag_mismatch_detected
cargo test -p oya-production-planning-bom-usecase -- cross_tenant_double_layer_denied
```

## D. Detailed mechanics

### D-1. Use-case signatures

```rust
pub struct CreateBomRevisionUseCase<R, C, O, A> {
    pub repo: R,                 // BomRepository
    pub cedar: C,                // CedarEvaluator
    pub outbox: O,               // OutboxDispatcher
    pub audit: A,                // AuditEmitter
}

#[derive(Debug, Clone)]
pub struct CreateBomRevisionInput {
    pub tenant_id: TenantId,
    pub principal: Principal,
    pub policy_bundle_version: PolicyBundleVersion,
    pub idempotency_key: IdempotencyKey,
    pub material_id: MaterialId,
    pub plant_code: PlantCode,
    pub usage: BomUsage,
    pub base_quantity: Decimal,
    pub base_uom: UnitOfMeasure,
    pub positions: Vec<BomPositionDraft>,
    pub effective_from: Hlc,
}

#[derive(Debug, Clone)]
pub struct CreateBomRevisionOutput {
    pub bom_id: BomId,
    pub revision_no: RevisionNo,
    pub etag: Etag,
    pub event_id: EventId,
    pub cedar_decision_id: CedarDecisionId,
}
```

### D-2. Execution flow

```rust
impl<R: BomRepository, C: CedarEvaluator, O: OutboxDispatcher, A: AuditEmitter>
    CreateBomRevisionUseCase<R, C, O, A>
{
    pub async fn execute(&self, input: CreateBomRevisionInput) -> Result<CreateBomRevisionOutput, UseCaseError> {
        // 1. Idempotency check (read-only)
        if let Some(prior) = self.repo.find_by_idempotency_key(&input.tenant_id, &input.idempotency_key).await? {
            return Ok(prior.into());
        }
        // 2. Cedar gate
        let decision = self.cedar.evaluate(CedarRequest {
            principal: input.principal.clone(),
            action: action_id("production_planning::bom::create"),
            resource: bom_resource(&input),
            context: cedar_context(&input),
        }).await?;
        if !decision.is_permit() { return Err(UseCaseError::PermissionDenied { reason: decision.reasons() }); }
        // 3. Domain construction
        let positions: Vec<BomPosition> = input.positions.iter().map(BomPositionDraft::to_position).collect::<Result<_, _>>()?;
        let aggregate = BomRevision::new(
            input.tenant_id, BomId::new(), RevisionNo::initial(),
            input.material_id.clone(), input.plant_code, input.usage,
            input.base_quantity, input.base_uom, positions, input.effective_from,
        )?;
        aggregate.validate_acyclic()?;
        // 4. Begin transaction
        let tx = self.repo.begin_tx().await?;
        // 5. Persist
        let etag = self.repo.save(&tx, &aggregate).await?;
        // 6. Dispatch outbox
        let event = make_event(EVT_BOM_REVISION_CREATED, &aggregate, &decision, &input);
        self.outbox.append(&tx, &event).await?;
        // 7. Audit
        self.audit.emit(&tx, AuditEntry::from(&decision, &aggregate)).await?;
        // 8. Commit
        tx.commit().await?;
        Ok(CreateBomRevisionOutput {
            bom_id: aggregate.bom_id(), revision_no: aggregate.revision_no(),
            etag, event_id: event.event_id, cedar_decision_id: decision.decision_id,
        })
    }
}
```

### D-3. Port traits

```rust
#[async_trait]
pub trait BomRepository: Send + Sync {
    async fn begin_tx(&self) -> Result<RepoTx, RepoError>;
    async fn find_by_idempotency_key(&self, tenant_id: &TenantId, key: &IdempotencyKey) -> Result<Option<BomRevisionRecord>, RepoError>;
    async fn save(&self, tx: &RepoTx, agg: &BomRevision) -> Result<Etag, RepoError>;
    async fn load(&self, tenant_id: &TenantId, bom_id: &BomId, revision_no: RevisionNo) -> Result<Option<BomRevision>, RepoError>;
    async fn supersede(&self, tx: &RepoTx, agg: &BomRevision, prior_etag: &Etag) -> Result<Etag, RepoError>;
}

#[async_trait]
pub trait CedarEvaluator: Send + Sync {
    async fn evaluate(&self, req: CedarRequest) -> Result<CedarDecision, CedarError>;
}

#[async_trait]
pub trait OutboxDispatcher: Send + Sync {
    async fn append(&self, tx: &RepoTx, event: &OutboxEvent) -> Result<(), OutboxError>;
}

#[async_trait]
pub trait AuditEmitter: Send + Sync {
    async fn emit(&self, tx: &RepoTx, entry: AuditEntry) -> Result<(), AuditError>;
}
```

### D-4. Typed errors

```rust
#[derive(thiserror::Error, Debug)]
pub enum UseCaseError {
    #[error("permission denied: {reason:?}")] PermissionDenied { reason: Vec<String> },
    #[error("etag mismatch: expected {expected}, got {actual}")] EtagMismatch { expected: Etag, actual: Etag },
    #[error("domain error: {0}")] Domain(#[from] BomError),
    #[error("repository error: {0}")] Repo(#[from] RepoError),
    #[error("cedar error: {0}")] Cedar(#[from] CedarError),
    #[error("outbox error: {0}")] Outbox(#[from] OutboxError),
    #[error("audit error: {0}")] Audit(#[from] AuditError),
    #[error("idempotency conflict — payload differs from prior")] IdempotencyConflict,
}
```

### D-5. Cedar policy fragment

```cedar
@id("production_planning::bom::create::v1")
@soak_started_at("2026-05-20T00:00:00Z")
permit (
  principal in ProductionPlanning::Role::"role-design-engineer",
  action == ProductionPlanning::Action::"bom_create",
  resource in ProductionPlanning::Bom::?
) when {
  context.tenant_id == principal.tenant_id &&
  context.policy_bundle_version >= "2026-05-20" &&
  resource.plant_code in principal.authorized_plants
};
```

### D-6. Outbox event schema

```yaml
event:
  audit_event_class: EVT-PRODUCTION_PLANNING-BOM_REVISION-CREATED
  tenant_id: <uuid>
  bom_id: <ulid>
  revision_no: <int>
  material_id: <string>
  plant_code: <string>
  occurred_at: <iso8601>
  hlc: <base32>
  correlation_id: <string>
  causation_id: <string>
  cedar_decision_id: <string>
  policy_bundle_version: <string>
  signature: <base64>
```

### D-7. SLO contribution

End-to-end usecase: ≤ 35ms P95 (Cedar eval 5ms + repo save 15ms + outbox append 5ms + audit 5ms + overhead 5ms). Idempotency hit path ≤ 8ms P95.

### D-8. Audit anchoring

Per ADR-0316, every audit row is signed Ed25519 by µservice sidecar; root-of-anchor written to immutable ledger.

## E. Failure modes & recovery

### E-1. Cedar deny on create
Operator sees `403 Forbidden` with `reason` array; runbook `runbooks/bom-cedar-deny.md`.

### E-2. Repo transaction failed mid-commit
Outbox row not persisted (rolled back). HTTP 503; idempotency key remains usable.

### E-3. Outbox dispatcher slow / queue full
Backpressure on use-case; 429 returned. Runbook `runbooks/outbox-backpressure.md`.

### E-4. Etag mismatch (concurrent edit)
HTTP 409. Client re-reads, re-applies, retries.

## F. Migration

Phase 1: domain (IP-001).
Phase 2: this usecase.
Phase 3 (IP-013): adapter wires real Postgres / NATS / Cedar.
Phase 4: HTTP / gRPC surface (IP-014).

Rollback: feature flag `production_planning_bom_usecase_v1` → false; existing legacy CRUD endpoints remain.

## G. References

- ADR-0105, ADR-0244, ADR-0252, ADR-0263, ADR-0294, ADR-0297, ADR-0316.
- Evans, E. (2003). *Domain-Driven Design.* ApplicationService pattern.
- SAP Help: function modules `CSAP_MAT_BOM_*`.
- Benchmarks: SAP CS01/CS02 | Oracle Fusion BOM | Microsoft Dynamics 365 BOM | NetSuite Manufacturing BOM | Siemens Teamcenter BOM management.

## H. Out-of-scope

- Adapter wiring (IP-013).
- HTTP surface (IP-014).
- Cross-µservice handoff (IP-016).

— end IP-007 —
