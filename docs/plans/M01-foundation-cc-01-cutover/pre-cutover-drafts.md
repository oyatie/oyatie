---
doc_status: published
---

# Pre-cutover drafts — oyatie SoT + grit/icm

<!--
status: Accepted
date: 2026-05-12
related_adrs: ADR-0052, ADR-0053, ADR-0054, ADR-0055
-->

Drafted 2026-05-12 by deep-dive orchestrator while ralplan consensus runs. These are scratch artifacts that feed the eventual ADRs and runbooks. Not authoritative; pre-approval.

---

## Draft 1 — Upstream grit bug report (P9 deliverable)

Title: `grit session start fails: "git checkout -b: <branch> is not a commit and a branch '--' cannot be created from it"`

Repository: `rtk-ai/grit`
Version: 0.3.0 (binary at `~/.cargo/bin/grit`, macOS Darwin 25.4.0)
Severity: high — blocks `grit session start` entirely; downstream blocks `grit session pr` and `grit session end`. Workaround exists (session-less mode using `grit claim` directly), but loses the session-PR primitive.

### Reproduction

```
$ grit init
grit initialized
  8771 symbols indexed
  155767 dependencies found

$ grit session start oyatie-cutover
Error: git checkout -b failed: fatal: 'grit/oyatie-cutover' is not a commit
and a branch '--' cannot be created from it
```

Reproduces deterministically. Repo is on `main` tracking `origin/main`. Working tree has some untracked + 2 modified files (`.gitignore`, `AGENTS.md`); same error reproduces on a clean tree (verified by running on a stash-saved clean state).

### Diagnosis

The error message shape suggests grit is invoking something like:

```
git checkout -b grit/oyatie-cutover --
```

That is, the **source-ref** positional argument is being passed as the literal `--` separator, with nothing after it. `git checkout -b NEW_BRANCH SOURCE_REF` requires `SOURCE_REF` to resolve to a commit; `--` is the end-of-options sentinel and is not a commit, hence the "is not a commit" half of the error. The "and a branch '--' cannot be created from it" half is `git`'s second-pass interpretation: since `--` is not a commit, git tries to read it as a branch name, which also fails because branch names cannot be `--`.

Likely root cause: the session-start command path in grit upstream is missing a default for the source-ref argument when the user does not supply one. The expected default is `HEAD` (or the configured base branch).

### Related symptom — FK on `grit claim` with non-code symbol

Independent bug, but worth documenting because the diagnostic path crossed it:

```
$ grit claim --agent dd-orchestrator --intent "test" \
    .omc/scratch/deep-dive-oyatie-sst-consolidation.md::SPEC
Error: FOREIGN KEY constraint failed
Caused by: Error code 787: Foreign key constraint failed
```

vs. with a real indexed code symbol:

```
$ grit claim --agent test-agent --intent "test claim with real symbol" \
    "crates/oya-cloud-billing-app/src/lib.rs::CloudBillingEventIngestAppStatus"
+ Worktree: /Users/jasonlee/oyatie/.grit/worktrees/test-agent
+ Granted:
  > crates/oya-cloud-billing-app/src/lib.rs::CloudBillingEventIngestAppStatus
```

The FK violation looks like a symbol-not-found case being surfaced as a raw sqlite FK error rather than a user-facing "symbol not in index" message. Suggested upstream fix: validate symbol path against the symbols table and return a clean error before attempting the FK insert.

### Suggested upstream changes

1. In session-start, default the source-ref to `HEAD` when no `--source` argument is provided, or document the required argument and improve the error message.
2. In claim, look up the symbol against the symbols table first; return a clean "symbol not in index" error when missing, rather than letting sqlite surface the FK violation.

### Workaround in use here

Operating in **session-less mode**: agents claim symbols directly on the orchestrator's current base branch. grit auto-creates `.grit/worktrees/<agent>/`. `grit done --agent <agent>` lands the worktree back to the base. The session-PR primitive (`grit session pr`) is deferred until the upstream fix.

---

## Draft 2 — New-crate creation policy (resolves chicken-and-egg with grit)

**Problem.** `grit claim` requires a real indexed code symbol (`<file>::<Identifier>`). A *new* crate (e.g., `tools/oya-agent-read/`) has no source files yet, hence no indexed symbols, hence cannot be locked via `grit claim`. The cutover plan creates new crates (A4, the helper CLI) — this is a real gap.

**Resolution.** Introduce a "scaffold claim" pattern that locks the **workspace-level coordination point** (`Cargo.toml::workspace.members`) for the duration of crate creation, plus the **parent directory** of the new crate for the duration of initial scaffolding. Once initial files land and grit re-indexes (or after the next `grit init` refresh), normal symbol-level claims take over.

Scaffold-claim sequence:

