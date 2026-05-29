---
doc_class: ImplementationPlan
ip_id: IP-049-seo-seam
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0263, ADR-0321, ADR-0328]
bounded_context: seo-seam
journey_id: J-MA-49-seo-recommendations-seam
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
delegation_destination: sites µservice
open_question_settled: Q-019
---

# IP-049: SEO Seam

## Context

HubSpot SEO Recommendations + Marketo SEO Tools provide keyword tracking, on-page recommendations, and SERP rank monitoring. Marketing-automation requires SEO metadata on landing pages but delegates SEO analysis to sites µservice per Q-019.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_seo_recommendation` | `rec_id` | `uuid primary key` | Recommendation id. |
| `marketing_seo_recommendation` | `tenant_id` | `uuid not null` | Tenant. |
| `marketing_seo_recommendation` | `subject_kind` | `text not null` | landing_page. |
| `marketing_seo_recommendation` | `subject_id` | `uuid not null` | FK. |
| `marketing_seo_recommendation` | `keyword` | `text` | Optional target keyword. |
| `marketing_seo_recommendation` | `recommendation_text` | `text not null` | Human-readable recommendation. |
| `marketing_seo_recommendation` | `severity` | `text not null` | info / warning / critical. |
| `marketing_seo_recommendation` | `acknowledged_at_hlc` | `hlc` | Operator acknowledgement. |
| `marketing_seo_recommendation` | `created_at_hlc` | `hlc not null` | HLC. |

## API Endpoints

REST `POST /v1/marketing-automation/seo/recommendations:request`:

```json
{"subject_kind": "landing_page", "subject_id": "...", "target_keyword": "marketing automation"}
```

REST `POST /v1/marketing-automation/seo/recommendations/{rec_id}:acknowledge`.

## Workflow Steps

1. `RequestSeoAnalysis` posts to sites µservice SEO seam.
2. `IngestRecommendation` writes row when sites responds.
3. `EmitReceived` emits event.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-SEO-RECOMMENDATION-RECEIVED` | `rec_id`, `subject_kind`, `subject_id`, `severity` |
| `EVT-MARKETING-SEO-RECOMMENDATION-ACKNOWLEDGED` | `rec_id`, `principal_id` |

## Cross-µservice Handoffs

- `sites` provides SEO analysis engine + keyword tracking + SERP monitoring.
- `landing-page` aggregate is the primary subject of recommendations.
- `audit-chain` seals events.
