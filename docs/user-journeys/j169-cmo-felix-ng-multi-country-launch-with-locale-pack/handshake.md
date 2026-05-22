---
doc_class: User-Journey-Handshake
journey_id: j169-cmo-felix-ng-multi-country-launch-with-locale-pack
date: 2026-05-20
authority_tier: 2
status: draft
---

# j169 — Handshake: per-µservice cross-tenant API surface

## §0 — Tenancies + principals

| Tenant | Role | Principals |
|---|---|---|
| `veritem-health-asia-pte-ltd-sg` | Veritem primary | Felix Ng (CMO), Priya Subramaniam-Tan (CEO), Wei Jia Tan (Compliance) |
| `veritem-health-singapore-sub-tenant` | SG sub | Hannah Goh (MD-SG) |
| `veritem-health-indonesia-sub-tenant` | ID sub | Bagas Hartono (MD-ID) |
| `veritem-health-thailand-sub-tenant` | TH sub | Chayanut Phongphan (MD-TH) |
| `veritem-health-vietnam-sub-tenant` | VN sub | Trần Thị Mỹ Linh (MD-VN) |
| `veritem-health-philippines-sub-tenant` | PH sub | Toni Ramos (MD-PH) |
| `veritem-health-malaysia-sub-tenant` | MY sub | Aisyah Mohd Rizal (MD-MY) |
| `ambassador-{n}-sub-tenant` | 12 ambassador identities | One per ambassador |
| `grabpay-sg-payment-processor-tenant` + GoPay-ID, TrueMoney-TH, MoMo-VN, GCash-PH, TouchnGo-MY, Stripe-fallback | Payment processors | system principals |
| `oya-governance-locale-pack-system-tenant` | Substrate governance dual-seal | system-principal |
| `pwc-singapore-soc2-auditor-tenant` | SOC2 auditor | `pwc-soc2-reader` |
| `sentinel-asia-asean-privacy-attestation-tenant` | ASEAN-Privacy auditor | `sentinel-asia-reader` |
| `dekra-singapore-eu-ai-act-art-50-tenant` | EU-AI-Act-Art-50 auditor | `dekra-art-50-reader` |

## §1 — Launch readiness read

### 1.1 — `marketing-automation.GET /v1/launches/{launch_id}/readiness`

| Field | Value |
|---|---|
| Source tenant | `veritem-health-asia-pte-ltd-sg` |
| Source principal | `felix.ng@...` |
| Cedar permit | `marketing-automation.launch_readiness_read` — cmo / ceo / md / compliance roles |
| Audit class | `EVT-J169-READINESS-COMPLETE-001` (read seal) |

Request:
```http
GET /v1/launches/asean-6-2026-06-15/readiness HTTP/3
oya-tenant: veritem-health-asia-pte-ltd-sg
oya-content-locale: en-SG
Authorization: Bearer <workload-identity-passkey-derived>
```

Response (excerpt):
```json
{
  "launch_id": "asean-6-2026-06-15",
  "status": "ready_for_go_no_go_review",
  "days_to_launch": 14,
  "countries": [
    {"code": "SG", "md_signoff_principal": "hannah.goh@...", "md_signoff_at": "2026-05-31T17:18:00+08:00", "checklist_green": 87, "checklist_total": 87, "tenant": "veritem-health-singapore-sub-tenant"},
    {"code": "ID", "md_signoff_principal": "bagas.hartono@...", "md_signoff_at": "2026-05-31T17:48:00+07:00", "checklist_green": 87, "checklist_total": 87, "tenant": "veritem-health-indonesia-sub-tenant"},
    {"code": "TH", "md_signoff_principal": "chayanut.phongphan@...", "md_signoff_at": "2026-05-31T18:00:00+07:00", "checklist_green": 87, "checklist_total": 87, "tenant": "veritem-health-thailand-sub-tenant"},
    {"code": "VN", "md_signoff_principal": "myLinh.tran@...", "md_signoff_at": "2026-05-31T18:12:00+07:00", "checklist_green": 87, "checklist_total": 87, "tenant": "veritem-health-vietnam-sub-tenant"},
    {"code": "PH", "md_signoff_principal": "toni.ramos@...", "md_signoff_at": "2026-05-31T18:24:00+08:00", "checklist_green": 87, "checklist_total": 87, "tenant": "veritem-health-philippines-sub-tenant"},
    {"code": "MY", "md_signoff_principal": "aisyah.rizal@...", "md_signoff_at": "2026-05-31T18:30:00+08:00", "checklist_green": 87, "checklist_total": 87, "tenant": "veritem-health-malaysia-sub-tenant"}
  ],
  "languages": ["id-ID", "ms-MY", "th-TH", "vi-VN", "tl-PH", "zh-Hant-SG", "en-SG"],
  "currencies": ["SGD", "IDR", "THB", "VND", "PHP", "MYR"],
  "ambassadors_confirmed": 12,
  "audit_seal": "EVT-J169-READINESS-COMPLETE-001"
}
```

