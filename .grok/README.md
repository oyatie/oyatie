# Multi-model delivery harness (fine-tunable)

Reusable Grok-native control plane for multi-agent delivery work.

**Does not use** `gjc`, `omc`, `omx`, or `hermes` CLIs. Ideas are incorporated; implementations live under `.grok/`.

**Not merge authority.** PR target `dev`; admission is reviewer APPROVE + `oya-ci-required` (ADR-0515).

## Authority substrate (P0)

The kit **must live on `origin/dev`** (tracked `.grok/` bins + harness + roles).  
`mm-runs/` and `memory/` stay gitignored. Side-branch-only or laptop-only kit is **F-KIT-NOT-ON-DEV**.

Worktrees: run `.grok/bin/mm-bootstrap` if kit missing before multi-model claims.

**Process assessment (2026-08-06):** `programs/PROCESS-ASSESSMENT-20260806.md`  
Dual-critic independence rules: `harness/safety.md` + `harness/drive.v1.json` (`require_cross_model_critics`).

Same-family session subagents may **inform** but must set `independence: same_family` and **must not** clear merge-check when cross-model is required.

## Intent lifecycle (prompt submit → automatic path)

**Doctrine:** `programs/INTENT-LIFECYCLE.md`

```text
prompt submit → CAPTURE_INTENT → PROMPT_HARDEN → DECOMPOSE → DISPATCH
             → PREFLIGHT → CONTRACT → PLAN → dual CRITIC → EXECUTE → …
             → SCORE_GRADE → LEARN → POSTMORTEM (self-improve)
```

| Stage | Schema | Role / action |
|-------|--------|----------------|
| CAPTURE_INTENT | `intent.v1` | harness (raw only) |
| PROMPT_HARDEN | `hardened_prompt.v1` | `PROMPT_HARDENER` via `mm-role` |
| DECOMPOSE | `work_graph.v1` | `PLANNER` |
| DISPATCH | dispatch ledger | harness (path-overlap fail closed) |

`pipeline.json` `entry.forbid_raw_execute=true`: free-form implement from raw chat is a **process defect**.

## Implement lifecycle (mechanical TDD)

**Doctrine:** `programs/IMPLEMENT-LIFECYCLE.md`

```text
RED_TEST → IMPLEMENT → GREEN_TEST → INTEGRATION_TEST → FALSE_GREEN_SCAN
→ REVIEW_DIFF → SIMPLIFY → HARDEN → VERIFY → ADMIT_SLICE
```

Fail closed: no implement without red proof; no skipped/deleted tests; false-green scan; dual review; simplify then harden; re-run green after each.

### Soft reds / blocks → work queue

```bash
.grok/bin/mm-queue-ingest --cwd . --base dev   # upsert soft_red, ci_red, block lanes
.grok/bin/mm-fabric-status --cwd .             # includes soft_red_ready counts
```

Soft CI legs must **not** be ignored: they become `task-board.v1.json` lanes until cleared.

### Autonomy

Agent dual-critic **APPROVE** + `oya-ci-required` green → `mm-drive merge` (human GH APPROVE not required).  
Non-APPROVE → fix until APPROVE. Human supervises only.

### Runner commands (`bin/mm-pipeline`)

```bash
mm-pipeline start --objective "…" --risk medium   # CAPTURE_INTENT + PREFLIGHT
mm-pipeline role --run-id ID --role PROMPT_HARDENER --stage PROMPT_HARDEN
mm-pipeline role --run-id ID --role PLANNER --stage DECOMPOSE
mm-pipeline dispatch --run-id ID                  # path-overlap fail closed
mm-pipeline role --run-id ID --role TEST_ENGINEER --stage RED_TEST
mm-pipeline role --run-id ID --role EXECUTOR --stage IMPLEMENT   # blocked without red proof
mm-pipeline false-green-scan --run-id ID
mm-pipeline admit-slice --run-id ID
mm-pipeline admit-dual-critic --packet path/to/dual-critic.json  # rejects same_family
mm-pipeline close-run --run-id ID                 # score→grade→evaluate→learn
```

## Ideas absorbed (not vendored)

