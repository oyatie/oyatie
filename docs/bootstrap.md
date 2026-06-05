# Contributor Bootstrap

Single-command setup for the oyatie contributor environment: canonical CLI, encouragement
hooks, and vendored lifecycle skills.

---

## Single-command install

```bash
./tools/hook-bootstrap/install.sh
```

Idempotent — safe to run multiple times. Exits 0 on success.

Options:
- `--dry-run` — preview all changes without writing anything
- `--skip-skills` — skip agent-skills fetch (offline contributors)
- `--sync-skills` — force re-vendor of agent-skills

---

## What you get

| Component | Installed at | Purpose |
|-----------|-------------|---------|
| Hook entries | `.claude/settings.json` | 12 encouragement hooks for Claude Code |
| Codex hooks (if detected) | `.codex/hooks.json` | Same hooks for Codex CLI |
| Gemini hooks (if detected) | `.gemini/settings.json` | Same hooks for Gemini CLI (event names: SessionStart/BeforeAgent/AfterAgent/BeforeTool/AfterTool) |
| Retired local CLI wrapper | none | The former `bin/oya`/`oya-dev-cli` developer wrapper is retired; use Buck2/Prow-native checks and substrate-owned APIs instead. |
| Shell completions | `tools/completions/{bash,zsh,fish}` | Tab-completion for all subcommands |
| Lifecycle skills | `tools/agent-skills/` | 23 vendored skills from addyosmani/agent-skills |
| Slash commands (Claude) | `.claude/commands/*.md` symlinks | 7 commands: `/spec /plan /build /test /review /code-simplify /ship` |
| Slash commands (Gemini) | `.gemini/commands/*.toml` symlinks | 7 commands (Gemini TOML format) — only created when Gemini detected |
| Skills discovery (per agent) | `.{claude,codex,gemini}/skills/` → `tools/agent-skills/skills/` | One symlink per detected agent, single-source-of-truth: edit upstream once, propagates to all surfaces |

If `.gemini/settings.json` already exists from prior Gemini use, install.sh refuses to overwrite and writes `.gemini/settings.json.oya-bootstrap-example` next to it — merge manually into your existing settings.

### What it doesn't do

- No user-level state (`~/.claude`, `~/.codex`, `~/.gemini`, `~/.local/bin` are never touched)
- No symlinks outside the repository
- No `~/.local/bin/oya` or similar global installs
- No modifications to your shell profile

---

## Per-shell PATH instructions (if direnv not installed)

If [direnv](https://direnv.net/) is installed, run once: `direnv allow`

Otherwise, add `bin/` to PATH manually in your shell profile:

**bash/zsh:**
```bash
export PATH="$PWD/bin:$PATH"
```

**fish:**
```fish
fish_add_path (string join / $PWD bin)
```

---

## What you get — Lifecycle Skills (23 skills)

Vendored from [addyosmani/agent-skills](https://github.com/addyosmani/agent-skills) (MIT).
See `tools/agent-skills/UPSTREAM.json` for the exact commit SHA and fetch timestamp.

| Phase | Skill | Purpose |
|-------|-------|---------|
| Define | `interview-me` | Extract real requirements before writing code |
| Define | `idea-refine` | Stress-test ideas before committing to a plan |
| Define | `spec-driven-development` | Write spec before writing code |
| Plan | `planning-and-task-breakdown` | Break work into ordered atomic tasks |
| Build | `incremental-implementation` | Build one step at a time with verification |
| Build | `test-driven-development` | Failing tests first, then implementation |
| Build | `source-driven-development` | Implementation grounded in source evidence |
| Build | `doubt-driven-development` | Challenge assumptions before proceeding |
| Build | `context-engineering` | Optimize agent context for quality output |
| Build | `api-and-interface-design` | Design contracts before implementation |
| Build | `frontend-ui-engineering` | UI-specific build patterns |
| Verify | `browser-testing-with-devtools` | Browser-based test execution |
| Verify | `debugging-and-error-recovery` | Systematic root-cause diagnosis |
| Review | `code-review-and-quality` | Multi-axis review (correctness/readability/security/perf) |
| Review | `code-simplification` | Reduce complexity without changing behavior |
| Review | `security-and-hardening` | Security review with remediation |
| Review | `performance-optimization` | Measure first, then optimize |
| Ship | `git-workflow-and-versioning` | Branching, commits, tagging |
| Ship | `ci-cd-and-automation` | Pipeline setup and quality gates |
| Ship | `deprecation-and-migration` | Safe removal of old APIs/systems |
| Ship | `documentation-and-adrs` | ADR authoring and doc coverage |
| Ship | `shipping-and-launch` | Final checklist before merge/release |
| (all) | `using-agent-skills` | Meta-skill: discover and invoke the right skill |

---

## Staying current — agent skills sync

A daily GitHub Actions workflow (`sync-agent-skills.yml`) checks
[addyosmani/agent-skills](https://github.com/addyosmani/agent-skills) for upstream drift.
On drift it opens a review PR (`chore/sync-agent-skills-<sha>`) — human review is required
before merge. Auto-merge is disabled.

To manually trigger a sync:
```bash
gh workflow run sync-agent-skills.yml --field force_sync=true
```

Or re-vendor locally:
```bash
./tools/hook-bootstrap/install.sh --sync-skills
```

---

## Offline contributors

If you are offline or behind a corporate proxy that blocks GitHub:

```bash
./tools/hook-bootstrap/install.sh --skip-skills
```

Hooks and CLI wrapper install normally. Agent skills can be vendored later when network
access is available.

---

## Uninstall

```bash
./tools/hook-bootstrap/uninstall.sh
```

Removes hook entries from `.claude/settings.json` and `.codex/hooks.json`. Optionally
removes `tools/agent-skills/` on prompt (preserved by default).

Preview what would be removed:
```bash
./tools/hook-bootstrap/uninstall.sh --dry-run
```

---

## Hook philosophy: encourage, don't prevent

All hooks exit 0. They print advisory suggestions to stderr/stdout; they never block
agent execution. The agent retains full autonomy.

CI gates (`registry/quality/lanes.yaml`) are the enforcement layer. Hooks are guidance
infrastructure delivered at the right moment — like a senior engineer pair-programming
with a junior agent.

Per ADR-0221 (Agentic Pipeline Hardening): hooks are guidance infrastructure, not
enforcement infrastructure.

---

## References

- ADR-0221: Agentic pipeline hardening doctrine
- `memory/feedback_oya_git_canonical_2026_05_18.md` — canonical oya git primitive
- `memory/feedback_oya_vcs_canonical_2026_05_16.md` — superseded rationale retained for history
- `evidence/pr-143-hooks-bootstrap-design-amendment-2026-05-18.json` — encouragement reframe
- Lifecycle skills: [addyosmani/agent-skills](https://github.com/addyosmani/agent-skills) (MIT)
- `tools/agent-skills/UPSTREAM.json` — provenance (SHA, fetch timestamp, attribution)
