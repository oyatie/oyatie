# Oyatie product/reorg pause handoff

Timestamp: 2026-07-26T06:08:00Z

State: PAUSED. No merge, push, dispatch, or product edit may resume until a later user resume.

## Invariants

- `HOLD(Planning)` remains. Nothing here is a Stage-1 PASS or roadmap-dispatch authorization.
- `specs/root-hub-pointers.json` points to `docs/AGENTS.md` as current operating authority.
- Use fresh isolated worktrees, signed commits, protected PRs to `dev`, exact-head review, resolved threads, exact `oya-ci-required`, squash merge, and postmerge receipts.
- Never hand-edit `*.generated.json`; use the sanctioned generator/materializer.
- New user authority: every `oya/**`, `oya-*`, `cloud/**`, and `cloud-*` path/name is a deprecated migration source. New product behavior must land in the debranded ADR-0562 capability-first/app composition tree with clean dependency direction.

## Stable remote anchor

- Remote `dev`: `0ac6e7654168b8a30f0e3c6627a74ba9e85157ed`
- No PR was merged during the pause transition.

## PR #1379 — OpenBao async secret-store boundary

- PR: https://github.com/jason931225/oyatie/pull/1379
- Worktree: `/Users/jasonlee/oyatie-worktrees/issue1268-openbao-runtime-20260726T0314Z`
- Branch/head: `codex/issue1268-openbao-runtime-20260726T0314Z` / `2d586d1df1f53660814443fbb6b7ab530b6b7537`
- Worktree clean; remote head matches local; mergeable but protected state remains BLOCKED.
- Independent local review and verification passed; zero known review threads.
- Protected run `30188574127` was still in progress at pause on affected-set job `89757798061`.
- The merge-train agent was interrupted before merge. Let the existing run finish, but do not merge while paused.

## PR #1380 — billing invoice application boundary

- PR: https://github.com/jason931225/oyatie/pull/1380
- Worktree: `/Users/jasonlee/oyatie-worktrees/issue1272-billing-invoice-20260726T0445Z`
- Branch/head: `codex/issue1272-billing-invoice-20260726T0445Z` / `10cdfff316819689fd7c1f0de8cd6f80479532c2`
- Worktree clean; signed commit; remote head matches local.
- Exact-head local reviewer/verifier passed.
- Protected run `30189683435` was intentionally cancelled after the metadata preflight exposed non-canonical Code Review verdict grammar. The PR body is repaired.
- Resume only after #1379 is promoted and the exact `dev` push run is green: rebase with signature, rerun exact local tests, force-with-lease push, run one protected admission, review, merge, and capture the postmerge packet.

## PR #1381 — legacy-path Leptos/Axum shell

- PR: https://github.com/jason931225/oyatie/pull/1381
- Worktree: `/Users/jasonlee/oyatie-worktrees/issue1273-leptos-axum-20260726T0412Z`
- Remote branch/head: `codex/issue1273-leptos-axum-20260726T0412Z` / `abfbec8c537219a2ba2f7bf06a2e14b7de6eb875`
- Remote review state: CHANGES_REQUESTED. Protected run `30189333563` failed.
- Do not merge or push this PR as an `oya/**` destination.
- Preserved uncommitted repair:
  - `docs/decisions/ADR-0393-leptos-canonical-app-shell-frontend.md`
  - `oya/application/crates/oya-application-shell-frontend/src/server.rs`
  - `oya/application/crates/oya-application-shell-frontend/tests/live_server.rs`
  - untracked `oya/application/crates/oya-application-shell-frontend/OWNERS`
- Bundle identity (`git diff --binary` plus untracked OWNERS content): SHA-256 `fe904825ca256cfbe38aa33ac4b9c8f7c7362043807ac6753fec936c1a212b50`.
- Repair behavior: remove unused unauthenticated wildcard POST `/api/{*fn_name}` because the crate registers zero server functions; live regression requires POST `/api/not-a-server-function` -> 404.
- Green receipts:
  - build `1c2d9aec-8060-47c5-9b1a-d3035f6f7cd8`: app unit 31/31, live integration 3/3, auth coverage unit 44/44, auth gate 7/7.
  - accounting build `fc0eb1a1-2e79-4f1e-b62b-10cd04f653ba`: unit 8/8, live corpus 3/3.
  - rustfmt and `git diff --check` passed; no generated JSON changed.
