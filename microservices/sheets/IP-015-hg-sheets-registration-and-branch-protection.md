---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-sheets-preview
phase: P01-sheets-foundation
impl_plan_id: IP-015-hg-sheets-registration-and-branch-protection
status: pending
owner: axis-sheets + council-architecture
acceptance_lanes: [oya-governance-hyperscaler-maturity-claims, oya-governance-authority-cohesion]
depends_on: [IP-014]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: HG-SHEETS registration + branch-protection + competitor-parity evidence pinning + Sheets launch verification

## Intent

Final HG-SHEETS gate registration in `/specs/hyperscaler-gates.json`; `.github/branch-protection.yaml` updates for the 9 new sheets governance lanes; release/sheets/{staging,production} pattern protection; pinning of competitor-parity-matrix evidence references; end-to-end Sheets launch verification per Phase-01 exit gate. Closes the phase.

## ChangeSet boundary

Three files updated + evidence emission:
- `/specs/hyperscaler-gates.json` — flip HG-SHEETS status from `registering` to `active`.
- `.github/branch-protection.yaml` — add 9 sheets governance lanes + release-pointer pattern protection.
- `microservices/sheets/competitor-parity-matrix.md` — pin evidence-source SHAs.
- `microservices/sheets/evidence/phase-01-exit.json` — emit phase-exit evidence per ADR-0110 ChangeSet contract.

## Code Shape

`evidence/phase-01-exit.json`:

```json
{
  "phase": "M03-sheets-preview/P01-sheets-foundation",
  "microservice": "sheets",
  "exit_gate_completed_at": "<ISO8601>",
  "ips_merged": [
    "IP-001", "IP-002", "IP-003", "IP-004", "IP-005",
    "IP-006", "IP-007", "IP-008", "IP-009", "IP-010",
    "IP-011", "IP-012", "IP-013", "IP-014", "IP-015"
  ],
  "crates_introduced": 115,
  "lanes_green_required_at_exit": [
    "cargo-check", "cargo-build", "cargo-clippy", "cargo-nextest", "cargo-deny",
    "lean-a1", "lean-a2", "port-location", "layer-correctness",
    "per-microservice-layout", "statelessness", "shardability",
    "sheets-crdt-no-silent-loss", "sheets-formula-engine-correctness",
    "sheets-recalc-determinism", "sheets-xlsx-roundtrip-best-effort",
    "sheets-range-acl-cedar-required", "sheets-import-sandboxed-and-avscan-required",
    "cedar-preview-required", "editor-execution-forbidden",
    "wasm-bundle-sri", "xss-vector-scan", "citus-rls-enforced",
    "cdn-cache-key-tenant-isolated", "no-tenant-branding-mid-render",
    "ai-formula-validation-required",
    "authority-cohesion", "hyperscaler-maturity-claims"
  ],
  "drills_passed": [
    "Cell value + SUM (100x100 grid)",
    "Formula-engine LibreOffice Calc reference corpus (≥ 400 functions)",
    "XLSX best-effort round-trip (100/100 golden workbooks)",
    "10-user concurrent collab no silent loss",
    "Per-range ACL hides PII",
    "Cedar per-seat gate (seat-overage refused)",
    "Sheet-open cold p95 ≤ 400ms; warm p95 ≤ 150ms",
    "Cell-edit-render p99 ≤ 50ms",
    "Recalc 100k-cell p95 ≤ 1s",
    "Recalc 1M-cell p95 ≤ 10s",
    "XLSX export 100k-cell p95 ≤ 5s",
    "Chart render p95 ≤ 200ms",
    "Smart-fill ≥ 80% accuracy on 3-cell seed corpus",
    "Connected-sheets refresh p95 ≤ 5s for 10k-row materialize",
    "AI-formula validation round-trip",
    "Offline buffer durability",
    "WASM bundle SRI verification",
    "XLSX import gVisor + ClamAV + OPSWAT"
  ],
  "branch_protection_active": [
    "dev: 9 new sheets governance lanes",
    "staging: 3 new lanes",
    "release/sheets/staging: pattern protection live",
    "release/sheets/production: pattern protection live"
  ],
  "hg_sheets_status": "active",
  "competitor_parity_evidence_pinned_at": "<ISO8601>",
  "next_phase": "M03/P02-sheets-marketplace-substrate (post-preview)"
}
```

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims
cargo run -p oya-dev-cli -- gate validate authority-cohesion
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice sheets
cargo nextest run --workspace --all-features
```

## Test Plan

| Test | Verifies |
|---|---|
| hyperscaler-maturity-claims lane green | HG-SHEETS active; all verifying lanes registered |
| authority-cohesion lane green | naming + structure conformance |
| competitor-parity evidence pinned | each competitor row has evidence_source_refs[*].snapshot_sha |
| phase-01-exit.json schema valid | matches changeset-payload schema |
| end-to-end editor flow (synthetic tenant) | open workbook → write 50 cells → SUM formula → save → reload → byte-equal |

## Halt Conditions

- HG-SHEETS not `active` after this IP — bug; root-cause.
- competitor-parity evidence missing snapshot_sha — pin before merge.
- Any phase-01 exit gate not green — STOP.

## Next IP

End of M03/P01 phase. Next phase: M03/P02-sheets-marketplace-substrate.

## References

- ADR-0110 ChangeSet state machine.
- ADR-0123 Hyperscaler maturity claim gate.
- ADR-0135 Sheets net-new µservice.
- ADR-0131 Per-microservice flat layout.
- microservices/sheets/PHASE-01-SHEETS-FOUNDATION.md.
- microservices/sheets/competitor-parity-matrix.md.
- /specs/hyperscaler-gates.json schema.
