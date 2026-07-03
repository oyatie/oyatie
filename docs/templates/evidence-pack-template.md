---
doc_status: published
---

# Evidence pack template (per regulator)

> Per [`docs/STANDARDS-AND-TEMPLATES.md`](../STANDARDS-AND-TEMPLATES.md) §2 + ADR-0003. Lives at `audits/<regulator>/<year>/evidence-pack.md`. Validated by `compliance-evidence-recency`.

## Pack metadata
- **Regulator:** <name + jurisdiction>
- **Pack version:** <semver>
- **Audit cycle:** <annual / per-incident / on-demand>
- **Coverage window:** <start> .. <end>
- **Owner team:** `ops-compliance` + (per-regulator pack maintainer)
- **Cosign attestation:** <signature URI>

## Control mapping table

| Control ID | Control name | Evidence type | Evidence link | Cadence | Owner | Last verified |
|---|---|---|---|---|---|---|
| <id> | <name> | (audit-chain segment / config snapshot / test result / SBOM / report) | <URI> | <daily / weekly / monthly / per-release> | <team> | <date> |

## Cross-references
- [`COMPLIANCE-MATRIX.md`](../COMPLIANCE-MATRIX.md) — full per-regulator × control matrix
- [`security-program.json`](../security-program.json) — control implementations
- [`PRIVACY-PROGRAM.md`](../PRIVACY-PROGRAM.md) — privacy-control evidence

## Trust-portal publication
- ☐ Mirror regenerated to `trust.oyatie.com/<regulator>/<year>/`
- ☐ Per-regulator notification of new pack version
- ☐ Audit-chain emission: `EVT-EVIDENCE-PACK-PUBLISHED`
