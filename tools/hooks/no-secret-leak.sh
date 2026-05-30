#!/usr/bin/env bash
# no-secret-leak.sh (PreToolUse: Bash + Write + Edit) — HARD-BLOCK secret commit/leak.
#
# Bash:  blocks `git add`/`git stage` of sensitive files (.env, *.pem, *.key, etc.)
# Write/Edit: blocks writing content matching secret patterns into tracked repo paths.
#
# CRITICAL ALLOWLIST (never blocked):
#   READING .env (awk/grep/source/cat to extract a var)
#   Masked push URL pattern: git push http://oya-admin:${TOKEN}@localhost:3000/...  (token in transient push URL is USE not leak)
#   .env.example / .env.template files
#
# Exit codes: 2 = block, 0 = allow

set -uo pipefail

payload="$(cat)"

# ── Extract tool name and inputs portably ────────────────────────────────────
tool_name="$(printf '%s' "$payload" | python3 -c '
import sys, json
try:
    d = json.load(sys.stdin)
    # Claude Code: {"tool_name": "...", "tool_input": {...}}
    v = d.get("tool_name", "")
    if v:
        print(v.lower())
        sys.exit(0)
    # Codex: {"type": "..."}
    v = d.get("type", "")
    if v:
        print(v.lower())
        sys.exit(0)
except Exception:
    pass
print("")
' 2>/dev/null || true)"

# ── BASH TOOL: git add/stage of sensitive files ──────────────────────────────
if [ "$tool_name" = "bash" ] || [ -z "$tool_name" ]; then
    cmd="$(printf '%s' "$payload" | python3 -c '
import sys, json
try:
    d = json.load(sys.stdin)
    v = d.get("tool_input", {}).get("command", "")
    if v:
        print(v)
        sys.exit(0)
    v = d.get("command", "")
    if v:
        print(v)
        sys.exit(0)
    v = d.get("input", {}).get("command", "")
    if v:
        print(v)
        sys.exit(0)
except Exception:
    pass
