# Platform Engineer — First Week on `application`

Audience: a platform / SRE engineer joining the `oya-application-*` lane. Goal: by EOD Friday you can claim a real ticket, ship a PR through the
native SCM/GitHub adapter path, and run a tenant-scoped dispatch experiment in a dev cell. No prior Oyatie knowledge assumed; Rust + Kubernetes +
Linux fundamentals are.

## Day 1 — orient and clone

1. **Read the contract before any tooling.** Open in order:
   - `docs/AGENTS.md` — operating contract.
   - `docs/decisions/ADR-0215-application-surface.md` — the binding definition of "what an Application is".
   - `docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md` — every code path you write must carry `tenant_id`.
   - `docs/decisions/ADR-0248-amazon-shape-cellular-architecture.md` — what a cell is and why shuffle-sharding matters.
2. Create an isolated worktree branch from `dev`:
   ```bash
   git fetch origin dev
   git worktree add -b onboarding/$USER-week1 .worktrees/$USER-week1 origin/dev
   cd .worktrees/$USER-week1
   ```
3. Bring up your loopback dev cell through the registered Buck2/Prow dev-cell harness for `CELL=loopback-1` and
   `PROFILE=application-dev`. The harness launches 3 Cloud Hypervisor pods on your local Kata stack (Linux) or via OrbStack VM (macOS), with a
   single-node `cockroachdb` and a dummy `governance` Cedar evaluator.

## Day 2 — read the code path end-to-end

Walk the request:
1. `oya/application/crates/oya-application-app/src/lib.rs` — current foundation app composition surface.
2. `oya/application/crates/oya-application-app/src/product_catalog.rs` — tenant product-catalog value objects and tests.
3. `oya/application/contracts/` — OpenAPI, AsyncAPI, and proto contracts for external consumers.
4. `oya/application/cedar/` — authorization policies that must stay tenant-scoped and default-deny.
5. `oya/application/runbooks/` — operational evidence and recovery guidance for the lane.

Read each file top-to-bottom; do **not** open downstream µservices yet. Stay in `application`.

## Day 3 — run the substance tests

Buck2 is the build/test/check authority. Build the lane target first and then run the registered Buck2/Prow test target for the changed shard:
```bash
buck2 build //oya/application/crates/oya-application-app:oya-application-app
buck2 test <registered-application-test-target>
```

Expected: 100 dispatch intents succeed with p95 <= 90 ms and the Prow job publishes multispectrum PR evidence. Cargo metadata may exist for
Rust tooling and Reindeer inputs, but Cargo commands are advisory only and are not merge evidence. If anything fails, do **not** patch the test to
hide the failure; attach the failing Buck2/Prow log and the suspected cause to the PR evidence.

## Day 4 — claim and ship a starter ticket

Pick from `oya/application/migration-playbooks/` a task tagged `starter`. Then:
1. Keep the work isolated in the worktree branch.
2. Implement only the starter ticket paths.
3. Run the Buck2 target(s) that cover those paths.
4. Open the PR with `gh pr create --base dev --head onboarding/$USER-week1`.
5. Merge readiness comes from reviewer approval plus the trusted Prow/Kubernetes-native `oya-ci-required` context. GitHub Actions is shadow
   compatibility only.

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
oya-app-dev experiment dispatch ... --report-out evidence/onboarding/$USER-week1/dispatch-report.json
```

## What "done with week 1" means

- [ ] You can name what an Application surface IS (one paragraph) and cite ADR-0215.
- [ ] You ran the integration test green against a loopback cell.
- [ ] One PR merged through native SCM/GitHub adapter flow on the `dev` branch.
- [ ] You produced a dispatch-report JSON in the evidence bundle.
- [ ] You attached at least one PR evidence note for a gap or wart you found.

## Common rookie traps

1. **Confusing `application` with `api-gateway`.** `api-gateway` is product-agnostic edge + protocol negotiation; `application` is tenant-aware + product-aware dispatch.
2. **Bypassing Cedar.** Never short-circuit the permit check, even in dev — the Cedar evaluator has a `dev-permissive` mode you can flag in `application.toml`.
3. **Forgetting `tenant_id`.** Any log line or trace span without a `tenant_id` will fail `lean-a3-tenant-trace` lint in CI.
4. **Editing `Intent` enum.** It's closed — submit an ADR amendment first.
