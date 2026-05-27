---
purpose: Oyatie Runbook — Forgejo Claim-Ref CAS Contract
doc_status: published
---

# Oyatie Runbook — Forgejo Claim-Ref CAS Contract

> **Status:** Contract/runbook for the Wave 3 board fallback.
> **Owner:** Governance tooling maintainers.
> **Last verified:** 2026-05-27 from leader-supplied live Forgejo evidence.
> **Scope:** Plain Git + Forgejo issues/labels/webhooks only. This runbook does not authorize `oya git`, `oya vcs`, GitHub Projects, native Forgejo Projects, or a long-running board service.

## Proven live evidence

- Forgejo target: `oya-forge`, Forgejo `11.0.14`.
- Native Forgejo Projects REST endpoints returned `404`, so board state must not depend on native Projects availability.
- Exclusive label projection works for board columns: `board/backlog` can be replaced by `board/claimed` as a visible issue-state projection.
- Native issue-assignee mutation is not an atomic claim primitive: two concurrent `PATCH` requests both succeeded, and the issue ended with two assignees.
- Git ref creation under `refs/heads/claims/<deliverable-id>` is atomic enough for claim ownership: one concurrent push succeeded; the other failed with remote ref-lock / reference-exists behavior.
- Push webhooks include sender identity, allowing the winning claimant to be reconciled from the ref update event.

## Contract invariants

1. The claim source of truth is exactly one Git ref: `refs/heads/claims/<deliverable-id>`.
2. Claim creation is compare-and-set by ref creation: create the ref only when it does not already exist.
3. Issue labels are projection-only. `board/backlog`, `board/claimed`, and related exclusive labels mirror claim state for humans; they are not locks.
4. Issue assignees are projection-only. They may be updated after a claim, but they do not prove exclusive ownership.
5. A losing claimant must not retry by overwriting, deleting, force-pushing, or moving another worker's claim ref.
6. Project availability is optional. Forgejo Projects REST `404` must degrade to the issue-label projection path.
7. Webhook reconciliation must treat push sender identity plus ref name as the authoritative audit signal for claim ownership.
8. All board sync and claim logic must be idempotent: rerunning sync after the same winning ref exists must produce no additional owner changes.

## Claim flow

1. Derive the deliverable id from the master plan entry.
2. Read the issue snapshot and display projection labels for operator context.
3. Attempt to create `refs/heads/claims/<deliverable-id>` pointing at the agreed claim commit or empty marker commit.
4. If ref creation succeeds:
   - record this worker/user as the claim winner;
   - project the issue from `board/backlog` to `board/claimed` using exclusive labels;
   - optionally project assignee metadata for convenience;
   - wait for or consume the push webhook and reconcile sender identity.
5. If ref creation fails because the remote already has the ref or reports a ref-lock/reference-exists conflict:
   - treat the attempt as a lost race;
   - fetch/read the existing claim ref;
   - report the current owner from the ref/webhook/issue projection;
   - do not mutate labels or assignees except to refresh local display state.
6. If Forgejo Projects endpoints return `404`, continue with issues, labels, Git refs, and push webhooks. Do not create a replacement daemon.

## Loser behavior

A loser exits cleanly with a conflict result that names the deliverable id and, when available, the winning ref tip and sender. The loser must not:

- force-push or delete `refs/heads/claims/<deliverable-id>`;
- PATCH the issue assignee as a second claim attempt;
- move labels to imply ownership;
- fall back to native Projects or any non-Git lock primitive.

## Projection label rules

- `board/*` labels are mutually exclusive within the board-state family.
- Claim success projects `board/backlog -> board/claimed`.
- Projection updates are best-effort and repeatable. They may lag the ref but must converge to the ref state.
- Label state is never sufficient to grant ownership without the matching claim ref.

## Future verification commands and tests

Future code that implements this contract should provide targeted checks equivalent to:

```bash
# Unit/integration: concurrent ref creation has one winner and one loser.
oya plan claim <deliverable-id> --repo <forgejo-remote> --actor worker-a &
oya plan claim <deliverable-id> --repo <forgejo-remote> --actor worker-b &
wait

git ls-remote <forgejo-remote> refs/heads/claims/<deliverable-id>

# Sync: issue labels project the winning ref and rerun idempotently.
oya gen board-sync --masterplan <path> --board-snapshot <snapshot> --dry-run
oya gen board-sync --masterplan <path> --board-snapshot <snapshot> --dry-run

# Gate: snapshot and masterplan agree in both directions.
oya gate validate board-masterplan-consistency --master-plan <path> --board-snapshot <snapshot>
```

Required assertions:

- exactly one concurrent claim creates `refs/heads/claims/<deliverable-id>`;
- the loser receives a typed conflict and performs no ownership projection;
- assignee races cannot be used as passing evidence for exclusive claim ownership;
- Forgejo Projects REST `404` keeps the label/ref fallback path healthy;
- push webhook fixtures include sender identity and the claim ref name;
- board-sync diff is empty on the second run after projection converges;
- orphan checks fail when a masterplan deliverable has no board issue or a board issue has no masterplan deliverable.

## Operator checklist

1. Confirm the deliverable id and target Forgejo remote.
2. Check for an existing claim ref before attempting a claim.
3. Create the claim ref without force or delete semantics.
4. Update only projection labels after a winning ref exists.
5. Confirm the push webhook sender matches the expected claimant.
6. Re-run board sync to verify an idempotent empty diff.
7. Preserve the losing attempt's conflict output as evidence when investigating races.

## Sources

Leader-supplied live evidence from the 2026-05-27 Wave 3 board implementation run: Forgejo `11.0.14` on `oya-forge`, Projects `404`, successful exclusive-label projection, non-atomic assignee race, atomic claim-ref race, and sender-bearing push webhook payloads.
