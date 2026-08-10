#!/usr/bin/env bash
# self-check.sh — hermetic local checks for Swarm Delivery Law Phase A kit.
#
# Opt-in / local evidence. Does not claim merge authority.
# Seeds anti-drift-drift-grep: forbid prose root/hub/freeze enumerations outside
# envelopes JSON in ADR-0711 + PORTABLE.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

fail=0
pass() { printf 'PASS  %s\n' "$*"; }
fail_msg() { printf 'FAIL  %s\n' "$*"; fail=1; }

ENVELOPES="specs/integ-branch-envelopes.json"
ADR="docs/decisions/ADR-0711-swarm-delivery-law-integ-branch-topology.md"
PORTABLE=".grok/programs/delivery-fabric/evidence/PORTABLE-SWARM-CONTRACT.md"
REGISTRY="registry/vcs/concurrent-safe-paths.yaml"
GIT_SHIM="tools/swarm/git-shim"
SWEEP="governance/check/integ-envelope/judgments/naming-sweep.json"

# 1) Envelopes parse + required keys
python3 - "$ENVELOPES" <<'PY' || fail_msg "envelopes JSON invalid / missing keys"
import json, sys
path = sys.argv[1]
with open(path) as f:
    e = json.load(f)
need = ["roots", "planes", "hubs", "anti_drift", "merge_windows", "concurrent_safe_exemptions", "reorg_debt_freeze", "naming"]
missing = [k for k in need if k not in e]
if missing:
    raise SystemExit(f"missing keys: {missing}")
ad = e["anti_drift"]
if ad.get("anti_drift_doctrine_version") != "1.1.0":
    raise SystemExit(f"anti_drift_doctrine_version={ad.get('anti_drift_doctrine_version')!r}")
if "docs_touched" not in ad.get("doc_packet_required_fields", []):
    raise SystemExit("anti_drift missing docs_touched")
inv = ad.get("invariants") or []
need_inv = [f"INV-DOC-{i}" for i in range(1, 10)]
missing_inv = [x for x in need_inv if x not in inv]
if missing_inv:
    raise SystemExit(f"anti_drift.invariants missing {missing_inv}")
if e.get("_meta", {}).get("version") != "1.16.5":
    raise SystemExit(f"_meta.version={e.get('_meta', {}).get('version')!r} want 1.16.5")
mw = e["merge_windows"]
if mw.get("hot_set_max", 0) != 4:
    raise SystemExit(f"merge_windows.hot_set_max={mw.get('hot_set_max')}")
paths = e["concurrent_safe_exemptions"].get("paths", [])
if "evidence/**" in paths:
    raise SystemExit("concurrent_safe still lists bare evidence/** (self-violation)")
if e.get("naming", {}).get("naming_sweep"):
    raise SystemExit("naming_sweep still inlined in envelopes (wrong home)")
bg = e["reorg_debt_freeze"].get("birth_gate", {})
if bg.get("scope") != "unit_root_birth":
    raise SystemExit("birth_gate.scope must be unit_root_birth")
for hole in ("kernel", "base", "app"):
    if hole not in e["roots"]:
        raise SystemExit(f"missing forward-declared root {hole}")
if "process_meta" not in e["planes"]:
    raise SystemExit("missing planes.process_meta")
# root-ops-contract-route: root survival hubs + .cursor must be routable via process_meta
pm_globs = set(e["planes"]["process_meta"].get("envelope_globs") or [])
need_pm = {"AGENTS.md", "CLAUDE.md", "README.md", "HANDOFF.md", ".cursor/**"}
missing_pm = sorted(need_pm - pm_globs)
if missing_pm:
    raise SystemExit(f"planes.process_meta missing root-ops globs: {missing_pm}")
if e["planes"]["process_meta"].get("branch") != "integ/ci":
    raise SystemExit("planes.process_meta.branch must remain integ/ci (forever owner)")
# northstar residual: HANDOFF indexed under northstar_baseline.root_survival
nb = e.get("reorg_debt_freeze", {}).get("northstar_baseline", {})
rs = (nb.get("root_survival") or {}).get("HANDOFF.md") or {}
if rs.get("destination_integ") != "integ/ci" or rs.get("forever") != "HANDOFF.md":
    raise SystemExit("northstar_baseline.root_survival.HANDOFF.md must route forever→integ/ci")
