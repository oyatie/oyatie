---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate
phase: P15-data-boundary
impl_plan_id: IP-001-data-boundary-kernel-scaffold
status: pending
owner: council-architecture
blocked_by:
- impl_plan: P14-policy/IP-001
  reason: CedarDubEvaluator calls PolicyEvaluator port from oya-policy-engine-kernel
    for HARD_DENY cedar evaluation
acceptance_lanes:
- cargo-check
- cargo-build
- cargo-clippy
- cargo-nextest
- cargo-deny
- lean-a1
- lean-a2
- lean-a3
- lean-a4
purpose: "Scaffolds all 8 data-boundary crates across 2 BCs, declares the 12-class DataClass enum, implements `DubEvaluator::evaluate()` with HARD_DENY semantics for PHI/PCI/PIPA/children."
execution_variant: merge-into-existing-crates
decided_at: "2026-05-17"
decided_by: user-directive-option-2
execution_variant_note: "Delta-1 merges ClassificationLevel, DataClassMatcher, RetentionPolicy into existing oya-data-boundary-kernel via new retention_policy.rs module. No new crates scaffolded. References F-M02B-PLAN-LIVE-CRATE-RECONCILIATION (filed P04). Adds 11 unit tests covering hard-deny tier mapping, matcher sets (HardDenySet/RegulatedFinancial/DirectPii/SearchIndexRestricted), and retention window/purge-action defaults."
---
# IP-001-data-boundary-kernel-scaffold: Scaffold DUB Engine + Classification Kernel/Domain/Application/Adapter/gRPC/App — 12 DataClass + HARD_DENY DDL + Cedar Fragment

## Intent

Scaffolds all 8 data-boundary crates across 2 BCs, declares the 12-class DataClass enum,
implements `DubEvaluator::evaluate()` with HARD_DENY semantics for PHI/PCI/PIPA/children,
authors the Cedar DUB policy fragment (`cedar/data_boundary.cedar`), and wires the
composition root. After this IP merges, every Ontology Action Type that touches a
regulated object calls `DubEvaluator::evaluate()` before committing — and the write is
unconditionally rejected on HARD_DENY.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `Cargo.toml` | update | Add 8 data-boundary workspace members |
| `crates/oya-data-boundary-engine-kernel/Cargo.toml` | create | Zero framework deps |
| `crates/oya-data-boundary-engine-kernel/src/lib.rs` | create | pub mod types; pub mod ports; pub mod errors |
| `crates/oya-data-boundary-engine-kernel/src/types.rs` | create | DataClass (12 variants), DataUseRequest, DubDecision, DubDenyEntry, DataUseContext |
| `crates/oya-data-boundary-engine-kernel/src/ports.rs` | create | DubEvaluator + DubAuditStore — sealed |
| `crates/oya-data-boundary-engine-kernel/src/errors.rs` | create | DubError enum |
| `crates/oya-data-boundary-engine-domain/Cargo.toml` | create | Depends on kernel only |
| `crates/oya-data-boundary-engine-domain/src/lib.rs` | create | DubRules: hard_deny_classes(); is_permitted(class, context); jurisdiction_overlay() |
| `crates/oya-data-boundary-engine-application/Cargo.toml` | create | Depends on domain + kernel |
| `crates/oya-data-boundary-engine-application/src/lib.rs` | create | EvaluateDubUseCase; RecordDubDenyUseCase |
| `crates/oya-data-boundary-engine-adapter/Cargo.toml` | create | Depends on application + domain + kernel + oya-policy-engine-kernel + sqlx |
| `crates/oya-data-boundary-engine-adapter/src/lib.rs` | create | module declarations |
| `crates/oya-data-boundary-engine-adapter/src/cedar_dub_evaluator.rs` | create | CedarDubEvaluator: impl DubEvaluator; calls PolicyEvaluator port for Cedar eval; returns HardDeny for PHI/PCI/PIPA/children |
| `crates/oya-data-boundary-engine-adapter/src/pg_dub_audit.rs` | create | PgDubAuditStore: impl DubAuditStore; RLS on data_boundary.deny_log |
| `crates/oya-data-boundary-engine-grpc/Cargo.toml` | create | tonic; depends on application + kernel |
| `crates/oya-data-boundary-engine-grpc/src/lib.rs` | create | DataBoundaryService gRPC: Evaluate + RecordDeny |
| `crates/oya-data-boundary-engine-app/Cargo.toml` | create | Composition root |
| `crates/oya-data-boundary-engine-app/src/main.rs` | create | DI assembly |
| `crates/oya-data-boundary-classification-kernel/Cargo.toml` | create | Zero framework deps |
| `crates/oya-data-boundary-classification-kernel/src/lib.rs` | create | DataClassificationStore port; ObjectDataClass type |
| `crates/oya-data-boundary-classification-adapter/Cargo.toml` | create | Depends on classification-kernel + sqlx |
| `crates/oya-data-boundary-classification-adapter/src/lib.rs` | create | PgDataClassificationStore: impl DataClassificationStore |
| `cedar/data_boundary.cedar` | create | Cedar DUB policy fragment (see Code Shape) |
| `contracts/data_boundary.proto` | create | DataBoundaryService rpc Evaluate |
| `migrations/data_boundary/V001__data_boundary_schema.sql` | create | Full DDL |
| `docs/standards/bounded-contexts.md` | update | Register data-boundary-engine + data-boundary-classification BCs |

