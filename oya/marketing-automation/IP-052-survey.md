---
doc_class: ImplementationPlan
ip_id: IP-052-survey
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0251, ADR-0263, ADR-0321, ADR-0328]
bounded_context: survey
journey_id: J-MA-52-feedback-survey-distribution
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-052: Survey

## Context

HubSpot Feedback Surveys (CSAT / NPS / CES) + Marketo Survey Form + Mailchimp Survey are universal. Surveys differ from forms in lifecycle (one-shot per subject) and analytical interpretation (NPS bands, CSAT mean, CES sum).

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_survey` | `survey_id` | `uuid primary key` | Survey id. |
| `marketing_survey` | `tenant_id` | `uuid not null` | Tenant. |
| `marketing_survey` | `kind` | `text not null` | nps / csat / ces / custom. |
| `marketing_survey` | `questions` | `jsonb not null` | Array of typed questions. |
| `marketing_survey` | `disclosure_block` | `jsonb` | Per pack overlay. |
| `marketing_survey` | `audience_descriptor` | `jsonb not null` | Target audience. |
| `marketing_survey` | `status` | `text not null` | draft / live / closed. |
| `marketing_survey_response` | `response_id` | `uuid primary key` | Response row. |
| `marketing_survey_response` | `survey_id` | `uuid not null` | FK. |
| `marketing_survey_response` | `subject_hash` | `text not null` | Subject. |
| `marketing_survey_response` | `answers` | `jsonb not null` | Answer payload. |
| `marketing_survey_response` | `signed_token_verified` | `boolean not null` | Response token verification. |
| `marketing_survey_response` | `recorded_at_hlc` | `hlc not null` | HLC. |

## API Endpoints

REST `POST /v1/marketing-automation/surveys`.

REST `POST /v1/marketing-automation/surveys/{survey_id}:send` distributes via email.

REST `POST /v1/marketing-automation/surveys/{survey_id}/responses` records (public, signed-token-gated).

## Workflow Steps

1. `ValidateQuestions` for kind-specific shape (NPS: 0-10 scale + open text; CSAT: 1-5 + open text; CES: 1-7 + open text).
2. `Authorize` Cedar.
3. `Publish` transitions status to live.
4. On send, `MintResponseTokens` per subject (signed) and dispatch email with unique link.
5. On response, `VerifySignedToken` + `EnforceOneResponsePerSubject`.
6. `EmitResponse` emits event.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-SURVEY-CREATED` | `survey_id`, `kind`, `tenant_class` |
| `EVT-MARKETING-SURVEY-RESPONSE-RECORDED` | `survey_id`, `response_id`, `subject_hash` |

## Migration Notes

HubSpot Feedback Surveys export with question text + scale + responses; preserved. Marketo Survey Form export similar.

## Cross-µservice Handoffs

- `email` distributes the survey.
- `customer-analytics` runs NPS / CSAT / CES report.
- `audit-chain` seals events.
