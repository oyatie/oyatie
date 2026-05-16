---
doc_class: Runbook
runbook_class: Tombstone
runbook_id: RB-GRIT-PARALLEL-CLAIM-DEMO
status: retired
adr_ref: docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md
doc_status: published
---

# Runbook: grit parallel-claim demo (RETIRED 2026-05-16)

This runbook is retired. The `grit` parallel-symbol-claim acceptance drill is no longer applicable: `grit` is retired from the prescribed agent surface per ADR-0116. Per-agent isolation is now achieved by `git worktree` (Layer 0), with concurrent-safe-paths handled by the Foundry pipeline admission-gate (ADR-0111). The companion script `grit-parallel-claim-demo.sh` is also retired.

See `docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md` for the replacement.
