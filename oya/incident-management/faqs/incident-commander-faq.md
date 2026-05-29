---
doc_class: FAQ
microservice: incident-management
persona: incident-commander
related_adrs: [ADR-0316, ADR-0251]
date: 2026-05-20
doc_status: published
---

# incident-management — Incident Commander FAQ

## Q1: What's the difference between SEV-1 / SEV-2 / SEV-3 / SEV-4?

- **SEV-1**: Customer-facing outage or severe degradation. Multiple customers affected OR a critical customer affected OR data loss / security breach risk. Examples: payment processing down, primary login broken, ePHI exposed.
- **SEV-2**: Single-customer outage OR widespread non-critical degradation. Examples: a feature flag broken for one tenant, search latency 5× normal for all tenants.
- **SEV-3**: Internal-only impact OR minor customer issue with workaround. Examples: a non-critical dashboard broken, a batch job slow but completing eventually.
- **SEV-4**: Cosmetic, low-urgency, or known issue. Examples: a typo in a UI label, a slow query that doesn't impact customers, a deprecated API still being called.

The substrate auto-classifies based on alert signals (error rate, latency, customer-impact estimator) but the IC has the authority to upgrade or downgrade. Bias toward upgrading early — it's easier to downgrade a SEV-1 to SEV-2 than to recover from initially mistreating a SEV-1 as a SEV-3.

## Q2: How fast does paging actually arrive? 15 s p99 sounds slow for SEV-1.

The 15 s p99 is end-to-end from "alert ingest into the incident-management µservice" to "first delivery attempt acknowledged by the carrier". The breakdown:
- Alert ingest → state-machine eval: 200 ms p99.
- State eval → escalation policy lookup: 100 ms p99.
- Policy lookup → channel routing decision: 50 ms p99.
- Channel routing → carrier API (Twilio/Bandwidth/Slack): 2-10 s p99 (carrier-dependent).
- Carrier ack → delivery acknowledged: 1-5 s p99 (varies by destination phone/device).

For SEV-1, the substrate fires ALL configured channels in parallel (SMS + voice + Slack + email + Telegram). The fastest channel wins; on-call typically sees a Slack DM in 3 s and an SMS in 6 s. Voice takes longer (~ 10-15 s) because the carrier dials.

If your tenant needs sub-5-s p99 paging (rare; usually a financial or healthcare requirement), promote to paid tier where the multi-provider parallel routing reduces the tail latency.

## Q3: Can I integrate alerts from Datadog / Splunk / New Relic instead of the oyatie observability µservice?

Yes. Portal → Integrations → "Alert sources". Configure webhook receivers for any external monitoring tool. Each integration includes a mapping from the source tool's alert payload to oyatie's incident schema (service, severity, summary, runbook URL).

Built-in integrations: Datadog, New Relic, Splunk, Sentry, Honeycomb, Rollbar, Bugsnag, AWS CloudWatch, Google Cloud Monitoring, Azure Monitor, Grafana, Prometheus Alertmanager.

For tools without a built-in integration, use the generic webhook endpoint: `https://incident-webhook.<tenant>.oyatie.io/<integration-id>` with HMAC signing.

## Q4: Our team uses Slack DMs and channel pings. Can the substrate respect "do not disturb" hours for non-SEV-1 pages?

Yes via on-call rotation policy. Configure per-rotation:
- "Always page" (default): on-call gets pinged regardless of time.
- "Quiet hours": for SEV-3/SEV-4 only, suppress during configured hours (e.g. 22:00-06:00 local).
- "SEV-1-only override": only SEV-1 pages override quiet hours.

The substrate respects the rotation member's local timezone (auto-detected from their IAM profile).

## Q5: Can multiple people be paged simultaneously for the same incident?

Yes — this is the "war room paging" pattern for SEV-1. Configure the escalation policy:
- Level 1: page primary on-call + IC + EM simultaneously.
- All three have 5 min to ack; first ack stops further paging on that level.
- Level 2 only fires if NO ONE acks within 5 min.

For multi-team incidents (e.g. an incident spanning database + network), use the "fan-out" pattern: a single trigger fires pages to multiple rotations in parallel.

## Q6: Our incident took 4 hours to resolve. The post-mortem says action items are "Improve monitoring" + "Better runbook". Is that good enough?

No. "Improve monitoring" is not actionable. Convert to:
- "Add Prometheus alert for the metric X with threshold Y, owned by Alice, due 2026-06-15. Verified by triggering a synthetic breach + receiving the page."
- "Author runbook for failure class Z (the one we hit), owned by Bob, due 2026-06-08. Verified by 2 other team members reading + following it for a simulated breach."

