#!/usr/bin/env bash
# .omc/hooks/grit-claim-intent-gate.sh
# Authority: .omc/plans/ralplan-freelance-prevention-controls-2026-05-14.md §6(d)
#
# PreToolUse hook for Claude Code. Receives JSON on stdin:
#   {"session_id":..., "tool_name":"Bash", "tool_input":{"command":"..."}}
#
# Rejects (exit 2) `grit claim` / `grit begin` commands that touch crates/oya-*/
# unless --intent text cites BOTH:
#   (a) a phase ID — regex: (M\d\d-[a-z-]+/P\d\d|M\d\d-P\d\d|Wave \d|IP-\w+)
#   (b) an Accepted ralplan or ADR — regex: (ralplan-[a-z0-9-]+-\d{4}-\d{2}-\d{2}|consensus-masterplan-\d{4}-\d{2}-\d{2}|ADR-\d{4})
#
# Read-side grit operations (status, show-session, symbols, watch) are exempt.
# Non-crate paths are exempt.

set -uo pipefail

# Parse stdin (Claude Code provides tool_input as JSON). Tolerate missing jq:
if command -v jq >/dev/null 2>&1; then
  payload=$(cat)
  tool_name=$(echo "$payload" | jq -r '.tool_name // empty')
  command=$(echo "$payload" | jq -r '.tool_input.command // empty')
else
  # Fallback: cannot parse stdin reliably; allow (don't block legitimate work)
  exit 0
fi

# Only inspect Bash invocations.
[ "$tool_name" = "Bash" ] || exit 0

# Only inspect grit claim/begin invocations.
if ! echo "$command" | grep -qE '\bgrit (claim|begin)\b'; then
  exit 0
fi

# Only inspect when the claim touches crates/oya-*/ paths.
if ! echo "$command" | grep -qE 'crates/oya-'; then
  exit 0
fi

# Extract --intent value. Accepts:
#   --intent "long text"
#   --intent='long text'
#   --intent long_text_no_spaces
# This captures the FIRST --intent block; if multi-line text, we capture the whole command line.
intent="$command"

phase_re='(M[0-9]{2}-[a-z-]+/P[0-9]{2}|M[0-9]{2}-P[0-9]{2}|Wave [0-9]|IP-\w+)'
plan_re='(ralplan-[a-z0-9-]+-[0-9]{4}-[0-9]{2}-[0-9]{2}|consensus-masterplan-[0-9]{4}-[0-9]{2}-[0-9]{2}|ADR-[0-9]{4})'

phase_ok=0
plan_ok=0
if echo "$intent" | grep -qE -- "$phase_re"; then phase_ok=1; fi
if echo "$intent" | grep -qE -- "$plan_re"; then plan_ok=1; fi

if [ $phase_ok -eq 1 ] && [ $plan_ok -eq 1 ]; then
  exit 0
fi

# Reject. stderr is shown to user/model.
{
  echo "grit-claim-intent-gate: REJECT"
  echo "  authority: .omc/plans/ralplan-freelance-prevention-controls-2026-05-14.md §6(d)"
  echo "  reason: grit claim --intent on crates/oya-* paths must cite BOTH:"
  echo "    (a) phase ID  — regex: $phase_re"
  echo "    (b) accepted plan/ADR — regex: $plan_re"
  echo "  found: phase_cited=$([ $phase_ok -eq 1 ] && echo yes || echo NO)  plan_cited=$([ $plan_ok -eq 1 ] && echo yes || echo NO)"
  echo "  fix: include phase ID + ralplan-*-YYYY-MM-DD (or ADR-NNNN) in the --intent text"
} >&2

exit 2
