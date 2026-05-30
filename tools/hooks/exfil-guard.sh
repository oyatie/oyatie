#!/usr/bin/env bash
# exfil-guard.sh (PreToolUse:Bash) — HARD-BLOCK data-exfiltration attempts.
#
# Blocks: curl|wget|nc|ncat|socat|telnet|dig|nslookup to non-allowlisted external hosts,
#         /dev/tcp|/dev/udp bash redirects, base64|xxd piped to network cmd,
#         python -c with socket/urllib/requests/httpx egress, scp/sftp to non-allowlisted.
#
# CRITICAL ALLOWLIST (never blocked):
#   git (push/fetch/clone to any configured remote incl github.com + localhost:3000 Forgejo)
#   buck2, cargo, rustup, package managers (brew/apt/npm/pip install)
#   localhost / 127.0.0.1 / *.local / internal hosts
#
# Exit codes: 2 = block, 0 = allow
# Logs blocks to: /tmp/oya-security-audit.log

set -uo pipefail

AUDIT_LOG="${OYA_SECURITY_AUDIT_LOG:-/tmp/oya-security-audit.log}"

payload="$(cat)"

# ── Extract command portably ─────────────────────────────────────────────────
# Support Claude Code (tool_input.command), Codex (command), and generic fallback.
cmd="$(printf '%s' "$payload" | python3 -c '
import sys, json
try:
    d = json.load(sys.stdin)
    # Claude Code shape: {"tool_input": {"command": "..."}}
    v = d.get("tool_input", {}).get("command", "")
    if v:
        print(v)
        sys.exit(0)
    # Codex shape: {"command": "..."}
    v = d.get("command", "")
    if v:
        print(v)
        sys.exit(0)
    # Gemini shape: {"input": {"command": "..."}}
    v = d.get("input", {}).get("command", "")
    if v:
        print(v)
        sys.exit(0)
except Exception:
    pass
print("")
' 2>/dev/null || true)"

# If extraction fails, fall back to raw payload scan
if [ -z "$cmd" ]; then
    cmd="$payload"
fi

block() {
    local reason="$1"
    local ts
    ts="$(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date)"
    {
        echo "[$ts] EXFIL-GUARD BLOCK: $reason"
        echo "  cmd: $(printf '%s' "$cmd" | head -c 200)"
    } >> "$AUDIT_LOG" 2>/dev/null || true
    {
        echo "BLOCKED [exfil-guard]: $reason"
        echo "Command: $(printf '%s' "$cmd" | head -c 200)"
        echo ""
        echo "If this is a legitimate operation, check the allowlist in tools/hooks/exfil-guard.sh."
        echo "Audit log: $AUDIT_LOG"
    } >&2
    exit 2
}

# ── ALLOWLIST fast-paths (return before any block checks) ───────────────────
# These patterns are ALWAYS allowed regardless of what follows.

# git commands (push/fetch/clone/pull/ls-remote etc.) — any remote including github.com + Forgejo
if printf '%s' "$cmd" | grep -Eq '(^|[;&|([:space:]])(git)[[:space:]]'; then
    exit 0
fi

# buck2 / cargo / rustup
if printf '%s' "$cmd" | grep -Eq '(^|[;&|([:space:]])(buck2|cargo|rustup)[[:space:]]'; then
    exit 0
fi

# Package managers (install only — not arbitrary network)
if printf '%s' "$cmd" | grep -Eq '(^|[;&|([:space:]])(brew|apt|apt-get|yum|dnf|npm|pip|pip3|yarn|pnpm)[[:space:]]+(install|update|upgrade|add|i)[[:space:]]'; then
    exit 0
fi

# ── /dev/tcp and /dev/udp bash redirects ────────────────────────────────────
if printf '%s' "$cmd" | grep -Eq '/dev/(tcp|udp)/'; then
    # Allow localhost targets
    if printf '%s' "$cmd" | grep -Eq '/dev/(tcp|udp)/(localhost|127\.[0-9]+\.[0-9]+\.[0-9]+|0\.0\.0\.0)'; then
        exit 0
    fi
    block "/dev/tcp|/dev/udp redirect to external host detected"
