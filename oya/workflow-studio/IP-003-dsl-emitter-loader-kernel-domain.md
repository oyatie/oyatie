---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-studio-preview
phase: P01-visual-authoring-substrate
impl_plan_id: IP-003-dsl-emitter-loader-kernel-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness, oya-governance-workflow-spec-roundtrip]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: dsl-emitter + dsl-loader — kernel + domain

## Intent

Author the kernel + domain layers of the paired BCs `dsl-emitter` (visual → canonical workflow_spec.v1.json) and `dsl-loader` (workflow_spec.v1.json → visual). The load(emit(visual)) == visual round-trip byte-equality is the load-bearing invariant per AC-02; property-tested over a 100-spec reference corpus.

## ChangeSet boundary

Four crates:
- `oya-workflow-studio-dsl-emitter-kernel` — port-trait `SpecEmitter` + entity types (`EmitContext`, `EmittedSpec`, `EmitDiagnostic`).
- `oya-workflow-studio-dsl-emitter-domain` — pure visual→spec mapping; deterministic.
- `oya-workflow-studio-dsl-loader-kernel` — port-trait `SpecLoader` + entities (`LoadContext`, `LoadedDefinition`, `LoadDiagnostic`).
- `oya-workflow-studio-dsl-loader-domain` — pure spec→visual mapping; deterministic.

Round-trip property test asserts: `load(emit(canvas)) == canvas` AND `emit(load(spec)) == spec` (byte-identical).

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/workflow-studio/src/crates/oya-workflow-studio-dsl-emitter-kernel/{Cargo.toml,src/lib.rs,src/entities.rs,src/ports.rs}` | create |
| `microservices/workflow-studio/src/crates/oya-workflow-studio-dsl-emitter-domain/{Cargo.toml,src/lib.rs,src/emit.rs,tests/emit.rs}` | create |
| `microservices/workflow-studio/src/crates/oya-workflow-studio-dsl-loader-kernel/{Cargo.toml,src/lib.rs,src/entities.rs,src/ports.rs}` | create |
| `microservices/workflow-studio/src/crates/oya-workflow-studio-dsl-loader-domain/{Cargo.toml,src/lib.rs,src/load.rs,tests/load.rs}` | create |
| `microservices/workflow-studio/capabilities/eval/round-trip-reference-corpus.jsonl` | create | 100 reference spec corpus per FR-06 |
| `microservices/workflow-studio/src/crates/oya-workflow-studio-dsl-loader-domain/tests/round_trip_byte_equal.rs` | create | the load-bearing AC-02 property test |
| `microservices/workflow-studio/catalog/oya-workflow-studio-dsl-emitter-{kernel,domain}.yaml` | create |
| `microservices/workflow-studio/catalog/oya-workflow-studio-dsl-loader-{kernel,domain}.yaml` | create |

## Code Shape

`dsl-emitter-kernel/src/ports.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmitContext {
    pub tenant_id: String,
    pub definition_id: String,
    pub jurisdiction_overlay: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmittedSpec {
    pub spec_id: String,
    pub version_sha: String,
    pub body_canonical_json: Vec<u8>,
    pub diagnostics: Vec<EmitDiagnostic>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmitDiagnostic {
    pub severity: String,
    pub json_pointer: String,
    pub message: String,
}

pub trait SpecEmitter: Send + Sync {
    fn emit(&self, ctx: &EmitContext, canvas: &oya_workflow_studio_visual_canvas_kernel::entities::Canvas)
        -> Result<EmittedSpec, EmitError>;
}

#[derive(thiserror::Error, Debug)]
pub enum EmitError {
    #[error("schema violation")] SchemaViolation,
    #[error("tenant mismatch")] TenantMismatch,
    #[error(transparent)] Other(#[from] anyhow::Error),
}
```

`dsl-loader-domain/tests/round_trip_byte_equal.rs`:

```rust
use std::fs::read_to_string;
use proptest::prelude::*;

#[test]
fn test_load_emit_byte_equal_over_corpus() {
    let corpus_path = "microservices/workflow-studio/capabilities/eval/round-trip-reference-corpus.jsonl";
    let lines = read_to_string(corpus_path).expect("corpus exists");
    let mut ok = 0; let mut total = 0;
    for line in lines.lines() {
        total += 1;
        let spec: serde_json::Value = serde_json::from_str(line).unwrap();
        let canonical = serde_json::to_string(&spec).unwrap();
        let canvas = oya_workflow_studio_dsl_loader_domain::load::load_canvas(&spec).expect("loadable");
        let re_emitted = oya_workflow_studio_dsl_emitter_domain::emit::emit_spec(&canvas).expect("emittable");
        let re_canonical = serde_json::to_string(&re_emitted.body).unwrap();
        if re_canonical == canonical { ok += 1; }
    }
    assert_eq!(ok, total, "round-trip byte-equality failed on {} of {}", total - ok, total);
}
```

## Acceptance Gates

```bash
cargo check -p oya-workflow-studio-dsl-emitter-kernel -p oya-workflow-studio-dsl-emitter-domain \
  -p oya-workflow-studio-dsl-loader-kernel -p oya-workflow-studio-dsl-loader-domain
cargo nextest run -p oya-workflow-studio-dsl-loader-domain --test round_trip_byte_equal
buck2 build //:quality-lane-registry-authority-check # lane=workflow-spec-roundtrip --microservice workflow-studio \
  --spec-corpus microservices/workflow-studio/capabilities/eval/round-trip-reference-corpus.jsonl
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_load_emit_byte_equal_over_corpus` | AC-02; 100/100 byte-equality |
| `test_emit_deterministic` (property) | emit(canvas) == emit(canvas) for 1000 canvases |
| `test_load_deterministic` (property) | load(spec) == load(spec) |
| `test_emit_diagnostic_when_invalid` | invalid spec returns precise per-line error (AC-05 supports LLM-assist) |
| `test_schema_strict_additionalProperties_false` | unknown fields rejected per threat-model T-I-03 |

## Halt Conditions

- Any spec in the reference corpus fails round-trip — BLOCKER per AC-02; do not merge.
- Emitter non-deterministic — bug; root-cause and fix.
- Schema validation lane fails — bug; the strict schema is load-bearing.

## Next IP

[`IP-004-dsl-emitter-loader-usecase-api-adapter-sdk.md`](IP-004-dsl-emitter-loader-usecase-api-adapter-sdk.md)

## References

- ADR-0164 (Bominal): Workflow canonical spec format — inherited verbatim.
- ADR-0105 13-layer.
- PRD AC-02 round-trip byte-equality.
- JSON Schema 2020-12 — `json-schema.org/draft/2020-12/json-schema-core.html`.
- proptest docs — `proptest.rs`.

## Counterpart Anchors
This workflow-studio IP is measured against the local Workflow Studio benchmark envelope: n8n for visual workflow authoring depth, Zapier for broad trigger/action accessibility, Make for visual branching and scenario ergonomics, and Workato for enterprise workflow governance. The IP must keep Oyatie's differentiator intact: canonical workflow_spec.v1 round-trip, Cedar-gated save/publish, tenant-scoped collaboration, and audit evidence rather than counterpart-specific runtime authority.

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`workflow-studio`].
- surface_evidence_paths: [`microservices/workflow-studio/IP-003-dsl-emitter-loader-kernel-domain.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/templates/index.json`, `microservices/workflow-studio/templates/schemas/workflow-template.schema.json`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/ARCHITECTURE.md`].
