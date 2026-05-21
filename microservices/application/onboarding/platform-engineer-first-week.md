# Platform Engineer — First Week on `application`

Audience: a platform / SRE engineer joining the `oya-application-*` lane. Goal: by EOD Friday you can claim a real ticket, ship a PR through Foundry,
and run a tenant-scoped dispatch experiment in a dev cell. No prior Oyatie knowledge assumed; Rust + Kubernetes + Linux fundamentals are.

## Day 1 — orient and clone

1. **Read the contract before any tooling.** Open in order:
   - `docs/AGENTS.md` — operating contract.
   - `docs/decisions/ADR-0215-application-surface.md` — the binding definition of "what an Application is".
   - `docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md` — every code path you write must carry `tenant_id`.
   - `docs/decisions/ADR-0248-amazon-shape-cellular-architecture.md` — what a cell is and why shuffle-sharding matters.
2. Clone via the worktree path (ADR-0116 retired external coord tooling; use `oya git`, **not** raw `git clone`):
   ```bash
   ./bin/oya git worktree-add --base dev --branch onboarding/$USER-week1 .worktrees/$USER-week1
   cd .worktrees/$USER-week1
   ```
3. Bring up your loopback dev cell:
   ```bash
   make dev-cell.up CELL=loopback-1 PROFILE=application-dev
   ```
   This launches 3 Cloud Hypervisor pods on your local Kata stack (Linux) or via OrbStack VM (macOS),
   with a single-node `cockroachdb` and a dummy `governance` Cedar evaluator.

## Day 2 — read the code path end-to-end

Walk the request:
1. `crates/oya-application-app/src/router.rs` — Axum router; routes are derived from `application.toml` per-tenant overrides.
2. `crates/oya-application-kernel/src/dispatch.rs` — `dispatch::Pipeline::run(intent, ctx)` — the contract surface.
3. `crates/oya-application-domain/src/intent.rs` — `Intent` enum (closed; adding a variant requires ADR amendment).
4. `crates/oya-application-port-cedar/src/lib.rs` — Cedar permit evaluation port.
5. `crates/oya-application-adapter-workflow-engine/src/lib.rs` — outbound dispatch into `workflow-engine`.

Read each file top-to-bottom; do **not** open downstream µservices yet. Stay in `application`.

## Day 3 — run the substance tests

```bash
cargo test -p oya-application-kernel --features dev-cell
cargo test -p oya-application-app -- --include-ignored
```

Now the canonical integration test:
```bash
make ms.application.integration CELL=loopback-1 TENANT=oyatie.community.dev-sample
```

Expected: 100 dispatch intents succeed with p95 ≤ 90 ms; the test writes its evidence bundle to `.foundry/evidence/$USER-week1/`.
If anything fails, do **not** patch the test — file an evidence note via:
```bash
./bin/oya vcs note --agent platform-eng-$USER --evidence "test_path:$path failure:$why"
```

## Day 4 — claim and ship a starter ticket

Pick from `microservices/application/migration-playbooks/from-aws-app-runner.md` (or another playbook) a task tagged `starter`. Then:
```bash
./bin/oya vcs claim \
  --agent platform-eng-$USER \
  --intent application-starter-$ticket \
  microservices/application/migration-playbooks
```

Implement, then:
```bash
./bin/oya vcs verify --agent platform-eng-$USER --evidence "tests_passed:N integration:green" <paths>
./bin/oya vcs done    --agent platform-eng-$USER --evidence "tests_passed:N integration:green" <paths>
```

Open the PR via `gh pr create --base dev` — the Foundry admission gate (ADR-0112) handles the rest.

## Day 5 — run a dispatch experiment in a dev cell

Use the SDK to fire 1,000 dispatches through your loopback cell and inspect the audit chain:
```bash
oya-app-dev experiment dispatch \
  --tenant oyatie.community.dev-sample \
  --intent application::Intent::CreateWorkspace \
  --count 1000 \
  --report
```

The report dumps p50/p95/p99 + Cedar permit count + cell hop count. Save it to your week-1 evidence bundle:
```bash
oya-app-dev experiment dispatch ... --report-out .foundry/evidence/$USER-week1/dispatch-report.json
```

## What "done with week 1" means

- [ ] You can name what an Application surface IS (one paragraph) and cite ADR-0215.
- [ ] You ran the integration test green against a loopback cell.
- [ ] One PR merged through Foundry on the `dev` branch.
- [ ] You produced a dispatch-report JSON in the evidence bundle.
- [ ] You filed at least one `oya vcs note` against a gap or wart you found.

## Common rookie traps

1. **Confusing `application` with `api-gateway`.** `api-gateway` is product-agnostic edge + protocol negotiation; `application` is tenant-aware + product-aware dispatch.
2. **Bypassing Cedar.** Never short-circuit the permit check, even in dev — the Cedar evaluator has a `dev-permissive` mode you can flag in `application.toml`.
3. **Forgetting `tenant_id`.** Any log line or trace span without a `tenant_id` will fail `lean-a3-tenant-trace` lint in CI.
4. **Editing `Intent` enum.** It's closed — submit an ADR amendment first.
