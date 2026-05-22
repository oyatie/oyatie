---
doc_class: User-Journey-Handshake
journey_id: j166-cso-mira-goldberg-strategic-acquisition-go-no-go
date: 2026-05-20
authority_tier: 2
status: draft
---

# j166 — Handshake matrix

Every named µservice call for the 9-day MRT acquisition decision cycle (May 15 07:42 EDT → May 25 11:18 EDT). Order matches `story.md`. Transport HTTPS over QUIC per ADR-0253. Cross-tenant calls NDA-bound + Cedar-validated per ADR-0244 + ADR-0251. Hebrew + German + diacritics preserved UTF-8 NFC byte-exact.

## Notation

- `[SKY]` Skylark tenant principal
- `[MRT]` MRT tenant principal
- `[XT]` Cross-tenant channel
- `→` synchronous request
- `←` synchronous response
- `↪` side-effect on bus
- `⟂` denied path

## §1 Workspace open (May 15 07:42 EDT)

`[SKY] → governance` — `POST /v1/governance/m-and-a/workspace-open`

```json
{
  "tenant_id": "skylark-logistics-solutions-inc",
  "deal_id": "mrt-acquisition-2027-q2",
  "target_tenant_id": "mendelsohn-routing-technologies-inc-de",
  "deal_class": "strategic_acquisition_saas_mid_market",
  "working_price_usd": 186000000,
  "price_range_usd": {"low": 172000000, "high": 202000000},
  "structure": {"cash_pct": 0.60, "stock_pct": 0.40},
  "earnout_usd": 30000000,
  "board_decision_target": "2027-05-25T09:00:00-04:00",
  "cso_principal": "mira.goldberg@skylark-logistics-solutions-inc",
  "opened_at": "2027-05-15T07:42:48-04:00"
}
```

Cedar: permit (cso + tenant + passkey + nda_active). Audit: `EVT-J166-WORKSPACE-OPEN-000`.

## §2 Cross-tenant diligence document arrival (May 15 02:14–06:18 EDT)

### 2.1 Cross-tenant channel envelope (4 documents)

`[MRT:bjorn] → connect` — `POST /v1/connect/cross-tenant/send`

```json
{
  "channel_id": "cross-tenant-channel-skylark-mrt-2027-q2",
  "from_tenant": "mendelsohn-routing-technologies-inc-de",
  "from_principal": "bjorn.mendelsohn@mendelsohn-routing-technologies-inc-de",
  "to_tenant": "skylark-logistics-solutions-inc",
  "to_principals": [
    "mira.goldberg@skylark-logistics-solutions-inc",
    "reginald.otis@skylark-logistics-solutions-inc",
    "daphne.harrowgate@skylark-logistics-solutions-inc"
  ],
  "nda_record_id": "nda-skylark-mrt-2027-03-08",
  "payload_class": "diligence_response_anonymized",
  "document_id": "doc-mrt-q1-2027-cohort-churn-anonymized",
  "filename": "mrt-q1-2027-cohort-churn-anonymized.csv",
  "size_bytes": 1221408,
  "e2ee_envelope_b64": "<MLS group encrypted bundle>",
  "sent_at": "2027-05-15T08:14:18+02:00"
}
```

Cedar: permit (nda_active + payload_class in whitelist + sender authorized). Audit: `EVT-J166-DOC-ARRIVED-{n}-Δ001`.

### 2.2 NDA-payload validation

`[SKY] → compliance` — `POST /v1/compliance/nda-channel/validate-payload`

```json
{
  "document_id": "doc-mrt-q1-2027-cohort-churn-anonymized",
  "payload_class": "diligence_response_anonymized",
  "nda_record_id": "nda-skylark-mrt-2027-03-08",
  "pii_scanner_results": {"pii_count": 0, "categories_checked": ["name", "email", "phone", "address", "national_id", "financial_account"]},
  "financial_value_scanner_results": {"value_count": 0},
  "payload_size_bytes": 1221408,
  "payload_size_within_nda_envelope": true,
  "e2ee_envelope_intact": true,
  "sender_authorization_valid": true
}
```

