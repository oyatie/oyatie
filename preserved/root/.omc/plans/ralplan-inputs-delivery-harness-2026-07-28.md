# ralplan INPUTS — the reusable delivery harness (2026-07-28)

Assembled for /ralplan. Everything below is MEASURED this session, not assumed.
Goal (founder): a reusable workflow where agents complete work that passes CI
FIRST TIME — CI catches only genuine edge cases, never rediscoverable obligations.
Applies to the reorg AND all future implementation work.

## The three-layer target

| Layer | Catches | Miss cost |
|---|---|---|
| 1. Obligation table (pre-dispatch DATA) | enumerated obligations | zero |
| 2. Local preflight (pre-push) | what the table missed | minutes |
| 3. CI | genuine edge cases only | ~71 min median + round trip |

## Measured evidence

### The table works — 5x measured
ADR-0628's nine projection edits took FIVE discover-fix-rerun cycles (no table).
ADR-0629's identical edits took ONE pass (table known). Same task, same person.

### The preflight engine exists and runs
`buck2 run //ci/facade/affected-target-set:oya-cloud-ci-affected-set-bin --
--policy ci/facade/affected-target-set/affected-set-policy.json --base origin/dev
--head HEAD --derive-only` — verified locally: classified 13 changed files,
printed decision + reasons. Preceded by the two materializer runs.

### Why nobody runs it: escalation to FULL
Any new-gate change escalates to `//...` (55-60 min) because three path classes
have NO buck2 owner and NO synthetic_dependencies declaration:
`Cargo.lock`, `OWNERS`, `registry/catalog/*.yaml`.
THE LEVER: declare synthetic owners for those three classes once; the preflight
collapses to minutes. (affected-set policy already supports synthetic_dependencies
— see memory affected-set-full-tier-root-cause.)

