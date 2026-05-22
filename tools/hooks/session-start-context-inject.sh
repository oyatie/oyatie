#!/usr/bin/env bash
# tools/hooks/session-start-context-inject.sh
#
# Trigger:  Claude Code SessionStart
# Purpose:  Inject canonical primitives cheat sheet into the new session's context.
#           Stdout from SessionStart hooks becomes agent context in Claude Code.
# Behavior: Prints a formatted primitives summary sourced from
#           tools/hooks/_canonical-primitives.md (single source of truth).
# Non-blocking guarantee: exits 0 always; never modifies any project state.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
PRIMITIVES="$REPO_ROOT/tools/hooks/_canonical-primitives.md"

echo "============================================================"
echo " OYATIE CANONICAL PRIMITIVES — session context (2026-05-18)"
echo "============================================================"
echo ""

if [ -f "$PRIMITIVES" ]; then
    cat "$PRIMITIVES"
else
    # Fallback inline summary when file not found (should not happen in normal use)
    echo "VCS:       oya git <git-subcommand> is the cutover target; current policy ratchet remains oya vcs <claim|work|verify|done|status|symbols|queue|watch|promote>"
    echo "VCS policy: Oya VCS state transitions plus oya git drop-in; ADR-0116 owns the transition boundary"
    echo "Contracts: OpenAPI 3.2.0 + AsyncAPI 3.1.0 + proto3"
    echo "AI:        microservices/intelligence/ (consumer) | microservices/foundry/ (internal)"
    echo "Taxonomy:  plugin-app-store / marketplace / community — 3 distinct µservices"
    echo "Quality:   100+ artifacts per µservice (ADR-0212)"
    echo ""
    echo "See tools/hooks/_canonical-primitives.md for full reference."
fi

echo ""
echo "============================================================"
echo " LIFECYCLE SKILL MAP"
echo "============================================================"
echo ""
echo "Vendored skills at tools/agent-skills/skills/ (MIT — Addy Osmani and contributors)"
echo ""
echo "Define:  interview-me | idea-refine | spec-driven-development"
echo "Plan:    planning-and-task-breakdown"
echo "Build:   incremental-implementation | test-driven-development | source-driven-development"
echo "         doubt-driven-development | context-engineering | api-and-interface-design"
echo "         frontend-ui-engineering"
echo "Verify:  browser-testing-with-devtools | debugging-and-error-recovery"
echo "Review:  code-review-and-quality | code-simplification | security-and-hardening"
echo "         performance-optimization"
echo "Ship:    git-workflow-and-versioning | ci-cd-and-automation | deprecation-and-migration"
echo "         documentation-and-adrs | shipping-and-launch"
echo ""
echo "Persona agents (tools/agent-skills/agents/):"
echo "  review   → code-reviewer"
echo "  security → security-auditor"
echo "  tests    → test-engineer"
echo ""
echo "Discovery rule: invoke the skill matching the task phase BEFORE producing output."
echo "Process skills (Define/Plan) come before implementation skills (Build/Verify/Ship)."
echo ""
echo "============================================================"
echo " Hooks are GUIDANCE only. CI gates (registry/quality/lanes.yaml) enforce."
echo " ADR-0221: hooks are guidance infrastructure, not enforcement infrastructure."
echo "============================================================"

exit 0
