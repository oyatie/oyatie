---
doc_class: Runbook
runbook_class: Tombstone
id: RB-SANCTIONED-PRIMITIVES-PREFLIGHT
status: retired
adr_ref: docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md
---

# Sanctioned-primitives preflight — RETIRED 2026-05-16

This runbook is retired. Sanctioned-primitive preflight (grit/icm/rtk/vox checks) is no longer applicable; those tools are retired from the prescribed agent surface. See `docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md` for the replacement: the Foundry pipeline (M-CC-P11) admission-gate + merge-queue. Direct `cargo` (no shim) and `git` + `gh` are the only build/coordination primitives required.