Audit: `EVT-J166-NDA-PAYLOAD-VALIDATED-Δ001a`.

### 2.3 Drive archive (cross-tenant evidence flag)

`[SKY] → drive` — `POST /v1/drive/rooms/{room}/files`

```json
{
  "drive_room": "skylark/m-a/2027-q2/mrt/diligence-inbox",
  "filename": "mrt-q1-2027-cohort-churn-anonymized.csv",
  "content_type": "text/csv",
  "size_bytes": 1221408,
  "cross_tenant_evidence_flag": true,
  "source_tenant": "mendelsohn-routing-technologies-inc-de",
  "nda_return_or_destroy_obligation_active": true,
  "nda_return_or_destroy_deadline": "2027-09-30T23:59:59-04:00",
  "audit_event_id": "EVT-J166-DRIVE-WRITE-DILIGENCE-Δ001b"
}
```

Audit: `EVT-J166-DILIGENCE-DOCS-EXCHANGED-002` (after all 28 docs over 9 days; rolling summary).

## §3 Financial model compute (May 15 14:42 EDT)

`[SKY] → financial-planning` — `POST /v1/financial-planning/m-and-a/compute`

```json
{
  "tenant_id": "skylark-logistics-solutions-inc",
  "deal_id": "mrt-acquisition-2027-q2",
  "model_template": "m-and-a-acquisition-saas-mid-market-v4",
  "inputs": {
    "target_arr_usd": 42000000,
    "target_arr_growth_yoy": 0.31,
    "target_gross_margin": 0.78,
    "target_customer_count": 340,
    "target_avg_arr_per_customer_usd": 123500,
    "target_cac_ltm_usd": 48000,
    "target_ltv_cac_ratio": 4.2,
    "target_net_dollar_retention": 1.17,
    "target_gross_dollar_retention": 0.94,
    "target_customer_concentration_top10_pct": 0.31,
    "acquirer_arr_usd": 148000000,
    "acquirer_customer_count": 1820,
    "acquirer_net_dollar_retention": 1.28,
    "deal_terms": {
      "price_low_usd": 172000000,
      "price_working_usd": 186000000,
      "price_high_usd": 202000000,
      "structure_cash_pct": 0.60,
      "structure_stock_pct": 0.40,
      "earnout_usd": 30000000,
      "integration_cost_assumption_usd": 12000000
    }
  }
}
```

Response (truncated):

```json
{
  "scenarios": [
    {
      "price_usd": 172000000,
      "revenue_multiple_arr": 4.1,
      "ntm_accretive_months": 18,
      "year_3_irr": 0.22,
      "year_5_irr": 0.27,
      "year_3_cumulative_cash_usd": -48000000,
      "year_5_cumulative_cash_usd": 104000000,
      "dilution_to_skylark_stock_pct": 0.042
    },
    {
      "price_usd": 186000000,
      "revenue_multiple_arr": 4.4,
      "ntm_accretive_months": 23,
      "year_3_irr": 0.18,
      "year_5_irr": 0.22,
      "year_3_cumulative_cash_usd": -62000000,
      "year_5_cumulative_cash_usd": 84000000,
      "dilution_to_skylark_stock_pct": 0.046
    },
    {
      "price_usd": 202000000,
      "revenue_multiple_arr": 4.8,
      "ntm_accretive_months": 34,
      "year_3_irr": 0.12,
      "year_5_irr": 0.16,
      "year_3_cumulative_cash_usd": -78000000,
      "year_5_cumulative_cash_usd": 54000000,
      "dilution_to_skylark_stock_pct": 0.050
    }
  ]
}
```

Audit: `EVT-J166-M-A-MODEL-COMPUTED-003`.

## §4 ML scenario modeling (May 18 06:42–06:54 EDT)

`[SKY] → intelligence` — `POST /v1/intelligence/m-and-a/scenario-model`

