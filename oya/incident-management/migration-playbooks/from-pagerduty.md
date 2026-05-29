---
doc_class: MigrationPlaybook
microservice: incident-management
source_vendor: PagerDuty
related_adrs: [ADR-0316, ADR-0251]
date: 2026-05-20
doc_status: published
---

# Migration Playbook — PagerDuty → oyatie incident-management

Audience: an SRE/engineering ops team currently on PagerDuty Business or Digital Operations tier who wants to move to oyatie's substrate over 6-10 weeks.

Outcome: all services + escalation policies + on-call rotations migrated, paging continuous (no missed pages), historical incidents archived with audit-chain anchoring, PagerDuty decommissioned per minimum-commit.

## Phase 0 — discovery (week 1)

1. Inventory PagerDuty configuration via the PagerDuty API:
   ```sh
   pd-cli services list --json > services.json
   pd-cli escalation-policies list --json > policies.json
   pd-cli schedules list --json > schedules.json
   pd-cli teams list --json > teams.json
   pd-cli users list --json > users.json
   pd-cli integrations list --json > integrations.json
   ```
2. Inventory commercial exposure:
   - PagerDuty contract end date.
   - Per-responder pricing tier (Free / Starter / Business / Digital Ops / Enterprise).
   - Number of active responders (vs purchased seats).
   - Minimum-seat commit.
3. Identify migration priorities:
   - High-volume services (most pages/month) first.
   - SEV-1-critical services first.
   - Pack-bound services (KR-PIPA, EU NIS2) first.

Deliverable: `migration-plan.md` enumerating PagerDuty assets + target oyatie tier.

## Phase 1 — stand up oyatie + dual-paging prep (weeks 2-3)

1. Deploy oyatie incident-management IaC into the target cell.
2. Sign up for SMS providers in your target jurisdiction:
   - US: Twilio + Bandwidth.com.
   - EU: Vonage + Plivo.
   - KR (if pack-bound): NHN Cloud SMS + Kakao Talk Bizmessage.
3. Configure your channel preferences in oyatie: portal → Channels → "Provider preferences". Set primary + secondary + tertiary for SMS + voice.
4. Run the paging diagnostic: portal → Diagnostics → "Test all channels". Confirm each channel delivers within 15 s p99.

## Phase 2 — user migration (week 3)

For each PagerDuty user:
1. Verify the user exists in oyatie's `iam` µservice (most teams already have IAM, so users exist).
2. Bind the appropriate Cedar role: `incident::responder` for normal users, `incident::commander` for ICs, `oncall::scheduler` for ops leads.
3. Import the user's notification preferences (PagerDuty stores: phone, email, push, Slack handle) via:
   ```sh
   oya incident-management user-import \
     --source pagerduty \
     --input users.json
   ```

The substrate validates each user's notification channels via a "test page" before considering them migration-ready.

## Phase 3 — schedule + rotation migration (week 4)

PagerDuty schedules use the proprietary "layered schedule" model. oyatie uses a similar layered model with semantic differences.

For each PagerDuty schedule:
1. Document the layers (e.g. "Layer 1: Weekly rotation primary, members [Alice, Bob, Charlie]; Layer 2: Weekend coverage, members [Dave]").
2. Recreate in oyatie:
   ```sh
   oya incident-management schedule-create \
     --name "Platform Team Primary" \
     --timezone "America/Los_Angeles" \
     --layer 'rotation:weekly,members:alice@,bob@,charlie@,handoff:Monday 09:00' \
     --layer 'rotation:weekly,members:dave@,days:saturday,sunday'
   ```
3. Verify with a 4-week look-ahead:
   ```sh
   oya incident-management schedule-preview --schedule platform-primary --days 28
   ```
   Compare against the equivalent PagerDuty schedule. Tune until they match.

## Phase 4 — escalation policy migration (week 5)

For each PagerDuty escalation policy:
1. Document the levels + timeouts.
2. Recreate in oyatie:
   ```yaml
   # microservices/incident-management/escalation-policies/platform-sev1.yaml
   name: "platform-sev1"
   levels:
     - timeout_minutes: 2
       targets:
         - schedule: platform-primary
         - schedule: platform-secondary
     - timeout_minutes: 3
       targets:
         - schedule: platform-em
     - timeout_minutes: 5
       targets:
         - user: cto@your-tenant.com
   ```
3. Commit + deploy:
   ```sh
   oya incident-management policy-apply --file escalation-policies/platform-sev1.yaml
   ```

## Phase 5 — service + integration migration (week 6)

For each PagerDuty service:
1. Identify the monitoring tools currently sending alerts to PagerDuty (Datadog / New Relic / Prometheus / etc).
2. Create the oyatie equivalent service:
   ```sh
   oya incident-management service-create \
     --name "cloud-billing-tax-app" \
     --escalation-policy platform-sev1 \
     --runbook-url "https://github.com/oyatie/oyatie/blob/dev/microservices/cloud-billing-tax-app/runbooks/availability-breach.md"
   ```
3. Reconfigure the monitoring tool to dual-fire (PagerDuty + oyatie) for 2 weeks. Compare paging behaviour.

## Phase 6 — historical incident archive (week 7)

PagerDuty's incident-history API allows bulk export:
```sh
pd-cli incidents list --since 2023-01-01 --until $(date +%Y-%m-%d) --json > incidents-history.json
```

Import to oyatie:
```sh
oya incident-management incident-archive-import \
  --source pagerduty \
  --input incidents-history.json \
  --target-retention 7y \
  --emit-audit-chain-anchor true
```

The import:
- Creates read-only incident records in oyatie.
- Cross-emits Merkle anchors to audit-chain.
- Preserves PagerDuty incident IDs for back-reference.
- Imports any attached post-mortem documents (if PagerDuty's Postmortems feature was used).

## Phase 7 — cutover (week 8)

1. Disable PagerDuty's notification delivery (don't delete the account yet; just stop paging).
2. oyatie becomes the sole paging path.
3. Monitor for 7 days. Verify: paging continues, no missed pages, on-call schedules functioning, escalation policies fire as expected.
4. If issues: re-enable PagerDuty as a safety net; debug oyatie config.

## Phase 8 — PagerDuty wind-down (weeks 9-10)

1. Cancel PagerDuty contract per minimum-commit.
2. Receive final invoice.
3. Update tenant ARCHITECTURE.md to reference oyatie exclusively.

## Common pitfalls

| Pitfall | Mitigation |
|---|---|
| PagerDuty's "anyone in rotation" semantics don't match oyatie's "first responder ack stops paging" | Configure oyatie escalation level with multiple targets in the SAME level; first-ack stops paging |
| PagerDuty Stakeholder users (read-only) | Map to oyatie `incident::observer` Cedar role |
| PagerDuty's Slack integration (rich Slack messages) | oyatie Slack integration is equivalent; provision via portal → Integrations → Slack |
| PagerDuty's email-to-incident integration | oyatie supports via `incident-email-ingest` integration; configure the dedicated email address per service |
| Custom PagerDuty webhooks | Reimplement as oyatie incident-state webhooks; oyatie's webhook payload is FHIR-Subscription-shaped |
| PagerDuty's Event Intelligence (group similar alerts) | oyatie's similar feature is "Incident Fingerprinting" (paid tenant_class); group rules are different — review per service |
| On-call schedules with complex layered shifts that PagerDuty handled implicitly | The oyatie layered model is explicit; you must encode each layer. Take 1-2 days per complex schedule for verification |
