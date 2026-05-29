# REMEDIATION-NOTES-2026-05-21-tier-scrub

Service: application

Files modified with current line counts:
- `README.md` — 5 lines
- `manifest.json` — 451 lines
- `capabilities/module-load.yaml` — 93 lines
- `capabilities/session-emit.yaml` — 87 lines
- `capabilities/shell-render.yaml` — 96 lines
- `capabilities/tenant-admin-console-control.yaml` — 55 lines
- `competitor-parity-matrix.md` — 141 lines
- `compliance.md` — 1038 lines
- `policy/route-isolation.md` — 188 lines

capability-tiers/ dir deleted: Y

Vocabulary replacement count: ~55 direct and derived replacements.

Design decisions:
- Replaced `capability_tiers` manifest metadata with `tenant_class_eligibility` and `paid_billing_components_emitted`.
- Collapsed retired routing examples into tenant_class, deployment-context, and cell_topology language.
- Preserved route-isolation SLO intent while removing customer ladder wording.

Outstanding follow-ups: none for assigned forbidden vocabulary.