---

## Crate Naming

```
NAME: oya-data-boundary-engine-kernel
JUSTIFICATION:
- microservice = data-boundary: 2-token microservice name; ADR-0008; ADR-0056 v4.1
  allows up to 3 tokens; "data-boundary" is the registered µservice name
- bc-tokens = engine: evaluation loop BC; separate from classification BC
- layer = kernel: DataClass enum + sealed ports DubEvaluator + DubAuditStore
- exemptions claimed: none
```

---

## Code Shape

### `cedar/data_boundary.cedar`

```cedar
// Data-Use-Boundary Cedar policy fragment
// Loaded into policy.tenant_rule_packs as pack_name = "system:dub-v1" (system pack; not tenant-editable)

entity Tenant;
entity Principal in [Tenant] = { role: String, org_level: String };
entity DataObject in [Tenant] = {
    data_class: String,   // one of 12 DataClass values
    pillar: String        // "person" | "org"
};

action Read    appliesTo { principal: Principal, resource: DataObject };
action Write   appliesTo { principal: Principal, resource: DataObject };
action Export  appliesTo { principal: Principal, resource: DataObject };
action Share   appliesTo { principal: Principal, resource: DataObject };

// HARD_DENY rules — unconditionally forbid regulated class mutations
// PHI: only clinical principals may write; nobody may export without explicit release
forbid (principal, action in [Write, Export, Share], resource is DataObject)
    when { resource.data_class == "Phi" && principal.role != "clinical" };

// PCI: no export or share under any circumstance from this engine
forbid (principal, action in [Export, Share], resource is DataObject)
    when { resource.data_class == "Pci" };

// PIPA: cross-border transfer forbidden without explicit consent flag in context
forbid (principal, action in [Export, Share], resource is DataObject)
    when { resource.data_class == "Pipa" };

// Children: read + write restricted to designated child-data roles
forbid (principal, action in [Read, Write, Export, Share], resource is DataObject)
    when { resource.data_class == "Children" && principal.role != "child_data_custodian" };

// Org-admin cannot read person-pillar regulated objects (ADR-0132 pillar rule)
forbid (principal, action == Read, resource is DataObject)
    when { principal.role == "org_admin" && resource.pillar == "person"
           && resource.data_class in ["Phi", "Pipa", "BiometricId", "Genetic"] };
```

### `migrations/data_boundary/V001__data_boundary_schema.sql`

```sql
CREATE SCHEMA IF NOT EXISTS data_boundary;

-- Per-object data class declarations
CREATE TABLE data_boundary.object_classifications (
    classification_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    object_id uuid NOT NULL,        -- FK to ontology.objects(object_id) at app layer
    data_class text NOT NULL CHECK (data_class IN (
        'Phi','Pci','Pipa','Children','BiometricId',
        'FinancialAccount','GovernmentId','Employment',
        'LocationHistory','Communications','Genetic','Behavioral'
    )),
    declared_by uuid NOT NULL,
    declared_at timestamptz NOT NULL DEFAULT now(),
    revoked_at timestamptz NULL
);
ALTER TABLE data_boundary.object_classifications FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON data_boundary.object_classifications
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE UNIQUE INDEX idx_obj_class_active ON data_boundary.object_classifications
    (tenant_id, object_id) WHERE revoked_at IS NULL;
COMMENT ON TABLE data_boundary.object_classifications IS 'distribution_column:tenant_id';

-- HARD_DENY audit log (append-only; every denied request recorded)
CREATE TABLE data_boundary.deny_log (
    deny_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    principal_type text NOT NULL,
    principal_id uuid NULL,
    action text NOT NULL,
    object_id uuid NULL,
    data_class text NOT NULL,
    denial_reason text NOT NULL,
    denied_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE data_boundary.deny_log FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON data_boundary.deny_log
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- Append-only trigger
CREATE OR REPLACE FUNCTION data_boundary.deny_log_immutable() RETURNS trigger AS $$
BEGIN RAISE EXCEPTION 'data_boundary.deny_log is append-only'; END
$$ LANGUAGE plpgsql;
CREATE TRIGGER no_update BEFORE UPDATE ON data_boundary.deny_log
    FOR EACH ROW EXECUTE FUNCTION data_boundary.deny_log_immutable();
CREATE TRIGGER no_delete BEFORE DELETE ON data_boundary.deny_log
    FOR EACH ROW EXECUTE FUNCTION data_boundary.deny_log_immutable();

CREATE INDEX idx_deny_log_tenant_class ON data_boundary.deny_log
    (tenant_id, data_class, denied_at DESC);
COMMENT ON TABLE data_boundary.deny_log IS 'distribution_column:tenant_id';
```