## §2 — NLLB-200 content-localization handshake

### 2.1 — `intelligence.POST /v1/localizations/batch`

| Field | Value |
|---|---|
| Source tenant | `veritem-health-asia-pte-ltd-sg` |
| Source principal | Felix Ng (cmo role) OR system-principal `localization-orchestrator` |
| Cedar permit | `intelligence.localize_content_batch` — cmo or system-orchestrator + ai-content-transparency-disclosure-policy active |
| Audit class | `EVT-J169-LOCALIZATION-BATCH-002a` (per batch); aggregate `EVT-J169-LOCALIZATION-QA-COMPLETE-002` |

Request:
```json
{
  "source_language": "en-SG",
  "target_languages": ["id-ID", "ms-MY", "th-TH", "vi-VN", "tl-PH", "zh-Hant-SG"],
  "strings": [
    {"key": "onboarding.welcome.title", "source": "Welcome to Veritem — your daily health partner"},
    {"key": "onboarding.welcome.body", "source": "Track your blood sugar after each meal — small habits make big differences."}
  ],
  "cultural_adaptation_overlays": {
    "id-ID": ["halal-dietary-aware", "lebaran-reset-aware"],
    "ms-MY": ["halal-dietary-aware"],
    "th-TH": ["buddhist-uposatha-aware"],
    "vi-VN": ["coffee-culture-aware"],
    "tl-PH": ["family-centric-tone"],
    "zh-Hant-SG": ["multi-generational-tone"]
  },
  "ai_content_transparency_required": true,
  "human_editor_review_required": true
}
```

Response (excerpt):
```json
{
  "batch_id": "loc-batch-asean-6-2026-06-15-001",
  "localized": [
    {
      "key": "onboarding.welcome.body",
      "translations": [
        {
          "lang": "id-ID",
          "text": "Catat gula darahmu setelah setiap makan — kebiasaan kecil bisa membawa perubahan besar.",
          "ai_source": "nllb-200-distilled-1.3B",
          "ai_raw": "Lacak gula darah Anda setelah setiap makan — kebiasaan kecil membuat perbedaan besar.",
          "human_editor": "indah.rahmawati@veritem-localization-id-team",
          "edit_diff_token_count": 5,
          "transparency_attestation": "ai_translated_then_human_edited",
          "cultural_overlay_applied": [],
          "audit_seal": "EVT-J169-LOCALIZATION-STRING-{key}-{lang}-002b"
        },
        {
          "lang": "th-TH",
          "text": "การเดิน 10,000 ก้าวต่อวันสามารถช่วยควบคุมระดับน้ำตาลในเลือดได้",
          "ai_source": "nllb-200-distilled-1.3B",
          "ai_raw": "(same as text)",
          "human_editor": "siriporn.nakhonpathom@veritem-localization-th-team",
          "edit_diff_token_count": 0,
          "transparency_attestation": "ai_translated_human_reviewed_no_edit",
          "cultural_overlay_applied": [],
          "audit_seal": "EVT-J169-LOCALIZATION-STRING-{key}-{lang}-002b"
        }
      ]
    }
  ],
  "merkle_root": "sha384-..."
}
```

## §3 — Ambassador onboarding

### 3.1 — `community.POST /v1/ambassadors/credential-issue`

Request:
```json
{
  "ambassador_principal": "tania.putri@ambassador-1-sub-tenant",
  "country_code": "ID",
  "tier": "tier-1",
  "retainer_idr_per_month": 24000000,
  "per_signup_commission_idr": 12000,
  "attribution_tracking_url": "https://veritem.id/signup?ambassador=tania-putri-001",
  "content_languages": ["id-ID"],
  "social_channels": [
    {"platform": "instagram", "handle": "@sehat.bersama.tania", "follower_count": 312000},
    {"platform": "tiktok", "handle": "@tania.diabetes", "follower_count": 84000}
  ],
  "ndab_signed": true,
  "consent_gdpr_equivalent_signed": true
}
```