print("")
' 2>/dev/null || true)"

    # Check for git add/stage of sensitive files
    if printf '%s' "$cmd" | grep -Eq '(^|[;&|([:space:]])(git)[[:space:]]+(add|stage)[[:space:]]'; then
        # Allowlist: masked Forgejo push URL (token in transient push URL, not being committed)
        # This pattern is: git push http://oya-admin:${TOKEN}@localhost:3000/...
        # (add/stage wouldn't apply here, but belt-and-suspenders)

        # Check if any sensitive file patterns are in the git add arguments
        # Sensitive: .env (exact, not .env.example/.env.template), *.pem, *.key, id_rsa, id_ed25519, *.p12, *.pfx, *credentials*, *.kubeconfig
        if printf '%s' "$cmd" | grep -Eq '(^|[[:space:]])-f[[:space:]]'; then
            # -f flag present: force-add a potentially gitignored secret file — extra suspicious
            if printf '%s' "$cmd" | grep -Eq "([[:space:]]|/)(\.env|[^[:space:]]*\.pem|[^[:space:]]*\.key|id_rsa|id_ed25519|[^[:space:]]*\.p12|[^[:space:]]*\.pfx|[^[:space:]]*credentials[^[:space:]]*|[^[:space:]]*\.kubeconfig)([[:space:]]|$)"; then
                if ! printf '%s' "$cmd" | grep -Eq "(\.env\.(example|template|sample)|\.env\.test|\.env\.ci)"; then
                    {
                        echo "BLOCKED [no-secret-leak]: git add -f of sensitive credential file detected"
                        echo "Command: $(printf '%s' "$cmd" | head -c 200)"
                        echo ""
                        echo "Sensitive files (.env, *.pem, *.key, id_rsa, id_ed25519, *.p12, *.pfx, *credentials*, *.kubeconfig)"
                        echo "must not be force-added to git. Add them to .gitignore instead."
                    } >&2
                    exit 2
                fi
            fi
        fi

        # Without -f: also block adding sensitive files.
        # Use two-part check: is it a git add/stage AND does the command contain a sensitive filename?
        # (A single combined regex fails because [[:space:]]+ in the first part consumes the separator.)
        if printf '%s' "$cmd" | grep -Eq '(^|[;&|([:space:]])(git)[[:space:]]+(add|stage)([[:space:]]|$)' && \
           printf '%s' "$cmd" | grep -Eq '(^|[[:space:]]|/)(\.env|[^[:space:]]*\.pem|[^[:space:]]*\.key|id_rsa|id_ed25519|[^[:space:]]*\.p12|[^[:space:]]*\.pfx|[^[:space:]]*credentials[^[:space:]]*|[^[:space:]]*\.kubeconfig)([[:space:]]|$)'; then
            # Exclude .env.example/.env.template
            if ! printf '%s' "$cmd" | grep -Eq "(\.env\.(example|template|sample)|\.env\.test|\.env\.ci)"; then
                {
                    echo "BLOCKED [no-secret-leak]: git add of sensitive credential file detected"
                    echo "Command: $(printf '%s' "$cmd" | head -c 200)"
                    echo ""
                    echo "Sensitive files (.env, *.pem, *.key, id_rsa, id_ed25519, *.p12, *.pfx, *credentials*, *.kubeconfig)"
                    echo "must not be committed. Add them to .gitignore instead."
                } >&2
                exit 2
            fi
        fi

        # Catch `git add .` or `git add -A` or `git add -u` — these stage everything including .env
        # Only block if we're in a directory that has known sensitive files nearby
        if printf '%s' "$cmd" | grep -Eq 'git[[:space:]]+(add|stage)[[:space:]]+(-A|--all|\.[[:space:]]|\.?$)'; then
            # Check if .env or other sensitive files would be staged (advisory level only — don't hard-block blanket add)
            # Hard-block only if a .env file exists in the current working context
            # This is a best-effort check; false negatives possible but false positives on blanket add are too disruptive
            : # allow blanket git add — controlled by .gitignore
        fi
    fi

    # Allowlist: reading .env (awk/grep/cat/source patterns are fine — these are reads not writes)
    # Already handled by not blocking non-git-add commands.
fi

# ── WRITE/EDIT TOOL: secret patterns in content ──────────────────────────────
if [ "$tool_name" = "write" ] || [ "$tool_name" = "edit" ] || [ "$tool_name" = "multiedit" ]; then
    # Extract file path and content
    file_path="$(printf '%s' "$payload" | python3 -c '
import sys, json
try:
    d = json.load(sys.stdin)
    # Claude Code Write: tool_input.file_path
    v = d.get("tool_input", {}).get("file_path", "")
    if v:
        print(v)
        sys.exit(0)
    # Claude Code Edit: tool_input.file_path
    v = d.get("tool_input", {}).get("path", "")
    if v:
        print(v)
        sys.exit(0)
    # Generic
    v = d.get("file_path", d.get("path", ""))
    if v:
        print(v)
except Exception:
    pass
print("")
' 2>/dev/null || true)"

    content="$(printf '%s' "$payload" | python3 -c '
import sys, json
try:
    d = json.load(sys.stdin)
    ti = d.get("tool_input", d)
    # Write: content field
    v = ti.get("content", "")
    if v:
        print(v[:4000])  # Limit scan to first 4000 chars
        sys.exit(0)
    # Edit: new_string field
    v = ti.get("new_string", "")
    if v:
        print(v[:4000])
        sys.exit(0)
except Exception:
    pass
