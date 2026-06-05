---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-studio-preview
phase: P01-visual-authoring-substrate
impl_plan_id: IP-010-jurisdiction-overlay-renderer-full
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
acceptance_lanes: [cargo-check, cargo-nextest, layer-correctness]
depends_on: [IP-004]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: jurisdiction-overlay-renderer — full BC (5 layers)

## Intent

Author the `jurisdiction-overlay-renderer` BC: pure overlay resolution + visual diff over jurisdictions. Renders Studio's jurisdiction-overlay UX (per-pack visual differences, base-reachable invariant per AC-04). Library-only; no rest/worker/sdk/app exemptions per PRD §"Bounded Contexts".

## ChangeSet boundary

Five crates:
- `oya-workflow-studio-jurisdiction-overlay-renderer-{kernel,domain,usecase,api,adapter}`

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-workflow-studio-jurisdiction-overlay-renderer-kernel/{Cargo.toml,src/{lib.rs,entities.rs,ports.rs}}` | create |
| `src/crates/oya-workflow-studio-jurisdiction-overlay-renderer-domain/{Cargo.toml,src/{lib.rs,resolve.rs,diff.rs},tests/{resolve.rs,diff.rs,jurisdiction_view_switch.rs,base_reachable_invariant.rs}}` | create |
| `src/crates/oya-workflow-studio-jurisdiction-overlay-renderer-usecase/{Cargo.toml,src/lib.rs}` | create |
| `src/crates/oya-workflow-studio-jurisdiction-overlay-renderer-api/{Cargo.toml,src/lib.rs}` | create |
| `src/crates/oya-workflow-studio-jurisdiction-overlay-renderer-adapter/{Cargo.toml,src/lib.rs}` | create |
| `microservices/workflow-studio/catalog/oya-workflow-studio-jurisdiction-overlay-renderer-*.yaml` | create | 5 catalog records |

## Code Shape

`jurisdiction-overlay-renderer-kernel/src/entities.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Jurisdiction {
    Base,
    Kr, Eu, Us, UsHc, Jp, Sg, Au, In, Br, Ae, Ksa,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Overlay {
    pub jurisdiction: Jurisdiction,
    pub version_sha: String,
    pub patches: Vec<OverlayPatch>,
    pub signature: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OverlayPatch {
    pub json_pointer: String,
    pub op: OverlayOp,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OverlayOp {
    Add(serde_json::Value),
    Replace(serde_json::Value),
    Remove,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResolvedView {
    pub jurisdiction: Jurisdiction,
    pub spec_body: serde_json::Value,
    pub overlay_version_sha: String,
    /// AC-04: base view always reachable from resolved view.
    pub base_reachable: bool,
}
```

`jurisdiction-overlay-renderer-domain/tests/base_reachable_invariant.rs`:

```rust
use proptest::prelude::*;

proptest! {
    /// AC-04 invariant: after applying any overlay, the base view is reachable
    /// (i.e., overlays are reversible).
    #[test]
    fn test_base_reachable_after_any_overlay(
        base_spec in proptest::collection::vec(any::<u32>(), 1..50),
        overlay_patches in proptest::collection::vec(any::<u32>(), 0..20),
    ) {
        let resolved = oya_workflow_studio_jurisdiction_overlay_renderer_domain::resolve::apply_overlay(
            &base_spec, &overlay_patches,
        );
        prop_assert!(resolved.base_reachable);
        let unresolved = oya_workflow_studio_jurisdiction_overlay_renderer_domain::resolve::unapply_overlay(
            &resolved.spec_body, &overlay_patches,
        );
        prop_assert_eq!(unresolved, base_spec);
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-workflow-studio-jurisdiction-overlay-renderer-kernel \
  -p oya-workflow-studio-jurisdiction-overlay-renderer-domain \
  -p oya-workflow-studio-jurisdiction-overlay-renderer-usecase \
  -p oya-workflow-studio-jurisdiction-overlay-renderer-api \
  -p oya-workflow-studio-jurisdiction-overlay-renderer-adapter
cargo nextest run -p oya-workflow-studio-jurisdiction-overlay-renderer-domain --test base_reachable_invariant
cargo nextest run -p oya-workflow-studio-jurisdiction-overlay-renderer-domain --test jurisdiction_view_switch
buck2 build //:quality-lane-registry-authority-check # lane=layer-correctness --microservice workflow-studio
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_jurisdiction_view_switch` | AC-04; switch overlay; resolved view differs; base reachable |
| `test_base_reachable_after_any_overlay` (property) | AC-04 invariant over 1000 random overlays |
| `test_overlay_diff_idempotent` | applying same overlay twice yields same view |
| `test_overlay_signature_required` | unsigned overlay rejected (per threat-model T-T-04) |
| `test_jurisdiction_drift_detection` | overlay_version_sha mismatch surfaces "refresh" |

## Halt Conditions

- base_reachable invariant violated — STOP. AC-04 breach.
- Overlay signature bypass succeeds — STOP. T-T-04 breach.

## Next IP

[`IP-011-replay-debugger-frontend-full.md`](IP-011-replay-debugger-frontend-full.md)

## References

- PRD AC-04 + FR-08.
- threat-model.md T-T-04, T-D-06.
- data-residency.md (per-pack overlay residency).
- JSON Patch RFC 6902 — `tools.ietf.org/html/rfc6902`.

## Counterpart Anchors
This workflow-studio IP is measured against the local Workflow Studio benchmark envelope: n8n for visual workflow authoring depth, Zapier for broad trigger/action accessibility, Make for visual branching and scenario ergonomics, and Workato for enterprise workflow governance. The IP must keep Oyatie's differentiator intact: canonical workflow_spec.v1 round-trip, Cedar-gated save/publish, tenant-scoped collaboration, and audit evidence rather than counterpart-specific runtime authority.

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`workflow-studio`].
- surface_evidence_paths: [`microservices/workflow-studio/IP-010-jurisdiction-overlay-renderer-full.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/templates/index.json`, `microservices/workflow-studio/templates/schemas/workflow-template.schema.json`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/ARCHITECTURE.md`].
