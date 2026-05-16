#!/usr/bin/env bash
# RETIRED 2026-05-16 per docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md
# `grit` is no longer prescribed in the agent surface. Per-agent isolation is
# achieved via plain `git worktree` and the Foundry pipeline (M-CC-P11)
# admission-gate concurrent-safe-paths. This script is preserved as history
# only; invoking it exits non-zero with a retirement notice.

echo "grit-parallel-claim-demo.sh: RETIRED — see docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md" >&2
exit 64
