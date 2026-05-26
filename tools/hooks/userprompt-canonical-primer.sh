#!/usr/bin/env bash
# tools/hooks/userprompt-canonical-primer.sh
#
# Trigger:  Claude Code UserPromptSubmit
# Purpose:  Append a one-line canonical primitive reminder to each user prompt's
#           context so agents consistently see the correct tool names and versions.
# Behavior: Prints a single reminder line to stdout (becomes session context).
#           Read-only; no project state is modified.
# Non-blocking guarantee: exits 0 always.

set -euo pipefail

echo "Canonical: plain git for VCS work; governance gates are ./bin/oya verify --ci-required and oya gate run-all. OpenAPI 3.2.0 + AsyncAPI 3.1.0. See tools/hooks/_canonical-primitives.md."

exit 0