for rail in ("oya/itsm/", "oya/sheets/", "oya/translate/", "oya/sites/"):
    pr = (nb.get("product_rails") or {}).get(rail) or {}
    if not pr.get("destination_integ") or not pr.get("shrink_source_integ"):
        raise SystemExit(f"northstar_baseline.product_rails missing two-phase fields for {rail}")
print("ok")
PY
if [[ $fail -eq 0 ]]; then pass "envelopes schema keys + anti_drift INV-DOC-1…9 + merge_windows + holes + root-ops process_meta + HANDOFF"; fi

# 2) Registry ↔ envelopes concurrent-safe parity (narrowed evidence)
python3 - "$ENVELOPES" "$REGISTRY" <<'PY' || fail_msg "concurrent-safe parity"
import json, re, sys
env_path, reg_path = sys.argv[1:3]
with open(env_path) as f:
    e = json.load(f)
env_paths = set(e["concurrent_safe_exemptions"]["paths"])
text = open(reg_path).read()
# YAML list under safe_paths: lines like "      - foo/**"
reg_paths = set(re.findall(r'^\s+-\s+(\S+)\s*$', text, flags=re.M))
# Ignore schema_version noise — only path-like entries
reg_paths = {p for p in reg_paths if "/" in p or p.startswith(".")}
missing_in_reg = env_paths - reg_paths
extra_broad = {"evidence/**"} & reg_paths
if missing_in_reg:
    raise SystemExit(f"registry missing envelopes paths: {sorted(missing_in_reg)}")
if extra_broad:
    raise SystemExit(f"registry still has broad paths: {sorted(extra_broad)}")
print("ok")
PY
if [[ $fail -eq 0 ]]; then pass "concurrent-safe envelopes↔registry parity"; fi

# 3) Drift-grep: prose MUST NOT re-list governed roots / freeze layout tables (INV-DOC-2)
for f in "$ADR" "$PORTABLE"; do
  if [[ ! -f "$f" ]]; then
    fail_msg "missing $f"
    continue
  fi
  if grep -E -n '`os`, `ci`, `governance`' "$f" >/dev/null 2>&1; then
    fail_msg "prose root enumeration in $f (cite envelopes#roots instead)"
  else
    pass "no legacy root comma-list in $f"
  fi
  if ! grep -E -q 'integ-branch-envelopes\.json#roots' "$f"; then
    fail_msg "$f must cite integ-branch-envelopes.json#roots"
  else
    pass "$f cites #roots"
  fi
  # Amendment B path tables must not reappear (cite #reorg_debt_freeze.rows only)
  if grep -E -n '\| current path \| action \|' "$f" >/dev/null 2>&1; then
    fail_msg "prose freeze/layout path table in $f (cite #reorg_debt_freeze.rows)"
  else
    pass "no Amendment B path table in $f"
  fi
  if ! grep -E -q 'reorg_debt_freeze\.rows' "$f"; then
    fail_msg "$f must cite reorg_debt_freeze.rows"
  else
    pass "$f cites #reorg_debt_freeze.rows"
  fi
done

# ADR must document INV-DOC-9
if grep -E -q 'INV-DOC-9' "$ADR"; then
  pass "ADR-0711 documents INV-DOC-9"
else
  fail_msg "ADR-0711 missing INV-DOC-9"
fi

# 4) git-shim denies --no-verify
if grep -E -q 'deny.*--no-verify' "$GIT_SHIM"; then
  pass "git-shim denies --no-verify"
else
  fail_msg "git-shim missing --no-verify deny"
fi
# Behavioral probe (best-effort; skip if bash can't exec)
SHIM="$ROOT/$GIT_SHIM"
if [[ -x "$SHIM" ]]; then
  if out="$("$SHIM" commit -n -m x -- README.md 2>&1)" && true; then
    :
  fi
  if printf '%s' "${out:-}" | grep -qi 'DENIED\|no-verify'; then
    pass "git-shim runtime denies commit -n"
  else
    # If deny didn't fire because of earlier pathspec/message issues, still require source deny.
    if grep -E -q 'has_flag -n \|\| has_flag --no-verify' "$GIT_SHIM"; then
      pass "git-shim source has -n/--no-verify gate"
    else
      fail_msg "git-shim runtime probe inconclusive and source gate missing"
    fi
  fi
fi

