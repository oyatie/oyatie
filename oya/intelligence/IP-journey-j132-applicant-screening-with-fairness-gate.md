---
doc_class: Implementation-Plan
ip_id: IP-journey-j132-applicant-screening-with-fairness-gate
journey_ref: docs/user-journeys/j132-hr-mass-hiring-event-100-roles/
status: draft
date: 2026-05-20
microservice: intelligence
related_adrs: [ADR-0308, ADR-0311, ADR-0263, ADR-0247]
---

# IP — Intelligence's role in j132 AI-screening + EU-AI-Act fairness

## Scope

Intelligence runs the `applicant-screening-v2` scorer over the 1,040 received applications,
runs per-jurisdiction fairness audits (pre-deployment + post-deployment per EU-AI-Act),
produces per-applicant Article 86 explanations (6-year retention), and runs the post-hire
fairness audit at T+90d. Per ADR-0308 ML-lifecycle, the scorer is in stage=PRODUCTION
with active fairness monitoring; per ADR-0247, it runs as Foundry principal
`oyatie:foundry:scorer-applicant-screening-v2`.

## Acceptance criteria

1. `applicant-screening-v2` model registered in model-registry with conformity certificate ref.
2. Scoring API: `POST /intelligence/applicant-screen` batch + per-applicant.
3. Per-applicant output: match_score, top_3_strengths, top_3_concerns, confidence, fairness_band.
4. Per-applicant Article 86 explanation persisted (6-year retention).
5. Fairness audit produces a verdict (green/yellow/red) + per-jurisdiction findings.
6. EU-AI-Act pre-deployment conformity check ties to BNetzA certificate ID.
7. Post-hire fairness audit auto-runs at T+90d.
8. SLO: P95 per-applicant inference latency ≤ 350ms; sustained 1k/sec batch throughput.

## Atomic deliverables

| Step | Change | Verification |
|---|---|---|
| 1 | Register `applicant-screening-v2` in model-registry with metadata (training data hash, conformity cert) | model-registry test passes |
| 2 | Implement `POST /intelligence/applicant-screen` (single + batch) | T-301 passes |
| 3 | Implement per-applicant explanation persistence (Article 86 store) | T-305 passes |
| 4 | Implement fairness audit: 4/5ths rule + demographic-parity + age-distribution + sample-size | T-303 passes |
| 5 | Implement fairness verdict producer (green/yellow/red) | T-303, T-304 pass |
| 6 | Implement EU-AI-Act preflight: certificate validation | T-302 passes |
| 7 | Implement NY AEDT bias-audit summary export | T-104 integration pass |
| 8 | Implement post-hire fairness audit (T+90d trigger from workflow-engine) | T-601 passes |
| 9 | Add audit-event classes: IntelligenceApplicantScored, IntelligenceFairnessAuditCompleted, IntelligencePostHireFairnessAuditCompleted | Registry green |
| 10 | Add Cedar permits: applicant_screening.run, fairness_audit.run, post_hire_fairness.run, applicant_screening_explanation_read | Cedar tests pass |

## Model lifecycle (per ADR-0308)

- **stage**: PRODUCTION
- **conformity_cert**: BNetzA-EU-AI-ACT-2026-001-Marcus-Tenant (valid through 2026-12-31)
- **training_data_hash**: hash-pinned at registration
- **fairness_baseline**: green for 100% of registered jurisdictions
- **active_monitoring**: enabled (per-event fairness audit at use time)
- **retraining_cadence**: quarterly or on fairness-drift red

## Cedar permits authored

```cedar
// b2b.intelligence.applicant_screening.run.cedar
permit (
  principal,
  action == Action::"b2b.intelligence.applicant_screening.run",
  resource is JobApplicationBatch
) when {
  principal == User::"oyatie:foundry:scorer-applicant-screening-v2" ||
  principal.audience_type == "B2B_HR_ADMIN"
};
```

```cedar
// b2b.intelligence.fairness_audit.run.cedar
permit (
  principal,
  action == Action::"b2b.intelligence.fairness_audit.run",
  resource is FairnessAudit
) when {
  context.tenant.compliance_pack_active("pack-eu-ai-act-2026-baseline") &&
  context.eu_ai_act_conformity_certificate_valid == true
};
```

```cedar
// b2b.intelligence.applicant_screening_explanation_read.cedar
permit (
  principal,
  action == Action::"b2b.intelligence.applicant_screening_explanation_read",
  resource is ApplicantExplanation
) when {
  (principal.audience_type == "B2B_HR_ADMIN" && principal.tenant_id == resource.tenant_id) ||
  (principal == resource.applicant_principal_pseudo_id.resolve_to_real_principal())
};
```

```cedar
// b2b.intelligence.post_hire_fairness.run.cedar
permit (
  principal,
  action == Action::"b2b.intelligence.post_hire_fairness.run",
  resource is PostHireFairnessAudit
) when {
  principal == User::"oyatie:foundry:scheduler" ||
  principal.audience_type == "B2B_HR_ADMIN"
};
```

## API surfaces

### `POST /intelligence/applicant-screen`

