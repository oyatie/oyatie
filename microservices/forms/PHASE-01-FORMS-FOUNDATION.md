---
doc_class: PhasePlan
microservice: forms
milestone: M03-workspace-tier-foundation
phase: P01-forms-foundation
status: pending
date: 2026-05-17
owner: axis-forms + council-architecture
related_artifacts:
  - microservices/forms/PRD.md
  - /specs/microservices/forms.json
related_adrs: [ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-FORMS-0001, ADR-FORMS-0002, ADR-FORMS-0003, ADR-FORMS-0004, ADR-FORMS-0005, ADR-FORMS-0006]
doc_status: published
---

# PHASE-01: forms foundation (M03-P01)

## Intent

Stand up the forms µservice from zero to GA-eligible foundation. This phase delivers:

1. Layer-A substrate (Postgres + Citus + Redis + Meilisearch + ClamAV + WAF + CDN + hCaptcha sidecar).
2. Core domain kernel (form / field / section / response / submission with type-safe schema).
3. Form-builder Leptos-WASM editor + form-renderer (Leptos-WASM + plain HTML fallback for accessibility).
4. Response-collector REST surface (anonymous + authenticated + pre-filled-link).
5. Cedar policy bundle (tenant + ci + auditor + public-read) with default-deny baseline.
6. ChangeSet integration for form-definition versioning (ADR-0110).
7. Audit-chain integration for submission seal.
8. Webhook delivery + bulk-distribute + export workers.
9. AI-form-build T2 capability (gated; opt-in only).
10. Per-pack overlays: pack-kr (launch) + scaffolds for pack-eu / pack-us / pack-us-healthcare / pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa.

## Exit Criteria (SLO-gated per ADR-0139)

All of:

- `oya-forms-form-render-latency` p95 ≤ 200ms over 7-day window — green.
- `oya-forms-field-validate-latency` p99 ≤ 50ms — green.
- `oya-forms-submission-latency` p95 ≤ 150ms — green.
- `oya-forms-analytics-render-latency` p95 ≤ 500ms — green.
- `oya-forms-bulk-distribute-latency` p95 ≤ 30s — green.
- `oya-forms-export-csv-latency` p95 ≤ 5s — green.
- `oya-forms-ai-form-build-latency` p95 ≤ 8s — green.
- `oya-forms-pii-encryption-correctness` = 100% — green.
- `oya-forms-accessibility-wcag-correctness` = 100% — green.
- Every AC-01..AC-28 in PRD passing in CI.
- pack-kr live with one external tenant + at least one HIPAA-tagged tenant for pack-us-healthcare smoke (BAA-required).
- DPIA + AI-Act-conformity signed by council-privacy + council-legal-compliance.

## Per-IP Test Coverage Threshold

| IP class | Coverage threshold | Notes |
|---|---|---|
| IaC (IP-001) | ≥ 1 helm-install + helm-test per chart | smoke on kind cluster |
| Domain kernel (IP-002..IP-005) | ≥ 90% line coverage; ≥ 100% on PII paths | property tests on conditional-logic |
| Adapter layers (IP-006..IP-009) | ≥ 80% line coverage; contract tests vs counterpart µservice | mock the counterpart in unit tests; integration in e2e |
| REST surface (IP-010..IP-012) | OpenAPI conformance ≥ 100%; round-trip on every endpoint | k6 load test before promotion |
| Worker (IP-013..IP-014) | property tests on idempotency + dead-letter | chaos test on bulk-distribute |
| App (IP-015) | end-to-end smoke; pack-kr launch tenant | golden-path + 5 failure-mode scenarios |

## Per-IP Sequence

| IP | Title | Layer focus |
|---|---|---|
| IP-001 | Layer-A IaC (Postgres + Citus + Redis + Meilisearch + ClamAV + WAF + CDN + hCaptcha + Turnstile + Friendly Captcha sidecar) | infra |
| IP-002 | Form / field / section / response domain kernel | kernel + domain |
| IP-003 | Conditional-logic engine (declarative DAG; ADR-FORMS-0004) | domain |
| IP-004 | Validation engine (per-field + cross-field; JSON Schema bridge) | domain |
| IP-005 | Versioning + ChangeSet binding (ADR-0110) | domain |
| IP-006 | Postgres adapter (Citus shard; column-level envelope encryption per ADR-FORMS-0003) | adapter |
| IP-007 | Redis adapter (rate-limit + session) | adapter |
| IP-008 | Meilisearch adapter (response search) | adapter |
| IP-009 | Captcha adapter (hCaptcha + Turnstile + Friendly Captcha) | adapter |
| IP-010 | Form-builder Leptos-WASM (authoring UI) | app/frontend |
| IP-011 | Form-renderer (Leptos-WASM + plain-HTML fallback) | app/frontend |
| IP-012 | Response-collector REST surface | rest/adapter |
| IP-013 | Bulk-distribute worker (Kafka-backed; back-pressured) | worker |
| IP-014 | Export worker (CSV / XLSX / JSON; streaming) | worker |
| IP-015 | HG-FORMS hyperscaler-gate registration + GA cutover | governance |

## Cross-product Boundaries (Workflow + Ontology adapter mandatory)

- Forms NEVER calls a sibling µservice directly; every cross-product flow goes through the Workflow + Ontology adapter pattern (per `feedback_workflow_objectgraph_adapter_layer.md`).
- Workflow-trigger on submission → workflow-engine: adapter via `oya-forms-workflow-trigger-adapter` (no direct workflow-engine REST call from form-rest).
- Response-bridge to sheets: adapter via `oya-forms-sheets-bridge-adapter`.
- File-upload backend to drive: adapter via `oya-forms-drive-upload-adapter`.

## Risk Register

| Risk | Mitigation |
|---|---|
| AI-form-build emits GDPR Art. 22 high-risk form (e.g., automated hiring filter) | ADR-FORMS-0005 high-risk classification + DPIA prompt + reviewer-agent gate |
| Per-tenant DEK rotation breaks at-rest encryption read-path | Quarterly chaos drill; rolling re-encryption with dual-key window |
| Captcha provider outage (hCaptcha) | Multi-provider fallback (Turnstile → Friendly Captcha → manual review queue) |
| Citus shard skew (mega-tenant 10M+ responses) | Per-tenant cell migration; ADR-0164 cell-pinning policy |
| Webhook delivery dead-letter blocks tenant workflow | DLQ with per-tenant cap; SLI on DLQ depth; ops runbook |
| WCAG 2.2 AA failure regression on a builder release | Pre-publish CI gate `oya-governance-wcag22-conformance`; blocking |

## Next Phase

PHASE-02-FORMS-AI-AND-ANALYTICS (T2 expansion, analytics dashboards, A/B testing, template marketplace expansion).

## References

- PRD.md.
- /specs/microservices/forms.json.
- ADR-0139 SLO-gated promotion.
- ADR-0131 per-microservice flat layout.
- ADR-0132 single-concern microservices.
- ADR-0133 compliance review cadence.
- ADR-FORMS-0001..0006.