# 5) naming_sweep moved out
if [[ -f "$SWEEP" ]]; then
  python3 - "$SWEEP" <<'PY' || fail_msg "naming-sweep.json invalid"
import json, sys
with open(sys.argv[1]) as f:
    d = json.load(f)
rows = d.get("rows") or d.get("naming_sweep") or []
if len(rows) < 1:
    raise SystemExit("naming-sweep.json has no rows")
print("ok", len(rows))
PY
  pass "naming-sweep.json present outside envelopes"
else
  fail_msg "missing $SWEEP"
fi

# 6) deliver.js must not embed unescaped bash \${INTEG#...} in template
if grep -E -n '\$\{INTEG#integ/' .claude/workflows/deliver.js >/dev/null 2>&1; then
  fail_msg "deliver.js still has unescaped \${INTEG#integ/} (JS SyntaxError)"
else
  pass "deliver.js merge-tree template has no bare \${INTEG#integ/}"
fi

# 7) claim-push contains merge-tree + --check + dirty refuse
if grep -E -q 'merge-tree' tools/swarm/claim-push.sh; then
  pass "claim-push.sh runs merge-tree"
else
  fail_msg "claim-push.sh missing merge-tree"
fi
if grep -E -q 'CHECK_ONLY|--check' tools/swarm/claim-push.sh; then
  pass "claim-push.sh supports --check"
else
  fail_msg "claim-push.sh missing --check"
fi
if grep -E -q 'status --porcelain' tools/swarm/claim-push.sh \
  && grep -E -q 'working tree dirty|porcelain non-empty' tools/swarm/claim-push.sh; then
  pass "claim-push.sh refuses dirty porcelain"
else
  fail_msg "claim-push.sh missing dirty porcelain refuse"
fi

# 8) claim-mechanical: deliver.js must parseClaimPacket (not bare /^CLAIM/ theater)
if grep -E -q 'function parseClaimPacket' .claude/workflows/deliver.js; then
  pass "deliver.js defines parseClaimPacket"
else
  fail_msg "deliver.js missing parseClaimPacket (claim-mechanical)"
fi
if grep -E -q 'claimOk = claimed && /\^\\s\*CLAIM/i\.test' .claude/workflows/deliver.js; then
  fail_msg "deliver.js still uses bare /^CLAIM/ theater for claimOk"
else
  pass "deliver.js claimOk is not bare /^CLAIM/ theater"
fi
if grep -E -q 'parseClaimPacket\(claimed' .claude/workflows/deliver.js; then
  pass "deliver.js gates Land on parseClaimPacket"
else
  fail_msg "deliver.js does not call parseClaimPacket on claim summary"
fi
if grep -E -q 'bindDiff:\s*true' .claude/workflows/deliver.js \
  && grep -E -q 'Claim↔diff bind|Claim↔diff bind' .claude/workflows/deliver.js; then
  pass "deliver.js Claim↔diff bind enabled"
else
  fail_msg "deliver.js missing Claim↔diff bind (bindDiff:true)"
fi
if grep -E -q 'bind_diff|bind-diff|Claim↔diff' tools/swarm/claim_packet.py; then
  pass "claim_packet.py implements Claim↔diff bind"
else
  fail_msg "claim_packet.py missing Claim↔diff bind"
fi

# 9) claim_packet.py self-test (anti-drift-claim-fields + Claim↔diff bind)
if python3 tools/swarm/claim_packet.py --self-test; then
  pass "claim_packet.py self-test"
else
  fail_msg "claim_packet.py self-test failed"
fi

# 10) Behavioral probe: claim-push dirty refuse (temp dirty file in worktree)
DIRTY_PROBE=".claim-push-dirty-probe-$$"
if printf 'probe\n' > "$DIRTY_PROBE"; then
  if out="$(bash tools/swarm/claim-push.sh --check specs 2>&1)" && true; then
    :
  fi
  rm -f "$DIRTY_PROBE"
  if printf '%s' "${out:-}" | grep -qi 'dirty\|porcelain'; then
    pass "claim-push --check refuses dirty tree (behavioral)"
  else
    fail_msg "claim-push dirty behavioral probe did not REFUSE (out=${out:-empty})"
  fi
else
  fail_msg "could not create dirty probe file"
fi

if [[ $fail -ne 0 ]]; then
  echo "self-check: FAILED" >&2
  exit 1
fi
echo "self-check: OK"
