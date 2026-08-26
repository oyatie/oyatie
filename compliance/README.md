# Compliance

`compliance/` owns the evidence engine and Compliance-as-a-Service facade:
catalog, bind, list/preview/project, evidence coverage, and export. Policy owns
authorization evaluation, Audit owns the tamper-evident record, and root
`packs/` owns Compliance-as-Code data.

Read the owner law before changing this directory:

- [ADR.md](ADR.md) — decisions in force
- [PRD.md](PRD.md) — product requirements and promotion targets
- [SPEC.md](SPEC.md) — behavior and contract
- [PLAN.md](PLAN.md) — sequenced remaining work

The current tree is pre-product structural debt: typed Rust validation models,
one retained retention oracle, and unconsumed artifacts. It is not a running
CaS facade, evidence engine, compliance certification, or production SLO.
