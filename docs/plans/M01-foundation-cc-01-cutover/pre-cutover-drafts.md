---
doc_status: published
---


<!--
status: Accepted
date: 2026-05-12
related_adrs: ADR-0052, ADR-0053, ADR-0054, ADR-0055
-->

Drafted 2026-05-12 by deep-dive orchestrator while ralplan consensus runs. These are scratch artifacts that feed the eventual ADRs and runbooks. Not authoritative; pre-approval.

---




### Reproduction

```
  8771 symbols indexed
  155767 dependencies found

and a branch '--' cannot be created from it
```

Reproduces deterministically. Repo is on `main` tracking `origin/main`. Working tree has some untracked + 2 modified files (`.gitignore`, `AGENTS.md`); same error reproduces on a clean tree (verified by running on a stash-saved clean state).

### Diagnosis


```
```

That is, the **source-ref** positional argument is being passed as the literal `--` separator, with nothing after it. `git checkout -b NEW_BRANCH SOURCE_REF` requires `SOURCE_REF` to resolve to a commit; `--` is the end-of-options sentinel and is not a commit, hence the "is not a commit" half of the error. The "and a branch '--' cannot be created from it" half is `git`'s second-pass interpretation: since `--` is not a commit, git tries to read it as a branch name, which also fails because branch names cannot be `--`.



Independent bug, but worth documenting because the diagnostic path crossed it:

```
    .omc/scratch/deep-dive-oyatie-sst-consolidation.md::SPEC
Error: FOREIGN KEY constraint failed
Caused by: Error code 787: Foreign key constraint failed
```

vs. with a real indexed code symbol:

```
    "crates/cloud-billing-app/src/lib.rs::CloudBillingEventIngestAppStatus"
+ Granted:
  > crates/cloud-billing-app/src/lib.rs::CloudBillingEventIngestAppStatus
```

The FK violation looks like a symbol-not-found case being surfaced as a raw sqlite FK error rather than a user-facing "symbol not in index" message. Suggested upstream fix: validate symbol path against the symbols table and return a clean error before attempting the FK insert.

### Suggested upstream changes

1. In session-start, default the source-ref to `HEAD` when no `--source` argument is provided, or document the required argument and improve the error message.
2. In claim, look up the symbol against the symbols table first; return a clean "symbol not in index" error when missing, rather than letting sqlite surface the FK violation.

### Workaround in use here


---




Scaffold-claim sequence:

```
     Cargo.toml::workspace_members
2. <agent creates tools/agent-read/{Cargo.toml, src/lib.rs, src/main.rs}>
3. <agent edits root Cargo.toml to add the new crate to workspace.members>
```




---

## Draft 3 — Parallel-claim demo script (A7 deliverable scaffold)


### Demo scenario


### Demo script

```
# Terminal 1 (agent-A)
  --intent "demo: rename CloudBillingEventIngestAppStatus to CloudBillingIngestStatus" \
  "crates/cloud-billing-app/src/lib.rs::CloudBillingEventIngestAppStatus"
#         + Granted: ...CloudBillingEventIngestAppStatus

# edit crates/cloud-billing-app/src/lib.rs — rename only that one type
# (in real demo: use a no-op edit that just touches the line — keeps the demo replayable)
cd /Users/jasonlee/oyatie
# expect: rebase + merge succeeds; lock released
```

Run terminal 2 IN PARALLEL with terminal 1:

```
# Terminal 2 (agent-B)
  --intent "demo: tweak doc comment on CloudBillingMeterUnitRecord" \
  "crates/cloud-billing-app/src/lib.rs::CloudBillingMeterUnitRecord"
# expect: + Granted (because agent-A's lock is on a different symbol in the same file)

# edit doc comment on CloudBillingMeterUnitRecord only
cd /Users/jasonlee/oyatie
# expect: rebase + merge succeeds; lock released
```

### Capture
- `git log --oneline -5` (run by **human**, not agent — this is a recording-time observation, allowed) showing two merge commits landed without manual conflict resolution.

### Negative case to capture
A third agent (`agent-C`) attempts to claim a symbol already locked by `agent-A`. Expect:

```
  "crates/cloud-billing-app/src/lib.rs::CloudBillingEventIngestAppStatus"
# expect: error or queue (depending on flags); use --queue --wait 10 to demonstrate FIFO
```

### Runbook structure

2. Setup (three terminals, recording on)
3. Steps (the three blocks above)
4. Expected output excerpts (with placeholders for real timestamps)
5. Negative-case run

---

## Draft 4 — `omx ultragoal checkpoint/complete-goals` deprecation notice

Target location: `bominal/agents/ultragoal/DEPRECATED.md` (NEW file alongside the archive directory).

Body (≤30 lines):

```markdown
# DEPRECATED — omx ultragoal checkpoint flow

As of 2026-05-12, `omx ultragoal checkpoint` and `omx ultragoal complete-goals`
glue this flow provided (codex-goal-*.json, ledger.jsonl, goals.json, PAUSE.md,
G004-reconciliation-blocker.md) has been moved to

## Replacement mapping

| Old | New |
|---|---|

## Why

`docs/decisions/specs/deep-dive-oyatie-sst-consolidation.md`.

## Last-known good state

```

---

## Draft 5 — `governance-portfolio-citation` lane logic (A1 scaffold)

The lane validates two invariants:

1. `oyatie/docs/PRD.md` contains a citation block referencing `bominal/docs/consolidated/PRD.md` as portfolio parent.
2. `bominal/docs/consolidated/PRD.md` contains a citation block referencing `oyatie/docs/PRD.md` as canonical implementation home.

Pseudo-implementation (Rust, lives at `crates/governance-portfolio-citation-kernel/`):

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

Lane runner (out-of-kernel) reads the two PRD files, extracts citation blocks via a small markdown-frontmatter parser, calls `verify`, and exits non-zero on incomplete. Lives at `tools/governance-portfolio-citation/`.

---

## Draft 6 — `governance-banned-primitives` lane logic (A5 scaffold)

The lane greps agent-instruction sections of these files for banned tokens:

- `oyatie/CLAUDE.md` (agent-instruction sections only)
- `oyatie/AGENTS.md` (agent-instruction sections only)
- `oyatie/docs/AGENTS.md` (agent-instruction sections only)
- `agents/settings/claude.settings.json` (whole file)
- Any project-level skill prompts under `.claude/skills/` or `.codex/skills/` (if any)


Pseudo-pattern:

```
for f in TARGETED_FILES:
  for section in extract_agent_instruction_sections(f):
    for token in [r"\bgit\b", r"\bgh\b", r"\brtk git\b", r"\brtk gh\b"]:
      if matches(section, token):
        emit_violation(f, section_line, token)
exit_nonzero if any_violation else exit_zero
```

Lane lives at `tools/governance-banned-primitives/`; kernel at `crates/governance-banned-primitives-kernel/` for the matcher logic (pure, deterministic).

---

## Notes

- All six drafts above are planning scratch artifacts, pre-approval. None of them are authoritative until the Planner→Architect→Critic consensus lands and the user approves execution.
- The new-crate chicken-and-egg (Draft 2) is the most architecturally-significant of these drafts — it surfaces a real gap in the spec and adds a small ADR target (ADR-0054).
- The bug report (Draft 1) should be filed upstream once the user approves; until then it's a draft.
