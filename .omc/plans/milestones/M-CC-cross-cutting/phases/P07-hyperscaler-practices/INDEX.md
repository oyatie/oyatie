---
purpose: Auto-backfilled purpose for INDEX.md
---

---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M-CC-P07
title: Hyperscaler-Practice Adoption (Working Backwards / Design Doc / Postmortem / 1ES / Eng-Excellence)
status: complete
purpose: Adopt named practices from AWS, Google, Microsoft, Oracle; thread through every milestone.
---

# M-CC-P07 — Hyperscaler-Practices

## Purpose
Per MASTERPLAN §2 Directives 6 and 9. Hyperscaler-research output (pending) lands at [`../../../../specs/hyperscaler-best-practices-2026-05-12.md`](../../../../specs/hyperscaler-best-practices-2026-05-12.md).

## Acceptance
- AWS Working Backwards / PRFAQ: every product/axis launch has a PRFAQ in `docs/products/<axis>/PRFAQ.md`.
- Google Design Doc per phase: every phase has `DESIGN-DOC.md` under its directory.
- SRE postmortem-blameless: every Sev-1/2 generates a postmortem in `docs/postmortems/`.
- Microsoft 1ES CI templates: every CI lane uses a templated entry.
- Oracle Engineering Excellence Council–style merge gate: council-architecture signs every cross-axis-contract PR.
- Rust toolchain gates: `cargo-deny`, `cargo-audit`, `cargo-nextest`, `cargo-semver-checks`, `sccache`, `cargo-llvm-cov` all integrated into `scripts/check.sh`.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | PRFAQ + Design-Doc + Postmortem templates | complete | [`IP-001-prfaq-designdoc-postmortem.md`](IP-001-prfaq-designdoc-postmortem.md) |
| IP-002 | 1ES-templated CI pipelines | complete | [`IP-002-1es-ci-templates.md`](IP-002-1es-ci-templates.md) |
| IP-003 | Engineering Excellence Council merge gate | complete | [`IP-003-eng-excellence-merge-gate.md`](IP-003-eng-excellence-merge-gate.md) |
| IP-004 | Rust toolchain hyperscaler-gate set (cargo-deny / audit / nextest / semver-checks / sccache / llvm-cov) | complete | [`IP-004-rust-toolchain-gates.md`](IP-004-rust-toolchain-gates.md) |

## Estimated parallelism
4 agents.

## Symbols-touched
`docs/standards/prfaq-template.md`, `docs/standards/design-doc-template.md`, `docs/standards/postmortem-template.md`, `.github/workflows/`, `scripts/check.sh`.

## Agent-handoff
```
icm store -t context-oyatie -c "M-CC-P07 complete: hyperscaler practices adopted; PRFAQ + Design Doc + Postmortem + 1ES + Eng-Excellence + Rust toolchain gates live" -i critical -k "M-CC,P07,hyperscaler-practices,complete"
```
