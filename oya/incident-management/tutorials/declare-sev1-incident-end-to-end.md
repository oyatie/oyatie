---
doc_class: Tutorial
microservice: incident-management
related_adrs: [ADR-0316, ADR-0251]
date: 2026-05-20
doc_status: published
---

# Tutorial — Declare a SEV-1 incident end-to-end with war-room + status-page + post-mortem

Goal: walk through declaring a real SEV-1 incident from first-page to post-mortem-publish. By the end, you'll have: a paged on-call team, an automated war-room channel, a public status-page update, a customer-impact estimate, and a published post-mortem.

Scenario: a critical service (`cloud-billing-tax-app`) starts returning 5xx errors for 35 % of requests at 14:22 UTC. The `observability` µservice's availability SLO breach-detector fires a multi-window burn-rate alert at 14:23 UTC.

Prereqs:
- `incident::commander` Cedar role.
- paid tier or higher (war-room automation + status-page integration are paid tenant_class).
- Your tenant configured with Slack + SMS + voice channels.
- ~ 1 hour for the tutorial; the real incident response can take many hours.

## Step 1 — alert ingest + page delivery (14:23-14:24 UTC)

The observability µservice cross-emits the alert to `incident-management`. Within 200 ms, the substrate:
1. Creates an incident record with auto-assigned ID `INC-2026-05-20-001`.
2. Looks up the service's escalation policy: "cloud-billing-tax-app SEV-1 policy".
3. Pages the primary on-call (Alice), the secondary (Bob), the EM (Charlie), and the IC (you) in parallel via SMS + voice + Slack.

Within 6 s, all four parties receive notifications. The Slack notification includes:
- Incident ID + service + severity + alert source.
- Runbook URL.
- "Acknowledge" button.
- War-room channel link (auto-created).

## Step 2 — acknowledge + take command (14:24-14:25 UTC)

You ack the page via Slack button. The substrate stops further paging to you.

You enter the war-room channel `#incident-sev1-inc-2026-05-20-001`. The substrate has auto-populated:
- Pinned message with incident details + runbook URL.
- Inviter bot with quick actions ("update status", "page another team", "escalate to next level").
- Linked dashboards from the observability µservice.
- Recent commits to the affected service (`cloud-billing-tax-app`) — pulled from `oya git`.

You announce: "@here I'm taking IC. We have a 35 % 5xx error rate on cloud-billing-tax-app since 14:22 UTC. Alice owns mitigation. Bob owns customer comms. I'll handle external escalation."

Use the `/incident assign-roles` slash command:

```
/incident assign-roles inc-2026-05-20-001 \
  --primary-on-call alice@your-tenant.com \
  --mitigation-lead alice@your-tenant.com \
  --comms-lead bob@your-tenant.com \
  --incident-commander self
```

Substrate logs the role assignments to audit-chain.

## Step 3 — first customer status-page update (within 15 min of detection)

Bob (comms-lead) opens the status-page integration:

```
/incident statuspage post INC-2026-05-20-001 \
  --component cloud-billing-tax-app \
  --status investigating \
  --message "We are investigating elevated error rates in our billing API. Customers may see failed billing operations. Updates every 30 minutes."
```

