---
doc_class: ImplementationPlan
template_id: TPL-IMPL
microservice: workflow-studio
milestone: M03-studio-preview
phase: P02-native-canvas-shells
impl_plan_id: IP-023-presence-awareness-protocol
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-frontend
co_owners: [axis-platform-shared]
date: 2026-05-18
related_adrs: [ADR-0145, ADR-0204]
acceptance_lanes: [presence-correctness, presence-isolation, oya-vcs-promotion-readiness]
depends_on: [IP-022]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-023 — Presence awareness protocol (shared cursors + selection)

## Goal

Layer a presence-awareness protocol on top of the Loro doc from IP-022 so collaborators see each other's cursor positions and selection halos. Implementation uses Loro's `Awareness` map keyed by `participant_id`, value contains `{cursor: {x,y}, selection: [NodeId], color, displayName}`. State pruned 30s after last heartbeat. Each emit costs ≤ 64 bytes wire. Tenant isolation enforced by the room key from IP-022.

## Files to create or modify

| Path | Action | Line range (approx) |
|---|---|---|
| `microservices/workflow-studio/src/collab/presence/Cargo.toml` | create | ~40 LoC; deps loro, serde, tokio |
| `microservices/workflow-studio/src/collab/presence/src/lib.rs` | create | ~220 LoC; `PresenceMap`, `Heartbeat`, `Prune` actor |
| `microservices/workflow-studio/src/collab/presence/src/payload.rs` | create | ~80 LoC; bounded payload struct; NaN/Inf reject |
| `microservices/workflow-studio/src/collab/presence/src/color.rs` | create | ~40 LoC; deterministic color assignment per participant_id |
| `clients/web-sveltekit/lib/collab/presence-binding.ts` | create | ~140 LoC; SvelteKit reactive store |
| `clients/apple/WorkflowStudio/Collab/PresenceBinding.swift` | (covered by IP-018) | — |
| `clients/android/workflowstudio/.../PresenceBinding.kt` | (covered by IP-019) | — |
| `clients/gtk4/workflow-studio/src/presence_binding.rs` | (covered by IP-020) | — |
| `clients/winui3/WorkflowStudio/Collab/PresenceBinding.cs` | (covered by IP-021) | — |
| `microservices/workflow-studio/tests/presence_correctness.rs` | create | ~220 LoC; 5 tests |
| `microservices/workflow-studio/runbooks/presence-disconnect.md` | create | ~80 LoC operator playbook |
| `microservices/workflow-studio/decisions/ADR-0145.md` | append §"Awareness protocol shipped" | +4 LoC |

## Code shape

`presence/src/lib.rs` (excerpt):

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct PresencePayload {
    pub cursor: Option<Cursor>,
    pub selection: SmallVec<[NodeId; 16]>,
    pub color: Color,
    pub display_name: BoundedString<64>,
    pub heartbeat_ms: u64,
}

impl PresencePayload {
    pub fn validate(&self) -> Result<(), PresenceError> {
        if let Some(c) = &self.cursor {
            if !c.x.is_finite() || !c.y.is_finite() { return Err(NonFiniteCursor); }
            if c.x.abs() > MAX_COORD || c.y.abs() > MAX_COORD { return Err(CursorOutOfBounds); }
        }
        if self.selection.len() > MAX_SELECTION { return Err(SelectionTooLarge); }
        Ok(())
    }
}
```

## Tests to write (acceptance)

| Test name | File | Asserts |
|---|---|---|
| `presence_two_peer_cursor_visible_within_200ms` | tests/presence_correctness.rs | Peer A moves cursor; Peer B sees it within 200ms p95 |
| `presence_stale_entry_pruned_after_30s` | tests/presence_correctness.rs | Peer A disconnects; entry pruned within 30s |
| `presence_cross_tenant_isolation` | tests/presence_correctness.rs | Wrong tenant room key → no presence leak |
| `presence_payload_nan_inf_rejected` | tests/presence_correctness.rs | `cursor.x = NaN` → reject without panic |
| `presence_payload_budget_le_64_bytes` | tests/presence_correctness.rs | Typical payload ≤ 64 bytes serialized |
| `presence_color_deterministic` | tests/presence_correctness.rs | Same participant_id → same color across reconnects |
| `presence_selection_too_large_rejected` | tests/presence_correctness.rs | Selection len > MAX_SELECTION → reject |

Minimum 5 required; 7 specified.

## Evidence to emit

- `evidence/microservices/workflow-studio/presence-correctness-{date}.json`
- Audit-chain seal: `oya audit-chain seal --kind presence-correctness --ms workflow-studio --window 30d`
- Metrics: `oya_workflow_studio_presence_visible_latency_ms_bucket`, `oya_workflow_studio_presence_active_participants{room=...}`, `oya_workflow_studio_presence_pruned_total`, `oya_workflow_studio_presence_reject_total{reason=...}`

## Rollback procedure

1. Revert presence ChangeSet.
2. Flip feature flag `workflow_studio_presence=disabled` → cursors no longer rendered; Loro doc sync continues.
3. Emit rollback evidence JSON.

## Blocking dependencies

- IP-022 — Loro doc + awareness substrate.
- `oya-shared-presence-kernel` shared crate — payload validation primitives.

## Acceptance gates

```bash
cargo run -p oya-dev-cli -- gate validate presence-correctness
cargo run -p oya-dev-cli -- gate validate presence-isolation
cargo run -p oya-dev-cli -- gate validate oya-vcs-promotion-readiness --microservice workflow-studio
cargo test -p oya-collab-presence --tests
```

## Halt conditions

- Cross-tenant isolation test fails: STOP, security-critical.
- NaN/Inf cursor rejected too late (panic instead of error): STOP.
- Stale-entry pruning fails: STOP (memory leak risk).

## Exit criteria

1. All 7 tests green.
2. `presence-correctness`, `presence-isolation`, `oya-vcs-promotion-readiness` lanes green.
3. Evidence ledger sealed.
4. Runbook published.
5. ADR-0145 awareness section updated.

## Next IP

[`IP-024-1000-node-perf-bench.md`](IP-024-1000-node-perf-bench.md)

## References

- ADR-0145 Loro awareness.
- ADR-0204 canvas.
- `oya-shared-presence-kernel` crate README.
- Yjs Awareness protocol (Loro inherits design).

## Counterpart Anchors
This workflow-studio IP is measured against the local Workflow Studio benchmark envelope: n8n for visual workflow authoring depth, Zapier for broad trigger/action accessibility, Make for visual branching and scenario ergonomics, and Workato for enterprise workflow governance. The IP must keep Oyatie's differentiator intact: canonical workflow_spec.v1 round-trip, Cedar-gated save/publish, tenant-scoped collaboration, and audit evidence rather than counterpart-specific runtime authority.

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/workflow-studio/IP-023-presence-awareness-protocol.md` matched [`cost`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/workflow-studio/IP-023-presence-awareness-protocol.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/capacity-model.md`, `microservices/workflow-studio/compliance.md`, `microservices/workflow-studio/ARCHITECTURE.md`].

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`workflow-studio`].
- surface_evidence_paths: [`microservices/workflow-studio/IP-023-presence-awareness-protocol.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/templates/index.json`, `microservices/workflow-studio/templates/schemas/workflow-template.schema.json`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/ARCHITECTURE.md`].