Response:
```json
{
  "credential_id": "ambassador-cred-tania-putri-2026-06-02",
  "issued_at": "2026-06-02T11:42:18+07:00",
  "tier": "tier-1",
  "passkey_enrolled": true,
  "attribution_tracking_active": true,
  "audit_seal": "EVT-J169-AMBASSADOR-CREDENTIAL-ISSUE-003a-tania"
}
```

## §4 — A/B cohort split rule write

### 4.1 — `marketing-automation.PUT /v1/cohort-rules/per-country/{country_code}`

```http
PUT /v1/cohort-rules/per-country/ID HTTP/3
oya-tenant: veritem-health-asia-pte-ltd-sg
oya-truetime-uncertainty-ms: 2.2
Content-Type: application/json
```
```json
{
  "country_code": "ID",
  "cohorts": [
    {"label": "control", "percentage": 33.33, "onboarding_flow": "4-step-existing"},
    {"label": "treatment_a", "percentage": 33.33, "onboarding_flow": "6-step-clinical-detail"},
    {"label": "treatment_b", "percentage": 33.34, "onboarding_flow": "3-step-deferred-clinical"}
  ],
  "signer_md": "bagas.hartono@veritem-health-asia-pte-ltd-sg",
  "signer_cmo": "felix.ng@veritem-health-asia-pte-ltd-sg",
  "signer_compliance": "wei.jia@veritem-health-asia-pte-ltd-sg",
  "research_ethics_review_passed": true,
  "research_ethics_reviewer_principal": "dr.endang.sutarwati@local-research-ethics-board-id",
  "effective_at_utc": "2026-06-15T01:00:00Z"
}
```

Response:
```json
{
  "rule_bundle_id": "cohort-rules-ID-2026-06-15",
  "audit_seal": "EVT-J169-COHORT-SPLIT-ID-004a",
  "dual_seal_tenants": ["veritem-health-asia-pte-ltd-sg", "veritem-health-indonesia-sub-tenant"]
}
```

## §5 — Go/no-go Cedar quorum vote

### 5.1 — `governance.POST /v1/launches/{id}/go-no-go-vote`

```json
{
  "launch_id": "asean-6-2026-06-15",
  "decision": "PERMIT",
  "voter_principal": "felix.ng@veritem-health-asia-pte-ltd-sg",
  "voter_role": "cmo",
  "rationale_en_SG": "All 522 readiness items green. 12 ambassadors confirmed. Localization QA passed. Cohort splits signed. Approved.",
  "voter_passkey_attestation": "<webauthn-assertion>"
}
```

After 8-of-8 PERMIT:
```json
{
  "quorum_decision": "PERMIT",
  "audit_seal": "EVT-J169-GO-LIVE-PERMIT-005",
  "dual_seal_tenants": ["veritem-health-asia-pte-ltd-sg", "oya-governance-locale-pack-system-tenant"],
  "truetime_uncertainty_ms": 1.6,
  "trigger_workflow": "feature-flags.schedule_per_country_launch_flips"
}
```

## §6 — Per-country feature-flag launch flip

### 6.1 — `feature-flags.PUT /v1/flags/launch_active/per-country/{country_code}`

Request (one per country, scheduled at exact local 08:00):
```json
{
  "flag_id": "launch_active",
  "country_code": "SG",
  "value": true,
  "effective_at_utc": "2026-06-15T00:00:00Z",
  "effective_at_local_iana": "2026-06-15T08:00:00+08:00",
  "scope": "country:SG",
  "rationale": "ASEAN-6 launch — SG country flip per CHG-LAUNCH-ASEAN-6-2026-06-15"
}
```

Response:
```json
{
  "flag_id": "launch_active",
  "country": "SG",
  "applied_at_utc": "2026-06-15T00:00:00.018Z",
  "first_signup_at_utc": "2026-06-15T00:00:18.412Z",
  "audit_seal": "EVT-J169-LAUNCH-LIVE-SG-006a"
}
```

## §7 — Payment processor cross-tenant handshake

### 7.1 — `payments.POST /v1/processors/{processor_id}/initialize`

