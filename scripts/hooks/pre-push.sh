#!/usr/bin/env bash
# Canonical local pre-push hook: dispatches into the Rust
# `oya verify --ci-required --include-deferred` subcommand path (which in turn
# invokes `gate run-all --ci-required --include-deferred`, the canonical
# aggregator plus hosted required-check mirrors and the deferred-gate roster).
#
# Two forms are accepted:
#   1. Installed binary form: `oya verify --ci-required --include-deferred`
#      (when `oya` is on PATH).
#   2. Workspace-clone form:
#      `cargo run -q -p oya-dev-cli -- verify --ci-required --include-deferred`
#      (for contributors who haven't yet installed the `oya` binary).
#
# This file is the source-of-truth that the
# `oya-foundry-fitness-pre-push` lane reads as evidence; the lane
# rejects any hook that stops at plain `oya verify` because plain
# verify does not prove hosted required-check coverage.
#
# Sunset trigger: the long-term goal is to replace this two-line
# shim with a Rust-binary git hook installed via `oya hook install`
# (tracked under M01-P17 follow-up). git itself requires hooks to
# be executable files; the minimum-viable Rust replacement is a
# one-line shim that `exec`s into the oya binary, which is what
# this file is.
set -euo pipefail

if command -v oya >/dev/null 2>&1; then
  exec oya verify --ci-required --include-deferred "$@"
fi
exec cargo run -q -p oya-dev-cli -- verify --ci-required --include-deferred "$@"