print("")
' 2>/dev/null || true)"

    # Skip .env.example, .env.template, .env.sample — these are safe templates
    if printf '%s' "$file_path" | grep -Eq '\.(example|template|sample)(\..*)?$|\.env\.(example|template|sample|test|ci)$'; then
        exit 0
    fi

    # Skip gitignored paths: .env at root
    if printf '%s' "$file_path" | grep -Eq '(^|/)\.env$'; then
        # Writing to .env is suspicious if content has real secrets
        : # fall through to content scan
    fi

    # Secret pattern checks on content
    SECRET_FOUND=""
    SECRET_TYPE=""

    # OpenBao unseal key / root token
    if printf '%s' "$content" | grep -Eq 'OPENBAO_ROOT_TOKEN[[:space:]]*=[[:space:]]*[A-Za-z0-9/+.]{10,}'; then
        SECRET_FOUND=1; SECRET_TYPE="OPENBAO_ROOT_TOKEN value"
    fi

    # Forgejo admin token (literal value, not ${TOKEN} variable reference which is a USE pattern)
    if [ -z "$SECRET_FOUND" ] && printf '%s' "$content" | grep -Eq 'FORGEJO_ADMIN_TOKEN[[:space:]]*=[[:space:]]*[A-Za-z0-9_\-]{10,}[^$\{]'; then
        SECRET_FOUND=1; SECRET_TYPE="FORGEJO_ADMIN_TOKEN literal value"
    fi

    # PEM private key blocks
    if [ -z "$SECRET_FOUND" ] && printf '%s' "$content" | grep -Eq '\-\-\-\-\-BEGIN[[:space:]][A-Z ]*PRIVATE KEY\-\-\-\-\-'; then
        SECRET_FOUND=1; SECRET_TYPE="PEM PRIVATE KEY block"
    fi

    # AWS access key ID
    if [ -z "$SECRET_FOUND" ] && printf '%s' "$content" | grep -Eq 'AKIA[0-9A-Z]{16}'; then
        SECRET_FOUND=1; SECRET_TYPE="AWS Access Key ID (AKIA...)"
    fi

    # Generic high-entropy secret assignment: api_key=, secret=, token=, password= with value ≥16 chars
    # ALLOWLIST: ${VAR} references and variable interpolation patterns (these are USEs not values)
    if [ -z "$SECRET_FOUND" ]; then
        # Match: key/secret/token/password = "value" or = value (non-variable, ≥16 alphanum chars)
        if printf '%s' "$content" | grep -Eiq '(api[_-]?key|api[_-]?secret|secret[_-]?key|auth[_-]?token|access[_-]?token|password|passwd)[[:space:]]*[=:][[:space:]]*[A-Za-z0-9/+]{16,}'; then
            # Exclude lines where the value is a variable reference: ${VAR}, $VAR, %(VAR)s, <PLACEHOLDER>
            # Check if the match is NOT a variable substitution
            if ! printf '%s' "$content" | grep -Eiq '(api[_-]?key|secret|token|password)[[:space:]]*[=:][[:space:]]*(\$\{|\$[A-Z]|<[A-Z]|YOUR_|PLACEHOLDER|EXAMPLE|CHANGE_ME|REPLACE)'; then
                # Further check: it's in a tracked file path (not .env or .env.* files outside the repo)
                # For .env.example etc. already excluded above
                SECRET_FOUND=1; SECRET_TYPE="generic API key/secret/token/password literal (≥16 chars)"
            fi
        fi
    fi

    if [ -n "$SECRET_FOUND" ]; then
        # Allowlist: masked Forgejo push URL (transient use, not stored)
        # Pattern: http://oya-admin:${TOKEN}@localhost:3000/... — this is a USE pattern
        if printf '%s' "$content" | grep -Eq 'https?://[^:]+:\$\{[^}]+\}@(localhost|127\.|[a-z0-9-]+\.local)'; then
            exit 0
        fi

        {
            echo "BLOCKED [no-secret-leak]: $SECRET_TYPE detected in write/edit content"
            echo "File: $file_path"
            echo ""
            echo "Writing secret values into the repository is prohibited."
            echo "Use \${VARIABLE} references or a secrets manager (OpenBao) instead."
            echo "Template files (.env.example, .env.template) are allowed — rename if this is a template."
        } >&2
        exit 2
    fi
fi

exit 0