- Body: applicant_batch (≤200/batch)
- Per-applicant: pseudo_id, resume_text (from Drive), req_metadata, jurisdiction
- Response: per-applicant {match_score, top_3_strengths, top_3_concerns, confidence, fairness_band, explanation_uri}
- Cedar: `b2b.intelligence.applicant_screening.run`
- Audit: IntelligenceApplicantScored ×N

### `POST /intelligence/fairness-audit`

- Body: event_id + audit_scope (jurisdiction filter)
- Response: AiScreeningFairnessAudit schema
- Cedar: `b2b.intelligence.fairness_audit.run`
- Audit: IntelligenceFairnessAuditCompleted

### `GET /intelligence/applicant-explanation/{pseudo_id}/{req_id}`

- Cedar: `b2b.intelligence.applicant_screening_explanation_read`
- Response: per-applicant Article 86 explanation
- Audit: ApplicantExplanationRead

### `POST /intelligence/post-hire-fairness-audit`

- Body: event_id
- Response: post-hire fairness report with hire-rate vs applicant-pool comparison
- Cedar: `b2b.intelligence.post_hire_fairness.run`
- Audit: IntelligencePostHireFairnessAuditCompleted

## Fairness rules

### 4/5ths rule (US EEOC)

- selection_rate(class) ≥ 0.8 × selection_rate(highest_class)
- Computed per-jurisdiction (US-only by default; extensible to other jurisdictions)
- Per-applicant inferred protected-class proxy from: name (Bayesian inference), university (proxy), self-disclosed (optional)

### Demographic parity (EU Anti-Discrimination Directive)

- |selection_rate(class_A) - selection_rate(class_B)| ≤ 0.10
- Per-jurisdiction (DE-BER, KR-SEO)

### Age distribution (US ADEA, applied to US-AUS only)

- selection_rate(age≥40) ≥ 0.7 × selection_rate(age<40)

### Sample-size handling

- If a req has < 10 applicants: yellow flag (cannot statistically determine)
- If a req has < 5 applicants: skip statistical test, mark `insufficient_sample_yellow`

### Verdict aggregation

- All-green per jurisdiction → green
- Any-yellow per jurisdiction → yellow (event-level)
- Any-red per jurisdiction → red (event-level), halts decisions

## Per-applicant Article 86 explanation shape

```json
{
  "explanation_id": "uuid",
  "applicant_pseudo_id": "pseu_xxx",
  "req_id": "REQ-IBLR-SWE-II-001",
  "model_ref": "oyatie:intelligence:model:applicant-screening-v2:v1.0.0",
  "match_score": 0.78,
  "top_3_strengths": ["5 yrs Rust experience", "Distributed systems at Google", "Published Substrate Engineering blog"],
  "top_3_concerns": ["No ML production experience", "Career break 2023-2024 unexplained", "Junior to expected level"],
  "confidence": "high",
  "input_features_used": ["resume_text_embeddings", "experience_years", "skills_extraction", "university_tier"],
  "input_features_NOT_used": ["name", "age", "gender_inferred", "ethnicity_inferred", "photo_inferred"],
  "decision_logic_summary_text": "Candidate scored high on Rust + distributed systems but lacks the specific ML production background the req requires. Match score reflects this trade-off.",
  "comparison_to_archetype": "Top-15% of received applicants in Rust skills; bottom-30% in ML production years.",
  "retention_until": "2032-05-26",
  "compliant_with": ["EU-AI-Act-Article-86", "US-ECOA-Reg-B"]
}
```

## Dependencies