The substrate posts to your configured status-page provider (oyatie's `community` µservice public-status, or Statuspage.io / Instatus / etc). Subscribers (customers who opted in) get notified via email + SMS + webhook.

Status-page post is logged to audit-chain.

## Step 4 — mitigation investigation (14:25-14:50 UTC)

Alice opens the runbook and follows the diagnostic commands. The runbook's "Top-3 likely causes":
1. Upstream PostgreSQL primary failure.
2. Tax-calculator dependency (Avalara) outage.
3. Misconfigured circuit-breaker threshold after recent deploy.

Diagnostic 1 (PostgreSQL): `kubectl -n cloud-billing logs deploy/postgres-primary --tail=200 | grep ERROR`. No errors. Cause 1 ruled out.

Diagnostic 2 (Avalara): `curl https://api.avalara.com/api/v2/utilities/ping`. Returns 200 OK with 80 ms latency. Cause 2 ruled out.

Diagnostic 3 (circuit breaker): `kubectl -n cloud-billing exec deploy/cloud-billing-tax-app -- curl -s localhost:9090/metrics | grep oya_circuit_breaker`. Shows `oya_circuit_breaker_state{name="tax-calc-down"} = 1` (tripped) for the last 5 minutes.

Root cause: a deploy at 14:21 UTC reduced the circuit-breaker threshold from 50 errors/min to 5 errors/min. Normal background error rate (intermittent transient failures) immediately tripped the breaker.

Mitigation: rollback the deploy. Alice runs:

```sh
oya deploy rollback cloud-billing-tax-app \
  --to-version v2.3.4 \
  --reason "INC-2026-05-20-001 circuit-breaker threshold regression"
```

Deploy rolls back in 90 s. Error rate returns to normal within 60 s after rollback.

## Step 5 — mitigation confirmed + state transitions (14:50-15:00 UTC)

You verify in the observability dashboard: error rate has dropped from 35 % to 0.1 %. Burn rate alert resolves automatically.

Update incident state:

```
/incident state inc-2026-05-20-001 --to mitigated --message "Rolled back to v2.3.4. Error rate back to baseline."
```

Post follow-up status-page update:

```
/incident statuspage post INC-2026-05-20-001 \
  --status monitoring \
  --message "We have rolled back the change and error rates have returned to normal. We are monitoring for stability."
```

Wait 30 min. Verify stability.

Transition to resolved:

```
/incident state inc-2026-05-20-001 --to resolved
```

Final status-page update:

```
/incident statuspage post INC-2026-05-20-001 \
  --status resolved \
  --message "The incident is resolved. We will publish a post-mortem within 5 business days."
```

## Step 6 — customer-impact estimation

The substrate's customer-impact estimator (paid tenant_class tier) reports:

```
Incident INC-2026-05-20-001 customer impact:
  Duration: 28 minutes (14:22 UTC start, 14:50 UTC mitigation)
  Affected customers: 47 (out of 312 active billing-active)
  Affected transactions: ~ 2 400 failed
  Estimated lost transaction value: $18 200
  Estimated SLA-credit exposure: $4 600 (per the cloud-billing-tax-app SLA: 5 % credit per 0.1 % monthly availability shortfall)
```

This estimate is preserved in the incident record for the post-mortem.

## Step 7 — post-mortem authoring (within 5 business days)

Portal → Post-Mortems → "New from incident" → INC-2026-05-20-001.

The substrate pre-populates:
- Timeline from incident state transitions.
- Pages issued.
- Mitigation actions (deploy rollback).
- Customer impact estimate.
- Linked Slack war-room transcript.

You author the narrative sections:

**Summary**: A deploy at 14:21 UTC reduced the circuit-breaker threshold below the normal background error floor, causing the circuit-breaker to trip and 35 % of requests to fail with 5xx for 28 minutes until rolled back.

**Root cause**: The circuit-breaker threshold was tuned manually 6 months ago to 50 errors/min based on then-current background error rate of ~ 30/min. The deploy at 14:21 UTC reduced this to 5/min based on a hypothesis that the new code had lower expected error rate. The hypothesis was wrong — background error rate didn't decrease — and the new threshold was below the baseline, causing immediate tripping.

**Contributing factors**:
- The circuit-breaker threshold change was not flagged in the deploy review as a high-risk change.
- The deploy did not include a synthetic load-test of the new threshold against current background error rate.
- The deploy went out at 14:21 UTC (Friday afternoon NYC time / 14:21 NYC, near close-of-business) reducing the on-call window for rapid response.

**Action items**:
1. Add circuit-breaker threshold changes to the high-risk deploy checklist (owner: Alice, due 2026-05-28). Verified by reviewing the next 3 high-risk deploys and confirming the checklist was followed.
2. Add a synthetic check that validates circuit-breaker threshold > current background error rate × 1.5, blocking the deploy if not (owner: Bob, due 2026-06-15). Verified by attempting a synthetic mis-configured deploy and seeing it blocked.
3. Update the cloud-billing-tax-app runbook to include "Diagnostic 3: circuit-breaker state" as the FIRST diagnostic (it was Diagnostic 3 in this incident's runbook; should have been first) (owner: Alice, due 2026-05-25).

**Customer impact**: 47 customers affected, ~ 2 400 failed transactions, estimated $4 600 SLA-credit exposure. The CSM team will reach out to affected customers within 48 h.

## Step 8 — publish + track action items

```
/postmortem publish INC-2026-05-20-001 --version 1.0.0
```

The substrate emits `post-mortem::published` to audit-chain. Subscribers (your engineering org) are notified. Action items are tracked in the portal; overdue items auto-escalate to the EM.

## Step 9 — verify compliance evidence

```sh
oya audit-chain query --tenant <tenant-id> \
    --event-class "incident::*,post-mortem::*" \
    --incident-id INC-2026-05-20-001
```

You should see the full lifecycle: triggered → ack → investigate → mitigated → resolved → post-mortem-published. Plus every page issued + every status-page update + every role assignment.

This evidence supports SOC 2 CC7.3 / ISO 27001 A.5.24 / NIST SP 800-61r2 audit requirements.

## What you've done

A full SEV-1 incident from detection to post-mortem with:
- Automated paging across multiple channels.
- War-room auto-creation.
- Customer status-page communication.
- Mitigation via runbook-guided rollback.
- Customer-impact estimation.
- Blameless post-mortem with specific action items.
- Cryptographic audit-trail for regulator evidence.

## Common pitfalls

| Pitfall | Mitigation |
|---|---|
| IC + mitigation-lead being the same person | Designate different roles; the IC coordinates while the mitigation-lead executes |
| First status-page update > 15 min after detection | Configure the war-room template to remind the comms-lead at 10 min if no status-page post yet |
| Action items without owners + due dates | Substrate enforces these fields; cannot publish a post-mortem with vague action items |
| Closing incident before mitigation is verified | Use the "monitoring" intermediate state for 30 min before transitioning to "resolved" |
| Blame-language in post-mortem ("Alice made an error") | Re-frame as system ("the deploy checklist didn't include circuit-breaker changes") |
