---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-meta-iac-pipeline-substrate
impl_plan_id: IP-007-iac-applier-kernel-domain-usecase
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-cloud-iac
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, layer-correctness, check-iac-apply-scope, cloud-iac-provenance-slsa-l3]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: cloud-iac-iac-applier-{kernel,domain,usecase,api,adapter,adapter-argocd}

## Intent

Scaffold the iac-applier BC's core stack: kernel (port traits) + domain (dependency-ordering + retry policy) + usecase (apply orchestrator) + api + adapter (Kubernetes API client) + adapter-argocd (ArgoCD reconciler client). Applier enforces per-µservice scope per Cedar + SLSA L3 verification pre-apply.

## ChangeSet boundary

Six new crates per ADR-0105: `-kernel`, `-domain`, `-usecase`, `-api`, `-adapter`, `-adapter-argocd`. Catalog rows.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cloud-iac/src/crates/cloud-iac-iac-applier-kernel/{Cargo.toml,src/lib.rs,src/entities.rs,src/ports.rs,src/errors.rs}` | create |
| `microservices/cloud-iac/src/crates/cloud-iac-iac-applier-domain/{Cargo.toml,src/lib.rs,src/dep_order.rs,src/retry_policy.rs}` | create |
| `microservices/cloud-iac/src/crates/cloud-iac-iac-applier-usecase/{Cargo.toml,src/lib.rs,src/apply_orchestrator.rs,src/slsa_verifier.rs}` | create |
| `microservices/cloud-iac/src/crates/cloud-iac-iac-applier-api/{Cargo.toml,src/lib.rs}` | create |
| `microservices/cloud-iac/src/crates/cloud-iac-iac-applier-adapter/{Cargo.toml,src/lib.rs,src/k8s_client.rs,src/event_emitter.rs}` | create |
| `microservices/cloud-iac/src/crates/cloud-iac-iac-applier-adapter-argocd/{Cargo.toml,src/lib.rs,src/argocd_client.rs}` | create |
| `microservices/cloud-iac/catalog/cloud-iac-iac-applier-*.yaml` | create (6 rows) |

## Code Shape

```rust
// kernel/src/entities.rs
pub struct ApplyJob {
    #[data_class(AUDIT)]         pub apply_id: String,  // ULID
    #[data_class(INTERNAL_ONLY)] pub microservice: String,
    #[data_class(INTERNAL_ONLY)] pub pack: String,
    #[data_class(INTERNAL_ONLY)] pub environment: Environment,
    #[data_class(INTERNAL_ONLY)] pub sha: String,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)] pub apply_scope: ApplyScope,
    #[data_class(AUDIT)]         pub state: ApplyState,
}

pub struct ApplyScope {
    pub microservice: String,
    pub namespaces: Vec<String>,
    pub cluster_roles: Vec<String>,
}
```

```rust
// usecase/src/apply_orchestrator.rs
pub struct ApplyOrchestrator<M, R, S, E> {
    cluster_mutator: M,
    reconciler: R,
    slsa_verifier: S,
    event_emitter: E,
}

impl<M, R, S, E> ApplyOrchestrator<M, R, S, E>
where M: ClusterMutator, R: ReconcilerClient, S: SlsaVerifier, E: ApplyEventEmitter {
    pub async fn apply(&self, job: &ApplyJob) -> Result<ApplyResult, ApplyError> {
        // 1. Verify SLSA L3 attestation chain
        self.slsa_verifier.verify(&job.content_digest).await?;
        // 2. Verify apply-scope (Cedar policy already evaluated upstream; double-check here)
        if !self.in_scope(&job.target_resources, &job.apply_scope) {
            return Err(ApplyError::ScopeViolation);
        }
        // 3. Emit ApplyStarted
        self.event_emitter.emit_apply_started(&job).await?;
        // 4. Orchestrate via ArgoCD reconciler (or direct k8s API)
        let result = self.reconciler.sync(&job).await?;
        // 5. Emit ApplyCompleted
        self.event_emitter.emit_apply_completed(&job, &result).await?;
        Ok(result)
    }
}
```

## Acceptance Gates

```bash
cargo check --workspace -p cloud-iac-iac-applier-kernel -p cloud-iac-iac-applier-domain -p cloud-iac-iac-applier-usecase -p cloud-iac-iac-applier-api -p cloud-iac-iac-applier-adapter -p cloud-iac-iac-applier-adapter-argocd --all-features
cargo nextest run --workspace -p cloud-iac-iac-applier-* --all-features
cloud-ci/ci governance gate `iac-apply-scope` for --microservice cloud-iac is green in the branch-protected `presubmit` context
cloud-ci/ci governance gate `provenance-slsa-l3` for --microservice cloud-iac is green in the branch-protected `presubmit` context
```

## Test Plan

| Test | Layer | Verifies |
|---|---|---|
| `test_dep_order_topological` | domain | apply order respects dependencies |
| `test_retry_policy_exponential_backoff` | domain | retry budget honored |
| `test_apply_orchestrator_slsa_refuse` | usecase | unsigned chart refused |
| `test_apply_orchestrator_scope_refuse` | usecase | out-of-scope resource refused |
| `test_apply_orchestrator_happy_path` | usecase | in-scope + signed → applied |
| `integration_argocd_sync` | adapter-argocd | against kind ArgoCD instance |

## Halt Conditions

- SLSA verification path made optional — refuse.
- Apply-scope check bypassable via crafted manifest — refuse.

## Next IP

[`IP-008-iac-registry-postgres.md`](IP-008-iac-registry-postgres.md)

## References

- ADR-0105; ADR-0139; ADR-0140 (retired per ADR-0145).
- PRD §"Bounded Contexts" iac-applier BC.
- ArgoCD reconciler API — `argo-cd.readthedocs.io/en/stable/operator-manual/`.
- OpenSSF SLSA L3 — `slsa.dev/spec/v1.0/`.
