---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-bnf-v4-adrs-finalized
impl_plan_id: IP-001-bnf-v4-adrs
status: merged
owner: council-architecture
blocked_by: []
acceptance_lanes:
  - cargo-check
---

# IP-001-bnf-v4-adrs: Author BNF v4.1 ADRs + rename plan approval

## Intent

Authors ADR-0056 (BNF v4.1 flat microservice grammar + 12-layer enum), ADR-0057
(Hybrid C cutover mechanics), and companion ADRs ADR-0058 through ADR-0062.
Produces the user-approved rename plan v4.1 that gates Shard 1 execution.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `docs/decisions/ADR-0056-rust-clean-architecture-bnf.md` | create | BNF v4.1 + 12-layer enum + check-namespace + microservice registry |
| `docs/decisions/ADR-0057-cutover-mechanics-rename-plan-v4.md` | create | Hybrid C topology; Shard 0+1; lockfile-rename xtask; 4-partition reviewers |
| `docs/plans/rename-plan-v4-clean-arch-2026-05-13.md` | create | 140-crate audit; §3.1–§3.5 per-crate table; §3.6 summary; §8.1 gates |
| `tools/xtask-metadata-augment/src/main.rs` | update | Add `generate-rename-map --bnf-version v4.1` flag; dual-schema parser for 9-col + 11-col audit rows |

---

## Crate Naming

No new crates. All crate names cited in ADR-0056 carry inline justifications
per `feedback_naming_justification.md`.

---

## Code Shape

ADR-0056 §"Canonical BNF v4.1":
```bnf
crate        ::= "oya" "-" microservice ( "-" bc-tokens )? "-" layer
               | "oya" "-" "check" "-" rule-name
microservice ::= kebab-token ( "-" kebab-token )*
bc-tokens    ::= kebab-token ( "-" kebab-token )*   (* OPTIONAL *)
layer        ::= "kernel"|"domain"|"application"|"app"|"adapter"|"infrastructure"
               | "cli"|"rest"|"grpc"|"graphql"|"worker"|"sdk"
```

---

## Acceptance Gates

```bash
# Compile xtask with new generate-rename-map logic
cargo check -p xtask-metadata-augment --all-features   # exit 0

# Generate TSV — must produce 114 rows
cargo run -p xtask-metadata-augment -- generate-rename-map \
  --plan docs/plans/rename-plan-v4-clean-arch-2026-05-13.md \
  --map-out /tmp/rename-map-v4.1.tsv \
  --names-out /tmp/old-crate-names-v4.1.txt
# Expected: "generate-rename-map: 114 rename pairs written"
```

---

## Test Plan

### Unit tests

Doc-only phase; no production code tests. TSV output count is the acceptance criterion.

---

## Clean Architecture Compliance

No new crates. ADR-0056 establishes the compliance matrix that all future crates
must satisfy. The 12-layer enum + microservice registry constitute the compile-time
enforcement surface.

---

## Load Test

Not applicable — doc-only phase.

---

## Grit Symbol-Locks

Doc-only work. ICM `scaffold-locks-oyatie` fallback per ADR-0054.

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-001-bnf-v4-adrs merged. ADR-0056 v4.1 + ADR-0057 authored. Rename plan v4.1 user-approved. xtask generate-rename-map dual-schema parser produces 114 rows. Next IP: P02/IP-001-shard-1-atomic-rename." \
  -i high \
  -k "M01,P01,IP-001,ADR-0056,ADR-0057,xtask"
```

---

## Halt Conditions

1. TSV generation produces < 114 or > 120 rows — audit parser bug; fix parser.
2. ADR-0056 and rename plan conflict on proposed names — reconcile with §3 audit body.

---

## Next IP Pointer

`../P02-shard-1-atomic-rename/impl-plan.md`

---

## Cross-References

- Phase spec: `phase-spec.md`
- ADR-0056: `docs/decisions/ADR-0056-rust-clean-architecture-bnf.md`
- ADR-0057: `docs/decisions/ADR-0057-cutover-mechanics-rename-plan-v4.md`
- Memory: `feedback_naming_justification.md`, `feedback_grit_claim_work_done.md`
