# Pipeline Engineer — First Week on `foundry`

Audience: an engineer with merge-queue / CI / Bors / Spr / GitHub Merge Queue experience joining the `oya-foundry-*` lane.
NOTE 2026-05-21: This onboarding doc is HISTORICAL. The `foundry` µservice is RETIRED per ADR-0335 (Wave 15I); AI substrate absorbed into `microservices/intelligence/`. The agentic-pipeline doctrine (changeset state, admission gate, merge queue, completion gate, webhook-driven invocation, VCS orchestrator) lives in ADRs 0110, 0111, 0112, 0113, 0116, 0247, 0255 and is implemented across vcs-orchestrator + intelligence + workflow + audit-chain + observability + identity + tenancy + policy-engine. The "Hermes" name is RETIRED corpus-wide per ADR-0247 D-10 + ADR-0328 D-9.22 + ADR-0335 D-26..D-36. The substantive walkthrough below is preserved for historical reference; for live authority cite `microservices/intelligence/manifest.json` plus the listed ADRs.

Foundry was the internal agentic development pipeline; engineers operated it, they did not build product on top of it.

## Day 1 — read everything

- `docs/AGENTS.md` — operating contract for agents using the pipeline.
- `docs/decisions/ADR-0110-changeset-state-machine.md`
- `docs/decisions/ADR-0111-merge-queue-projected-state-fix-at-any-stage.md`
- `docs/decisions/ADR-0112-webhook-driven-foundry-agent-invocation.md`
- `docs/decisions/ADR-0113-vcs-orchestrator-end-to-end.md`
- `docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md`
- `docs/decisions/ADR-0136-amendment-foundry-not-consumer-facing.md`
- `docs/decisions/ADR-0220-per-agent-claim-isolation.md`
- `docs/decisions/ADR-0247-self-modification.md`
- `feedback_pipeline_clog_gotchas_2026_05_17.md` (in user MEMORY) — 23 real clog gotchas.

You are not allowed to touch Foundry production until you have read all of the above. The clog gotchas list in particular is the
distilled wisdom of "what goes wrong in production" — re-read whenever you propose a structural change.

## Day 2 — claim → work → done → verify → promote walkthrough

Local pipeline:
```bash
make dev-cell.up CELL=foundry-loopback-1 PROFILE=foundry-dev
```

In an isolated worktree, walk one cycle:
```bash
./bin/oya vcs claim \
  --agent pipeline-eng-$USER \
  --intent foundry-onboarding-week1 \
  docs/onboarding-scratch

# create a trivial change
echo "$USER onboarding 2026-05-20" > docs/onboarding-scratch/$USER.md

./bin/oya vcs verify --agent pipeline-eng-$USER --evidence "trivial:true" docs/onboarding-scratch
./bin/oya vcs done    --agent pipeline-eng-$USER --evidence "trivial:true" docs/onboarding-scratch
./bin/oya vcs promote --agent pipeline-eng-$USER \
  --bundle pipeline-eng-week1-onboarding \
  --environment dev \
  --evidence "trivial:true" \
  docs/onboarding-scratch
```

Each step writes audit-chain entries. Inspect:
```bash
./bin/oya audit query --agent pipeline-eng-$USER --window 1h
```

## Day 3 — read the merge queue implementation

Open `crates/oya-foundry-merge-queue/src/queue.rs`. The function `Queue::admit(pr)` is the canonical entry point. Trace through:
1. **Projected state computation** — rebases the PR onto current `target` (in-memory) and runs validators against the projection.
2. **Fence token** — assigns a per-base-branch fence token; admitting PRs out of fence order is illegal.
3. **Validator pipeline** — runs the `lean-a*` lanes + multispectrum facets per tier.
4. **Reviewer-agent loop** — fires `oyatie.foundry.reviewer-*` principals; gathers verdicts; consensus per ADR-0111.
5. **Merge** — fast-forwards target if projected state holds; otherwise re-projects and retries.

Read `crates/oya-foundry-coordinator/src/coordinator.rs` to see how cluster-wide caps + per-agent caps are enforced.

## Day 4 — diagnose a clog (drill)

Spin up a synthetic clog:
```bash
./bin/oya foundry drill clog \
  --scenario shared-crate-cascade \
  --agents 5 \
  --target-branch dev
```

You will see 5 PRs in-flight, all targeting the same shared crate. Without the coordinator's shared-crate lock, the queue would
re-rebase O(N²) times. With the lock, the queue serializes the shared-crate PRs and parallelizes everything else.

Verify the coordinator behavior:
```bash
./bin/oya foundry status --window 10m
```
Look for `coordinator.shared_crate_locks` ≥ 1 and `queue.rebases_per_pr` ≤ 2.

## Day 5 — ship a real change through Foundry

Pick a starter ticket. Claim, implement, verify, done. Open PR through the Foundry-managed `gh pr create`:
```bash
gh pr create --base dev --title "foundry: pipeline-eng-$USER starter"
```

Watch the admission gate progress through the GitHub Check Runs panel. The full multispectrum verdict appears as a check run named
`reviewer-agent / multispectrum-v2.4.0`.

## Done with week 1

- [ ] You completed the claim→work→done→verify→promote loop end-to-end at least 3 times.
- [ ] You can name the 5 stages of `Queue::admit(pr)` from memory.
- [ ] You ran the clog drill and saw the coordinator prevent cascade.
- [ ] You shipped at least one real PR through Foundry to `dev`.
- [ ] You've internalised the 23 clog gotchas — quiz yourself.

## Rookie traps (most-common)

1. **Bulk-resolving Codex P2 threads.** Per `feedback_codex_bulk_resolve_antipattern.md`: never. Every P2 is real. Sweep is REPORT-ONLY.
2. **Bypassing the merge queue.** Direct push to `dev` is blocked by the GitHub branch protection; emergency override requires the
   `oyatie.governance.break-glass-operator.*` principal.
3. **Forgetting to claim.** Edits without an active claim trigger `lean-a11-claim-discipline` lane failure.
4. **Over-broad positional scopes.** Claim narrowly — overlapping claims block other agents.
5. **--no-verify on commits.** Hooks fail for a reason; never skip.
6. **Cross-shard claims without governance lock.** Read ADR-0111 §C-3.
