---
microservice: connect
doc_class: Phase
phase_id: PHASE-01
milestone: M01-foundation
date: 2026-05-20
owner_team: axis-integration
status: Accepted
related_adrs: [ADR-0056, ADR-0105, ADR-0145, ADR-0245, ADR-0246, ADR-0249, ADR-0255, ADR-0263, ADR-0294, ADR-0295, ADR-0296, ADR-0297]
companion_docs:
  - microservices/connect/PRD.md
  - microservices/connect/ARCHITECTURE.md
doc_status: published
---

# PHASE-01 — Integration Substrate Foundation

## Intent

Stand up the connect µservice as the canonical integration substrate: 8 BCs × 8 layers (kernel/domain/usecase/api/adapter/rest/grpc/worker) = 64 base crates, plus 30 seed connector adapters, plus the OAuth broker + webhook receiver runtime, plus abuse-defence wiring (ADR-0297), plus full Cedar gate roster.

## Scope

In:
- 8 BCs across 8 layers
- 30 seed connector adapters (top-30 by demand: Slack, Salesforce, Stripe, Shopify, GitHub, AWS Lambda/S3/SQS/SNS, Notion, Linear, Jira, GitLab, Twilio, SendGrid, Mailgun, HubSpot, Mongo, Postgres, Toss Payments, KakaoPay, Datadog, Sentry, Mixpanel, Segment, Amplitude, GA4, Algolia, Pinecone, Snowflake, BigQuery)
- Cedar policies (≥7 fragments)
- OpenAPI 3.2.0 + AsyncAPI 3.1.0 + proto3 contracts
- 7 SLOs
- Helm chart + Terraform module + OpenBao policy + network policy
- Abuse-defence baseline per ADR-0297
- Audit-chain event emission per ADR-0263

Out:
- The remaining 470 connector adapters (M02 + M03)
- Marketplace publishing flow (depends on marketplace µservice GA)
- AI-assisted data-mapping (depends on intelligence µservice GA)

## Acceptance gates

1. All 64 base crates compile + nextest pass.
2. All Cedar fragments pass `oya-governance-cedar-baseline`.
3. OpenAPI conformance lane green.
4. AsyncAPI channel validation lane green.
5. Top-30 connector adapters: real-vendor-sandbox integration tests pass.
6. SLO targets met on staging cell for 7d soak.
7. Abuse-defence CI lanes green: `oya-governance-anti-{bot,spoof,scrape}-coverage` + `oya-governance-abuse-defence-ux-floor`.
8. Doc-rigor lane `oya-governance-doc-rigor` PASS for every artifact in this µservice.

## Sequencing

Per ADR-0217 vertical-slice rollout order:
1. **Slice A** — `connector-catalog` BC end-to-end (kernel → rest); enables catalog browse.
2. **Slice B** — `oauth-broker` BC end-to-end; enables grant lifecycle.
3. **Slice C** — `webhook-receiver` + `signature-verification` BCs; enables inbound webhooks.
4. **Slice D** — `connector-adapter` + `retry-and-DLQ` BCs; enables outbound action invocation.
5. **Slice E** — `data-mapping` BC; enables visual mapper.
6. **Slice F** — 30 seed adapter implementations; tests against real vendor sandboxes.

## Exit criteria

Phase ends when all 64 crates ship to `staging` cell with all CI lanes green for 7d. Promotion to `production` per ADR-0139 agentic SLO-gated promotion.

## References

- IP-001 through IP-016 in this directory
- ADR-0217 vertical-slice rollout order
- ADR-0139 agentic SLO-gated promotion