- Accepted mapping: ADR-0562 membership coverage maps `oya/application` to `app/application`. ADR-0615 keeps the reusable shell substrate under `console/`; the full multi-capability application composition belongs under `app/application`.
- Existing `console/**` workspace-shell/docs-portal crates are not equivalent to the Leptos frontend.
- Resume as a migration lane: create the exact move plan/codemod queue, debrand crate/target identities, preserve clean architecture, migrate catalog/SLO/IaC/dependency references, regenerate faces, and transplant the tested fail-closed behavior. Close or replace #1381 rather than admitting a legacy-only destination.

## PR #1382 — fail-closed OIDC discovery

- PR: https://github.com/jason931225/oyatie/pull/1382
- Worktree: `/Users/jasonlee/oyatie-worktrees/issue778-oidc-metadata-20260726T051223Z`
- Local branch/head: `codex/issue778-oidc-metadata-20260726T051223Z` / signed local commit `f58eebc0c311ae87a65e93b49dadafa994c416e3`
- Worktree clean.
- Remote PR still points to old head `6e714207647c0e20da9c69bb408d31c5cbff9cd4`; the verified repair is intentionally unpushed.
- Exact repair: live discovery returns 404; canonical `/oauth/v2/keys` and legacy `/oauth/jwks` remain 200; no authorization/token/userinfo/session surface is invented.
- Verifier PASS:
  - kernel build `f9c0b689-955a-4f5b-960e-bad228444e8d`: unit 36/36, integration 32/32.
  - service build `a7ee8316-1968-4eea-80a5-06d9ff6432df`: unit 47/47, live E2E 6/6.
  - rustfmt/diff/generated-face hygiene passed.
- Old protected run `30189533657` was intentionally cancelled.
- Resume after preceding train entries: rebase signed local commit onto promoted `dev`, rerun exact tests, update PR body to exact `Verdict: APPROVED`, force-with-lease push, protected admission, review, merge, postmerge packet.

## Pipeline findings

- Local agent coordinator had soft FD limit 256 and 248 open descriptors; completed subagent rollout files remained open. Direct command execution intermittently failed with `EMFILE`.
- 37 idle Oyatie Buck2 daemon/forkserver groups were safely reaped.
- Evidence: https://github.com/jason931225/oyatie/issues/1377#issuecomment-5082240331
- The affected-set FULL lane repeats a cold merge-base build/test before candidate execution. On #1379: setup/reclaim/restore/materialization consumed about 9 minutes, the cold merge-base baseline consumed 32 minutes, and candidate binding started about 41 minutes after job start.
- Evidence: https://github.com/jason931225/oyatie/issues/899#issuecomment-5082254252
- Correct scheduling: parallelize implementation, local tests, review, reorg mapping, and prepared signed commits; serialize only exact-base protected admission/merge/promotion until trusted exact-SHA reuse exists.

## Interrupted agents

- `issue1268_merge_train` — interrupted before any merge.
- `reorg_application_arch` — interrupted while deciding whole-product move versus first shell strangler slice.
- `issue778_metadata_arch` — interrupted; no implementation depended on it.
- Other lanes used for this checkpoint had already completed or paused.

## Exact resume order

1. Re-read this handoff, `specs/root-hub-pointers.json`, and `docs/AGENTS.md`; fetch `origin/dev`.
2. Recheck PR #1379 run `30188574127`. If exact-head green and the user has resumed, revalidate base/head/review/threads, squash merge, wait for exact promoted-`dev` `oya-ci-required`, then post the immutable packet.
3. Rebase/admit/merge #1380 and capture its promoted-dev packet.
4. Finish the `oya/application` -> `app/application` architecture decision, create a fresh migration worktree and generator-owned move plan, and transplant #1381 behavior into the debranded destination. Do not push the preserved old-path repair.
5. Rebase/admit/merge signed local #1382.
6. Resume only debranded product-code and migration queues; reject documentation-only work unless it is the minimum admission/authority edge for code.

