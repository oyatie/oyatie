---
doc_class: Implementation-Plan-Journey-Slice
journey_id: j144
microservice: intelligence
status: draft
date: 2026-05-20
authority_tier: 3
intern_buildable: true
adr_anchors: [ADR-0247, ADR-0255, ADR-0311]
---

# intelligence — IP slice for j144 (consumer-tier filter + drafter)

## Scope

1. **FilterSpec authoring** with closed-schema enforcement (no protected-characteristic fields).
2. **Filter.Apply** that classifies postings → strong/soft/blocked.
3. **CoverLetter.Draft** with transparency floor (model_id, prompt_template_hash, temperature, EU-AI-Act explainability record).
4. **Filter.Retrain** with locality enforcement (compute on personal-tenant only).
5. **Filter.Blacklist** for fraud-reporting integration with Community.
6. **Context.Bind** for résumé + portfolio indexing — local.

## API surface

```proto
service FilterSpec {
  rpc Author(AuthorRequest) returns (AuthorResponse);
  rpc Update(UpdateRequest) returns (UpdateResponse);
}

service Filter {
  rpc Apply(ApplyRequest) returns (ApplyResponse);
  rpc Retrain(RetrainRequest) returns (RetrainResponse);
  rpc Blacklist(BlacklistRequest) returns (BlacklistResponse);
}

service CoverLetter {
  rpc Draft(DraftRequest) returns (DraftResponse);
}

service Context {
  rpc Bind(BindRequest) returns (BindResponse);
  rpc Unbind(UnbindRequest) returns (UnbindResponse);
}
```

## Implementation tasks

### T1 — FilterSpec schema validation

Reject specs with non-empty `protected_characteristic_filters`. Reject specs missing required fields.

### T2 — Filter.Apply

- Compute relevance via consumer-tier model (smaller, local).
- For each posting: structured scoring on (role_match, seniority, location, comp, industry, recency).
- Output buckets: strong (≥0.75), soft (0.45-0.75), blocked (<0.45 or rule-block).
- Performance: 100 postings p99 ≤ 8s.

### T3 — CoverLetter.Draft

- Inputs: posting, context_ref, tone, length.
- Output: draft with explainability record:
  ```
  {
    text,
    model_id,
    prompt_template_hash,
    temperature,
    eu_ai_act_explainability_record: {
      decision_path, key_input_features_cited, model_capabilities, model_limitations
    }
  }
  ```
- Performance p99 ≤ 4s per draft.

### T4 — Filter.Retrain

- Compute strictly on personal-tenant compute nodes.
- Audit: `IntelligenceFilterRetrained{compute_node_tenant_id=<chris-personal-tenant>, example_count, threshold_shift}`.
- Locality invariant verified by reading the actual compute-node tenant_id assertion.

### T5 — Filter.Blacklist

- Adds sender_principal to filter's blacklist.
- Triggers when EmploymentFraudReported event fires.

### T6 — Context.Bind

- Indexes attached files locally (vector embeddings on local model).
- No external upload.

## Cedar permits

| Permit | Granted to | Purpose |
|---|---|---|
| `b2c.intelligence.filter_spec.author` | self | Author closed-form spec |
| `b2c.intelligence.filter.apply` | self via workflow-engine | Apply spec to postings |
| `b2c.intelligence.filter.retrain` | self via workflow-engine | Local retrain |
| `b2c.intelligence.cover_letter.draft` | self via workflow-engine | Draft cover |
| `b2c.intelligence.context.bind` | self | Bind résumé + portfolio |

## Audit emissions

- `FilterSpecAuthored`
- `IntelligenceFilterCompleted{raw, blocked, soft, strong}`
- `CoverLetterDrafted{model_id, prompt_template_hash}`
- `IntelligenceFilterRetrained{compute_node_tenant_id, example_count, threshold_shift}`
- `FilterBlacklistUpdated`
- `IntelligenceContextBound`

## Performance

- See above per-task.

## Acceptance criteria

- [ ] B.2 closed-schema enforcement (rejects protected-characteristic fields).
- [ ] B.3 retraining locality (compute_node tenant_id assertion present).
- [ ] B.4 transparency floor (model_id + prompt_template_hash + temperature stored).

