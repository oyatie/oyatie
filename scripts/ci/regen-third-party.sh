#!/usr/bin/env bash
#
# regen-third-party.sh — DURABLY regenerate third-party/BUCK.
#
# `reindeer buckify` regenerates third-party/BUCK from Cargo.lock + the static
# third-party/fixups/*/fixups.toml. But several cross-platform-correctness edits
# CANNOT be expressed in TOML/reindeer fixups (per-OS `select()`s and `$(location ...)`
# macros), so a bare `reindeer buckify` would silently re-break the aarch64-linux build by
# re-introducing darwin-hardcoded values:
#   - aws-lc-sys  LDFLAGS=-nostartfiles  (build-script feature-test double-CRT, #93)
#   - openssl     DEP_OPENSSL_* per-OS select() (E0425 EVP_idea_*, #91)
#   - psm         psm_asm preprocessor_flags per-OS select() (undefined
#                 rust_psm_stack_pointer at the rust_binary final link, #96/#78)
#   - aws-lc-sys  DEP_AWS_LC_*_INCLUDE = $(location ...) link env
# (See docs/decisions/ADR-0514 + the per-crate notes in third-party/fixups/*.)
#
# This wrapper runs buckify THEN applies a semantic, fail-closed overlay. The overlay
# uses rule names and required anchors, never hunk offsets, so dependency churn cannot
# silently produce a partially patched generated face. ALWAYS regenerate via this script.
#
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

REINDEER_REV="681727ce"
OVERLAY="tools/buck/apply-thirdparty-patches.py"

command -v reindeer >/dev/null 2>&1 || { echo "ERROR: reindeer not found on PATH (cargo install reindeer)"; exit 1; }
[ -f "$OVERLAY" ] || { echo "ERROR: missing $OVERLAY"; exit 1; }

# Reindeer does not expose its git revision through `--version`. The cargo-install
# receipt is therefore the authoritative local provenance check for this generated
# face. A different generator revision may produce a different rule graph.
if ! cargo install --list 2>/dev/null | grep -Fq "reindeer v0.0.0 (https://github.com/facebookincubator/reindeer#$REINDEER_REV)"; then
  echo "ERROR: required reindeer revision $REINDEER_REV is not installed"
  exit 1
fi

echo "[regen-third-party] reindeer buckify ..."
reindeer buckify

echo "[regen-third-party] applying semantic cross-platform overlay ($OVERLAY) ..."
python3 "$OVERLAY"

echo "[regen-third-party] done. The per-OS select()s + LDFLAGS + \$(location) DEP env are restored."
echo "[regen-third-party] Review: git diff third-party/BUCK should reflect ONLY intended dep changes."
