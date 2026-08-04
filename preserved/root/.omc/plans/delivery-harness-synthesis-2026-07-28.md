# Delivery harness — Bun-informed, measured (2026-07-28)

Input to /ralplan. Everything MEASURED this session. Companion:
`ralplan-inputs-delivery-harness-2026-07-28.md`.

Founder frame: be productive while ensuring quality and engineering excellence.
Classify each friction as PATTERN (keep — it buys quality) or ANTI-PATTERN
(remove — it is accidental cost). CI is a BACKSTOP, not the feedback loop.

## What Bun actually did, and what transfers

| Bun mechanism | Transfers? | Evidence here |
|---|---|---|
| `PORTING.md` — decide once, agents never re-derive | **YES, strongest** | ADR-0628 took 5 discover-fix-rerun cycles; ADR-0629 (identical work, list known) took **1**. Round 1 surfaced **21 more rows** I did not know. |
| 1 implementer → 2 **diff-only** reviewers → 1 fixer | **YES** | Round-1 reviewers caught a **blocker** I had also found independently (circular emitter) plus 10 defects across 3 changes; 6/6 verdicts were REQUEST_CHANGES. |
| 4 worktrees × 16 agents | **PARTLY** | Bun's port was embarrassingly parallel (per-file). Here a global ADR counter serializes ADR-bearing work; everything else parallelises. |
| 24/7 coverage-guided fuzzing | **NOT YET** | 0 fuzz targets repo-wide. Unclosed gap. |

Bun's real lesson is not "many agents." It is **front-load the decisions so agents
never re-derive, then review adversarially on the diff alone.**

## Friction classified — PATTERN vs ANTI-PATTERN

### PATTERN (keep; it buys quality)
- **Born-accounting** (a file exists because an ADR names it). Prevents orphan
  artifacts. Cost is one ADR section. KEEP.
- **Adversarial diff-only review.** 58% of DELETE verdicts overturned earlier;
  round-1 reviewers found a blocker. Highest-value single mechanism. KEEP.