---

## Acceptance Gates

```bash
cargo check --workspace --all-features                                          # exit 0
cargo build --workspace --all-features                                          # exit 0
cargo clippy --workspace --all-features -- -D warnings                          # exit 0
cargo nextest run --workspace --all-features                                    # exit 0
cargo nextest run -p oya-data-boundary-engine-adapter --test hard_deny_phi      # exit 0
cargo nextest run -p oya-data-boundary-engine-adapter --test hard_deny_pci      # exit 0
cargo nextest run -p oya-data-boundary-engine-adapter --test hard_deny_pipa     # exit 0
cargo nextest run -p oya-data-boundary-engine-adapter --test hard_deny_children # exit 0
cargo nextest run -p oya-data-boundary-engine-adapter --test cedar_dub_policy_load  # exit 0
cargo deny check                                                                # exit 0
oya gate validate lean-a1 --phase P15-data-boundary
oya gate validate lean-a2 --phase P15-data-boundary
oya gate validate shardability --phase P15-data-boundary
```

---

## Test Plan

### Unit tests

| Test name | What it verifies |
|---|---|
| `test_data_class_hard_deny_set` | PHI/PCI/PIPA/Children → is_hard_deny_class() = true; others = false |
| `test_dub_rules_phi_write_non_clinical` | DubRules: Write on PHI by non-clinical principal → Deny |
| `test_dub_rules_pci_export` | Export on PCI → always Deny regardless of role |
| `test_dub_rules_children_read_non_custodian` | Read on Children by non-custodian → Deny |
| `test_deny_log_append_only` | UPDATE/DELETE on deny_log raises exception |

### Integration tests

| Test name | What it verifies |
|---|---|
| `integration_hard_deny_phi` | CedarDubEvaluator: PHI Write by non-clinical → HardDeny; write not committed |
| `integration_hard_deny_pci` | PCI Export → HardDeny; deny_log entry created |
| `integration_hard_deny_pipa` | PIPA Share → HardDeny; audit-chain event forwarded |
| `integration_hard_deny_children` | Children Read by non-custodian → HardDeny |
| `integration_classification_rls` | Tenant A cannot read tenant B object classifications |
| `integration_cedar_dub_policy_load` | cedar/data_boundary.cedar loads without parse errors |

---

## Load Test

```javascript
// tests/load/smoke-data-boundary.js
export const options = {
  vus: 100, duration: '60s',
  thresholds: {
    http_req_duration: ['p(99)<10'],   // DUB eval p99 ≤10ms
    http_req_failed: ['rate<0.001'],
  },
};
```

| Scenario | Target | Pass criterion |
|---|---|---|
| DUB evaluate (Allow case) | p99 ≤10ms at 10k RPS | `http_req_duration{p(99)}<10` |
| DUB evaluate (HardDeny + audit write) | p99 ≤20ms at 1k RPS | `http_req_duration{p(99)}<20` |

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent council-architecture \
  --intent "IP-001-data-boundary-kernel-scaffold: 12 DataClass + HARD_DENY" \
  --ttl 3600 \
  crates/oya-data-boundary-engine-kernel/src/lib.rs::DubEvaluator \
  crates/oya-data-boundary-engine-kernel/src/lib.rs::DataClass \
  cedar/data_boundary.cedar::HardDenyPhi \
  migrations/data_boundary/V001__data_boundary_schema.sql::data_boundary.deny_log
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-001-data-boundary-kernel-scaffold merged; 12 DataClass; HARD_DENY for PHI/PCI/PIPA/children verified; deny_log append-only; Cedar DUB policy loaded; next: IP-002-data-boundary-cedar-dub-policy" \
  -i high \
  -k "M02,P15,IP-001,data-boundary"
```

---

## Halt Conditions

1. HARD_DENY can be bypassed by any code path — escalate immediately; this is a regulatory requirement.
2. Cedar DUB policy produces non-deterministic results — escalate to architect.
3. deny_log append-only trigger is removable by a non-superuser — escalate; revise schema permissions.
4. LEAN-A2 violation: data-boundary importing a product crate — escalate.

---

## Next IP Pointer

`IP-002-data-boundary-cedar-dub-policy.md`

---

## Cross-References

- Phase spec: `phase-spec.md`
- ADR-0008 (data-use-boundary), ADR-0007 (Cedar), ADR-0132 (pillars), ADR-0056 (BNF v4.1)