```json
{
  "tenant_id": "skylark-logistics-solutions-inc",
  "deal_id": "mrt-acquisition-2027-q2",
  "ml_models": [
    {
      "model_id": "monte-carlo-mid-market-saas-v7@oyatie-2027-02",
      "iterations": 10000,
      "scenarios": ["recession", "neutral", "tailwind"],
      "input_arr_baseline_usd": 42000000,
      "input_growth_distribution": "lognormal(mu=0.28,sigma=0.14)"
    },
    {
      "model_id": "cohort-churn-forecast-saas-v5@oyatie-2027-04",
      "horizon_years": 5,
      "cohort_input_csv_drive_pointer": "skylark/m-a/2027-q2/mrt/diligence-inbox/mrt-q1-2027-cohort-churn-anonymized.csv"
    },
    {
      "model_id": "integration-cost-forecast-cross-stack-v3@oyatie-2027-01",
      "integration_complexity_score": 0.62,
      "skylark_stack_summary": "<inline>",
      "mrt_stack_summary": "<from-diligence>"
    }
  ]
}
```

Response:

```json
{
  "monte_carlo_results": {
    "recession_p10": {"arr_year_5_usd": 58000000, "irr_year_5": 0.08, "neg_cf_year_3_plus_prob": 0.18},
    "neutral_p50": {"arr_year_5_usd": 98000000, "irr_year_5": 0.24, "neg_cf_year_3_plus_prob": 0.03},
    "tailwind_p90": {"arr_year_5_usd": 148000000, "irr_year_5": 0.38, "neg_cf_year_3_plus_prob": 0.00},
    "expected_probability_weighted": {"arr_year_5_usd": 94000000, "irr_year_5": 0.23}
  },
  "cohort_churn_5_year_forecast": [
    {"year": 1, "gross_churn": 0.062, "net_dollar_retention": 0.88},
    {"year": 2, "gross_churn": 0.068, "net_dollar_retention": 0.87},
    {"year": 3, "gross_churn": 0.071, "net_dollar_retention": 0.86},
    {"year": 4, "gross_churn": 0.074, "net_dollar_retention": 0.86},
    {"year": 5, "gross_churn": 0.076, "net_dollar_retention": 0.85}
  ],
  "integration_cost_forecast_usd": {
    "point_estimate": 14200000,
    "ci_95_lower": 9400000,
    "ci_95_upper": 19000000,
    "primary_drivers": [
      "postgres-14-to-16-migration-with-skylark-shard-alignment",
      "identity-unification-auth0-to-oyatie-identity",
      "route-optimization-engine-integration-4-to-6-senior-engineers-9-months"
    ]
  },
  "llm_provenance": {
    "models_invoked": 3,
    "total_inference_seconds": 712,
    "eu_ai_act_article_50_declaration_present": true
  }
}
```

Cedar: permit (cso + intelligence access + scenario_modeling). Audit: `EVT-J166-ML-SCENARIOS-004`.

## §5 Pack-manifest cross-check + merger filing requirements (May 18 14:18–17:32 EDT)

### 5.1 Pack cross-check

`[SKY] → compliance` — `POST /v1/compliance/pack-manifest/cross-check`

```json
{
  "acquirer_tenant_id": "skylark-logistics-solutions-inc",
  "target_tenant_id": "mendelsohn-routing-technologies-inc-de",
  "acquirer_active_packs": [
    "pack-soc2-type2-fy2026",
    "pack-gdpr-controller",
    "pack-ccpa-controller",
    "pack-hipaa-business-associate",
    "pack-pci-dss-saq-c",
    "pack-iso-27001-active"
  ],
  "target_active_packs": [
    "pack-soc2-type1-fy2026",
    "pack-gdpr-controller",
    "pack-iso-27001-active",
    "pack-german-bdsg",
    "pack-tisax-vda"
  ],
  "evaluate_blockers": true
}
```

Response: matches `story.md` §5 (overlap analysis + blockers + compatibility score).

Audit: `EVT-J166-PACK-CROSS-CHECK-005`.

### 5.2 Merger filings

`[SKY] → compliance` — `POST /v1/compliance/merger-filings/requirements-compute`

```json
{
  "deal_id": "mrt-acquisition-2027-q2",
  "deal_size_usd": 186000000,
  "acquirer_tenant_id": "skylark-logistics-solutions-inc",
  "target_tenant_id": "mendelsohn-routing-technologies-inc-de",
  "acquirer_turnover_usd": 148000000,
  "target_turnover_usd": 42000000,
  "jurisdictions_to_evaluate": ["HSR-US", "EU-Merger-Control", "Germany-BWB", "UK-CMA", "Israeli-IMC"]
}
```

