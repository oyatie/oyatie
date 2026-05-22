---
ip_id: IP-014
title: "IP-014: Compliance + critical-path edge-case wiring"
microservice: connect
bounded_context: cross-cutting
layers: [compliance]
acceptance_status: design-ready
date: 2026-05-20
related_adrs: [ADR-0244, ADR-0250, ADR-0251, ADR-0272, ADR-0276, ADR-0292, ADR-0296, ADR-0297]
companion_docs:
  - microservices/connect/compliance.md
  - microservices/connect/dpia.md
  - microservices/connect/policy/data-residency.md
doc_status: published
---

# IP-014: Compliance + critical-path edge-case wiring

## Purpose

Wire the compliance invariants from the keystone bundle 2026-05-20 into connect's runtime behavior: detection substrate binding, insider-threat controls, breach notification workflow, minor protection, key rotation cadence, crypto agility, and all §3.2.5 critical-path edge-case rows applicable to connect.

## §3.2.5 rows applicable to connect

| Row | Critical path | Connect handling |
|---|---|---|
| 1 | Emergency services | PagerDuty connector: never throttled, never circuit-opened for triggerIncident; bypass at edge WAF; elevated rate-limit floor |
| 6 | Whistleblower | Ethics-report connector (Vault platform, WhistleB, etc.): if enabled, submit path anonymized; sealed-sender; no IP log; per-pack ADR-0300 |
| 8 | DV-survivor | Incognito connector mode: connector wiring hidden from shared-tenant admin view; audit visible only to privacy team + survivor-designated contact |
| 11 | Disability | Assistive-tech connectors (screen-reader, voice-control): bypass CAPTCHA on connector-catalog browse; a11y-crawler audience type |
| 27 | Bug-bounty | `/.well-known/security.txt` served by catalog REST; abuse-report route per RFC 9116; security researcher allow-list |
| 28 | Delegated agent | IFTTT/n8n/Zapier-class workflows acting for human: `delegated_agent_token` model; tenant-attested delegation chain; audit chains to authorizing human |

## detection substrate binding (§3.2.6.A)

Connect contributes to detection families:
- **Family 2 (ATO)**: `WebhookSignatureVerifyFailed` spike → potential credential stuffing signal
- **Family 8 (Policy violation)**: Cedar permit denial → `PolicyViolationDetected`; cross-tenant access attempt → immediate block + signal

## insider-threat controls

- Connector-adapter-worker Kata isolation: no insider can access another tenant's credentials from the same pod
- OpenBao per-tenant path isolation: per `openbao-policy.hcl`; no cross-tenant reads
- Audit-event signing: every audit event signed by per-µservice Ed25519 key (ADR-0296 sidecar); no insider forgery possible

## key rotation cadence

| Secret | Rotation cadence | Mechanism |
|---|---|---|
| OAuth refresh tokens | On vendor schedule or ≤90d | Sidecar auto-rotation |
| Webhook signing secrets | ≤90d or on revocation | `webhook-receiver-domain::rotate_signing_secret()` |
| Audit-event signing key | ≤90d | External Secret Operator + OpenBao |
| TLS certificates | ≤90d | cert-manager + External Secret |
| ECH keys | ≥90d default | Per `docs/runbooks/cedar-fragment-emergency-rollback.md` cadence |

## crypto agility plan

- Current: TLS 1.3, AES-256-GCM, HMAC-SHA-256, Ed25519, X25519
- PQC hybrid: X25519MLKEM768 + ed25519+ml_dsa_65 (offered; degrade silently for non-PQ clients)
- Migration path: when NIST finalizes ML-KEM-768 → migrate from hybrid to pure PQC via ADR amendment (no code re-arch needed; only key-gen + TLS config change)

## Acceptance criteria

1. `compliance.md §critical-path-edge-cases` updated with rows 1/6/8/11/27/28.
2. `compliance.md §detection-substrate-binding` updated with families 2 + 8.
3. `compliance.md §insider-threat-controls` section authored.
4. `compliance.md §key-rotation-cadence` table matches secrets in `openbao-policy.hcl`.
5. `compliance.md §crypto-agility-plan` section authored.
6. CI lane `oya-governance-critical-path-coverage` green.

## Definition of done

- [ ] All `compliance.md` sections updated
- [ ] `oya-governance-adr-adherence-matrix` lane passes for connect (52/52 rows)


