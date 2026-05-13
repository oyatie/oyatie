---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M01-foundation
phase: P01-bnf-v4-adrs-finalized
status: Complete
acceptance_lanes: []
entry_gate: |
  None (first phase of M01). Pre-condition: workspace exists at main HEAD
  with v3 crate names and Shard 0 tooling (xtask-metadata-augment + 4 LEAN
  check crate scaffolds) committed.
exit_gate: |
  ADR-0056 v4.1, ADR-0057, ADR-0058, ADR-0059, ADR-0060, ADR-0061, ADR-0062
  all present in docs/decisions/ with status Accepted. Rename plan v4.1
  approved (frontmatter: execution: approved-by-user-2026-05-13). ICM
  context-oyatie row emitted.
depends_on: []
owner_team: council-architecture
---

# P01-bnf-v4-adrs-finalized: BNF v4.1 ADRs finalized and Wave 1 ADRs landed

## Purpose

This phase is a retrospective capture of the Wave 1 work completed on 2026-05-13.
It formalises the BNF v4.1 grammar (flat microservice catalog; no shared|vertical
binary; optional BC slot; 12-value layer enum), authors the companion ADRs
(ADR-0056 through ADR-0062), and obtains user approval for the rename plan v4.1
execution. All decisions landed in a single session; this phase-spec documents
the agreed-upon state so downstream phases have a canonical reference.

Advances Master Plan principles: clean architecture self-enforces via Cargo (BNF
encodes layer), flat catalog (no vertical/shared distinction), hyperscaler naming
convergence (AWS/Azure/GCP crate-name patterns).

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| (doc-only) | BNF + ADRs + rename plan | `docs/decisions/ADR-0056..0062`, `docs/plans/rename-plan-v4-clean-arch-2026-05-13.md` | n/a |

### Out-of-scope

- Actual crate renames — deferred to P02-shard-1-atomic-rename.
- Protocol classification of 26 `*-api` rows — deferred to P03.
- src-inspection of STUB-pending rows — deferred to P04.

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Author ADR-0056..0062 + rename plan v4.1 | merged | `council-architecture` |

---

## Acceptance Gates

### Cargo / CI gates (exit 0 required)

```bash
# No crate changes in this phase; xtask-metadata-augment must compile
cargo check -p xtask-metadata-augment --all-features   # exit 0
```

### Artifact gates

```
docs/decisions/ADR-0056-rust-clean-architecture-bnf.md       status: Accepted
docs/decisions/ADR-0057-cutover-mechanics-rename-plan-v4.md  status: Accepted
docs/plans/rename-plan-v4-clean-arch-2026-05-13.md           execution: approved-by-user-2026-05-13
```

---

## Clean Architecture Compliance

No new crates in this phase. All new BNF crate names are specified in ADR-0056
with justifications per `feedback_naming_justification.md`.

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| (All BCs defined in ADR-0056 §"Microservice registry") | various | ADR-0056 |

---

## Grit Claim Symbols

Doc-only phase. No grit symbol claims required. ICM `scaffold-locks-oyatie`
fallback used for coordination per ADR-0054.

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "P01-bnf-v4-adrs-finalized COMPLETE. ADR-0056 v4.1 BNF (flat microservice, optional BC, 12-layer enum) accepted. ADR-0057 cutover mechanics accepted. Rename plan v4.1 user-approved (execution: approved-by-user-2026-05-13). 114-row Shard 1 scope confirmed; 26 PROTOCOL-UNKNOWN deferred to Shard 1.5." \
  -i high \
  -k "M01,P01,bnf-v4.1,ADR-0056,ADR-0057,phase-complete"
```

---

## References

- ADR-0056: `docs/decisions/ADR-0056-rust-clean-architecture-bnf.md`
- ADR-0057: `docs/decisions/ADR-0057-cutover-mechanics-rename-plan-v4.md`
- Rename plan: `docs/plans/rename-plan-v4-clean-arch-2026-05-13.md`
- Memory: `feedback_flat_product_catalog.md`, `feedback_naming_justification.md`, `feedback_clean_architecture_requirements.md`
