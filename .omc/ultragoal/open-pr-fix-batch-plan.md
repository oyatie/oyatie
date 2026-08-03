## Open-PR fix batch — plan

### 1. Per-PR fix summary

| PR | CI-red cause | Fix (owned-Rust) | Reorg-entangled? | Effort |
|---|---|---|---|---|
| #1290 | `ci-baseline-ratchet-gate` total-accounting flags 5 new tracked files as unjustified debt (compliance-pack .py validator + fixtures never registered) | `git rm scripts/tests/compliance_pack_contract_slice_check.py`; add one slice entry to `ci/facade/contract-slice-conformance/contract-slice-policy.json`; trim ADR-0251 back to rationale; re-run accounting-registry producer; rebase (DIRTY) | No — root `/specs/` hub only | S |
| #1293 | `rust-first-automation-policy` gate: unregistered non-Rust automation at `scripts/tests/resilience_001_runtime_control_loop_check.py` | Delete .py; add declarative spec + one contract-slice-policy.json entry — **but only at the comms/ destination, not oya/messenger/** | **Yes, blocking** — author-flagged: must sequence after comms/ strangler move (#1308, ADR-0562 §10.16) | M |
| #1294 | Pure admission gate (`pr-traceability-admission-bin --require-code-review`) red: `reviewDecision=CHANGES_REQUESTED` is stale, left over from an older head; underlying Python validator already deleted | No code fix needed — get a fresh Review/fix pass approved against current head `1b301207a`; fold in 4 small CodeRabbit wording edits | No — `oya/feature-flags/` is canonical, unaffected by any pending rename | S |
| #1296 | `buck2` leg is infra-red (runner disk-exhaustion, not code); real mergeability blocker is CodeRabbit CHANGES_REQUESTED (8 findings) on `residency_001` Python validator | `git rm` the .py; revert its rust-first exception; add residency-001 + 4 sibling slice entries to contract-slice-policy.json, folding all 8 CodeRabbit fixes into the JSON/schema data; rerun on a clean runner | No — flat `/specs`, `/registry`, `/ci/facade` paths | M |
| #1297 | Two red gates: `rust-first-automation-hygiene` (new unregistered `talos_001_substrate_slice_check.py`) + `canonical-json (ADR-0546)` (non-canonical bytes in `specs/deployment-ops-contract.json`) | Run canonical-json fixer on the spec; `git rm` the .py (no exception); add one talos-001 slice entry to contract-slice-policy.json | No — `ci/facade/contract-slice-conformance/` already de-branded/merged on dev | S |

### 2. Reorg-safe order

Land/rebase in this order so no fix is stranded or reverted by the comms/ (messenger→comms) strangler move:

1. **#1297** (S, zero deps, zero entanglement) — land first, clears fastest.
2. **#1294** (S, process-only, zero reorg entanglement) — land in parallel with #1297; only waiting on a fresh Review/fix pass.
3. **#1290** (S, root-`/specs` hub only, zero entanglement) — land next; independent of everything else.
4. **#1296** (M, flat paths, zero entanglement) — land after the S-tier PRs clear the queue; needs a clean-runner rerun.
5. **#1293 (LAST, gated)** — do **not** convert its Python validator to the Rust contract slice at `oya/messenger/`. Sequence: (a) let the comms/ strangler move (issue #1308, ADR-0562 §10.16) land first, renaming `oya/messenger/` → `comms/` (SLO home `comms/observability/slos/messenger/`); (b) only then delete the .py and add the resilience-001 slice entry at the new comms/ path, using PR #1313's FINOPS-001 conversion as the template. Converting at the pre-move path first would create a second migration PR the moment the rename lands — this is the one PR in the batch where reorg ordering is a hard blocker, not just hygiene.

### 3. What parallelizes vs serializes

- **Parallel (independent worktrees), fix now**: #1290, #1294, #1296, #1297 — none share a path, none depend on each other or on the reorg move. Four separate lanes can run concurrently.
- **Serial, blocked on external event**: #1293 alone — cannot be finalized until the comms/ strangler move (#1308) lands. Can be *prepped* (spec drafted) in parallel, but the `git rm` + contract-slice-policy.json entry must be committed at the post-move path, so its merge is strictly after #1308.

### 4. Per-PR dispatch brief

**#1290** — Target `ci/facade/contract-slice-conformance/contract-slice-policy.json` (already-merged gate, no new crate). Assert compliance-001 fields (claim_boundary, cell_certification_state, cmp_consent, portability_export) as `required_fields`/`forbidden_markers`. Delete `scripts/tests/compliance_pack_contract_slice_check.py`, drop its rust-first exception row, re-run the accounting-registry producer (never hand-edit), then rebase off dev (mergeStateStatus DIRTY).

**#1293** — Target: new spec `specs/resilience-001-runtime-control-loop-contract.json` at the **comms/** destination (post-#1308 move), consumed by the existing `contract-slice-conformance` gate. Delete `scripts/tests/resilience_001_runtime_control_loop_check.py`; enumerate every nonclaim individually in `required_array_members` (PR #1313 precedent shows CodeRabbit catches dropped nonclaims). Do not touch `oya/messenger/` directly — wait for the strangler move.

**#1294** — No crate target; this is pure process. Get Kanban card `t_ed127d1c`'s Review/fix pass to re-evaluate current head `1b301207a` and flip `reviewDecision` off CHANGES_REQUESTED. While at it, reconcile the 4 CodeRabbit wording nits in `oya/feature-flags/release/runtime-safety-policy.json`, the rollout-kernel yaml, and ADR-0159 (soften "binds" language) — text-only, no code.

**#1296** — Target `ci/facade/contract-slice-conformance/contract-slice-policy.json`. Delete `scripts/tests/residency_contract_slice_check.py` + its rust-first-automation-policy.json exception. Add `residency-001-attestation` slice plus 4 sibling entries covering `residency-attestation-schema.json`, `residency-placement-audit-events.json`, `regions/registry.json`, `regulatory-regimes/registry.json` — fold all 8 CodeRabbit findings (real SHA-256 hash, canonical SOC2-T2 ids, typed `offlineChannelProtocol` properties, field-name alignment) directly into the JSON as `required_fields`/`enum_constraints`, not python string-matching. Ignore the PR body's stale "Verdict: Approved" claim.

**#1297** — Target `ci/facade/contract-slice-conformance/contract-slice-policy.json`. Run the canonical-json fixer on `specs/deployment-ops-contract.json` first (independent of the Python question). Delete `scripts/tests/talos_001_substrate_slice_check.py` (no exception row); add `talos-001-substrate-slice` entry porting every assertion from the deleted script (ADR authority list, 5 required substrate surfaces, ADR-0382 Proposed/design-only boundary) so the slice doesn't silently weaken the check.
