#!/usr/bin/env bash
#
# regen-third-party.sh — DURABLY regenerate third-party/BUCK.
#
# Reindeer regenerates third-party/BUCK from Cargo.lock + the static
# third-party/fixups/*/fixups.toml. Two cross-platform-correctness edits cannot
# currently be expressed in those fixups:
#   - aws-lc-rs needs aws-lc-sys DEP_* propagation with a `$(location ...)` macro.
#   - psm needs a per-OS preprocessor_flags `select()` so Linux uses the ELF symbol.
# Bare Reindeer output is therefore incomplete and is never the canonical face.
# (See docs/decisions/ADR-0514 + the per-crate notes in third-party/fixups/*.)
#
# This wrapper renders into a temporary file, applies a semantic fail-closed
# overlay, and replaces third-party/BUCK only after the complete overlay succeeds.
# The overlay uses rule names and exact fragments, never hunk offsets, so dependency
# churn cannot silently produce a partially patched generated face.
#
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

REINDEER_REV="681727ced54a853977ac495e147ac54e1c0db115"
OVERLAY="tools/buck/apply-thirdparty-patches.py"
BUCK_FACE="third-party/BUCK"

REINDEER_BIN="$(command -v reindeer || true)"
[ -n "$REINDEER_BIN" ] || {
  echo "ERROR: reindeer not found on PATH"
  echo "Install: cargo install --git https://github.com/facebookincubator/reindeer --rev $REINDEER_REV reindeer"
  exit 1
}
REINDEER_BIN="$(python3 - "$REINDEER_BIN" <<'PY'
from pathlib import Path
import sys

try:
    print(Path(sys.argv[1]).resolve(strict=True))
except OSError as error:
    print(f"ERROR: cannot resolve reindeer executable: {error}", file=sys.stderr)
    raise SystemExit(2)
PY
)"
[ -f "$OVERLAY" ] || { echo "ERROR: missing $OVERLAY"; exit 1; }

# Reindeer does not expose its git revision through `--version`. Bind the exact
# executable selected from PATH to the full-revision Cargo receipt in that same
# install root. Merely finding a matching receipt elsewhere is insufficient.
python3 - "$REINDEER_BIN" "$REINDEER_REV" <<'PY'
import json
from pathlib import Path
import sys

binary = Path(sys.argv[1])
revision = sys.argv[2]
install_root = binary.parent.parent
expected_binary = (install_root / "bin" / "reindeer").resolve()
receipt = install_root / ".crates2.json"
expected_install = (
    "reindeer 0.0.0 "
    f"(git+https://github.com/facebookincubator/reindeer#{revision})"
)

if binary != expected_binary:
    print(
        f"ERROR: resolved reindeer binary {binary} is not {expected_binary}",
        file=sys.stderr,
    )
    raise SystemExit(2)
try:
    installs = json.loads(receipt.read_text()).get("installs", {})
except (OSError, json.JSONDecodeError) as error:
    print(f"ERROR: cannot read Cargo install receipt {receipt}: {error}", file=sys.stderr)
    raise SystemExit(2)
record = installs.get(expected_install)
if not isinstance(record, dict) or "reindeer" not in record.get("bins", []):
    print(
        "ERROR: selected reindeer binary is not bound to required revision "
        f"{revision} in {receipt}",
        file=sys.stderr,
    )
    print(
        "Install: cargo install --git https://github.com/facebookincubator/reindeer "
        f"--rev {revision} reindeer",
        file=sys.stderr,
    )
    raise SystemExit(2)
PY

TEMP_FACE="$(mktemp "third-party/.BUCK.regen.XXXXXX")"
trap 'rm -f "$TEMP_FACE"' EXIT

echo "[regen-third-party] rendering with pinned reindeer $REINDEER_REV ..."
"$REINDEER_BIN" buckify --stdout >"$TEMP_FACE"

echo "[regen-third-party] applying exact semantic overlay ($OVERLAY) ..."
python3 "$OVERLAY" --buck-file "$TEMP_FACE"

chmod 0644 "$TEMP_FACE"
if [ -f "$BUCK_FACE" ] && cmp -s "$TEMP_FACE" "$BUCK_FACE"; then
  echo "[regen-third-party] canonical face unchanged"
else
  mv "$TEMP_FACE" "$BUCK_FACE"
  echo "[regen-third-party] replaced $BUCK_FACE atomically"
fi

echo "[regen-third-party] done. The per-OS select() + \$(location) DEP env are present."
echo "[regen-third-party] Review: git diff third-party/BUCK should reflect ONLY intended dep changes."
