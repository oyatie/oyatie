---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-studio-preview
phase: P01-visual-authoring-substrate
impl_plan_id: IP-015-hg-workflow-studio-registration-final
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow + council-architecture
acceptance_lanes: [oya-governance-hyperscaler-maturity-claims, oya-governance-authority-cohesion]
depends_on: [IP-014]
---

# IP-015: HG-WORKFLOW-STUDIO final registration + competitor-parity evidence pinning + Studio launch verification

## Intent

Final HG-WORKFLOW-STUDIO gate registration in `/specs/hyperscaler-gates.json`, pinning of competitor-parity-matrix.md evidence references, end-to-end Studio launch verification per Phase-01 exit gate. This IP closes the phase.

## ChangeSet boundary

Three files updated + evidence emission:
- `/specs/hyperscaler-gates.json` — flip HG-WORKFLOW-STUDIO status from `registering` to `active`.
- `microservices/workflow-studio/competitor-parity-matrix.md` — pin evidence-source SHAs (snapshot competitor docs at registration time).
- `microservices/workflow-studio/evidence/phase-01-exit.json` — emit phase-exit evidence per ADR-0110 ChangeSet contract.

## Concrete File Targets

| Path | Action |
|---|---|
| `/specs/hyperscaler-gates.json` | update | status: registering → active for HG-WORKFLOW-STUDIO |
| `microservices/workflow-studio/competitor-parity-matrix.md` | update | pin evidence_source_refs[*].snapshot_sha per row |
| `microservices/workflow-studio/evidence/phase-01-exit.json` | create | exit-gate evidence per ADR-0110 |
| `microservices/workflow-studio/evidence/multispectrum/ip-015-<change_id>-<unix_ts>.json` | create | per-IP changeset evidence |

## Code Shape

`evidence/phase-01-exit.json`:

```json
{
  "phase": "M03-studio-preview/P01-visual-authoring-substrate",
  "microservice": "workflow-studio",
  "exit_gate_completed_at": "<ISO8601>",
  "ips_merged": [
    "IP-001", "IP-002", "IP-003", "IP-004", "IP-005",
    "IP-006", "IP-007", "IP-008", "IP-009", "IP-010",
    "IP-011", "IP-012", "IP-013", "IP-014", "IP-015"
  ],
  "crates_introduced": 52,
  "lanes_green_required_at_exit": [
    "cargo-check", "cargo-build", "cargo-clippy", "cargo-nextest", "cargo-deny",
    "lean-a1", "lean-a2", "port-location", "layer-correctness",
    "per-microservice-layout", "statelessness", "shardability",
    "workflow-spec-roundtrip", "cedar-preview-required",
    "editor-execution-forbidden", "node-library-determinism",
    "node-library-signature-verification", "wasm-bundle-sri",
    "xss-vector-scan", "citus-rls-enforced",
    "cdn-cache-key-tenant-isolated", "no-tenant-branding-mid-render",
    "llm-assist-validation-required",
    "authority-cohesion", "hyperscaler-maturity-claims"
  ],
  "drills_passed": [
    "round-trip byte-equality (100/100 golden specs)",
    "10-user concurrent collab (no silent loss)",
    "Cedar per-seat gate (seat-overage refused)",
    "TTI budget (p99 ≤ 2s GA)",
    "save round-trip (p99 ≤ 200ms stable)",
    "node-library 3x re-load determinism",
    "WASM bundle SRI verification",
    "jurisdiction overlay switch",
    "LLM-assist validation round-trip",
    "offline buffer durability"
  ],
  "branch_protection_active": [
    "dev: 11 new Studio governance lanes",
    "staging: 2 new lanes",
    "release/workflow-studio/staging: pattern protection live",
    "release/workflow-studio/production: pattern protection live"
  ],
  "hg_workflow_studio_status": "active",
  "competitor_parity_evidence_pinned_at": "<ISO8601>",
  "next_phase": "M03/P02-collab-marketplace-substrate (post-preview)"
}
```

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims
cargo run -p oya-dev-cli -- gate validate authority-cohesion
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice workflow-studio
cargo nextest run --workspace --all-features  # full workspace passes
```

## Test Plan

| Test | Verifies |
|---|---|
| hyperscaler-maturity-claims lane green | HG-WORKFLOW-STUDIO active; all verifying lanes registered |
| authority-cohesion lane green | naming + structure conformance |
| competitor-parity evidence pinned | each competitor row has evidence_source_refs[*].snapshot_sha |
| phase-01-exit.json schema valid | matches changeset-payload schema |
| end-to-end editor flow (synthetic tenant) | open editor → drag 5 nodes → save → reload → byte-equal |

## Halt Conditions

- HG-WORKFLOW-STUDIO not yet `active` after this IP — bug; root-cause.
- competitor-parity evidence missing snapshot_sha — pin before merge.
- Any phase-01 exit gate not green — STOP.

## Next IP

End of M03/P01 phase. Next phase: M03/P02-collab-marketplace-substrate (definition marketplace + multi-domain pack expansion); separate phase doc.

## References

- ADR-0110 ChangeSet state machine.
- ADR-0123 Hyperscaler maturity claim gate.
- ADR-0131 Per-microservice flat layout.
- microservices/workflow-studio/PHASE-01-VISUAL-AUTHORING-SUBSTRATE.md §"Acceptance Gates".
- microservices/workflow-studio/competitor-parity-matrix.md.
- /specs/hyperscaler-gates.json schema.
