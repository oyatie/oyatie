---
doc_class: ImplementationPlan
template_id: TPL-IMPL
microservice: workflow-studio
milestone: M03-studio-preview
phase: P02-native-canvas-shells
impl_plan_id: IP-022-loro-crdt-sync-binding
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-frontend
co_owners: [axis-data, axis-platform-shared]
date: 2026-05-18
related_adrs: [ADR-0145, ADR-0204, ADR-0208]
acceptance_lanes: [crdt-correctness-no-silent-loss, perf-loro-merge-latency, oya-governance-promotion-readiness]
depends_on: [IP-016]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-022 — Loro CRDT sync binding (graph state replicated across collaborators)

## Goal

Build the canonical Loro CRDT integration that backs every Workflow Studio shell (web SvelteKit + Leptos + GTK + WinUI + SwiftUI + Compose). One Rust core crate (`oya-collab-loro`) exports a stable C ABI plus uniffi bindings for Swift/Kotlin/C#, plus a TS package compiled to WASM for the web. Per ADR-0145 Loro is canonical; per ADR-0208 WebSocket is the transport. Each canvas binds its node-graph to a `LoroMap<NodeId, NodeRecord>` + `LoroList<EdgeRecord>` and updates fan out to peers via per-tenant room channels.

## Files to create or modify

| Path | Action | Line range (approx) |
|---|---|---|
| `microservices/workflow-studio/src/collab/loro_core/Cargo.toml` | create | ~50 LoC; deps: `loro = "1.x"`, `tokio`, `tokio-tungstenite`, `serde`, `uniffi` |
| `microservices/workflow-studio/src/collab/loro_core/src/lib.rs` | create | ~240 LoC; `LoroBinding` struct: `connect()`, `apply_local()`, `subscribe()`, `tenant_isolation_guard()` |
| `microservices/workflow-studio/src/collab/loro_core/src/transport.rs` | create | ~200 LoC; WebSocket client per ADR-0208; reconnect + session-resume token state machine |
| `microservices/workflow-studio/src/collab/loro_core/src/room.rs` | create | ~140 LoC; `RoomKey { tenant_id, document_id }`; rejects mismatched joins |
| `microservices/workflow-studio/src/collab/loro_core/src/uniffi.udl` | create | ~80 LoC; UDL for Swift/Kotlin/.NET bindings |
| `microservices/workflow-studio/src/collab/loro_core/uniffi-cs/build.rs` | create | ~40 LoC; `uniffi-cs` codegen for WinUI consumer |
| `microservices/workflow-studio/src/collab/loro_wasm/Cargo.toml` | create | ~40 LoC; `wasm-bindgen` target |
| `microservices/workflow-studio/src/collab/loro_wasm/src/lib.rs` | create | ~140 LoC; `wasm-bindgen` thin wrapper |
| `clients/web-sveltekit/lib/collab/loro-binding.ts` | create | ~180 LoC; TS facade consuming `loro_wasm` |
| `microservices/workflow-studio/tests/loro_correctness.rs` | create | ~320 LoC; 6 correctness tests (see below) |
| `microservices/workflow-studio/tests/loro_perf.rs` | create | ~120 LoC; merge-latency p95/p99 bench |
| `microservices/workflow-studio/runbooks/crdt-merge-conflict.md` | create | ~120 LoC; merge-conflict debug procedure |
| `microservices/workflow-studio/runbooks/loro-session-resume.md` | create | ~80 LoC; reconnect + resume token playbook |
| `microservices/workflow-studio/decisions/ADR-0145.md` | append §"Loro core + uniffi bindings shipped" | +6 LoC |

## Code shape

`loro_core/src/lib.rs` (excerpt):

```rust
pub struct LoroBinding {
    doc: Arc<LoroDoc>,
    transport: Arc<Transport>,
    room: RoomKey,
    awareness: Arc<Awareness>,
}

impl LoroBinding {
    pub async fn connect(room: RoomKey, endpoint: Url, token: SessionToken)
        -> Result<Self, LoroError>
    {
        let transport = Transport::connect(endpoint, token).await?;
        transport.join(&room)?;  // server rejects mismatched tenant_id
        let doc = Arc::new(LoroDoc::new());
        let awareness = Arc::new(Awareness::new(doc.clone()));
        Ok(Self { doc, transport, room, awareness })
    }

    pub fn apply_local(&self, op: LocalOp) -> Result<(), LoroError> {
        self.doc.with_txn(|txn| op.apply(txn))?;
        self.transport.broadcast(self.doc.export_from(&Default::default()))?;
        Ok(())
    }
}
```

## Tests to write (acceptance)

