---
doc_class: ImplementationPlan
ip_id: IP-055-chatflow
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0251, ADR-0263, ADR-0321, ADR-0328]
bounded_context: chatflow
journey_id: J-MA-55-conversational-bot-and-handoff
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-055: Chatflow

## Context

HubSpot Chatflows + Conversational Bots cover bot decision trees + live-agent handoff. Mailchimp and Marketo do not have first-class chatflow primitives. This slice owns the marketing-side chatflow with a hard PII-redaction boundary at handoff — counterparts ship full conversation transcripts to agents, which is a sharp surface for HIPAA / GDPR tenants.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_chatflow` | `chatflow_id` | `uuid primary key` | Chatflow id. |
| `marketing_chatflow` | `tenant_id` | `uuid not null` | Tenant. |
| `marketing_chatflow` | `decision_tree_dag` | `jsonb not null` | Bot DAG. |
| `marketing_chatflow` | `handoff_rules` | `jsonb not null` | When-to-handoff conditions. |
| `marketing_chatflow` | `pii_redaction_strategy` | `text not null` | strict / standard / off. |
| `marketing_chatflow` | `status` | `text not null` | draft / published / retired. |
| `marketing_chatflow_session` | `session_id` | `uuid primary key` | Per-conversation. |
| `marketing_chatflow_session` | `chatflow_id` | `uuid not null` | FK. |
| `marketing_chatflow_session` | `subject_hash` | `text not null` | Subject. |
| `marketing_chatflow_session` | `transcript` | `jsonb not null` | Bot exchange. |
| `marketing_chatflow_session` | `handed_off_to` | `text` | messenger / contact-center / null. |
| `marketing_chatflow_session` | `handoff_at_hlc` | `hlc` | HLC. |
| `marketing_chatflow_session` | `redacted_transcript_for_handoff` | `jsonb` | PII-redacted copy. |

## API Endpoints

REST `POST /v1/marketing-automation/chatflows` defines.

REST `POST /v1/marketing-automation/chatflows/{chatflow_id}/sessions/{session_id}:advance` advances the conversation.

REST `POST /v1/marketing-automation/chatflows/{chatflow_id}/sessions/{session_id}:handoff` triggers handoff.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.ops"` | `marketingAutomation::PublishChatflow` | `MarketingChatflow::*` | `tenant_class`, `pii_redaction_strategy` |
| `Service::"chatflow-runner"` | `marketingAutomation::AdvanceChatflow` | `MarketingChatflowSession::*` | `session_id` |
| `Service::"chatflow-runner"` | `marketingAutomation::HandoffChatflow` | `MarketingChatflowSession::*` | `handoff_destination`, `redaction_applied` |

For HIPAA-pack tenants, Cedar denies handoff unless `pii_redaction_strategy == 'strict'` and `redacted_transcript_for_handoff` is populated.

## Workflow Steps

1. `ValidateDecisionTreeDag`.
2. `ValidateHandoffRules`.
3. `Authorize` Cedar.
4. On session advance, walk decision tree; capture user input.
5. On handoff trigger, `ApplyPiiRedaction` per strategy (strict: redact all detected PII; standard: redact obvious PII; off: pass-through).
6. `RouteHandoff` to messenger or contact-center.
7. `EmitHandoff` event.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-CHATFLOW-DEFINED` | `chatflow_id`, `pii_redaction_strategy`, `tenant_class` |
| `EVT-MARKETING-CHATFLOW-HANDOFF-TRIGGERED` | `session_id`, `handed_off_to`, `redaction_applied`, `cedar_decision_id` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Advance turn | 80 ms | 400 ms | 1 s | 2000 rps/cell | 99.95% |
| Handoff | 150 ms | 700 ms | 2 s | 500 rps/cell | 99.9% |

## Cross-µservice Handoffs

- `messenger` accepts handoff for chat continuation.
- `contact-center` accepts handoff for live agent.
- `data-boundary` provides PII detection + redaction.
- `audit-chain` seals every handoff.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-055-chatflow.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-055-chatflow.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].