### Round-1 experiment findings (parallel impl + diff-only review + consolidation)
Implementer 1 (adr-index-renderer) FALSIFIED the task premise:
- render_markdown/render_json render FROM decisions.json records, NOT from ADR
  frontmatter. Only the id-SET is grounded against docs/decisions/*.md. The parity
  gate is SELF-REFERENTIAL: title/status/owner/date drift is invisible.
- 245 of 437 ADR files fail the strict corpus parser (MissingRequiredField: date).
  Deriving the index from frontmatter is blocked on a corpus-wide frontmatter fix.
- "Wire emitter, don't de-commit" is contradictory: materializer targets ARE the
  de-committed face set; materializing a committed path IS a de-commit.
New obligation rows surfaced (agent-reported, table-worthy):
- premise says "renderer already produces X" -> probe the LIVE corpus first
- wiring into materializer -> confirm path is in GENERATED_FACE_PATHS first
- before editing -> capture CLEAN-TREE baseline of the affected gate targets
  (live_history_only_retirement_facts_* is expected-red locally without
  --github-event OIDs)
- buck2 test takes NO libtest args (no --nocapture, no name filter)
- probe files -> Write/Edit only; worktree-isolation hook rejects shell redirects
- adr_index_projection_parity only grounds the id-set (self-referential corpus)

### ADR serialization point (measured)
decisions.json + ADR-INDEX.md carry 9 hand-maintained counter fields; two
ADR-adding PRs cannot both be correct. Matrix rows collide textually (append-only,
trivial). Cargo.lock collides on every pair (regenerate, trivial).
True fix is upstream: make docs/decisions/*.md the parse-clean source of truth
(needs the 245-file frontmatter fix) THEN derive. Until then: one ADR-PR in
flight at a time.

## Cross-model delegation (founder directive: codex exec + agy where appropriate)

DeepSWE v1.1 (the designated routing reference; catalog in memory, 18d old,
LIVE-RECONFIRM ids): sol 73%/$8.39 (most capable), terra 70%/$4.95
(capability-per-$), luna 67%/$3.03 (most efficient); terra+luna BEAT
opus-4.8 (59%/$13.22) and sonnet-5 (54%/$26.40) on capability AND cost.
Fable 70%/$21.63 CAPPED AT HIGH (founder).

Live probes this session:
- codex exec -c model=gpt-5.6-luna --sandbox read-only: launched OK (grading pending)
- agy -p headless: auto-DENIES tool permissions without settings.json allow-rules
  -> needs one-time permission provisioning before any delegation use.

LiveBench 2026-06-25 (VERIFIED live pull 2026-07-28, raw CSVs, not the SPA):
global: sol-max 82.4 ($0.485/q) > fable-max 80.8 ($1.271/q) > opus-5-xhigh 80.3
($0.391/q) > terra-max 79.8 ($0.397/q). Category splits that drive routing:
- AGENTIC CODING: terra-max 68.0 BEATS sol-max 65.6; opus-5 ~61;
  **fable-max 46.9 — craters.** Corroborates founder "Fable = cross-ref +
  frontend only" with independent data.
- Pure Coding: fable-max 86.0 tops. Language: fable 90.7 tops. Math: sol 96.2.
- opus-5-high = Claude-family efficiency point: 79.0 global at $0.295/q.
TWO independent benchmarks (DeepSWE repo-tasks, LiveBench question-level) agree.

Candidate routing (to be ratified in plan; each lane needs a GRADED PROBE first
— founder 2026-07-28: experiment and verify, never trust tables blindly):
- terra: DEFAULT agentic implementation lane (best agentic score + cap-per-$)
- sol: hardest reasoning/judge/math stages only
- opus-5-high: in-family review + standard Claude lanes (cheapest strong Claude)
- luna: cheap read-only verification/census (graded probe in flight)
- Fable(High): pure-coding/language/frontend lanes ONLY — 46.9 agentic disqualifies
  it from implementation/review lanes. NOTE: session main loop currently runs
  Fable; orchestration/judgment is fine, but Workflow implementer agents should
  carry an explicit model override rather than inheriting it.
- CROSS-MODEL REVIEW is the high-value slot: same-model reviewers share blind
  spots; terra reviews Claude work at ~1/3 the cost of a second Claude reviewer.
- agy headless: BLOCKED until settings.json permissions.allow provisioning
  (verified: auto-denies command tool in -p mode).

## Worktree provisioning (founder: "set number of worktrees completely aware")

Bun model: N long-lived worktrees, each pre-provisioned with the procedure.
Here: worktree + obligation table + preflight script + clean-tree baseline
captured AT PROVISION TIME (so agents diff against known-good, not re-derive).

## Open decisions for the plan (consensus, not ad-hoc)

1. Obligation table: enforced DATA (gate) vs versioned doc read at dispatch?
2. Preflight: convention in agent prompt vs git hook vs gate-that-checks-it-ran?
3. Order: synthetic_dependencies lever first (small, unblocks preflight adoption)
   vs table-as-data first?
4. Worktree count + lifecycle (long-lived pre-provisioned vs per-task)?
5. ADR pipeline: frontmatter cleanup (245 files) -> derive projections; interim
   rule = one ADR-PR in flight.
6. Where does cross-model delegation slot into the workflow primitives
   (review lane? verification lane? cheap census lane?)
7. The 105 disconnected libs/ kernels + register_crate invocation surface —
   same "logic exists, invocation missing" class; sequence within the harness work?

## Constraints (non-negotiable, from doctrine + session rulings)

- No CLI-shaped automation; materializer-invoked producers or gate-invoked libs.
- No serializer edits to governed JSON; line edits; match file's escape convention.
- ADRs land Proposed. New files need an ADR naming them (born-accounting).
- ADR-0512: structural migration exclusive in the drain; gate work may parallel.
- Every gate: RED fixture + corpus floor + matrix row + catalog row +
  no_autofix_reason (or --fix).