- **RED fixture + corpus floor per gate.** Cheap; the only proof a gate can fail.
- **Shrink-only baselines.** Makes debt visible and directional.
- **Anti-narrowing ratchet** (cannot silently drop a scan root) — but see
  scan-root liveness: it needed a liveness counterpart to stop preserving
  dead roots. Now shipped (ADR-0628 / #1440).

### ANTI-PATTERN (remove or change the pipeline)
1. **CI as the feedback loop.** ~71 min median per discovery. Every obligation I
   learned today came from a red run. FIX: obligation table (layer 1) + local
   preflight (layer 2).
2. **Hand-maintained derived state.** ADR-INDEX.md + decisions.json = 9 hand-edits
   per ADR, and they are a **global serialization point**. The renderers exist
   (private) in `libs/oya-check-adr-index`. FIX: expose + emit. CAVEAT (round-1
   finding): the emitter would be **circular** — records are read from
   decisions.json itself; only the id-SET is grounded against docs/decisions/*.md,
   and 245 of 437 ADR files fail the strict frontmatter parser. So the honest
   sequence is: fix frontmatter corpus → derive from ADR files → de-commit.
3. **Preflight escalates to FULL (55–60 min).** `Cargo.lock`, `OWNERS`,
   `registry/catalog/*.yaml` have no buck2 owner and no `synthetic_dependencies`.
   FIX: declare synthetic owners once → preflight becomes minutes → adoption.
   **This is the single highest-leverage change in the whole harness.**
4. **OMX contamination of the delegation surface.** `~/.codex/config.toml`
   `developer_instructions` hijacked a read-only codex run into an OMX
   state-repair loop that could not terminate (needs write access; sandbox is
   read-only). FIX: `--ignore-user-config` on every delegated invocation.
   ANTI-PATTERN: an orchestration layer that requires write access to answer a
   read-only question.
5. **Invocation surfaces missing for shipped logic.** Three instances:
   architecture-map producer, `register_crate` orchestrator, ADR renderers. Plus
   105 of 111 `libs/` check kernels unreachable from merge authority. The logic is
   written and tested; only the call site is missing. FIX: wire, do not rewrite.
6. **buck2 test ergonomics.** No libtest args (`--nocapture`, name filters
   rejected); multi-target output interleaves so `| tail` hides other targets;
   gate target naming is a convention, not a guarantee. Accept as constraint,
   but ENCODE it in the table (rows exist now).

## Model + agent routing — GRADED, not assumed

Founder rule: experiment and verify; a routing table without a graded probe is a
hypothesis.

| Lane | Model | Evidence |
|---|---|---|
| read-only verification / census | **gpt-5.6-luna** | **GRADED 4/4 exact** on the dead-scan-roots task (known ground truth), 43k tokens ≈ pennies. VALIDATED. |
| agentic implementation (default) | **gpt-5.6-terra** | LiveBench Agentic Coding **68.0 — beats sol 65.6** at $0.397/q; DeepSWE agrees. PROBE PENDING. |
| hardest reasoning / judge | gpt-5.6-sol | LiveBench global 82.4, Math 96.2. PROBE PENDING. |
| in-family review, cheap Claude | claude-opus-5-high | 79.0 global at **$0.295/q** — cheapest strong Claude. |
| pure coding / language / frontend ONLY | claude-fable-5 (High) | Coding 86.0 + Language 90.7 top, but **Agentic Coding 46.9 — craters**. Independently corroborates the standing "never default reviewer" ruling. |
| quick local tasks | agy | **BLOCKED**: headless `-p` auto-denies tool permissions; needs one-time `settings.json` permissions.allow provisioning. |

**Cross-model review is the high-value slot** — a same-model reviewer shares blind
spots with the implementer. terra reviews Claude work at ~1/3 the cost of a second
Claude reviewer.

### Verified codex delegation invocation (5 failed attempts to get here)
```
codex exec -m <model> -s read-only -C <worktree> \
  --skip-git-repo-check --ephemeral --ignore-user-config < /dev/null
```
- `-m`, NOT `-c model=` (the config-override form did not take)
- `-C <worktree>` is REQUIRED — without it, it runs in the caller's cwd
- `--ignore-user-config` is REQUIRED — else OMX hooks hijack the turn
- `< /dev/null` is REQUIRED — else it blocks on "Reading additional input from stdin"
- `-s workspace-write` + `--add-dir` for implementation lanes

### SDKs for programmatic use (workflow integration)
- **Codex SDK** (`@openai/codex-sdk`, TS; `openai_codex`, Python): `new Codex()`,
  `codex.startThread({model, sandbox})`, `thread.run(prompt)`,
  `codex.resumeThread(id)`, `runStreamed()`. Wraps the CLI over JSONL on
  stdin/stdout. Sandbox presets: `read_only` / `workspace_write` / `full_access`.
- **Claude Agent SDK** (TS/Python): `query({prompt, options})` with
  `permissionMode` (`default|acceptEdits|bypassPermissions|plan|dontAsk|auto`),
  programmatic `agents` (each with own prompt/tools/model), `setPermissionMode()`,
  `interrupt()`. Subagents inherit parent permission mode.
- Implication: the Workflow tool's `agent()` can dispatch Claude natively; codex
  lanes go through `codex exec` (or the SDK) as a shell-invoked peer. Keep the
  SAME obligation table + preflight contract for both, so lane choice is an
  economics decision, not a behaviour change.

## ROUND 1 VERDICT: parallelism did NOT pay — and the reason inverts my assumption

10 agents · 1,157,743 tokens · 467 tool calls · 47 min wall (~7.5 agent-hours)
→ **0 of 3 lanes mergeable.** 27 defects; 5 of 6 review axes blocking.

**Consolidation was CHEAP.** Exactly one shared file across lanes
(`oya-ci-required.yml`), hunks 6 unchanged lines apart — wider than git's 3-line
context, so it auto-merges. Total integration cost < 30 min. I had assumed shared
files would be the bottleneck. They were not.

**Parallelism failed on OUTPUT, not integration.** Three cost buckets, measured:
1. **Triplicated discovery ~65 min** — all three lanes independently rediscovered
   the SAME materializer defect, with no channel to warn each other.
2. **2 of 3 briefs were factually WRONG.** Each lane burned 20–50 min disproving
   its own brief in isolation, then shipped anyway.
3. **No abort path.** Both refuted lanes pushed REQUEST_CHANGES branches instead
   of stopping. An unrecoverable brief produced a branch, not a signal.

### Harness defects found (all fixable, all specific)
- **Self-reported shared-file lists are unreliable in BOTH directions.**
  architecture-map declared 3 files, touched **14**, and declared `Cargo.lock`
  which it never modified. The declaration concealed the round's only real
  collision. FIX: derive from `git status --porcelain` per worktree, never trust
  self-report.
- **A lane can be mechanically self-inconsistent and still consume 2 reviews.**
  architecture-map added 3 Cargo.toml dependency edges with a CLEAN Cargo.lock —
  catchable in seconds, it would red the very freshness gate it was renaming.
- **True chokepoints exist and are narrow:** `gate-baseline.signoff.json`
  exemptions are explicitly ONE-REGEN (two lanes cannot both spend one);
  length-encoded const arrays (`[&str; 7]`) are two-place edits inside one line;
  `gate-baseline.generated.json` is merge-base-anchored so each merge shifts the
  remaining lanes' baseline.
- **Two exclusion lists already disagree** (gate_registration.rs vs
  gate-self-conformance policy).

### Round-2 harness changes (from the consolidation analysis)
1. **Falsifiable premises at dispatch** — every "X already does Y" claim carries
   ONE runnable check (parse-rate, byte-parity diff, `buck2 targets` listing).
2. **Premise-failure ABORT path** — "brief refuted, here is the evidence" is a
   valid terminal state that does NOT produce a branch.
3. **Mid-flight friction broadcast** — shared append-only friction file read
   before each lane's first gate run; converts 2nd/3rd rediscovery into a lookup.
   (SendMessage already exists for this.)
4. **Derived shared-file lists** via `git status --porcelain`.
5. **Cheap mechanical self-check before review** (`cargo metadata --locked`, own
   freshness gate) — never spend two reviewers on a self-inconsistent branch.
6. **Serialize only true chokepoints**, parallelize the rest.
7. **Fixed merge order, smallest CI cone first.**
8. **Affected-set check as a dispatch PRECONDITION** — grep every path the lane
   will touch against `affected-set-policy.json`; unmapped ⇒ FULL 55–60 min tier.

**Verdict on the model:** Bun's 16-agent fan-out works when tasks are
independent AND briefs are correct. Our briefs were the failure. Fix dispatch
quality before adding agents — more parallelism would have multiplied the waste,
not the output.

## The harness (proposed; /ralplan to ratify)

```
PROVISION   N long-lived worktrees. Each carries:
            - the obligation table (versioned DATA, in-repo)
            - the preflight script
            - a CLEAN-TREE baseline of the affected gate targets
DISPATCH    task + table + destination + model lane (per routing above)
IMPLEMENT   worktree-isolated; agent runs preflight itself
REVIEW      2 diff-only reviewers, at least one CROSS-MODEL
FIX         single fixer applies review findings
PREFLIGHT   materialize faces → affected-set → run derived targets
LAND        CI = backstop only
HARVEST     every friction the table missed becomes a new table row
```

The HARVEST step is what makes it compound: round 1 produced **21 new rows**.

## Sequenced next actions

1. **Pull the `synthetic_dependencies` lever** (Cargo.lock / OWNERS /
   registry/catalog) — makes preflight cheap; unblocks the whole convention.
2. **Author the obligation table as in-repo DATA** (11 known + 21 harvested).
3. **Graded probes for terra and sol** before adopting their lanes.
4. **agy permission provisioning**, or drop agy from the harness.
5. ADR pipeline: frontmatter corpus fix → derive → de-commit. Interim: one
   ADR-bearing PR in flight.
6. Wire the 3 missing invocation surfaces + the 105 stranded kernels.
