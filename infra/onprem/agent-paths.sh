#!/usr/bin/env bash
# agent-paths.sh — shared inventory of paths where coding agents read/write.
#
# Sourced by:
#   security/scan.sh   → scan every path for secrets (read-only)
#   cleanup/install.sh → reap stale entries under the *transient* subset
#
# Two lists:
#   AGENT_SCAN_PATHS     — anywhere an agent stores state, config, or scratch
#                          (scanned read-only; nothing is deleted)
#   AGENT_CLEANUP_GLOBS  — transient scratch / cache patterns safe to age out
#                          (matched against mtime > AGENT_CLEANUP_DAYS)
#
# Add new agents here; both consumers pick the additions up automatically.

REAL_USER=${SUDO_USER:-${USER:-oyatie}}
REAL_HOME=$(getent passwd "$REAL_USER" 2>/dev/null | cut -d: -f6)
[ -z "${REAL_HOME:-}" ] && REAL_HOME="/home/$REAL_USER"

AGENT_SCAN_PATHS=(
  # Claude Code
  "$REAL_HOME/.claude"
  "$REAL_HOME/.claude/projects"
  "$REAL_HOME/.cache/claude"
  "$REAL_HOME/.local/share/claude"
  "$REAL_HOME/.config/claude"

  # OpenAI Codex CLI
  "$REAL_HOME/.codex"
  "$REAL_HOME/.cache/codex"
  "$REAL_HOME/.local/share/codex"
  "$REAL_HOME/.config/codex"

  # Google Gemini CLI
  "$REAL_HOME/.gemini"
  "$REAL_HOME/.cache/gemini"
  "$REAL_HOME/.local/share/gemini"
  "$REAL_HOME/.config/gemini"

  # Editor-embedded agents
  "$REAL_HOME/.cursor"
  "$REAL_HOME/.vscode"
  "$REAL_HOME/.vscode-server"
  "$REAL_HOME/.vscode-insiders"
  "$REAL_HOME/.config/Code"
  "$REAL_HOME/.zed"
  "$REAL_HOME/.windsurf"

  # CLI / IDE-bridge AI coders
  "$REAL_HOME/.aider"
  "$REAL_HOME/.aider.chat.history.md"
  "$REAL_HOME/.cline"
  "$REAL_HOME/.continuedev"
  "$REAL_HOME/.config/Continue"

  # Generic config / cache buckets where agents drop state
  "$REAL_HOME/.config"
  "$REAL_HOME/.cache"
  "$REAL_HOME/.local/state"
  "$REAL_HOME/.local/share"

  # Transient
  /tmp
  /var/tmp
  /dev/shm

  # System logs
  /var/log

  # Catch-all for anything we haven't enumerated
  "$REAL_HOME"
)

# Transient scratch / cache GLOBS — agents lay these down and never reap.
# Cleanup reaps entries older than AGENT_CLEANUP_DAYS (default 7).
AGENT_CLEANUP_GLOBS=(
  /tmp/claude-*
  /tmp/codex-*
  /tmp/gemini-*
  /tmp/agent-*
  /tmp/oyatie-*
  /tmp/worktree-*
  /tmp/aider-*
  /tmp/cursor-*
  /tmp/zed-*
  /tmp/.cline-*
  /tmp/.aider*
  /var/tmp/claude-*
  /var/tmp/codex-*
  /var/tmp/gemini-*
  /var/tmp/agent-*
  /dev/shm/claude-*
  /dev/shm/codex-*
  /dev/shm/gemini-*
)

# Cache subdirs we may safely age out at the file level (>AGENT_CLEANUP_DAYS).
AGENT_CACHE_DIRS=(
  "$REAL_HOME/.cache/claude"
  "$REAL_HOME/.cache/codex"
  "$REAL_HOME/.cache/gemini"
  "$REAL_HOME/.cache/aider"
  "$REAL_HOME/.cache/cursor"
)

# Days of inactivity before transient state qualifies for reaping.
AGENT_CLEANUP_DAYS=${AGENT_CLEANUP_DAYS:-7}
