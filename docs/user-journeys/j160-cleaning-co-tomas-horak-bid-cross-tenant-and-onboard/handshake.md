---
doc_class: User-Journey-Handshake
journey_id: j160-cleaning-co-tomas-horak-bid-cross-tenant-and-onboard
date: 2026-05-20
authority_tier: 2
status: draft
---

# j160 — Handshake matrix

Every named µservice call across the seven tenants (`uklid-horak-sro-plzen-cz` + `polycraft-bohemia-as-plzen-cz` + `cz-ossz-state-tenant` + `cz-vzp-zp-tenant` + `cz-financni-urad-tenant` + `cz-datova-schranka-tenant` + `cz-cleaning-industry-owner-operators-community`) for the 81-day journey from bid prep through first-shift live. Order matches `story.md`. Every row names source + target tenant, API surface, wire shape, Cedar permit, ADR-0263 audit class.

## Notation

- `[H]` Úklid Horák tenant
- `[P]` PolyCraft Bohemia tenant
- `[O]` Czech OSSZ state tenant
- `[Z]` Czech zdravotní pojišťovna tenant
- `[F]` Czech Finanční úřad tenant
- `[D]` Czech Datová schránka tenant
- `[C]` Czech cleaning-industry community tenant
- `→` synchronous; `↪` side-effect; `⟂` denied

Transport: HTTPS/HTTP-3 (QUIC) per ADR-0253. Cedar p95 ≤ 180 ms. UTF-8 NFC for Czech/Vietnamese/Slovak/Ukrainian. Diacritic-strict mode default. CZ data residency = `eu-prague-primary`.

## §1 Bid request read

### 1.1 Tomáš reads bid request

`[H] → marketplace` — `GET /v1/marketplace/bid-requests/{bid_id}`

Path: `bid_id = bid-polycraft-plzen-cleaning-2027-01-04`

Response (excerpt):

```json
{
  "bid_id": "bid-polycraft-plzen-cleaning-2027-01-04",
  "issuing_tenant": "polycraft-bohemia-as-plzen-cz",
  "title_cs": "Otevřená výzva k podání nabídky · facilities cleaning Plzeňský závod",
  "title_en": "Open call for bid · facilities cleaning Plzeň plant",
  "service_class": "facilities_cleaning_industrial",
  "site_area_m2_breakdown": {
    "production": 9200,
    "warehouse": 1800,
    "admin": 980,
    "common": 420
  },
  "contract_window": {
    "start": "2027-01-04",
    "end": "2028-12-31",
    "months": 24
  },
  "max_annual_excl_vat_czk": 4200000,
  "csn_en_13549_min_grade": 4,
  "iso_9001_required": true,
  "bid_deadline": "2026-10-17T17:00:00+02:00",
  "diacritic_policy": {"unicode_normalization": "NFC", "diacritics_preserved": true}
}
```

Audit: `EVT-J160-BID-REQUEST-READ-001` sealed in `uklid-horak-sro-plzen-cz`.

## §2 Site walk evidence capture

### 2.1 Site walk task complete with photos

`[H] → tasks` — `POST /v1/tasks/{task_id}/complete`

```json
{
  "task_id": "task-j160-001-site-walk-polycraft",
  "completed_by": "tomas.horak@uklid-horak-sro-plzen-cz",
  "completed_at": "2026-10-15T14:18:00+02:00",
  "evidence": {
    "photo_ids": ["photo-...01", "photo-...02", "...", "photo-...47"],
    "voice_notes": ["vn-2026-10-15-080812-recovery-cleaning-zone1", "...×12"],
    "site_walked_with": "m.prochazkova@polycraft-bohemia-as-plzen-cz",
    "zones_walked": ["admin", "canteen", "lockers_showers", "restrooms", "warehouse", "production_zones_1_through_7"]
  }
}
```

Audit: `EVT-J160-SITE-WALK-002` sealed.

## §3 Bid submission

### 3.1 Bid submit

`[H] → marketplace` — `POST /v1/marketplace/bid-requests/{bid_id}/submit-bid` at 16:42:18 CET Wed Oct 15