```
1. grit claim --agent <id> --intent "scaffold tools/oya-agent-read crate" \
     Cargo.toml::workspace_members
2. <agent creates tools/oya-agent-read/{Cargo.toml, src/lib.rs, src/main.rs}>
3. <agent edits root Cargo.toml to add the new crate to workspace.members>
4. grit done --agent <id>          # lands the scaffold to base
5. grit init                       # re-index so the new crate's symbols are claimable
6. <subsequent agents use normal grit claim against the new symbols>
```

The `Cargo.toml::workspace_members` symbol must be indexed by grit (verify before relying on it; if not, the scaffold-claim coordination point is `Cargo.toml` at the file level, which today is not lockable — escalate as a grit feature request for file-level locks).

**Mitigation if `Cargo.toml::workspace_members` is not indexable.** Use **icm as the coordination lock** during the scaffold window: `icm store -t scaffold-locks-oyatie -c "agent=<id> path=tools/oya-agent-read window=open" -i critical` before scaffolding, then `-c "agent=<id> path=tools/oya-agent-read window=closed"` after. Other agents `icm recall -t scaffold-locks-oyatie` and back off if any window is open against an overlapping path. Slower; correct as a fallback.

This pattern lands as ADR-0054: `ADR-0054-grit-scaffold-claim-pattern.md`.

---

## Draft 3 — Parallel-claim demo script (A7 deliverable scaffold)

Target location for the recorded runbook: `oyatie/docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.md`.

### Demo scenario

Two agents (`agent-A`, `agent-B`) work in parallel on non-overlapping symbols inside the same crate. They both land via `grit done`. No conflict.

### Demo script

```
# Terminal 1 (agent-A)
grit claim --agent agent-A \
  --intent "demo: rename CloudBillingEventIngestAppStatus to CloudBillingIngestStatus" \
  "crates/oya-cloud-billing-app/src/lib.rs::CloudBillingEventIngestAppStatus"
# expect: + Worktree: .grit/worktrees/agent-A
#         + Granted: ...CloudBillingEventIngestAppStatus

cd .grit/worktrees/agent-A
# edit crates/oya-cloud-billing-app/src/lib.rs — rename only that one type
# (in real demo: use a no-op edit that just touches the line — keeps the demo replayable)
cd /Users/jasonlee/oyatie
grit done --agent agent-A
# expect: rebase + merge succeeds; lock released
```

Run terminal 2 IN PARALLEL with terminal 1:

```
# Terminal 2 (agent-B)
grit claim --agent agent-B \
  --intent "demo: tweak doc comment on CloudBillingMeterUnitRecord" \
  "crates/oya-cloud-billing-app/src/lib.rs::CloudBillingMeterUnitRecord"
# expect: + Granted (because agent-A's lock is on a different symbol in the same file)

cd .grit/worktrees/agent-B
# edit doc comment on CloudBillingMeterUnitRecord only
cd /Users/jasonlee/oyatie
grit done --agent agent-B
# expect: rebase + merge succeeds; lock released
```

### Capture
- Timestamps from each terminal for each `grit` invocation.
- `grit watch` output in a third terminal showing `claim_granted`, `done`, `lock_released` events for both agents in interleaved order.
- `git log --oneline -5` (run by **human**, not agent — this is a recording-time observation, allowed) showing two merge commits landed without manual conflict resolution.

### Negative case to capture
A third agent (`agent-C`) attempts to claim a symbol already locked by `agent-A`. Expect:

```
grit claim --agent agent-C --intent "..." \
  "crates/oya-cloud-billing-app/src/lib.rs::CloudBillingEventIngestAppStatus"
# expect: error or queue (depending on flags); use --queue --wait 10 to demonstrate FIFO
```

### Runbook structure

1. Prerequisites (grit installed, repo on `main`, `grit init` has run, no active locks)
2. Setup (three terminals, recording on)
3. Steps (the three blocks above)
4. Expected output excerpts (with placeholders for real timestamps)
5. Negative-case run
6. Cleanup (`grit release --agent agent-A` etc., revert any demo edits)
7. Glossary of grit terms used

---

## Draft 4 — `omx ultragoal checkpoint/complete-goals` deprecation notice

Target location: `bominal/agents/ultragoal/DEPRECATED.md` (NEW file alongside the archive directory).

Body (≤30 lines):

```markdown
# DEPRECATED — omx ultragoal checkpoint flow

As of 2026-05-12, `omx ultragoal checkpoint` and `omx ultragoal complete-goals`
are retired. The agentic pipeline is now `grit` + `icm` only; the orchestration
glue this flow provided (codex-goal-*.json, ledger.jsonl, goals.json, PAUSE.md,
G004-reconciliation-blocker.md) has been moved to
`archive/pre-grit-cutover-2026-05-12/` and removed from the active path.

## Replacement mapping

| Old | New |
|---|---|
| `omx ultragoal checkpoint --goal-id <id> --status complete` | `grit done --agent <id>` + `icm store -t context-oyatie -c "..." -i high` |
| `omx ultragoal complete-goals` | series of per-agent `grit done` invocations |
| `codex-goal-<id>-active.json` (intent record) | `grit claim --agent <id> --intent "..."` (intent lives in the lock) |
| `ledger.jsonl` (action log) | `grit watch` event stream (real-time) + `icm recall -t context-oyatie` (historical) |
| `goals.json` (goal manifest) | `icm store -t goals-oyatie` + plan files under `docs/plans/` |
| `G004-reconciliation-blocker.md` (omx mismatch failure record) | does not exist under grit — no objective-state to mismatch |
| `PAUSE.md` | no equivalent; agents halt via `grit release` or TTL expiry |

## Why

See ADR-0053-grit-icm-as-sanctioned-primitives.md and the spec at
`docs/decisions/specs/deep-dive-oyatie-sst-consolidation.md`.

## Last-known good state

`archive/pre-grit-cutover-2026-05-12/` preserves the deprecated artifacts.
```