## A. Problem
`IP-014: Compliance + critical-path edge-case wiring` closes a concrete `connect` integration-substrate gap, not a generic planning slot. The issue is that connector behavior spans catalog metadata, OAuth or webhook trust, vendor rate limits, DLQ replay, policy decisions, and SLO evidence; a short line-count shell cannot prove those boundaries. Domain vocabulary for this IP: ConnectorCatalog, OAuthBrokerService, WebhookReceiverService, ConnectorAdapter, DLQ, provider-BYOK, per-tenant webhook DNS, vendor rate-limit profile.

## B. Approach
Compliance wiring: critical-path rows, detection families, insider-threat controls, key rotation, and crypto agility become explicit service gates rather than narrative claims. The implementation remains substrate-only: `workflow-engine` orchestrates, while `connect` supplies connector directory, credential broker, webhook receive, adapter invocation, mapping, retry/DLQ, and audit evidence.

## C. Deliverables
- `microservices/connect/PRD.md` — concrete artifact to verify or update.
- `microservices/connect/ARCHITECTURE.md` — concrete artifact to verify or update.
- `microservices/connect/contracts/openapi/connect-integration.yaml` — concrete artifact to verify or update.
- `microservices/connect/contracts/proto/connect_integration.proto` — concrete artifact to verify or update.
- `microservices/connect/contracts/asyncapi/connect-integration-events.yaml` — concrete artifact to verify or update.
- `microservices/connect/policy/connector-authorization.cedar` — concrete artifact to verify or update.
- `microservices/connect/slos/connector-availability.openslo.yaml` — concrete artifact to verify or update.
- `microservices/connect/competitor-parity-matrix.md` — concrete artifact to verify or update.
- `microservices/connect/compliance.md` — concrete artifact to verify or update.
- `microservices/connect/dpia.md` — concrete artifact to verify or update.
- `microservices/connect/policy/data-residency.md` — concrete artifact to verify or update.
- `microservices/connect/policy/abuse-defence.cedar` — concrete artifact to verify or update.
- Declared Rust crates/types such as `ConnectorCatalog`, `OAuthBrokerService`, `WebhookReceiverService`, `ConnectorAdapter`, or `DlqService` must be added only by implementation PRs that also add tests; this documentation scrub does not fake source existence.

## D. Implementation Steps
1. Confirm the bounded-context row in `microservices/connect/PRD.md` and the retirement/substrate boundary in `microservices/connect/ARCHITECTURE.md`.
2. Trace each public command or event to `contracts/openapi/connect-integration.yaml`, `contracts/proto/connect_integration.proto`, or `contracts/asyncapi/connect-integration-events.yaml`.
3. Check the relevant Cedar policy before adding publish, OAuth, webhook, invoke, replay, or catalog mutation behavior.
4. Bind credentials through `iac/openbao-policy.hcl` and never through raw tenant tokens in docs, tests, or examples.
5. Attach an SLO, dashboard, runbook, or audit-event class for every failure mode named in this IP.
6. Run the IP-specific cargo/gate/contract/load command when source exists; otherwise record the missing crate as implementation debt.

## E. Acceptance
- Artifact links above resolve in this checkout.
- Vendor-specific probes include at least one real connector catalog entry, not a hypothetical vendor.
- Credential, webhook, and DLQ paths have policy plus audit evidence before runtime claims.
- The counterpart matrix row is updated when parity changes.

## F. Evidence
- `microservices/connect/PRD.md`
- `microservices/connect/ARCHITECTURE.md`
- `microservices/connect/contracts/openapi/connect-integration.yaml`
- `microservices/connect/contracts/proto/connect_integration.proto`
- `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`
- `microservices/connect/policy/connector-authorization.cedar`
- `microservices/connect/slos/connector-availability.openslo.yaml`
- `microservices/connect/competitor-parity-matrix.md`
- `microservices/connect/compliance.md`
- `microservices/connect/dpia.md`

## G. Counterparts
| Counterpart pressure | Oyatie closure for this IP |
|---|---|
| Zapier, n8n, Workato, Boomi, MuleSoft, Tray.io, Pipedream, AWS EventBridge, Stripe, Salesforce, Slack, GitHub, GitLab, HubSpot, Notion, Linear, Snowflake, and Twilio | MuleSoft/Boomi enterprise controls set the governance bar; Twilio/PagerDuty emergency paths test critical-path exceptions. This IP binds `014 compliance critical path` to concrete connect contracts, catalog records, policies, SLOs, runbooks, and IaC instead of a reusable stamp. |
