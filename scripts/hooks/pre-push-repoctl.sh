#!/usr/bin/env bash
# Purpose: local-developer git pre-push hook. Thin wrapper that forwards
# to `oya-dev-cli repoctl pre-push` (which itself now delegates to the
# canonical `oya verify` aggregator via `scripts/check.sh`). Installed
# by developers as `.git/hooks/pre-push`. No CI binding.
#
# User directive 2026-05-15: "pre-push should really just be part of
# some other check/validate" — fully folded behind the new top-level
# `oya verify` subcommand (see `crates/oya-dev-cli/src/commands/verify.rs`).
# The `repoctl pre-push` binary surface is retained because the
# `oya-check-pre-push` contract kernel encodes it as the canonical
# command name (lib.rs `CANONICAL_PRE_PUSH_COMMAND`). Folding the
# contract kernel itself into the verify surface is tracked as a
# follow-up so this slice does not collide with concurrent work on
# the contract kernel.
#
# Scheduled-replacement: full deletion of this .sh tracked under the
# transitional-.sh-removal sweep (audit row B-12 in
# `evidence/audits/shell-python-replacement-audit-2026-05-15.md`); the
# hook should be installed by `oya-dev-cli hook install` once available.
# Local developers can already invoke `oya verify` directly to bypass
# the wrapper.
set -euo pipefail
cargo run -p oya-dev-cli --bin repoctl -- pre-push "$@"