Response: matches `story.md` §5 — HSR required, German BWB required, UK CMA voluntary, EU MR and Israeli IMC not required.

Audit: `EVT-J166-MERGER-FILINGS-006`.

## §6 Counsel review (May 19 09:00–17:48 EDT)

`[Daphne] → governance` — `POST /v1/governance/counsel-review/submit-m-and-a`

```json
{
  "deal_id": "mrt-acquisition-2027-q2",
  "reviewer_principal": "daphne.harrowgate@skylark-logistics-solutions-inc",
  "redline_count": 4,
  "redlines": [
    {"id": "R1", "subject": "earnout structure team-retention pool"},
    {"id": "R2", "subject": "SOC 2 Type1→2 remediation as condition precedent"},
    {"id": "R3", "subject": "German co-counsel retention (Hengeler Mueller)"},
    {"id": "R4", "subject": "NDA Section 7.2 return-or-destroy timeline clarification"}
  ],
  "deal_term_clarification_requested": "Bjorn Mendelsohn relocation status",
  "review_duration_minutes": 524,
  "completed_at": "2027-05-19T17:42:18-04:00",
  "passkey_assertion_present": true
}
```

Audit: `EVT-J166-COUNSEL-REVIEW-007`.

## §7 CFO sign-off (May 18 10:42 EDT)

`[Reginald] → governance` — `POST /v1/governance/cfo-signoff/m-and-a`

```json
{
  "deal_id": "mrt-acquisition-2027-q2",
  "cfo_principal": "reginald.otis@skylark-logistics-solutions-inc",
  "financial_model_review_complete": true,
  "ml_scenario_acknowledgment": true,
  "concerns_documented": [
    "$14.2M integration cost higher than $12M working assumption",
    "key-person earnout structure",
    "ML cohort churn appropriately conservative"
  ],
  "signoff_at": "2027-05-18T10:42:08-04:00",
  "passkey_assertion_present": true
}
```

Audit: `EVT-J166-CFO-SIGNOFF-008`.

## §8 Committee endorsement (May 20 09:00–11:48 EDT)

`[committees] → governance` — `POST /v1/governance/committee/endorsement-submit`

```json
{
  "deal_id": "mrt-acquisition-2027-q2",
  "strategy_committee": {
    "chair": "margarita.velasco-heim@skylark-logistics-solutions-inc",
    "votes": [
      {"principal": "margarita.velasco-heim@skylark-logistics-solutions-inc", "vote": "endorse"},
      {"principal": "kenji.park-holloway@skylark-logistics-solutions-inc", "vote": "reservation"},
      {"principal": "<member3>@skylark-logistics-solutions-inc", "vote": "endorse"},
      {"principal": "<member4>@skylark-logistics-solutions-inc", "vote": "endorse"},
      {"principal": "<member5>@skylark-logistics-solutions-inc", "vote": "endorse"}
    ],
    "endorsement_count": 4,
    "committee_size": 5,
    "endorsed": true
  },
  "audit_committee": {
    "chair": "hannah.beauregard@skylark-logistics-solutions-inc",
    "endorsement_count": 3,
    "committee_size": 5,
    "endorsed": true
  }
}
```

Audit: `EVT-J166-COMMITTEE-ENDORSEMENT-009`.

## §9 Board vote (May 25 09:00 → 10:54 EDT)

`[board] → governance` — `POST /v1/governance/board/go-no-go-vote`