```json
{
  "principal": "tomas.horak@uklid-horak-sro-plzen-cz",
  "tenant_ctx": "uklid-horak-sro-plzen-cz",
  "bid_id": "bid-polycraft-plzen-cleaning-2027-01-04",
  "bid_total_excl_vat_czk": 7940000,
  "bid_total_with_vat_czk": 9607400,
  "vat_rate_pct": 21.0,
  "bid_window_months": 24,
  "line_items_ref": "cleaning-bid-line-items-uklid-horak-polycraft-2027",
  "attachments": [
    "iso-9001-cert-tuv-sud-2025-04-18.pdf",
    "issa-cims-2024-cert.pdf",
    "generali-liability-cert-czk-50m.pdf",
    "csn-en-13549-quality-protocol.pdf",
    "ref-letter-plzensky-prazdroj.pdf",
    "ref-letter-skoda-auto-plzen.pdf",
    "crew-composition-plan.pdf",
    "cover-narrative-cs.pdf"
  ],
  "narrative_cover_letter_text_b64": "<base64 Czech narrative>",
  "diacritic_policy": {"unicode_normalization": "NFC", "diacritics_preserved": true},
  "ares_business_register_verified": true,
  "cz_vat_id": "CZ27488123",
  "submitted_at": "2026-10-15T16:42:18+02:00"
}
```

Cedar permit: `marketplace.bid_submit` against `BidRequest::"bid-polycraft-..."`. Context per cedar-policy.cedar §1.

Audit: `EVT-J160-BID-SUBMITTED-003` dual-sealed in `uklid-horak-sro-plzen-cz` AND `polycraft-bohemia-as-plzen-cz`.

## §4 Bid evaluation + award

### 4.1 Procházková evaluates

`[P] → marketplace` — `POST /v1/marketplace/bid-requests/{bid_id}/evaluate` (Oct 19–25)

Internal-to-PolyCraft scoring against 5 evaluation criteria (price 30%, technical-protocol 35%, ČSN-EN-13549 fitness 15%, references 10%, ESG 10%).

Audit: `EVT-J160-BID-EVALUATED-004` sealed in `polycraft-bohemia-as-plzen-cz`.

### 4.2 Award notification

`[P] → messenger` (cross-tenant to `[H]`) — `POST /v1/messenger/groups/{group_id}/post`

```json
{
  "principal": "m.prochazkova@polycraft-bohemia-as-plzen-cz",
  "group_id": "thread-bid-polycraft-uklid-horak-2026-10",
  "post_at": "2026-10-27T14:00:12+01:00",
  "notification_class": "award_decision",
  "award_decision": "accepted",
  "bid_id": "bid-polycraft-plzen-cleaning-2027-01-04",
  "ciphertext_b64": "<MLS bundle>"
}
```

Audit: `EVT-J160-AWARD-RECEIVED-005` dual-sealed.

### 4.3 Workflow advance

`[H] → workflow-engine` — `POST /v1/workflows/bid-and-onboard/{instance_id}/transition`

```json
{
  "instance_id": "bid-onboard-uklid-horak-polycraft-2026",
  "from_state": "bid_evaluated",
  "to_state": "award_received",
  "transitioned_at": "2026-10-27T14:00:18+01:00",
  "evidence_links": ["EVT-J160-AWARD-RECEIVED-005"]
}
```

## §5 Contract negotiation + sign

### 5.1 Contract drafts iterate

`[H] ↔ [P]` via `contract-lifecycle-management` — 4 drafts Oct 28–Nov 6 with `POST /v1/clm/contracts/{contract_id}/draft-revision`.

### 5.2 Final sign with QES under TrueTime fence

`[H] + [P] → contract-lifecycle-management` — `POST /v1/clm/contracts/{contract_id}/qes-sign-dual-tenant`

```json
{
  "contract_id": "contract-uklid-horak-polycraft-2027-01-04",
  "signatories": [
    {
      "principal": "tomas.horak@uklid-horak-sro-plzen-cz",
      "qes_provider": "I.CA",
      "qes_certificate_serial": "<serial>",
      "signed_at": "2026-11-07T11:18:18+01:00"
    },
    {
      "principal": "m.prochazkova@polycraft-bohemia-as-plzen-cz",
      "qes_provider": "SecuSign",
      "qes_certificate_serial": "<serial>",
      "signed_at": "2026-11-07T11:18:38+01:00"
    }
  ],
  "truetime_uncertainty_ms": 6,
  "eidas_qualified": true,
  "contract_value_excl_vat_czk": 7940000,
  "contract_months": 24
}
```

Cedar permit: `contract.qes_sign_dual_tenant` with `context.truetime_uncertainty_ms <= 10`.

Audit: `EVT-J160-CONTRACT-SIGNED-006` dual-sealed under TrueTime fence.

### 5.3 Datová schránka notification

`[H] → cz-datova-schranka` — `POST /v1/cz-datova-schranka/notifications`