## Out of scope

- Enterprise-tier Intelligence (j132's mass-hiring screen lives there).
- provider-BYOK switching (covered separately in ADR-0255 IP).

## Journey execution rows — substance pass

| Journey row | Source trigger | Actor | Contract / Cedar probe | State effect | Evidence touch | Counterpart |
|---|---|---|---|---|---|---|
| Filter spec authored | Chris creates job-search filter in Workflow Studio | `ConsumerEndUser` self principal; Cedar `b2c.intelligence.filter_spec.author` and tenant-scope consent required | proto `FilterSpec.Author` in this IP; existing fallback `POST /dispatch` purpose=`job.filter_spec.author` | closed schema persisted; protected-characteristic filters rejected | `FilterSpecAuthored` audit emission | matches LinkedIn job alert preference authoring |
| Posting classified | workflow-engine sends 100 postings to Filter.Apply | `ConsumerEndUser` via workflow-engine; Cedar `b2c.intelligence.filter.apply` requires self/workflow delegation | proto `Filter.Apply` with local model scoring dimensions | strong/soft/blocked buckets returned; no application sent | `IntelligenceFilterCompleted{raw, blocked, soft, strong}` | matches LinkedIn/Indeed recommendation filter pass |
| Cover draft generated | user opens a strong-match posting | `ConsumerEndUser` self principal; context consent and AI budget checked by `tenant-scope.cedar` | proto `CoverLetter.Draft` with context_ref/tone/length | draft includes model_id, prompt_template_hash, temperature, explainability record | `CoverLetterDrafted` audit event | matches Grammarly/LinkedIn cover-letter assistant transparency |
| Local retrain | user corrects repeated soft/blocked classification | `ConsumerEndUser` via workflow-engine; personal-tenant compute_node_tenant_id assertion required | proto `Filter.Retrain` | threshold_shift recorded; no external upload | `IntelligenceFilterRetrained` with example_count | matches on-device personalization retrain boundary |
| Fraud blacklist | Community emits EmploymentFraudReported | `ConsumerEndUser` filter owner via workflow-engine; Cedar self/delegation only; Community owns fraud report source | proto `Filter.Blacklist` | sender_principal added to blacklist for this personal tenant only | `FilterBlacklistUpdated` | matches Gmail sender blocklist from fraud report |
| Context bind | user attaches resume/portfolio | `ConsumerEndUser` self principal; Cedar `b2c.intelligence.context.bind`; local vector embedding only | proto `Context.Bind` | context_ref created; external providers cannot receive raw files | `IntelligenceContextBound` | matches Apple/Spotlight local index privacy boundary |
| Provider/budget refusal | cover draft exceeds consumer daily AI budget | `ConsumerEndUser`; `dispatch-authorization.cedar` requires `consumer_daily_budget_remaining_usd > 0` | `DispatchEnvelope.secret_reference.kind=platform_default` budget path | draft refused with safe retry copy; no hidden provider call | `dispatch.refused` reason `cost_cap_exceeded` or budget policy metric | matches consumer SaaS AI quota controls |
| Appeal evidence | user challenges blocked posting classification | `ConsumerEndUser` owner; tenant-scope prevents other postings/context refs leaking | `GET /audit-tap/{envelope_id}` plus Filter.Apply output | appeal packet shows input hash, score dimensions, blacklist rule if any | audit-tap seal + filter metrics | matches LinkedIn job recommendation feedback appeal |

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/intelligence/IP-journey-j144-filter-and-drafter-consumer-tier.md` matched `p99`.
- Numeric target: `rto_p99_seconds=300`, `rpo_p99_seconds=60` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), EU-AI-ACT-2024-HIGH-RISK(1800s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-PIPA-2023-amendment(14400s/900s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/intelligence/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`, `microservices/intelligence/slos/streaming-throughput.openslo.yaml`, `microservices/intelligence/policy/abuse-defence.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-journey-j144-filter-and-drafter-consumer-tier.md` matched `cost, emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