Request (per processor, 6 processors + Stripe fallback):
```json
{
  "processor_id": "gopay-id",
  "country_code": "ID",
  "currency": "IDR",
  "pricing_tier_idr": {
    "monthly": 49000,
    "quarterly": 132000,
    "annual": 480000
  },
  "veritem_merchant_id_at_processor": "veritem-id-merchant-gopay-2026",
  "callback_url": "https://payments.veritem.id/callbacks/gopay",
  "compliance_attestation": "ID-OJK-payment-license-attested"
}
```

Response:
```json
{
  "processor_id": "gopay-id",
  "status": "active",
  "audit_seal": "EVT-J169-PAYMENT-PROCESSOR-GOPAY-ID-007b"
}
```

## §8 — Analytics + ambassador attribution

### 8.1 — `analytics.GET /v1/launches/{id}/day-7-report`

Request:
```http
GET /v1/launches/asean-6-2026-06-15/day-7-report HTTP/3
oya-tenant: veritem-health-asia-pte-ltd-sg
```

Response (Day-7 cumulative):
```json
{
  "launch_id": "asean-6-2026-06-15",
  "day_7_total_signups": 71400,
  "target_signups": 64000,
  "beat_pct": 11.6,
  "per_country": [
    {"code": "SG", "signups": 8420, "target": 8000},
    {"code": "ID", "signups": 26180, "target": 22000},
    {"code": "TH", "signups": 12400, "target": 10000},
    {"code": "VN", "signups": 10180, "target": 9000},
    {"code": "PH", "signups": 8840, "target": 9000},
    {"code": "MY", "signups": 5380, "target": 6000}
  ],
  "cohort_winners": {
    "SG": "treatment_a",
    "ID": "treatment_b",
    "TH": "treatment_b",
    "VN": "treatment_a",
    "PH": "treatment_b",
    "MY": "treatment_b"
  },
  "ambassador_attribution_pct": 38.4,
  "top_ambassadors": [
    {"name": "Tania Putri Wibowo (ID)", "attributed_signups": 8576, "pct": 12.0},
    {"name": "JM Cordero (PH)", "attributed_signups": 4998, "pct": 7.0},
    {"name": "Auntie Florence Wong (SG)", "attributed_signups": 2856, "pct": 4.0}
  ],
  "audit_seal": "EVT-J169-DAY-7-SIGNUPS-008",
  "merkle_root": "sha384-..."
}
```

## §9 — Cross-border compliance attestation

### 9.1 — `compliance.POST /v1/attestations/cross-border-transfer`

```json
{
  "scope": "launch:asean-6-2026-06-15",
  "transfer_records_count": 47218,
  "countries_involved": ["SG", "ID", "TH", "VN", "PH", "MY"],
  "asean_privacy_framework_attestation": true,
  "per_country_consent_obtained_pct": 100.0,
  "merkle_root": "sha384-...",
  "auditor_submissions": [
    {"auditor": "sentinel-asia-asean-privacy-attestation-tenant", "evidence_packs": ["ASEAN-Privacy-Framework", "cross-border-transfer-attestation"]},
    {"auditor": "dekra-singapore-eu-ai-act-art-50-tenant", "evidence_packs": ["EU-AI-Act-Art-50-content-transparency"]}
  ]
}
```

Response:
```json
{
  "attestation_id": "att-cross-border-asean-6-2026-06-22",
  "audit_seal": "EVT-J169-COMPLIANCE-ATTESTATIONS-011",
  "dekra_acknowledgement_expected_at": "2026-07-13T00:00:00Z"
}
```

## §10 — Cross-tenant invariants

- **Dual-seal**: every per-country launch event + cohort vote + ambassador credential + payment-processor init dual-seals in Veritem primary + country sub-tenant.
- **TrueTime**: ≤ 10 ms uncertainty at every gate.
- **Locale + diacritic**: Thai (ก้าว), Vietnamese (món ăn with tone marks), Bahasa, Tagalog, Traditional Chinese (餐單), all UTF-8 NFC.
- **Currency precision**: per ISO-4217 — IDR and VND with 0 decimals, others with 2.
- **MLS encryption**: ambassador comms + cross-tenant MD threads use MLS per RFC 9420.
- **HTTP/3 + QUIC** mandatory.
- **AI content transparency**: every AI-localized string carries `ai_content_transparency_attestation` per EU-AI-Act-Art-50.
