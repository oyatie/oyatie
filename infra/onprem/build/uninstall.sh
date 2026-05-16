#!/usr/bin/env bash
# uninstall.sh — paired with build/build-all-targets.sh.
# build-all-targets is a one-shot cross-compile driver; the only persistent
# state it leaves is the Rust target/ directory, which is repo-local.
# This uninstall is therefore a no-op pointer to `cargo clean`.
set -uo pipefail
echo "build/build-all-targets.sh has no persistent host state."
echo "To clean Rust artifacts: cd /home/oyatie/projects/oyatie && cargo clean"