| Test name | File | Asserts |
|---|---|---|
| `loro_two_peer_concurrent_edits_converge` | tests/loro_correctness.rs | Two peers do 100 random ops each; final state byte-equal |
| `loro_offline_edit_merge_no_silent_loss` | tests/loro_correctness.rs | Peer A edits offline; Peer B edits online; reconnect; both edits present |
| `loro_resume_after_disconnect_preserves_history` | tests/loro_correctness.rs | 30s disconnect; resume token replays missing ops |
| `loro_cross_tenant_isolation_rejected` | tests/loro_correctness.rs | Wrong tenant_id room key → server rejects join, client errors |
| `loro_room_eviction_after_idle` | tests/loro_correctness.rs | Idle 30min → room evicted from server; rejoin starts fresh |
| `loro_payload_budget_within_p95` | tests/loro_correctness.rs | 1000 ops; total payload ≤ 1MiB; per-op delta ≤ 16KiB p95 |
| `loro_merge_latency_under_50ms_p95` | tests/loro_perf.rs | 10k ops; merge latency p95 ≤ 50ms, p99 ≤ 100ms |
| `loro_concurrent_writers_no_lost_update` | tests/loro_correctness.rs | 8 concurrent writers; all 8 ops present after convergence |

Minimum 6 required; 8 specified.

## Evidence to emit

- `evidence/microservices/workflow-studio/loro-correctness-{date}.json` — per-test result + convergence proof traces
- `evidence/microservices/workflow-studio/loro-perf-{date}.json` — merge-latency histogram
- Audit-chain seal: `oya audit-chain seal --kind crdt-correctness --ms workflow-studio --window 30d`
- Metrics: `oya_workflow_studio_loro_merge_latency_ms_bucket`, `oya_workflow_studio_loro_payload_bytes_bucket`, `oya_workflow_studio_loro_room_active_total`, `oya_workflow_studio_loro_resume_token_redeemed_total`
- Structured logs at `target=collab/loro_core event=...`.

## Rollback procedure

1. Revert ChangeSet for `microservices/workflow-studio/src/collab/loro_core` + `loro_wasm` + per-shell binding files.
2. Flip feature flag `workflow_studio_loro_sync=disabled` → all shells fall back to single-user mode (no realtime collab; warning banner displayed).
3. Halt server `oya-workflow-studio-collab-server` per `runbooks/collab-server-halt.md`.
4. Drain active rooms; export per-room snapshots to `evidence/microservices/workflow-studio/rollback-snapshots-{date}/`.
5. Emit rollback evidence JSON.

## Blocking dependencies

- IP-016 — defines node-graph model the Loro doc must mirror.
- ADR-0145 — Loro pin (must remain authoritative).
- ADR-0208 — WebSocket transport contract.

## Acceptance gates

```bash
buck2 build //:quality-lane-registry-authority-check # lane=crdt-correctness-no-silent-loss --crate oya-collab-loro
buck2 build //:quality-lane-registry-authority-check # lane=perf-loro-merge-latency --crate oya-collab-loro
buck2 build //:quality-lane-registry-authority-check # lane=oya-governance-promotion-readiness --microservice workflow-studio
cargo test -p oya-collab-loro --tests
```

## Halt conditions

- Any correctness test fails (silent loss detected): STOP, escalate to ADR-0145 owner.
- Merge-latency p99 > 200ms: STOP, file regression IP.
- Cross-tenant isolation test fails: STOP, security-critical regression.

## Exit criteria

1. All 8 tests green on Linux + macOS + Windows CI runners.
2. `crdt-correctness-no-silent-loss`, `perf-loro-merge-latency`, `oya-governance-promotion-readiness` lanes green.
3. Evidence ledger sealed.
4. Uniffi bindings cross-compile cleanly for swift / kotlin / cs targets.
5. Wasm bundle ≤ 800KiB gzipped (CDN budget).
6. Runbooks published.
7. ADR-0145 status updated.

## Next IP

[`IP-023-presence-awareness-protocol.md`](IP-023-presence-awareness-protocol.md)

## References

- ADR-0145 — Loro CRDT canonical pin.
- ADR-0204 — canvas perf bar.
- ADR-0208 — realtime WebSocket transport.
- ADR-0064 — canonical base + localization overlay.
- Loro 1.x docs — `https://loro.dev/docs/`.
- Yjs paper (Loro design heritage) — `Preguica et al, A study of CRDTs`.

## Counterpart Anchors
This workflow-studio IP is measured against the local Workflow Studio benchmark envelope: n8n for visual workflow authoring depth, Zapier for broad trigger/action accessibility, Make for visual branching and scenario ergonomics, and Workato for enterprise workflow governance. The IP must keep Oyatie's differentiator intact: canonical workflow_spec.v1 round-trip, Cedar-gated save/publish, tenant-scoped collaboration, and audit evidence rather than counterpart-specific runtime authority.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/workflow-studio/IP-022-loro-crdt-sync-binding.md` matched [`p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/workflow-studio/IP-022-loro-crdt-sync-binding.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/ARCHITECTURE.md`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/multi-region.md`, `microservices/workflow-studio/capacity-model.md`].

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`workflow-studio`].
- surface_evidence_paths: [`microservices/workflow-studio/IP-022-loro-crdt-sync-binding.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/templates/index.json`, `microservices/workflow-studio/templates/schemas/workflow-template.schema.json`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/ARCHITECTURE.md`].