```json
{
  "ic_owner": "27488123",
  "notification_class": "contract_execution_metadata",
  "contract_hash_sha256": "<sha256 of signed contract PDF>",
  "counterparty_ic": "47714232",
  "contract_value_czk": 7940000,
  "notified_at": "2026-11-07T11:42:00+01:00"
}
```

Audit: `EVT-J160-DATOVA-SCHRANKA-NOTIFIED-006a` sealed.

## §6 Crew hiring

### 6.1 Labor-pool publishing

`[H] → marketplace` — `POST /v1/marketplace/labor-pool/positions/publish`

3 positions published Nov 10; integrates with Czech state Úřad práce.

### 6.2 Applicant intake + interview

`[H] → workflow-engine` — multiple `POST /v1/workflows/hire/{position_id}/interview-record`

### 6.3 Offer + accept

`[H] → workflow-engine` — `POST /v1/workflows/hire/{position_id}/offer-accept`

For each of: Hoàng Văn Long, Mária Kováčová, Іван Шевченко.

Audit: `EVT-J160-CREW-SELECTED-008-prep` sealed Wed Nov 26.

## §7 Employee onboarding to Czech state systems

### 7.1 ARES tenant verification (refresh)

`[H] → cz-ares` — `GET /v1/cz-ares/business/{ic}` for IČ 27488123 — confirms `cz_vat_id_active`.

### 7.2 OSSZ employee registration

`[H] → cz-ossz-state-tenant` — `POST /v1/cz-ossz/employee-registration`

For each new hire:

```json
{
  "employer_ic": "27488123",
  "employee": {
    "legal_name_strict_form_nfc": "Hoàng Văn Long",
    "birth_date": "1992-03-14",
    "rodne_cislo": "920314/XXXX",
    "residence_address_cz": "Plzeň, Skvrňany ...",
    "permanent_residence_status": "permanent_resident",
    "contract_class": "indefinite_full_time",
    "start_date": "2026-12-01"
  },
  "diacritic_policy": {"unicode_normalization": "NFC", "diacritics_preserved": true}
}
```

Audit: `EVT-J160-OSSZ-EMPLOYEE-REGISTERED-008-hoang` etc.

### 7.3 ZP (health insurance) registration

`[H] → cz-vzp-zp-tenant` — `POST /v1/cz-zp/{vzp|ozp|zpmv}/employee-registration`

Each new hire registers with their chosen ZP. Diacritic-strict preservation throughout.

### 7.4 Finanční úřad tax registration

`[H] → cz-financni-urad-tenant` — `POST /v1/cz-fu/employee-tax-registration` for each new hire's daň z příjmu fyzických osob (income tax withholding) setup.

## §8 Training

### 8.1 ČSN-262-2006 safety training

`[H] → learning-management` — `POST /v1/lms/courses/{course_id}/enrollment`

```json
{
  "course_id": "csn-262-2006-safety-cleaning-industry-v2",
  "enrolled_principals": [
    "hoang.van.long@uklid-horak-sro-plzen-cz",
    "maria.kovacova@uklid-horak-sro-plzen-cz",
    "ivan.shevchenko@uklid-horak-sro-plzen-cz"
  ],
  "delivery_window_start": "2026-12-01T08:00:00+01:00",
  "delivery_window_end": "2026-12-05T16:00:00+01:00",
  "delivery_hours": 8,
  "assessment_required": true,
  "instructor": "tomas.horak@uklid-horak-sro-plzen-cz (firm-specific portion); bozp-info.cz-instructor-23 (general portion)"
}
```

Audit: `EVT-J160-CSN-262-TRAINING-COMPLETE-008-{name}` sealed per crew member.

### 8.2 GDPR + CZ-110/2019 training

`[H] → learning-management` — similar enrollment for 6-hour GDPR module Dec 8–10.

Audit: `EVT-J160-GDPR-TRAINING-COMPLETE-008-{name}` sealed.

### 8.3 Tennant T7AMR equipment training

`[H] → learning-management` — equipment-cert enrollment Dec 11–12.

### 8.4 PolyCraft-specific induction

`[H] + [P] → workflow-engine` — `POST /v1/workflows/cross-tenant-induction/{instance_id}` Dec 15–17.

Includes biometric-badge enrollment (§9).

## §9 Biometric badge cross-tenant enrollment

### 9.1 Per-crew biometric badge enroll

`[H] → identity` (cross-tenant into `[P]`) — `POST /v1/identity/biometric-badge/enroll-to-client-access-system`

