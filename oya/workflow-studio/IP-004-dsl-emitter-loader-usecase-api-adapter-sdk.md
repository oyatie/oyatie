---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-studio-preview
phase: P01-visual-authoring-substrate
impl_plan_id: IP-004-dsl-emitter-loader-usecase-api-adapter-sdk
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness]
depends_on: [IP-003]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: dsl-emitter + dsl-loader — usecase + api + adapter + sdk

## Intent

Complete the dsl-emitter + dsl-loader BCs by adding usecase, api, adapter, and sdk layers atop IP-003's kernel + domain. Usecases orchestrate emit + validation; adapters provide protocol-neutral implementations; SDKs provide tenant-side client libraries for spec construction.

## ChangeSet boundary

Eight crates (4 per BC × 2 BCs):
- `oya-workflow-studio-dsl-emitter-{usecase,api,adapter,sdk}`
- `oya-workflow-studio-dsl-loader-{usecase,api,adapter,sdk}`

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-workflow-studio-dsl-emitter-usecase/{Cargo.toml,src/lib.rs,src/orchestrator.rs,tests/orchestrator.rs}` | create |
| `src/crates/oya-workflow-studio-dsl-emitter-api/{Cargo.toml,src/lib.rs,src/contracts.rs}` | create |
| `src/crates/oya-workflow-studio-dsl-emitter-adapter/{Cargo.toml,src/lib.rs,src/impl.rs,tests/impl.rs}` | create |
| `src/crates/oya-workflow-studio-dsl-emitter-sdk/{Cargo.toml,src/lib.rs,src/client.rs}` | create |
| `src/crates/oya-workflow-studio-dsl-loader-usecase/{Cargo.toml,src/lib.rs,src/orchestrator.rs,tests/orchestrator.rs}` | create |
| `src/crates/oya-workflow-studio-dsl-loader-api/{Cargo.toml,src/lib.rs,src/contracts.rs}` | create |
| `src/crates/oya-workflow-studio-dsl-loader-adapter/{Cargo.toml,src/lib.rs,src/impl.rs,tests/impl.rs}` | create |
| `src/crates/oya-workflow-studio-dsl-loader-sdk/{Cargo.toml,src/lib.rs,src/client.rs}` | create |
| `microservices/workflow-studio/catalog/oya-workflow-studio-dsl-emitter-{usecase,api,adapter,sdk}.yaml` | create |
| `microservices/workflow-studio/catalog/oya-workflow-studio-dsl-loader-{usecase,api,adapter,sdk}.yaml` | create |

## Code Shape

`dsl-emitter-usecase/src/orchestrator.rs`:

```rust
use oya_workflow_studio_dsl_emitter_kernel::ports::{SpecEmitter, EmitContext};
use oya_workflow_studio_visual_canvas_kernel::entities::Canvas;

pub struct EmitOrchestrator<E: SpecEmitter> {
    emitter: E,
}

impl<E: SpecEmitter> EmitOrchestrator<E> {
    pub fn new(emitter: E) -> Self { Self { emitter } }

    pub fn run(&self, ctx: &EmitContext, canvas: &Canvas)
        -> Result<oya_workflow_studio_dsl_emitter_kernel::ports::EmittedSpec, EmitOrchestratorError>
    {
        // 1. Pre-emit validation (canvas-side invariants).
        if canvas.tenant_id != ctx.tenant_id {
            return Err(EmitOrchestratorError::TenantMismatch);
        }
        // 2. Emit.
        let emitted = self.emitter.emit(ctx, canvas)?;
        // 3. Post-emit invariants (schema validation scheduled-for-distinct-tracked-work to upstream lane).
        Ok(emitted)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum EmitOrchestratorError {
    #[error("tenant mismatch")] TenantMismatch,
    #[error(transparent)] Emit(#[from] oya_workflow_studio_dsl_emitter_kernel::ports::EmitError),
}
```

## Acceptance Gates

```bash
cargo check -p oya-workflow-studio-dsl-emitter-usecase -p oya-workflow-studio-dsl-emitter-api \
  -p oya-workflow-studio-dsl-emitter-adapter -p oya-workflow-studio-dsl-emitter-sdk \
  -p oya-workflow-studio-dsl-loader-usecase -p oya-workflow-studio-dsl-loader-api \
  -p oya-workflow-studio-dsl-loader-adapter -p oya-workflow-studio-dsl-loader-sdk
cargo nextest run --workspace --tests
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice workflow-studio
cargo run -p oya-dev-cli -- gate validate lean-a1 --microservice workflow-studio
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_orchestrator_tenant_isolation` | canvas with wrong tenant_id → rejected |
| `test_orchestrator_emit_happy_path` | canvas valid → spec emitted |
| `test_sdk_client_round_trip` | SDK round-trips via in-process adapter |

## Halt Conditions

- Layer-correctness lane fails — restructure imports.
- usecase imports adapter directly — restructure (must use port).

## Next IP

[`IP-005-collab-crdt-kernel-domain-adapter.md`](IP-005-collab-crdt-kernel-domain-adapter.md)

## References

- ADR-0105 layer enum.
- ADR-0106 application → usecase rename.
- PRD §"Bounded Contexts" + §"Clean Architecture Compliance".

## Counterpart Anchors
This workflow-studio IP is measured against the local Workflow Studio benchmark envelope: n8n for visual workflow authoring depth, Zapier for broad trigger/action accessibility, Make for visual branching and scenario ergonomics, and Workato for enterprise workflow governance. The IP must keep Oyatie's differentiator intact: canonical workflow_spec.v1 round-trip, Cedar-gated save/publish, tenant-scoped collaboration, and audit evidence rather than counterpart-specific runtime authority.

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`workflow-studio`].
- surface_evidence_paths: [`microservices/workflow-studio/IP-004-dsl-emitter-loader-usecase-api-adapter-sdk.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/templates/index.json`, `microservices/workflow-studio/templates/schemas/workflow-template.schema.json`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/ARCHITECTURE.md`].