```json
{
  "deal_id": "mrt-acquisition-2027-q2",
  "meeting_at": "2027-05-25T09:00:00-04:00",
  "votes": [
    {"principal": "adrian.cheng-whitford@skylark-logistics-solutions-inc", "role": "chair", "vote": "yes"},
    {"principal": "hannah.beauregard@skylark-logistics-solutions-inc", "role": "audit_chair", "vote": "yes"},
    {"principal": "margarita.velasco-heim@skylark-logistics-solutions-inc", "role": "strategy_chair", "vote": "yes"},
    {"principal": "kenji.park-holloway@skylark-logistics-solutions-inc", "role": "independent", "vote": "abstain"},
    {"principal": "christine.adebayo-lin@skylark-logistics-solutions-inc", "role": "independent", "vote": "yes"},
    {"principal": "anil.subramaniam@skylark-logistics-solutions-inc", "role": "independent", "vote": "yes"},
    {"principal": "david.hofmann-reyes@skylark-logistics-solutions-inc", "role": "independent", "vote": "yes"},
    {"principal": "joon-ho.park@skylark-logistics-solutions-inc", "role": "independent", "vote": "yes"},
    {"principal": "patricia.wells-okonkwo@skylark-logistics-solutions-inc", "role": "NED", "vote": "no"}
  ],
  "yes_count": 7,
  "no_count": 1,
  "abstain_count": 1,
  "total_votes": 9,
  "majority_threshold": 5,
  "result": "GO",
  "voted_at": "2027-05-25T10:54:18-04:00"
}
```

Cedar: permit (board_voting_members + audit_committee_endorsement + counsel_review + CFO_signoff + passkeys). Audit: `EVT-J166-BOARD-VOTE-010`.

## §10 Decision record + Merkle anchor (May 25 11:18 EDT)

`[governance] → audit-chain` — `POST /v1/audit-chain/m-and-a-decision/anchor`

```json
{
  "deal_id": "mrt-acquisition-2027-q2",
  "bundle_components": [
    {"role": "executive_summary", "sha256": "..."},
    {"role": "financial_model", "sha256": "..."},
    {"role": "ml_scenarios", "sha256": "..."},
    {"role": "pack_cross_check", "sha256": "..."},
    {"role": "merger_filings", "sha256": "..."},
    {"role": "counsel_review", "sha256": "..."},
    {"role": "cfo_signoff", "sha256": "..."},
    {"role": "committee_endorsement", "sha256": "..."},
    {"role": "board_vote_roll_call", "sha256": "..."},
    {"role": "integration_playbook", "sha256": "..."}
  ],
  "super_merkle_root": "0xc2f8a4b7e1d6f3a9c4e7b2d5f8a1c6e9b3d7f0a4c8e2b5d9f3a7c0e4b8d2f6a1",
  "anchor_targets": [
    "audit-chain-spine-skylark-m-a-2027-q2",
    "external-transparency-log-batch-2027-05-25T1118"
  ]
}
```

Audit: `EVT-J166-DECISION-RECORDED-011`.

## §11 Denied paths

### 11.1 ⟂ Diligence response with PII

Cedar deny. Audit: `EVT-J166-CEDAR-DENY-NDA-PAYLOAD-PII-Δ010`.

### 11.2 ⟂ Non-NDA-bound cross-tenant data flow

Cedar deny. Audit: `EVT-J166-CEDAR-DENY-NON-NDA-FLOW-Δ011`.

### 11.3 ⟂ Board vote without CFO signoff

Cedar deny. Audit: `EVT-J166-CEDAR-DENY-VOTE-NO-CFO-Δ012`.

### 11.4 ⟂ Non-board member attempts go/no-go vote

Cedar deny. Audit: `EVT-J166-CEDAR-DENY-NON-BOARD-VOTE-Δ013`.

### 11.5 ⟂ NDA channel after NDA expiry

Cedar deny. Audit: `EVT-J166-CEDAR-DENY-NDA-EXPIRED-Δ014`.

## §12 SLA + latency summary

| Stage | SLA | Observed |
|---|---|---|
| Cross-tenant document arrival | within minutes | varies |
| NDA payload validation | ≤ 10s per doc | 6s avg |
| Financial model compute (3 prices) | ≤ 30s | 18s |
| ML scenario modeling (3 models) | ≤ 15 min | 12 min |
| Pack cross-check | ≤ 60s | 42s |
| Merger filings compute | ≤ 30s | 22s |
| Counsel review wall-clock | ≤ 2 business days | 1 |
| Board vote | within scheduled meeting | 09:00–10:54 EDT |
| Merkle anchor | ≤ 5 min post-vote | 24s |