```json
{
  "principal": "pavel.novak@uklid-horak-sro-plzen-cz",
  "principal_tenant": "uklid-horak-sro-plzen-cz",
  "target_client_tenant": "polycraft-bohemia-as-plzen-cz",
  "target_client_access_system": "polycraft-plzen-iso27001-access-control-v3",
  "contract_id": "contract-uklid-horak-polycraft-2027-01-04",
  "biometric_template_hash_sha256": "<sha256>",
  "biometric_class": "fingerprint_minutiae_isso19794-2",
  "enrolled_at": "2026-12-17T14:18:00+01:00",
  "auto_revoke_on_contract_end": true
}
```

Cedar permit: `identity.biometric_badge_enroll_to_client_access_system` against `ClientAccessSystem::"polycraft-plzen-..."`. Context per cedar-policy.cedar:

```
resource.client_tenant == "polycraft-bohemia-as-plzen-cz"
principal.crew_assigned_to_contract == "contract-uklid-horak-polycraft-2027-01-04"
principal.has_completed_training("csn-262-2006-safety") == true
principal.has_completed_training("gdpr-data-handling") == true
principal.has_completed_training("polycraft-induction") == true
```

Audit: `EVT-J160-BIOMETRIC-BADGE-ENROLLED-009-{name}` dual-sealed.

Repeated for: Pavel Novák, Lenka Šimková, Hoàng Văn Long, Mária Kováčová, Іван Шевченко.

## §10 Czech cleaning-industry community

### 10.1 Community posts

`[H] → community` (cross-tenant to `[C]`) — 4 posts during the journey:

```json
{
  "principal": "tomas.horak@uklid-horak-sro-plzen-cz",
  "principal_tenant_origin": "uklid-horak-sro-plzen-cz",
  "community_tenant": "cz-cleaning-industry-owner-operators-community",
  "group_id": "main-thread",
  "post_type": "question",
  "post_body_ciphertext_b64": "<MLS-encrypted Czech question text>",
  "tags": ["solvent-residue", "polycraft", "tennant-t7amr"]
}
```

Audit: `EVT-J160-COMMUNITY-PARTICIPATION-CSAR-{1..4}` sealed.

## §11 Pre-go-live readiness + first shift

### 11.1 Readiness check

`[H] → workflow-engine` — `POST /v1/workflows/bid-and-onboard/{instance_id}/readiness-check` Sat Jan 3 23:42 CET.

Audit: `EVT-J160-READINESS-CONFIRMED-009a` sealed.

### 11.2 First shift scan

`[P] → identity` (validating `[H]` crew badges) — biometric scans at 05:48 Mon Jan 4.

Audit: `EVT-J160-FIRST-SHIFT-GATE-SCAN-010-prep` sealed per crew member.

### 11.3 Contract live emission

`[H] + [P] → workflow-engine` — automatic emission at 06:00:18 Mon Jan 4.

Audit: `EVT-J160-CONTRACT-LIVE-007` dual-sealed.

### 11.4 First shift complete

`[H] → workflow-engine` — at 14:18 Mon Jan 4.

```json
{
  "instance_id": "bid-onboard-uklid-horak-polycraft-2026",
  "from_state": "crew_onboarding",
  "to_state": "contract_live",
  "first_shift_end": "2027-01-04T14:18:42+01:00",
  "shift_supervisor": "pavel.novak@uklid-horak-sro-plzen-cz",
  "incidents_count": 0,
  "csn_en_13549_visual_check_zones_passing": ["zone_1", "zone_2", "zone_3"],
  "ten_t_amr_pad_uptake_evidence_photo_id": "photo-2027-01-04-pad-zone-3"
}
```

Audit: `EVT-J160-FIRST-SHIFT-COMPLETE-010` dual-sealed.

## §12 Denied paths (must be tested — `⟂`)

| Probe | Cedar deny | Audit class |
|---|---|---|
| `⟂` Tomáš bid without ISSA-CIMS cert | FORBID-1 cert-missing | `EVT-J160-CEDAR-DENY-CERT-MISSING-014a` |
| `⟂` Bid submitted after window closes (Friday Oct 17 17:01 CET) | FORBID-2 bid-window-closed | `EVT-J160-CEDAR-DENY-BID-WINDOW-CLOSED-014b` |
| `⟂` Diacritic ASCII transliteration in legal field (Tomáš → Tomas) | FORBID-3 diacritic-strict | `EVT-J160-CEDAR-DENY-NAME-TRANSLITERATE-014c` |
| `⟂` Biometric badge enroll without ČSN-262 training | FORBID-4 training-prereq | `EVT-J160-CEDAR-DENY-BADGE-TRAINING-MISSING-014d` |
| `⟂` PolyCraft reads Úklid Horák payroll | FORBID-5 cross-tenant-payroll | `EVT-J160-CEDAR-DENY-PAYROLL-CROSS-TENANT-014e` |
| `⟂` Úklid Horák reads PolyCraft customer list | FORBID-6 cross-tenant-customer | `EVT-J160-CEDAR-DENY-CUSTOMER-CROSS-TENANT-014f` |
| `⟂` Contract sign without QES dual signature | FORBID-7 incomplete-signature | `EVT-J160-CEDAR-DENY-CONTRACT-INCOMPLETE-SIG-014g` |
| `⟂` ARES verification failure | FORBID-8 business-register-unverified | `EVT-J160-CEDAR-DENY-ARES-014h` |