---

## Draft 5 — `oya-governance-portfolio-citation` lane logic (A1 scaffold)

The lane validates two invariants:

1. `oyatie/docs/PRD.md` contains a citation block referencing `bominal/docs/consolidated/PRD.md` as portfolio parent.
2. `bominal/docs/consolidated/PRD.md` contains a citation block referencing `oyatie/docs/PRD.md` as canonical implementation home.

Pseudo-implementation (Rust, lives at `crates/oya-governance-portfolio-citation-kernel/`):

```rust
//! Foundry portfolio-citation fitness lane.
//!
//! Asserts the bidirectional citation between oyatie/docs/PRD.md and
//! bominal/docs/consolidated/PRD.md. Pure value-object kernel; the lane
//! runner pulls the two files and feeds the kernel the citation blocks.

pub struct CitationBlock {
    pub target_path: String,    // e.g. "bominal/docs/consolidated/PRD.md"
    pub role: CitationRole,     // PortfolioParent | CanonicalImplHome
    pub anchor: Option<String>, // optional anchor within the target
}

pub enum CitationRole {
    PortfolioParent,
    CanonicalImplHome,
}

pub struct PortfolioCitationVerdict {
    pub oyatie_cites_bominal: bool,
    pub bominal_cites_oyatie: bool,
}

impl PortfolioCitationVerdict {
    pub fn is_complete(&self) -> bool {
        self.oyatie_cites_bominal && self.bominal_cites_oyatie
    }
}

pub fn verify(
    oyatie_prd_citations: &[CitationBlock],
    bominal_prd_citations: &[CitationBlock],
) -> PortfolioCitationVerdict {
    let oyatie_cites_bominal = oyatie_prd_citations.iter().any(|c|
        matches!(c.role, CitationRole::PortfolioParent)
            && c.target_path == "bominal/docs/consolidated/PRD.md"
    );
    let bominal_cites_oyatie = bominal_prd_citations.iter().any(|c|
        matches!(c.role, CitationRole::CanonicalImplHome)
            && c.target_path == "oyatie/docs/PRD.md"
    );
    PortfolioCitationVerdict { oyatie_cites_bominal, bominal_cites_oyatie }
}
```

Lane runner (out-of-kernel) reads the two PRD files, extracts citation blocks via a small markdown-frontmatter parser, calls `verify`, and exits non-zero on incomplete. Lives at `tools/oya-governance-portfolio-citation/`.

---

## Draft 6 — `oya-governance-banned-primitives` lane logic (A5 scaffold)

The lane greps agent-instruction sections of these files for banned tokens:

- `oyatie/CLAUDE.md` (agent-instruction sections only)
- `oyatie/AGENTS.md` (agent-instruction sections only)
- `oyatie/docs/AGENTS.md` (agent-instruction sections only)
- `agents/settings/claude.settings.json` (whole file)
- Any project-level skill prompts under `.claude/skills/` or `.codex/skills/` (if any)

Banned tokens (in agent-instruction context only): `git`, `gh`, `rtk git`, `rtk gh`. Human-terminal RTK usage docs are NOT banned; the lane scopes its grep to `<!-- agent-instructions:start -->` / `<!-- agent-instructions:end -->` HTML comment fences (introduced as part of A5 rewrites).

Pseudo-pattern:

```
for f in TARGETED_FILES:
  for section in extract_agent_instruction_sections(f):
    for token in [r"\bgit\b", r"\bgh\b", r"\brtk git\b", r"\brtk gh\b"]:
      if matches(section, token):
        emit_violation(f, section_line, token)
exit_nonzero if any_violation else exit_zero
```

Lane lives at `tools/oya-governance-banned-primitives/`; kernel at `crates/oya-governance-banned-primitives-kernel/` for the matcher logic (pure, deterministic).

---

## Notes

- All six drafts above are planning scratch artifacts, pre-approval. None of them are authoritative until the Planner→Architect→Critic consensus lands and the user approves execution.
- The new-crate chicken-and-egg (Draft 2) is the most architecturally-significant of these drafts — it surfaces a real gap in the spec and adds a small ADR target (ADR-0054).
- The bug report (Draft 1) should be filed upstream once the user approves; until then it's a draft.