- **model-registry** (per IP-008 of intelligence)
- **identity** (resolve pseudo_id ↔ real principal for explanation-read by applicant)
- **compliance** (conformity certificate lookup)
- **drive** (read resume_text from candidate's Drive ref)
- **audit-chain** (EmitSealed)

## Observability

| Metric | Type | Labels |
|---|---|---|
| `oya_intelligence_applicant_screening_latency_ms` | histogram | model_ref, jurisdiction |
| `oya_intelligence_applicant_screening_p99_ms` | histogram | model_ref |
| `oya_intelligence_fairness_audit_verdict_total` | counter | verdict, jurisdiction |
| `oya_intelligence_explanation_retrieval_latency_ms` | histogram | n/a |
| `oya_intelligence_post_hire_audit_completed_total` | counter | event_id |

## SLOs

- P50 per-applicant inference: 180ms
- P95: 350ms
- P99: 800ms
- Batch throughput: 1k/sec sustained per cell
- Fairness audit P95: 12s for 1,040 applicants
- Explanation retrieval P95: 5s

## Failure modes

| Failure | Recovery |
|---|---|
| Model load failure | Fall back to v1; if v1 also down, halt phase |
| Fairness audit FAIL on one jurisdiction | Per-jurisdiction yellow/red; event-level rolls up |
| Conformity certificate expired | Preflight FAIL; halt; alert compliance |
| Drive read failure for resume_text | Per-applicant skip; mark `inference_skipped_data_unavailable` |
| Audit-chain degraded | Local WAL per ADR-0028 |

## Migration / rollout

- Lane: intelligence-rollout-j132 on dev → staging → production
- Pre-roll: load model artifact + run baseline fairness audit
- Roll: enable feature flag `intelligence.applicant_screening_v2` for marcus-tenant
- Validate: 1 week, fairness verdict green
- Promote: enable for all B2B tenants

## Test gates

- T-301 through T-305
- T-104 (NY AEDT bias-audit summary)
- T-302 (preflight FAIL)
- T-601 (post-hire audit)
- T-602 (drift detection)

## Notes

- Per ADR-0247, the scorer runs as Foundry principal under Cedar permit.
- Per ADR-0308, model-registry is the source-of-truth for stage transitions.
- Per ADR-0311, the scorer does NOT consume candidate personal-tenant data; only Community-public-surface data + Drive (via candidate-supplied resume_ref in their personal tenant).
- The explanation persistence honors EU-AI-Act Article 86 §1c (rejected applicants can request human re-review within 30 days).
- The post-hire fairness audit is scheduled by workflow-engine; intelligence is the executor.

— end of IP —

## Journey execution rows — substance pass

| Journey row | Source trigger | Actor | Contract / Cedar probe | State effect | Evidence touch | Counterpart |
|---|---|---|---|---|---|---|
| Batch screen admitted | workflow-engine submits 1,040 applications | `FoundryAgent` `oyatie:foundry:scorer-applicant-screening-v2`; Cedar `b2b.intelligence.applicant_screening.run` admits HR admin or scorer principal | `POST /intelligence/applicant-screen` planned in this IP; existing fallback is `POST /dispatch` envelope with purpose=`hr.applicant_screen` | per-applicant score/explanation URI persisted | `IntelligenceApplicantScored` audit event and latency histogram | matches Workday/SAP SuccessFactors AI screening batch run |
| EU high-risk preflight | EU jurisdiction applicant batch present | `B2B_HR_ADMIN`; `eu-ai-act-high-risk.cedar` refuses category_4 employment without RFIA | `rfia_id` and `annex_iii_categories` fields in `DispatchEnvelope` | screening halts before model call when conformity missing | `dispatch.refused` reason `rfia_required` or Annex III reason | matches EU AI Act conformity gate in enterprise ATS |
| Fairness audit | pre-deployment or per-event fairness check requested | `B2B_HR_ADMIN` or compliance reviewer; Cedar `b2b.intelligence.fairness_audit.run` requires active EU pack certificate | `POST /intelligence/fairness-audit` planned; current eventing via `Eval.GetRecord` | green/yellow/red verdict; red halts decision handoff | `oya_intelligence_fairness_audit_verdict_total` | matches NYC AEDT / Greenhouse bias-audit evidence |
| Applicant explanation read | applicant requests Article 86 explanation | applicant principal resolving from pseudo_id or HR admin same tenant; Cedar `applicant_screening_explanation_read` restricts tenant/applicant | `GET /intelligence/applicant-explanation/{pseudo_id}/{req_id}` planned; audit readback via `GetAuditTapRecord` | explanation returns features used/not used and retention_until | `ApplicantExplanationRead` audit event | matches Workday candidate explanation export |
| Post-hire audit trigger | workflow-engine fires T+90d post-hire fairness event | `oyatie:foundry:scheduler`; Cedar `post_hire_fairness.run` admits scheduler or HR admin | `POST /intelligence/post-hire-fairness-audit` planned | hire-rate comparison appended to fairness report | `IntelligencePostHireFairnessAuditCompleted` event | matches SAP SuccessFactors post-hire adverse-impact review |
| Drive resume unavailable | candidate Drive ref cannot be read | `FoundryAgent` scorer; tenant and applicant scope still enforced; no cross-candidate fallback | dependency on Drive resume_text from IP dependencies | per-applicant `inference_skipped_data_unavailable` result | audit event with skipped applicant count | matches ATS skipped-document handling |
| Model/cert expiry | conformity certificate expired or model load fails | scorer principal; preflight fails closed; fallback v1 only if valid cert exists | model-registry/conformity_cert metadata in this IP | phase halt and compliance alert | `dispatch.failed`/fairness verdict metric | matches enterprise model registry promotion gate |
| Counterpart appeal packet | rejected candidate or regulator asks for review | `Auditor` scoped to tenant engagement; `auditor-scope.cedar` read-only scoped_tenants | `Dispatch.GetAuditTapRecord`, explanation URI, fairness verdict | packet includes score, factors, fairness verdict, model_ref, hashes | sealed audit-tap + explanation retention until 2032-05-26 | matches Workday/Greenhouse candidate appeal packet |

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/intelligence/IP-journey-j132-applicant-screening-with-fairness-gate.md` matched `SLO, p99`.
- Numeric target: `rto_p99_seconds=300`, `rpo_p99_seconds=60` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), EU-AI-ACT-2024-HIGH-RISK(1800s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-PIPA-2023-amendment(14400s/900s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/intelligence/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`, `microservices/intelligence/slos/streaming-throughput.openslo.yaml`, `microservices/intelligence/policy/abuse-defence.cedar`.