| Source | What we took |
|--------|----------------|
| [Bun rewrite](https://bun.com/blog/bun-in-rust) | Prep contract, dual adversarial split context, **edit the process** (harness/workflow/tool so agents cannot repeat the mistake — not one-off rebases or chat memory), fail closed, trial before fan-out. See `programs/hyperscaler-delivery-lanes/BUN-PARALLEL-DISCIPLINE.md` |
| [gaebal-gajae archive](https://blog.gaebal-gajae.dev/archive.html) | Lanes: Daily Reflection / Setup Tip / Behind; verified empty is work; judgment weight before speed |
| [GJC ultragoal](https://github.com/Yeachan-Heo/gajae-code) ideas only | Durable `goals.json` + `ledger.jsonl` + quality-gated checkpoints (`mm-goals`) |
| [Hermes Agent](https://github.com/nousresearch/hermes-agent) ideas only | Closed learning loop, skill drafts, memory, trajectories, toolset allowlists (`mm-learn`) |
| Oyatie AGENTS.md | 16 reasoning lenses as stage packs + orthogonal critic perspectives |

## Fine-tune knobs (edit these)

| File | Purpose |
|------|---------|
| `harness/lenses.v1.json` | 16 lens briefs / anti-patterns |
| `harness/stage-lens-packs.v1.json` | Stage → pack + orthogonal perspectives |
| `harness/model-routing.v1.json` | Role → model priors + complexity rules |
| `harness/learning-loop.v1.json` | When to write tips/skills/reflections |
| `harness/rubrics/run-grade.v1.json` | Score weights and letter bands |
| `harness/pipeline.json` | Stage DAG |
| `multi-model-roles.json` | CLI role system prompts / sandboxes |
| `workflows/*.rhai` | Multi-agent orchestration shape |

## Layout

```text
.grok/
  bin/           mm-preflight mm-role mm-pipeline mm-score mm-grade
                 mm-goals mm-lens-prompt mm-learn
  harness/       configs + schemas + rubrics
  workflows/     Grok Build Rhai workflows
  memory/        reflections/ tips/ behind/ trajectories/  (gitignored journals ok)
  skills/        learned procedural skills (drafts)
  mm-runs/       per-run journals (gitignored)
```

## Tasklist maintenance

| Layer | Command / artifact | Maintain how |
|-------|-------------------|--------------|
| Durable goals | `mm-goals create --brief-file …` | `goals.json` + `ledger.jsonl` |
| Activate slice | `mm-goals activate --goal-id G001 --lens-pack a,b` | status transitions |
| Checkpoint | `mm-goals checkpoint --status complete --evidence … --quality-gate-json …` | fail closed if gate not pass |
| Next work | `mm-goals next --run-id …` | pending → active |
| Portfolio (optional) | `bd` in Oyatie monorepo | separate from harness; not required |

Brief format (GJC-inspired, no CLI):

```text
Shared constraints…

@goal: Harden merge_group review path
Ensure pr-reviewer-evidence does not no-op on merge_group.

@goal: Document residual risk
Capture rollback and non-goals for the PR packet.
```

## Lens + model routing

```bash
# Render critic A pack for CRITIC_PLAN
.grok/bin/mm-lens-prompt --stage CRITIC_PLAN --perspective critic_a

# Defaults: Critic A = cartesian+red_team; Critic B = contrarian+systems
# Change in stage-lens-packs.v1.json — workflow prompts should stay in sync or call this helper.
```

## Workflows

| Workflow | What it does |
|----------|----------------|
| `lens-delivery-plan` | Contract → Plan → **orthogonal dual critics** → Synthesize (plan-only) |

Run (Grok Build):

```text
/workflow lens-delivery-plan
args: { "objective": "…", "focus": "false_green", "risk": "high" }
```

Or via tool: workflow name `lens-delivery-plan`.

## Score → grade → learn (close the loop)

```bash
.grok/bin/mm-pipeline start --objective "…" --risk medium
# … stages / roles …

.grok/bin/mm-score --run-id <id>
.grok/bin/mm-grade --run-id <id>
.grok/bin/mm-learn from-run --run-id <id>
```

Outputs:

- `scorecard.json` / `grade.json` / `process_edits.md`
- `memory/reflections/YYYY-MM-DD.md`
- `memory/tips/<class>.md` on process debt
- optional `skills/<name>/SKILL.md` draft
- `memory/trajectories/<run_id>.md`

## Multi-model (optional CLI workers)

```bash
.grok/bin/mm-role ARCHITECT --dry-run -- "…"
.grok/bin/mm-role PLANNER -- "…"
.grok/bin/mm-role CRITIC -- "…"
```

Grok workflows use the **session model** with **lens packs** for independence. External models attach via `mm-role` when you want Fable/Opus/Codex without GJC/OMC.

## Baseline rule

Always implement from `origin/dev` (behind=0). PREFLIGHT fails closed otherwise.

```bash
git fetch origin dev
git worktree add -b agent/mm-harness-YYYYMMDD /path origin/dev
```

## Continuous evaluation (not one-shot)

The workflow is **continuously evaluated** across:

| Axis | What we measure |
|------|-----------------|
| Concurrent throughput | Wall clock, parallel critics, serial fraction, path-overlap |
| Outcome quality | Grade, dual admit, hard fails, evidence |
| Delivery pipeline fit | origin/dev preflight, required CI context, no CLI merge authority |
| Prompt engineering | Lens packs, tool-use commands, orthogonal critics |
| Hyperscaler design | Blast radius, zero-trust, telemetry, day-2, finops |
| Portability | Bootstrap + project-profile for other repos (e.g. console) |
| Lifecycle fidelity | Real dev lifecycle ↔ stages (see LIFECYCLE-AND-GAPS.md) |

```bash
.grok/bin/mm-evaluate static --cwd .
.grok/bin/mm-evaluate full --run-id <id> --cwd .
```

Config: `harness/evaluation.v1.json` · narrative: `harness/LIFECYCLE-AND-GAPS.md`

### Port to another project

```bash
.grok/bin/mm-bootstrap ~/Developer/console
# edit ~/Developer/console/.grok/harness/project-profile.v1.json
```

### Where it fails / close-gap

Tracked as `known_failures` in `evaluation.v1.json` (throughput serial, prompt drift, no live CI poll, learn not auto-promoted, …). Priority order is machine-readable; close gaps by editing packs/pipeline and re-running `mm-evaluate`.

## Forbidden

- `gjc`, `omc`, `omx`, `hermes` as orchestration dependency
- Hand-edit `*.generated.json`
- Skip tests to go green
- Claim merge readiness without project required status context
