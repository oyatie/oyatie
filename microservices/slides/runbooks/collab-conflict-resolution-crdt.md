---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: slides
runbook_id: collab-conflict-resolution-crdt
status: Accepted
severity: Sev-2 (Sev-1 if silent-loss detected)
date: 2026-05-17
owner_team: axis-workspace + ops-sre-reliability
related_artifacts:
  - microservices/slides/threat-model.md (T-T-01)
  - microservices/slides/decisions/ADR-SLIDES-0001-crdt-library-selection.md
  - microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml
doc_status: published
---

# Runbook — Collab conflict resolution (CRDT)

## When to use

- Tenant reports unexpected conflict UI for what they believe was a non-overlapping edit.
- Conflict-rate SLI dashboard shows anomalous spike for a single deck or pack.
- Sev-1 alarm: `oya_slides_collab_silent_loss_attempt_total > 0`.

## Symptom triage

| Symptom | Likely cause | Run |
|---|---|---|
| Single deck, conflict UI fires repeatedly | Loro state divergence after WS reconnect storm | step 1 |
| Multiple decks per pack, conflict rate spike | Loro library regression after deploy | step 2 |
| Sev-1 `silent_loss_attempt > 0` | HMAC tampering OR Loro merge-algebra bug | step 3 |
| Tenant says "my edit disappeared" but no conflict surfaced | Investigate via replay (see `backfill-replay.md` R-2) | step 4 |

## Step 1 — Single-deck recovery

```bash
# Identify deck + tenant
DECK_ID=<reported_deck_id>
TENANT_ID=<tenant_id>

# Check WS gateway pod lease (single-writer invariant)
oya vcs --pod-shell collab-crdt-worker
> valkey-cli --no-auth-warning -h <valkey-host> GET "slides:lease:deck:${DECK_ID}"
# Expect: single pod ID; if multiple OR null, split-brain

# Reconcile from Postgres snapshot
oya vcs --service slides --action reconcile-crdt --deck-id $DECK_ID --tenant-id $TENANT_ID

# Force WS reconnect for affected clients
oya vcs --service slides --action force-ws-reconnect --deck-id $DECK_ID --reason "crdt-reconcile"
```

Verify: conflict UI no longer firing; clients reconnected; CRDT state byte-equal across pods.

## Step 2 — Library-regression suspect

```bash
# Identify Loro version + recent deploy
helm history slides -n workflow-studio
helm get values <revision_n> | grep loro_version

# Quick rollback if recent
helm rollback slides <revision_n-1> -n workflow-studio
```

After rollback, monitor `oya_slides_collab_conflict_rate` for 30 min. If conflict rate returns to baseline, the rollback is correct. Open issue with reproducer for upstream Loro maintainers.

## Step 3 — Silent-loss-attempt Sev-1

This is the most serious case. AC-06 invariant violated.

```bash
# Inspect counter source — was it HMAC mismatch or merge-algebra?
oya vcs --service slides --action describe-silent-loss-attempt --last 10
```

Two branches:

### 3a — HMAC mismatch source

This is tampering. Treat as security incident:

```bash
# Rotate per-session HMAC keys (forces all sessions to reconnect)
oya vcs --service slides --action rotate-collab-hmac-keys --tenant-id <tenant_id>

# Audit row already emitted; verify
oya vcs --service slides --action audit-tail --kind collab_hmac_mismatch --since 1h
```

Escalate to ops-security + DPO for breach evaluation.

### 3b — Merge-algebra source

This is a Loro library bug. Treat as Sev-1 correctness incident:

```bash
# Freeze T2 AI-content-generation cluster-wide
oya vcs --service slides --action freeze-capability --capability t2-content-gen --reason "crdt-correctness-investigation"

# Pin Loro to previous known-good version + redeploy
helm rollback slides <known_good_revision> -n workflow-studio

# Re-run AC-06 property test against last 24h captured op stream
cargo nextest run -p oya-slides-real-time-collaboration-domain --test test_no_silent_overwrite -- --ignored 24h-capture
```

Open ADR-SLIDES-0001 supersession if root-causes to Loro library defect.

## Step 4 — Tenant reports vanished edit; no conflict surfaced

```bash
# Replay CRDT op log for the deck (per backfill-replay.md R-2)
oya vcs --service slides --action replay-crdt --deck-id $DECK_ID --tenant-id $TENANT_ID --from <timestamp>

# Inspect output: is the edit op present in the log?
# YES — projection bug; emit fresh save event
# NO   — client never sent the op (network drop pre-WS); irretrievable; document for tenant
```

## Verification

- `oya_slides_collab_conflict_rate` returned to baseline (≤ 0.5% of merges).
- `oya_slides_collab_silent_loss_attempt_total = 0` in last 30min.
- AC-06 property test green.
- Audit row chain unbroken.

## Escalation

- Sev-1: leadership + DPO + legal.
- Open post-mortem within 5 business days per ADR-0123.

## References

- ADR-SLIDES-0001 (Loro CRDT).
- threat-model.md T-T-01.
- PRD AC-06.
- backfill-replay.md R-2.
- workflow-studio collab-conflict-resolution runbook (sibling pattern).