All deny paths dual-seal.

## §13 Diacritic + multi-script fidelity invariants

| Field | Expected stored form | Forbidden form |
|---|---|---|
| Tomáš Horák | "Tomáš Horák" UTF-8 NFC | "Tomas Horak" ASCII |
| Martina Procházková | "Martina Procházková" NFC | "Martina Prochazkova" |
| Lenka Šimková | "Lenka Šimková" NFC | "Lenka Simkova" |
| Hoàng Văn Long | "Hoàng Văn Long" NFC with Vietnamese tones | any tone-mark loss |
| Mária Kováčová | "Mária Kováčová" NFC | "Maria Kovacova" |
| Іван Шевченко | "Іван Шевченко" Cyrillic NFC OR "Ivan Shevchenko" Latin (user's choice; both stored) | any forced transliteration without user consent |
| Pavel Novák | "Pavel Novák" NFC | "Pavel Novak" |
| Anna Horáková (Tomáš's daughter, mentioned for completeness) | "Anna Horáková" NFC | none |
| ARES auto-fill business name | "Úklid Horák s.r.o." NFC | "Uklid Horak s.r.o." |
| Address: Skvrňanská třída | UTF-8 NFC | ASCII-decomposed |
| Hygienická stanice Plzeňského kraje | NFC | ASCII |
| OSSZ system: Czech rodné číslo + name | NFC strict; passport-ASCII NOT permitted unless explicit | passport-ASCII default |

## §14 Performance envelope

| Operation | p50 | p95 | p99 |
|---|---|---|---|
| Bid submit | 280 ms | 680 ms | 1.4 s |
| Bid evaluation per criterion | 140 ms | 380 ms | 640 ms |
| Award notification dual-seal | 180 ms | 420 ms | 720 ms |
| Contract QES dual-tenant sign with TrueTime fence | 1.8 s | 3.2 s | 5.4 s |
| Datová schránka notification | 480 ms | 1.2 s | 2.4 s |
| OSSZ employee registration | 320 ms | 780 ms | 1.6 s |
| ZP registration | 240 ms | 580 ms | 1.2 s |
| Biometric badge cross-tenant enroll | 480 ms | 1.1 s | 2.2 s |
| Community post MLS encrypt + seal | 120 ms | 280 ms | 480 ms |
| Cedar evaluation per cross-tenant action | 35 ms | 95 ms | 180 ms |
| First-shift-complete dual-seal | 180 ms | 420 ms | 720 ms |

## §15 Cell residency invariants

| Tenant | Cell |
|---|---|
| `uklid-horak-sro-plzen-cz` | `eu-prague-primary` |
| `polycraft-bohemia-as-plzen-cz` | `eu-prague-primary` |
| `cz-ossz-state-tenant` | `eu-prague-primary` (Czech state requires CZ residency) |
| `cz-vzp-zp-tenant` | `eu-prague-primary` |
| `cz-financni-urad-tenant` | `eu-prague-primary` |
| `cz-datova-schranka-tenant` | `eu-prague-primary` |
| `cz-cleaning-industry-owner-operators-community` | `eu-prague-primary` |
| DR replica | `eu-frankfurt-secondary` |
| Analytics read replica | `eu-vienna-tertiary` |

All cross-tenant audits dual-seal within `eu-prague-primary` for Czech residency compliance.

## §16 Stop condition

The handshake matrix is complete when every cross-tenant transition (bid-submit, award-accept, contract-sign, OSSZ/ZP/FÚ/Datová registrations, biometric-badge enroll, first-shift complete) dual-seals, every Cedar deny path produces audit, the diacritic+multi-script fidelity invariants hold across Czech/Vietnamese/Slovak/Ukrainian/Cyrillic names, performance gates are met, and the contract reaches `contract_live` state at 06:00:18 CET Mon Jan 4 2027.
