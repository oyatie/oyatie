#!/usr/bin/env bash
# Transitional wrapper: delegates to the canonical Rust
# `oya gate validate architecture-boundaries` subcommand that replaces
# this script per Wave 2 of the shell/python → Rust replacement program
# (audit row B-2,
# `evidence/audits/shell-python-replacement-audit-2026-05-15.md`).
#
# The Rust port lives at
# `crates/oya-dev-cli/src/commands/gate/architecture_boundaries.rs`.
# All validation logic + the 8-case self-test suite is now Rust. This
# wrapper remains only so that pre-existing `bash scripts/check-...`
# invocations in developer muscle memory and any unmigrated runbooks
# keep working until the next minor release, at which point it will be
# `git rm`-ed by the same ADR that drops the `scripts/check.sh`
# wrapper.
#
# Canonical entry points:
#     cargo run -q -p oya-dev-cli -- gate validate architecture-boundaries
#     cargo run -q -p oya-dev-cli -- gate validate architecture-boundaries --self-test
set -euo pipefail
if [[ -d "/opt/homebrew/opt/rustup/bin" ]]; then
  export PATH="/opt/homebrew/opt/rustup/bin:${PATH}"
fi
exec cargo run -q -p oya-dev-cli -- gate validate architecture-boundaries "$@"
