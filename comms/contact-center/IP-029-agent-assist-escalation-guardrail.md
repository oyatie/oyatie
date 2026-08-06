---
doc_class: ImplementationPlan
ip_id: IP-029-agent-assist-escalation-guardrail
microservice: contact-center
related_adrs: [ADR-0243, ADR-0255, ADR-0263, ADR-0321]
journey_id: J-CC-29-agent-assist-safe-escalation
status: proposed
date: 2026-05-20
owner: axis-contact-center
availability: paid
---

# IP-029: Agent Assist Escalation Guardrail

## Context

This net-new slice controls AI/knowledge suggestions and escalation during live interactions. It displaces Genesys Agent Assist, NICE Enlighten, Five9 Agent Assist, Talkdesk Copilot, and AWS Wisdom/Contact Lens by keeping suggestions advisory and Cedar-gated.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `contact_agent_assist_suggestion` | `suggestion_id` | `uuid primary key` | One generated or retrieved suggestion. |
| `contact_agent_assist_suggestion` | `tenant_id` | `uuid not null` | Tenant partition. |
| `contact_agent_assist_suggestion` | `interaction_id` | `uuid not null` | Live interaction. |
| `contact_agent_assist_suggestion` | `suggestion_kind` | `text not null` | `knowledge_article`, `script`, `escalation`, `compliance_warning`. |
| `contact_agent_assist_suggestion` | `confidence_bps` | `integer not null` | 0-10000 confidence. |
| `contact_agent_assist_suggestion` | `accepted_by_agent` | `boolean` | Nullable until action. |

## API Endpoints

REST `POST /v1/contact-center/interactions/{interaction_id}:suggest-next-action`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-cc000001",
  "interaction_id": "018f8ad2-cc-int-29",
  "agent_principal_id": "User::agent.8",
  "conversation_summary_ref": "transcript_summary:443",
  "allowed_kinds": ["knowledge_article", "escalation"]
}
```

gRPC `AgentAssistGuardrail.SuggestNextAction(SuggestNextActionRequest)` returns suggestions plus `guardrail_decision_id`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `Service::"agent-assist"` | `contactCenter::SuggestNextAction` | `Interaction::*` | `tenant_id`, `interaction_id`, `data_classes`, `allowed_kinds` |
| `User::"agent"` | `contactCenter::AcceptAssistSuggestion` | `AgentAssistSuggestion::*` | `suggestion_kind`, `confidence_bps`, `customer_visible` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Genesys Agent Assist Recommendation | `AgentAssistSuggestion` | recommendation maps to suggestion with source ref. |
| NICE Enlighten Suggestion | `AgentAssistSuggestion` | score maps to confidence bps. |
| Five9 Agent Assist Card | `AgentAssistSuggestion` | card type maps to suggestion kind. |
| Talkdesk Copilot Prompt | `AgentAssistSuggestion` | prompt output maps to advisory text. |
| AWS Wisdom Result | `AgentAssistSuggestion` | knowledge result maps to article ref. |

## Workflow Steps

1. `ClassifyConversationData` blocks restricted transcript fields.
2. `CallIntelligenceLayer` uses library-first dispatch where available.
3. `EvaluateGuardrail` checks customer-visible and escalation actions.
4. `PersistSuggestion` writes suggestion row.
5. `RecordAgentDecision` stores accept/reject evidence.

Branches: low confidence returns knowledge-only; regulated data blocks external model path; escalation suggestion requires supervisor route.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-CONTACT-CENTER-ASSIST-SUGGESTED` | `tenant_id`, `interaction_id`, `suggestion_kind`, `confidence_bps` |
| `EVT-CONTACT-CENTER-ASSIST-ACCEPTED` | `suggestion_id`, `agent_principal_id`, `customer_visible` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Suggest next action | 120 ms | 850 ms | 1.8 s | 1k suggestions/s/cell | 99.9% |
| Accept suggestion | 15 ms | 70 ms | 140 ms | 3k accepts/s/cell | 99.95% |

## Failure Modes + Recovery

- Intelligence unavailable: return deterministic knowledge search only.
- Guardrail denies customer-visible text: redact suggestion and log deny.
- Transcript summary missing: generate no suggestion and request summary refresh.

## Migration Notes

Vendor AI suggestions are imported as historical evidence only; Oyatie does not replay model output as policy-approved guidance unless it passes current guardrails.

## Cross-µservice Handoffs

- `intelligence` provides suggestions under audience/data-class tags.
- `knowledge` supplies article references.
- `policy-engine` evaluates guardrails.
- `audit-chain` seals suggestion and acceptance.
- `workflow-engine` receives escalation route requests.
