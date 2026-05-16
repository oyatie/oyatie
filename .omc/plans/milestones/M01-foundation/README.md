---
doc_class: MilestoneReadme
template_id: TPL-MILE-README
milestone_id: M01-foundation
parent: ../../../docs/MASTERPLAN.md
status: Complete
entry_gate: 'none (foundational milestone)

  '
exit_gate: '- 114 crates renamed atomically to BNF v4.1 (commit 55058f6 verified)

  - cargo {check,build,clippy,nextest,deny,doc} green across workspace

  - 4 LEAN architecture lanes (lean-a1..a4) flipped from --report-only to BLOCKER

  - 4 quality check crate scaffolds registered (statelessness/shardability/perf-budget/benchmark)

  - grit done emitted (or scaffold-locks-oyatie ICM fallback per ADR-0054)

  - ICM milestone-complete row emitted

  '
owner_team: axis-foundry
bominal_adrs_inherited:
- ADR-0100
- ADR-0101
- ADR-0102
- ADR-0105
- ADR-0125
oyatie_adrs_cited:
- ADR-0053
- ADR-0054
- ADR-0055
- ADR-0056
purpose: "Foundation milestone: atomic rename of all `oya-platform-*` / `oya-shared-*` / `oya-workspace-*` crates to BNF v4.1 flat µservice naming + 4 LEAN architecture-check lanes promoted from `--report-only` to `BLOCKER`."
---
# M01-foundation — BNF v4.1 cutover + LEAN BLOCKER promotion

## Intent

Foundation milestone: atomic rename of all `oya-platform-*` / `oya-shared-*` / `oya-workspace-*` crates to BNF v4.1 flat µservice naming + 4 LEAN architecture-check lanes promoted from `--report-only` to `BLOCKER`. Sets the canonical naming + enforcement baseline that M02-M12 build on.

## Status

**Complete.** Three commits on `main` land the milestone end-to-end:

| Commit | Phase | Outcome |
|---|---|---|
| `55058f6` | P02-shard-1-atomic-rename | 114 crates renamed; cargo gates all green |
| `8f86d4d` | (rollup) | Phase SPECs + impl plans P01-P05 authored |
| `d942b3d` | P05-post-cutover-hardening | LEAN-A1..A4 flipped to BLOCKER; 4 quality check crate scaffolds registered |

## Phases

| Phase ID | Path | Status |
|---|---|---|
| P01-bnf-v4-adrs-finalized | `phases/P01-bnf-v4-adrs-finalized/` | Complete |
| P02-shard-1-atomic-rename | `phases/P02-shard-1-atomic-rename/` | Complete |
| P03-shard-1-5-protocol-unknown-deferred | `phases/P03-shard-1-5-protocol-unknown-deferred/` | Complete (26 PROTOCOL-UNKNOWN rows deferred per ADR-0057) |
| P04-iter-4-src-inspection | `phases/P04-iter-4-src-inspection/` | Complete |
| P05-post-cutover-hardening | `phases/P05-post-cutover-hardening/` | Complete |

Legacy substrate phase directories (`P01-data-use-boundary-tenancy`, `P02-identity-cedar`, `P03-audit-chain-evidence`, `P04-eventing-object-graph`, `P05-cell-plane`, `P06-regional-pack-flattening`) are historical artifacts of an earlier numbering; their content has migrated to `M02b-substrate/phases/` under the new milestone-phase hierarchy.

## Unblocks

All M02 work (M02b-substrate phases P01-P22). The BNF v4.1 cutover is the precondition for every M02 crate authoring task.

## Acceptance evidence

`acceptance-evidence/README.md` plus `/evidence/foundation/m01-foundation-acceptance-audit-2026-05-14.json` record the current M01 acceptance closeout: G1/G2 live Cargo package set, M-CC-P01 P5+ lane evidence, M-CC-P00 scoped waiver, focused 65/65 package tests, and full `./scripts/check.sh` closeout under Rust 1.95.0 / edition 2024 / rustfmt 2024.

## References

- `docs/MASTERPLAN.md` §4 M01
- ADR-0056 BNF v4.1
- `.omc/plans/M01-M03-parallelization-manifest.md` Wave 0 + Wave 1
