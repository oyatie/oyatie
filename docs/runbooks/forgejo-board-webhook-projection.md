---
doc_status: published
---

# Runbook: Forgejo Board Webhook Projection

> **Owner:** `platform-governance` / `agentic-pipeline`
> **Severity supported:** Sev 3
> **Last verified:** 2026-05-27 by worker-6 in documentation review
> **Related:** [`RUNBOOKS-INDEX.md`](../RUNBOOKS-INDEX.md), [`TOOLCHAIN.md`](../TOOLCHAIN.md), [`MASTERPLAN.md`](../MASTERPLAN.md)

---

## Trigger

Open this runbook when the Forgejo issue board projection is stale or disputed after either of these inputs:

- Forgejo issue webhook for label changes on a deliverable issue.
- Git push webhook for `refs/heads/claims/<deliverable-id>` claim refs.
- Operator or agent report that a board column, exclusive label, or active claim no longer matches the masterplan deliverable state.

This runbook documents projection handling only. It does not authorize a new always-on board daemon, queue worker, or bespoke long-running board service.

---

## Pre-checks (5 minutes max)

- [ ] Confirm the event source is Forgejo for the expected repository and organization.
- [ ] Confirm the webhook signature is present and valid for the configured Forgejo webhook secret before trusting payload fields.
- [ ] Confirm the sender is allowed to mutate the projected board state: Forgejo system sender, approved bot identity, or a human maintainer with repository write permission.
- [ ] Confirm the event references a known deliverable id from the masterplan or an existing deliverable issue.
- [ ] Confirm this is a projection repair, not a request to introduce a new service. If a new service is requested, stop and require explicit later approval.

If any pre-check fails, do not update labels or claims. Record the rejected payload id and escalate to the agentic-pipeline owner.

---

## Event inputs

### Issue label webhook

Required fields:

- Repository full name and stable repository id.
- Issue id / number and issue URL.
- Label action (`added`, `removed`, or equivalent Forgejo issue-label mutation).
- Full label set after the mutation when available; otherwise fetch the issue labels before projecting.
- Sender id, sender login, event delivery id, event timestamp, and signature verification evidence.

Projection key: `issue:<repo-id>:<issue-id>:<label-set-digest>`.

### Claim-ref push webhook

Required fields:

- Repository full name and stable repository id.
- Ref name matching `refs/heads/claims/<deliverable-id>`.
- Old object id, new object id, before/after timestamps, and pusher identity.
- Commit metadata sufficient to identify the claiming worker or agent lane.
- Sender id, sender login, event delivery id, event timestamp, and signature verification evidence.

Projection key: `claim-ref:<repo-id>:<deliverable-id>:<new-object-id>`.

Ignore push events outside `refs/heads/claims/`. Do not infer a claim from unrelated branches, tags, pull requests, or comments.

---

## Projection rules

1. Derive the deliverable id from the issue mapping or `refs/heads/claims/<deliverable-id>`.
2. Load the current masterplan deliverable record and current Forgejo issue label set.
3. Normalize column labels into one exclusive board-state family. At most one active column label may remain after projection.
4. Apply the deterministic target column from the current deliverable state and claim state:
   - no active claim: backlog / ready label;
   - active claim with live claim ref: claimed / in-progress label;
   - completed deliverable evidence accepted: done label;
   - blocked claim or failed verification: blocked / needs-attention label.
5. Remove loser labels from the same exclusive family before adding the winning label.
6. Preserve non-board labels such as area, priority, severity, or evidence labels unless a separate approved policy says otherwise.
7. Emit or record projection evidence with event delivery id, projection key, before labels, after labels, and claim ref observed.

---

## Idempotency and ordering

Projection MUST be idempotent.

- Re-processing the same delivery id or projection key must produce the same final label set and must not create duplicate evidence.
- If webhooks arrive out of order, reconcile from current Git refs and current issue labels before applying changes.
- Treat the Git ref namespace as the source of truth for active claims. Treat Forgejo labels as a projection, not the authority.
- Use compare-and-swap semantics when updating a claim ref or label snapshot. If the observed base changed, reload and retry the projection from current state.
- Never process an event by appending another column label without first recomputing the exclusive label family.

---

## Loser-claim handling

A loser claim is any worker or agent claim that loses the `refs/heads/claims/<deliverable-id>` compare-and-swap race.

- Do not move the issue to claimed / in-progress for a loser claim.
- Do not delete the winning claim ref while handling a loser event.
- If a loser event already projected a claimed label, immediately reconcile from Git refs and replace it with the winner-derived label.
- Record the loser identity, rejected object id, winning object id, and event delivery id in projection evidence.
- Notify the losing lane through the normal agent/team channel; do not use board labels as the only loser notification.

---

## Reconciliation from Git refs

Use reconciliation when labels are stale, webhook delivery is uncertain, or an operator reports a board mismatch.

1. Enumerate `refs/heads/claims/*` for the repository.
2. For each deliverable id, resolve the current object id and claiming identity.
3. Load the matching masterplan deliverable and Forgejo issue.
4. Compute the desired exclusive board label from the current ref plus deliverable evidence state.
5. Diff desired labels against actual issue labels.
6. Apply only the minimal label changes needed to make the projection match the refs.
7. Record the reconciliation run id, ref snapshot, issue snapshot, diff, and outcome.

If a deliverable has no current claim ref, remove claimed / in-progress labels unless accepted completion evidence requires done.

---

## Explicit non-goals and guardrails

- Do not introduce `oya git` or `oya vcs` primitives.
- Do not depend on GitHub Projects or GitHub-specific project automation.
- Do not run `oya gate run-all` as part of webhook projection.
- Do not add a bespoke long-running board service, daemon, or controller unless a later approved ADR/task explicitly authorizes it.
- Do not treat Forgejo labels as the system of record for claims; labels are a board projection over masterplan deliverables and Git refs.

---

## Verification

After projection or reconciliation, verify:

- [ ] The issue has no more than one active board-column label from the exclusive family.
- [ ] The winning label matches the current `refs/heads/claims/<deliverable-id>` state and deliverable evidence state.
- [ ] Replaying the same webhook delivery id is a no-op.
- [ ] Reconciliation from Git refs produces an empty diff after the update.
- [ ] Loser-claim evidence exists when a CAS race was rejected.

---

## Post-incident updates

After this runbook is invoked for a real incident or material board mismatch, update:

- [ ] Incident notes with event delivery ids, ref snapshot, issue snapshot, and projection diff.
- [ ] The relevant deliverable evidence bundle if the mismatch affected claim or completion status.
- [ ] This runbook if the event shape, signature expectation, or reconciliation step was incomplete.
- [ ] [`MISTAKES-LEDGER.md`](../MISTAKES-LEDGER.md) only when a new reusable prevention is identified.

---

## Audit-chain emission

Record each invocation as `forgejo.board_projection.runbook_invoked` with:

- runbook id: `forgejo-board-webhook-projection`;
- invoker id and sender id;
- event delivery id or reconciliation run id;
- deliverable id and issue id;
- observed claim ref and object id;
- before / after exclusive label values;
- outcome: `projected`, `reconciled`, `rejected`, or `escalated`.

---

## Sources scanned

- Task 15 inbox assignment and scope constraints.
- [`docs/templates/runbook-template.md`](../templates/runbook-template.md).
- [`docs/RUNBOOKS-INDEX.md`](../RUNBOOKS-INDEX.md).
