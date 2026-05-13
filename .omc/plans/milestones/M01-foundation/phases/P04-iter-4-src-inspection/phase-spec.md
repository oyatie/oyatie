---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M01-foundation
phase: P04-iter-4-src-inspection
status: Complete
acceptance_lanes: []
entry_gate: |
  P02-shard-1-atomic-rename complete; workspace compiles clean; all 114 renamed
  crates present on disk. The 88 STUB-pending-iter-4-src-inspection cells in
  §3 audit body are the work items for this phase.
exit_gate: |
  Every `layer_evidence` cell in §3.1–§3.5 audit body contains a concrete
  file:line cite OR an explicit `PROTOCOL-UNKNOWN` deferral marker (the 26
  deferred rows are correct). Zero `STUB-pending` markers remain. Layer
  assignments confirmed or amended (with rationale). The §3 audit body is the
  source of truth for the post-Shard-1 workspace state. Rename plan updated
  in-place with evidence. ICM context-oyatie row emitted.
depends_on:
  - milestone: M01
    phase: P02-shard-1-atomic-rename
    reason: "src-inspection operates on renamed crate dirs; must run post-Shard-1"
owner_team: council-architecture
---

# P04-iter-4-src-inspection: Resolve STUB-pending-iter-4 layer evidence cells

## Purpose

The §3 audit body in `docs/plans/rename-plan-v4-clean-arch-2026-05-13.md`
contains 88 rows marked `STUB-pending-iter-4-src-inspection` in the
`layer_evidence` column. This phase resolves each stub by inspecting the crate's
`src/` directory against the canonical decision tree (§2.2.4), emitting a
concrete evidence cite (`file:line — pattern that fixes the layer`), and
confirming or amending the `proposed_name` layer suffix.

This phase also provides the protocol classification evidence that gates P03
(Shard 1.5) for the 26 PROTOCOL-UNKNOWN rows.

Advances Master Plan principles: no stubs, no placeholders, no deferrals within
agreed scope (per `feedback_autonomous_decision_principles.md`); the §3 audit
body becomes an accurate reflection of actual code shape.

---

## Scope

### In-scope

| Partition | STUB rows to resolve | PROTOCOL-UNKNOWN rows to classify |
|---|---:|---:|
| §3.1 platform | 18 | 5 |
| §3.2 cloud | 18 | 13 |
| §3.3.1 foundry non-check | 19 | 4 |
| §3.4 connect/workspace | 22 | 4 |
| **total** | **77** | **26** |

For each row: read `src/lib.rs` or `src/main.rs`; classify per §2.2.4 decision
tree; fill `layer_evidence` with concrete cite; confirm or amend `layer` + 
`proposed_name`; flag any kernel-vs-domain reclassifications.

Special case: `oya-data-boundary-kernel` (row 1, risk=5, ~95 consumers) — confirm
`kernel` layer (pure types + ports, named-by-identity) with explicit cite.

### Out-of-scope

- Actual code changes to reclassified crates (layer suffix changes are renames
  tracked in a follow-on micro-PR if any; this phase only updates the audit doc).
- New feature work.

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | src-inspect 88 STUB rows + 26 PROTOCOL-UNKNOWN rows | pending | `council-architecture` |

---

## Acceptance Gates

```bash
# Audit body has zero STUB markers
grep -c "STUB-pending" \
  docs/plans/rename-plan-v4-clean-arch-2026-05-13.md   # must be 0

# 26 PROTOCOL-UNKNOWN rows still present (deferred to P03)
grep -c "PROTOCOL-UNKNOWN" \
  docs/plans/rename-plan-v4-clean-arch-2026-05-13.md   # must be >= 26

# Workspace still compiles (no reclassification broke deps)
cargo check --workspace --all-features                   # exit 0
```

---

## Clean Architecture Compliance

This phase produces evidence; it does not change code. Any kernel→domain
reclassification discovered here becomes an input to a micro-PR that renames
the crate suffix (tracked as a child issue of P04).

### Row-1 special handling (oya-data-boundary-kernel)

`oya-data-boundary-kernel` is the highest-blast-radius crate (~95 consumers).
Per §3 audit row 1 and §2.2.4 decision tree rule 1 ("pure types + traits,
no logic? → kernel"), this crate MUST be confirmed as `kernel` before P04
exits. Evidence format: `crates/oya-data-boundary-kernel/src/lib.rs:1 — zero fn bodies; only struct/enum/trait declarations`.

---

## Grit Claim Symbols

```
docs/plans/rename-plan-v4-clean-arch-2026-05-13.md::Section3
```

TTL: 3600s per partition batch.

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "P04-iter-4-src-inspection COMPLETE. 88 STUB-pending cells resolved with file:line cites. 26 PROTOCOL-UNKNOWN rows confirmed (gate for P03). N kernel→domain reclassifications found (list). oya-data-boundary-kernel confirmed kernel (pure types+ports, ~95 consumers). Rename plan §3 audit body is now accurate." \
  -i high \
  -k "M01,P04,iter-4,src-inspection,layer-evidence,phase-complete"
```

---

## References

- Rename plan §3: `docs/plans/rename-plan-v4-clean-arch-2026-05-13.md`
- ADR-0056 §2.2.4: canonical decision tree
- ADR-0056 §"Layer semantics": 12-value closed enum
- Memory: `feedback_clean_architecture_requirements.md`, `feedback_autonomous_decision_principles.md`