Every action item must be: Specific (what), Measurable (verified by), Assigned (to whom), Realistic (achievable in the time), Time-bound (when). The substrate's action-item tracker enforces these fields; you cannot save an action item without all five.

## Q7: What's the customer-impact estimator at paid tier? How accurate is it?

The estimator cross-emits to `cloud-billing-tax-app` + `cloud-finops-api` + the service's traffic metrics to compute: (a) number of customers affected (by tenant_id + error rate), (b) duration of impact (from alert start to mitigation), (c) estimated financial impact (lost transactions × average transaction value + SLA-credit exposure).

Accuracy: within ± 30 % for typical incidents (validated against 12 months of incident post-mortem reconciliations against actual SLA credits issued). The estimator under-estimates for incidents that affect multiple downstream products (cascading impacts); for those, the IC should manually adjust the post-mortem's customer-impact figure.

The estimator is advisory; the substrate doesn't auto-issue SLA credits based on its output. Credits are issued via the `cloud-billing-tax-app` µservice with explicit human approval.

## Q8: Can I integrate with our customer-support tool (Zendesk / Intercom / Front)?

Yes. Configure a Zendesk/Intercom/Front integration: portal → Integrations → "Customer support". When an incident is declared, the substrate:
- Creates a high-priority macro/template in your support tool to triage customer tickets.
- For SEV-1: pins an in-product banner via the `community` µservice or your front-end status integration.
- Tracks support-ticket-volume during the incident as a signal of customer awareness.
- During post-mortem: links the related support tickets for retroactive sentiment + impact analysis.

## Q9: A regulator (FSC, FCC, FERC, etc) asks for our incident history over the last 3 years. How do I produce it?

Portal → Reports → "Regulator evidence export". Filter by severity, date range, service, customer impact. Export format:
- JSON: every incident with full lifecycle (triggered, ack, investigate, mitigate, resolve), pages issued, escalation path, war-room transcript (Slack/Discord/Telegram archive), post-mortem.
- PDF: human-readable narrative per incident.
- Audit-chain Merkle proofs for cryptographic verification.

The regulator can verify integrity independently. SLA: ≤ 24 h for ≤ 1 000 incidents; longer for larger ranges.

## Q10: Our pack is KR-PIPA-Finance. Are there special incident-disclosure requirements?

Yes. The KR-PIPA-Finance pack overlay enforces:
- Any SEV-1 incident affecting customer data triggers an FSC 정보처리시스템 사고통보 within 24 hours per 전자금융감독규정 § 73.
- Any incident with PII exposure triggers a KR-PIPA Art. 34 breach-notification (data subjects within 72 hours; PIPC within 72 hours).
- The substrate auto-drafts the FSC + PIPC notification using the incident-state machine + post-mortem evidence; your tenant's compliance officer signs + submits.
- Post-mortem retention 7 years minimum.

The substrate refuses to close a SEV-1 incident in the KR-PIPA-Finance pack until either (a) FSC notification confirmed sent, or (b) tenant compliance officer marks "no notification required" with a Cedar-permitted justification.

## Q11: How do I integrate the substrate with our existing on-call tool (we still use PagerDuty internally)?

Two patterns:
1. **Mirror to PagerDuty**: oyatie incidents fire pages via the PagerDuty Events API in parallel with oyatie's native paging. Useful during migration; eventually deprecate.
2. **Webhook bridge**: PagerDuty webhook fires → oyatie ingests as an alert → oyatie's state machine becomes canonical. Useful if you want PagerDuty for legacy reasons but oyatie for the post-mortem + customer-impact estimator.

The substrate doesn't recommend long-term dual-tool operation (introduces drift, increases cost). Pick one canonical source; mirror only during migration.

## Q12: A team disagrees on whether to declare an incident. What's the rule?

Bias toward declaring. The cost of a false-positive declared-incident is low (some Slack noise, a 15-min meeting); the cost of a false-negative un-declared incident is high (customer pain + delayed response). The substrate's IC playbook recommends: "If you're not sure if it's an incident, it's an incident — declare it as SEV-3, investigate for 10 min, then upgrade/downgrade/close."

For chronic disagreements between teams (e.g. between SRE and dev about whether a slow query counts as an incident), introduce explicit SLOs (via the `observability` µservice). If the SLO is breached, it's an incident by construction. If the SLO isn't breached, it's not an incident — file a bug ticket but don't declare.
