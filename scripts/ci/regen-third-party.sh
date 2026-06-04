#!/usr/bin/env bash
#
# regen-third-party.sh — DURABLY regenerate third-party/BUCK.
#
# `reindeer buckify` regenerates third-party/BUCK from Cargo.lock + the static
# third-party/fixups/*/fixups.toml. But several cross-platform-correctness edits
# CANNOT be expressed in TOML/reindeer fixups (per-OS `select()`s and `$(location ...)`
# macros), so a BARE `reindeer buckify` SILENTLY re-breaks the aarch64-linux build by
# re-introducing darwin-hardcoded values:
#   - aws-lc-sys  LDFLAGS=-nostartfiles  (build-script feature-test double-CRT, #93)
#   - openssl     DEP_OPENSSL_* per-OS select() (E0425 EVP_idea_*, #91)
#   - psm         psm_asm preprocessor_flags per-OS select() (undefined
#                 rust_psm_stack_pointer at the rust_binary final link, #96/#78)
#   - aws-lc-sys  DEP_AWS_LC_*_INCLUDE = $(location ...) link env
# (See docs/decisions/ADR-0514 + the per-crate notes in third-party/fixups/*.)
#
# This wrapper runs buckify THEN re-applies those hand-edits from a captured patch,
# so the regen is reproducible and the Linux build stays green. ALWAYS regenerate via
# this script, never bare `reindeer buckify`.
#
# When the underlying crate set changes (add/update/remove a dep), the patch may no
# longer apply (context drift). In that case: run `reindeer buckify`, re-apply the
# hand-edits by hand (search the fixups/* notes), confirm the aarch64-linux build is
# green, then re-capture the patch:
#     reindeer buckify
#     # ... re-apply hand-edits, verify green ...
#     git diff -R -- third-party/BUCK > scripts/ci/third-party-buckify-handedits.patch
#
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

PATCH="scripts/ci/third-party-buckify-handedits.patch"

command -v reindeer >/dev/null 2>&1 || { echo "ERROR: reindeer not found on PATH; install it from the pinned Buck2/Reindeer toolchain image"; exit 1; }
[ -f "$PATCH" ] || { echo "ERROR: missing $PATCH"; exit 1; }

echo "[regen-third-party] reindeer buckify ..."
reindeer buckify

echo "[regen-third-party] re-applying cross-platform hand-edits ($PATCH) ..."
if ! git apply "$PATCH"; then
  echo "ERROR: hand-edits patch failed to apply — the dep set likely changed."
  echo "       Re-apply the fixups/* hand-edits by hand, verify the aarch64-linux build,"
  echo "       then re-capture: git diff -R -- third-party/BUCK > $PATCH"
  exit 1
fi

echo "[regen-third-party] done. The per-OS select()s + LDFLAGS + \$(location) DEP env are restored."
echo "[regen-third-party] Review: git diff third-party/BUCK should reflect ONLY intended dep changes."