fi

# ── base64/xxd piped into a network command ──────────────────────────────────
if printf '%s' "$cmd" | grep -Eq '(base64|xxd)[^|]*\|[^|]*(curl|wget|nc |ncat|socat)'; then
    block "base64/xxd output piped into network command (potential data exfiltration)"
fi

# ── python -c with egress patterns ──────────────────────────────────────────
if printf '%s' "$cmd" | grep -Eq "python3?[[:space:]]+-c[[:space:]]+['\"]" && \
   printf '%s' "$cmd" | grep -Eq '(socket|urllib|requests|httpx)\.(connect|urlopen|get|post|request|open)'; then
    # Allow localhost
    if printf '%s' "$cmd" | grep -Eq '(localhost|127\.[0-9])'; then
        exit 0
    fi
    block "python -c with socket/urllib/requests/httpx network egress detected"
fi

# ── Network tool checks (curl/wget/nc/ncat/socat/telnet/dig/nslookup/scp/sftp) ──
# Extract the network-tool invocations and check their targets.

network_check() {
    local tool_pattern="$1"
    local tool_name="$2"

    if ! printf '%s' "$cmd" | grep -Eq "(^|[;&|([:space:]])$tool_pattern([[:space:]]|\$)"; then
        return 0
    fi

    # Allow if the only hosts are local
    # Strip common flags to get to the URL/host argument
    local stripped
    stripped="$(printf '%s' "$cmd" | sed 's/-[a-zA-Z][a-zA-Z0-9]*[[:space:]]*//g; s/--[a-zA-Z][a-zA-Z0-9-]*[[:space:]]*[^[:space:]]*//g')"

    # Allowlisted: localhost, 127.x.x.x, *.local, *.internal, 10.x, 172.16-31.x, 192.168.x
    # Also allow if no external-looking host pattern found
    if printf '%s' "$stripped" | grep -Eiq \
        '(localhost|127\.[0-9]+\.[0-9]+\.[0-9]+|0\.0\.0\.0|[a-z0-9-]+\.local[^a-z]|[a-z0-9-]+\.internal[^a-z]|10\.[0-9]+\.[0-9]+\.[0-9]+|172\.(1[6-9]|2[0-9]|3[01])\.[0-9]+\.[0-9]+|192\.168\.[0-9]+\.[0-9]+)'; then
        return 0
    fi

    # Allow well-known package/tool hosts for the given tool (brew uses curl internally etc.)
    if printf '%s' "$cmd" | grep -Eiq \
        '(formulae\.brew\.sh|homebrew|raw\.githubusercontent\.com|api\.github\.com|github\.com|crates\.io|registry\.npmjs\.org|pypi\.org|registry-1\.docker\.io|ghcr\.io|index\.crates\.io|static\.rust-lang\.org|rustup\.rs|dl\.google\.com|storage\.googleapis\.com)'; then
        return 0
    fi

    # Look for explicit external URLs / IPs in the full command
    if printf '%s' "$cmd" | grep -Eq \
        '(https?://|ftp://|[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}|[a-zA-Z0-9-]+\.[a-zA-Z]{2,}/)'; then
        # It has an explicit external-looking target — block
        block "$tool_name to potentially non-allowlisted external host"
    fi

    # No explicit target found — allow (ambiguous, safer to allow to avoid false positives)
    return 0
}

network_check 'curl' 'curl'
network_check 'wget' 'wget'
network_check 'ncat?' 'nc/ncat'
network_check 'socat' 'socat'
network_check 'telnet' 'telnet'
network_check 'dig' 'dig'
network_check 'nslookup' 'nslookup'
network_check 'host' 'host'

# scp/sftp: block any non-localhost target
if printf '%s' "$cmd" | grep -Eq "(^|[;&|([:space:]])(scp|sftp)[[:space:]]"; then
    if ! printf '%s' "$cmd" | grep -Eq '(localhost|127\.[0-9]+|[a-z0-9-]+\.local([[:space:]]|:)|[a-z0-9-]+\.internal([[:space:]]|:))'; then
        block "scp/sftp to non-allowlisted host"
    fi
fi

exit 0
