---
doc_class: User-Journey-Handshake
journey_id: j159-saanvi-mehta-mba-application-spans-personal-and-work
date: 2026-05-20
authority_tier: 2
status: draft
---

# j159 — Handshake matrix

Every named µservice call across the seven tenants involved in Saanvi Mehta's MBA Round 2 application cycle from Sunday Dec 6, 2026 21:47 IST through Tuesday Dec 22, 2026 22:48 IST. Order matches `story.md`. Every row names source + target tenant, API surface, wire shape, Cedar permit/forbid, and ADR-0263 audit class.

## Notation

- `[P]` Personal tenant `saanvi.mehta.personal`
- `[W]` Work tenant `stripe-india-pvt-ltd`
- `[CW]` Corporate work tenant `stripe-corporate-us` (HR audit principal)
- `[M]` Marico work tenant `marico-india-pvt-ltd` (Rajesh's home)
- `[C]` Community tenant `wharton-r2-2027-prospective-applicants-community`
- `[S]` Spousal personal tenant `arjun.mehta.personal`
- `[A]` Admissions tenant `wharton-mba-admissions-us` (transient cross-tenant ack)
- `→` synchronous request
- `←` synchronous response
- `↪` side-effect on bus
- `⟂` denied path

Transport: HTTPS over QUIC (HTTP/3) per ADR-0253. Cedar evaluations p95 ≤ 180 ms. Cross-tenant audit dual-seal mandatory under ADR-0263. All string fields preserve UTF-8 NFC for Devanagari + Latin diacritics; no transliteration to ASCII without explicit request.

## §1 Essay finalization in personal tenant

### 1.1 Notes µservice — finalize essay

`[P] → notes` — `POST /v1/notes/documents/{doc_id}/finalize`

Path: `doc_id = essay-wharton-r2-2027-why-mba-why-wharton-why-now-v9`

Request:

```json
{
  "principal": "saanvi.mehta.personal:saanvi",
  "tenant_ctx": "saanvi.mehta.personal",
  "doc_id": "essay-wharton-r2-2027-why-mba-why-wharton-why-now-v9",
  "word_count": 650,
  "word_count_max": 650,
  "locale": "en-US",
  "unicode_normalization": "NFC",
  "diacritics_preserved": true,
  "finalized_at": "2026-12-06T21:47:14+05:30",
  "destination_drive_path": "/saanvi/mba-2027/essays/wharton/essay-1-why-mba-why-wharton-why-now-final.docx"
}
```

Response (`200 OK`):

```json
{
  "doc_version_id": "doc-v9-final-2026-12-06T21-47-14",
  "drive_path_resolved": "personal-drive://saanvi.mehta.personal/saanvi/mba-2027/essays/wharton/essay-1-why-mba-why-wharton-why-now-final.docx",
  "merkle_leaf_b64": "<sha256>",
  "audit_class": "EVT-J159-ESSAY-FINALIZED-001"
}
```

Cedar permit: `notes.document_finalize` against `Document::"essay-wharton-..."`. Context: `principal.tenant == resource.tenant`. No cross-tenant complexity.

Audit: `EVT-J159-ESSAY-FINALIZED-001` sealed in `saanvi.mehta.personal`.

### 1.2 Spousal cross-tenant capability grant

`[P] → tenancy` — `POST /v1/tenancy/capabilities/grant`

Request:

```json
{
  "granting_principal": "saanvi.mehta.personal:saanvi",
  "granting_tenant": "saanvi.mehta.personal",
  "grantee_principal": "arjun.mehta.personal:arjun",
  "grantee_tenant": "arjun.mehta.personal",
  "capability_class": "spousal_read_only_folder",
  "scope_drive_path": "/saanvi/mba-2027/essays/wharton/",
  "scope_actions": ["drive.read_file", "drive.list_folder"],
  "scope_actions_forbidden": ["drive.download", "drive.share", "drive.propagate"],
  "expires_at": "2026-12-22T23:59:59+05:30",
  "relationship_basis": "joint_marriage_attestation_2026-10-04"
}
```

Response: `{"capability_id":"cap-spousal-saanvi-arjun-essays-wharton-2026-12-06","status":"granted","dual_sealed":true}`.

Audit: `EVT-J159-SPOUSAL-REVIEW-012` dual-sealed in `saanvi.mehta.personal` AND `arjun.mehta.personal`.

## §2 Recommender invitation — Wharton portal forwards to Priya

### 2.1 Wharton portal sends recommender invite

`[A] → mail` (inbound delivery to `[W]` Priya's work-tenant mail)

External email from `noreply@wharton-mba.upenn.edu` with structured headers:

```
X-Oya-Cross-Tenant-Capability-Request: true
X-Oya-Capability-Class: recommendation_letter_write_once
X-Oya-Origin-Tenant: saanvi.mehta.personal
X-Oya-Target-Slot: slot-saanvi-wharton-r2-2027-primary
X-Oya-Capability-Scope: write_once_no_browse
X-Oya-Auto-Revoke: 2027-01-06T23:59:59-05:00
```

`mail` µservice on Priya's work tenant recognizes the header, classifies the email as a recommender invite, and renders the inline UI card per ux-flow.md.

Audit: `EVT-J159-RECOMMENDER-INVITE-ARRIVED-002a` sealed in `stripe-india-pvt-ltd`.

### 2.2 Priya accepts cross-tenant capability

`[W] → tenancy` — `POST /v1/tenancy/capabilities/accept`

Request:

```json
{
  "accepting_principal": "priya.krishnamurthy@stripe-india-pvt-ltd",
  "accepting_tenant": "stripe-india-pvt-ltd",
  "target_tenant": "saanvi.mehta.personal",
  "target_slot_id": "slot-saanvi-wharton-r2-2027-primary",
  "capability_class": "recommendation_letter_write_once",
  "accept_basis": "work_context_principal_signing_as_manager",
  "accepted_at": "2026-12-07T09:34:42+05:30"
}
```

Response: `{"capability_id":"cap-priya-wharton-rec-2026-12-07","status":"accepted","dual_sealed":true}`.

Cedar permit: `tenancy.cross_tenant_capability_accept` against `RecommendationLetterSlot::"slot-saanvi-wharton-r2-2027-primary"`. Context:

```
capability_grant.granted_by = "saanvi.mehta.personal"
capability_grant.scope = "write_once_no_browse"
capability_grant.target_tenant = "saanvi.mehta.personal"
principal.tenant = "stripe-india-pvt-ltd"
principal.is_authentic_dual_tenant_match = false  // Priya is NOT Saanvi
principal.role_in_origin_tenant_unverified = "manager_at_employer"
```

Audit: `EVT-J159-RECOMMENDER-ACCEPT-002` dual-sealed in `saanvi.mehta.personal` AND `stripe-india-pvt-ltd`.

### 2.3 Priya drafts recommendation letter into cross-tenant slot

`[W] → notes` (writing to a slot whose home tenant is `[P]`) — `POST /v1/notes/cross-tenant-slot/{slot_id}/append`

Path: `slot_id = slot-saanvi-wharton-r2-2027-primary`

Request:

```json
{
  "principal": "priya.krishnamurthy@stripe-india-pvt-ltd",
  "principal_tenant": "stripe-india-pvt-ltd",
  "target_tenant": "saanvi.mehta.personal",
  "slot_id": "slot-saanvi-wharton-r2-2027-primary",
  "capability_id": "cap-priya-wharton-rec-2026-12-07",
  "draft_text_b64": "<base64 letter body, UTF-8 NFC>",
  "is_final_submit": false,
  "revision_number": 1
}
```

Response: `{"revision_id":"rev-saanvi-wharton-rec-priya-r1","capability_remaining_writes":"unlimited_until_final_submit"}`.

Audit: `EVT-J159-RECOMMENDER-PRIYA-DRAFT-002b` dual-sealed.

### 2.4 Priya final-submits

`[W] → notes` — `POST /v1/notes/cross-tenant-slot/{slot_id}/final-submit` at 18:42 IST Dec 7 (next day after refinements)

```json
{
  "principal": "priya.krishnamurthy@stripe-india-pvt-ltd",
  "slot_id": "slot-saanvi-wharton-r2-2027-primary",
  "capability_id": "cap-priya-wharton-rec-2026-12-07",
  "final_text_b64": "<base64 final letter body>",
  "is_final_submit": true,
  "submitted_at": "2026-12-07T18:42:18+05:30",
  "passkey_assertion_b64": "<priya passkey assertion>"
}
```

Response: `{"final_id":"final-saanvi-wharton-rec-priya","capability_status":"consumed","further_writes_forbidden":true}`. Audit: `EVT-J159-RECOMMENDER-PRIYA-FINAL-002c` dual-sealed.

## §3 Marico recommender (Rajesh)

### 3.1 Rajesh accepts

`[M] → tenancy` — `POST /v1/tenancy/capabilities/accept` at 14:18 IST Dec 7

```json
{
  "accepting_principal": "rajesh.subramanian@marico-india-pvt-ltd",
  "accepting_tenant": "marico-india-pvt-ltd",
  "target_tenant": "saanvi.mehta.personal",
  "target_slot_id": "slot-saanvi-wharton-r2-2027-supplementary",
  "capability_class": "recommendation_letter_write_once",
  "accept_basis": "former_skip_manager_at_hul_2017_2019",
  "accepted_at": "2026-12-07T14:18:38+05:30"
}
```

Audit: `EVT-J159-RECOMMENDER-MARICO-ACCEPT-003` dual-sealed in `saanvi.mehta.personal` AND `marico-india-pvt-ltd`.

### 3.2 Rajesh final-submits Dec 11

`[M] → notes` — `POST /v1/notes/cross-tenant-slot/{slot_id}/final-submit` at 11:30 IST Dec 11

Audit: `EVT-J159-RECOMMENDER-MARICO-FINAL-003a` dual-sealed.

## §4 HR audit sweep refused at personal-tenant boundary

### 4.1 HR sweep walks work-tenant drive

`[CW] → drive` — `POST /v1/drive/audit/walk` against `stripe-india-pvt-ltd`

Request:

```json
{
  "principal": "hr-systems@stripe-corporate-us",
  "sweep_id": "q4-2026-anti-leak-sweep-stripe-india",
  "target_tenant": "stripe-india-pvt-ltd",
  "target_user_set": ["saanvi.mehta@stripe-india-pvt-ltd"],
  "sweep_scope": "work_tenant_drive_only",
  "executed_at": "2026-12-09T14:18:00+00:00"
}
```

Response (`200 OK`):

```json
{
  "documents_walked": 217,
  "anomalies_detected": 0,
  "mba_references_found": 0,
  "wharton_references_found": 0,
  "personal_tenant_references_found": 0,
  "result": "clean"
}
```

Audit: `EVT-J159-HR-SWEEP-WORK-TENANT-WALK-004-prep` sealed in `stripe-corporate-us`.

### 4.2 HR sweep probes broader principal artifacts (REFUSED at personal-tenant boundary)

`[CW] → discovery` — `POST /v1/discovery/walk-all-principal-artifacts`

Request:

```json
{
  "principal": "hr-systems@stripe-corporate-us",
  "target_principal_human": "saanvi.mehta",
  "scope": "all_tenants_this_human_is_in",
  "executed_at": "2026-12-09T14:18:08+00:00"
}
```

Cedar evaluates: FORBID-1 (work-tenant principal ambient access to personal-tenant resource).

Response (`403 Forbidden`):

```json
{
  "error": "personal_tenant_boundary_enforced",
  "doctrine_anchor": "ADR-0311",
  "discoverable_tenants": ["stripe-india-pvt-ltd"],
  "non_discoverable_tenants_for_principal": ["personal_tenant_class_redacted"],
  "audit_class": "EVT-J159-CEDAR-DENY-WORK-TENANT-INTO-PERSONAL-014a"
}
```

`⟂` denied. Audit dual-seals in `stripe-corporate-us` (source) AND in `saanvi.mehta.personal` (target — Saanvi's transparency log shows an external query was refused).

### 4.3 HR sweep positive-attestation

`[CW] → audit-chain` — `POST /v1/audit/attestation`

```json
{
  "attestation_class": "anti_leak_sweep_clean",
  "principal_human": "saanvi.mehta",
  "tenants_walked": ["stripe-india-pvt-ltd"],
  "tenants_refused": ["personal_tenant_class"],
  "anomalies": 0,
  "executed_at": "2026-12-09T14:18:18+00:00"
}
```

Audit: `EVT-J159-HR-SWEEP-NO-PERSONAL-LEAK-004` sealed in `stripe-corporate-us`.

## §5 Wharton application fee payment

### 5.1 Payment authorization via personal-tenant provider-credential BYOK card

`[P] → payments` — `POST /v1/payments/authorize`

Request:

```json
{
  "principal": "saanvi.mehta.personal:saanvi",
  "tenant_ctx": "saanvi.mehta.personal",
  "amount": {"currency": "USD", "minor_units": 27500},
  "settlement_currency": "INR",
  "merchant": "wharton-mba-admissions-us:application-fee-r2-2027",
  "credential_id": "byok-cred-hdfc-millennia-7314",
  "credential_class": "personal_card_BYOK",
  "credential_tenant": "saanvi.mehta.personal",
  "context": {
    "personal_tenant_payment": true,
    "corporate_card_routing_attempted": false
  },
  "initiated_at": "2026-12-11T22:14:18+05:30"
}
```

Cedar evaluates: PERMIT (personal-tenant payment via personal-tenant provider-credential BYOK card, ADR-0255 §D-4).

Response: `{"auth_id":"auth-wharton-fee-2026-12-11","status":"authorized","3ds_required":true,"3ds_method":"sms_otp_hdfc"}`.

### 5.2 3D-Secure SMS OTP confirmation

`[P] → payments` — `POST /v1/payments/3ds/confirm`

```json
{
  "auth_id": "auth-wharton-fee-2026-12-11",
  "otp": "184729",
  "confirmed_at": "2026-12-11T22:14:22+05:30"
}
```

Response: `{"auth_id":"auth-wharton-fee-2026-12-11","status":"settled_t_plus_1","settlement_date":"2026-12-12"}`.

Audit: `EVT-J159-WHARTON-FEE-PAID-005` sealed in `saanvi.mehta.personal`.

### 5.3 Corporate card routing FORBIDDEN (sanity probe — required test)

`[P] → payments` — `POST /v1/payments/authorize` with corporate Amex credential

Request:

```json
{
  "principal": "saanvi.mehta.personal:saanvi",
  "tenant_ctx": "saanvi.mehta.personal",
  "amount": {"currency": "USD", "minor_units": 27500},
  "credential_id": "byok-cred-stripe-corporate-amex-4119",
  "credential_class": "corporate_card",
  "credential_tenant": "stripe-india-pvt-ltd"
}
```

Cedar evaluates: FORBID-2 (corporate card not eligible for personal-tenant payment).

Response (`403 Forbidden`):

```json
{
  "error": "corporate_card_not_eligible_for_personal_tenant_payment",
  "credential_tenant_mismatch": true,
  "doctrine_anchor": "ADR-0311 + ADR-0255",
  "audit_class": "EVT-J159-CEDAR-DENY-CORPORATE-CARD-PERSONAL-PAYMENT-014b"
}
```

`⟂` denied.

### 5.4 Wharton application submission

`[P] → workflow-engine` — `POST /v1/workflows/mba-application/{instance_id}/submit`

```json
{
  "instance_id": "mba-app-wharton-r2-2027-saanvi-mehta",
  "school": "wharton",
  "submitted_at": "2026-12-11T22:14:34+05:30",
  "essay_doc_ref": "personal-drive://saanvi.mehta.personal/saanvi/mba-2027/essays/wharton/",
  "recommender_final_refs": [
    "final-saanvi-wharton-rec-priya",
    "final-saanvi-wharton-rec-rajesh"
  ],
  "gmat_score_send_ref": "gmat-score-send-wharton-2026-12-09",
  "fee_payment_ref": "auth-wharton-fee-2026-12-11",
  "transcripts_ref": [
    "transcript-iit-bombay-saanvi-mehta-2014",
    "transcript-iim-calcutta-saanvi-mehta-2019"
  ]
}
```

### 5.5 Cross-tenant ack from Wharton

`[A] → workflow-engine` (back to `[P]`) at 22:14:42 IST

Audit: `EVT-J159-WHARTON-ACK-006` dual-sealed in `saanvi.mehta.personal` AND `wharton-mba-admissions-us`.

## §6 GMAT score-send and prep refresher

### 6.1 GMAT score-send to all 5 schools

`[P] → learning-management` — `POST /v1/lms/credentials/gmat/score-send`

```json
{
  "principal": "saanvi.mehta.personal:saanvi",
  "credential_class": "gmat_focus_score_official",
  "test_date": "2026-10-18",
  "score_total": 745,
  "score_breakdown": {"quant": 90, "verbal": 87, "data_insights": 88},
  "send_to_schools": [
    "wharton-mba-admissions-us",
    "stanford-gsb-admissions-us",
    "hbs-admissions-us",
    "chicago-booth-admissions-us",
    "insead-singapore-fontainebleau"
  ],
  "send_fee_paid_to_gmac": 35.00,
  "sent_at": "2026-12-09T16:18:00+05:30"
}
```

Audit: `EVT-J159-GMAT-SCORE-SEND-007` sealed.

### 6.2 Manhattan Prep refresher session

`[P] → learning-management` — `POST /v1/lms/sessions`

```json
{
  "principal": "saanvi.mehta.personal:saanvi",
  "course_id": "manhattan-prep-gmat-focus-quant-di-advanced",
  "session_id": "session-2026-12-12-19-30-saanvi",
  "started_at": "2026-12-12T19:30:00+05:30",
  "completed_at": "2026-12-12T21:02:00+05:30",
  "questions_attempted": 28,
  "questions_correct": 26,
  "accuracy_pct": 92.86,
  "competency_band_demonstrated": "q90_plus_data_insights"
}
```

Audit: `EVT-J159-GMAT-PREP-LMS-SESSION-007a` sealed.

## §7 Community participation

### 7.1 Saanvi posts to community

`[P] → community` (cross-tenant into `[C]`) — `POST /v1/community/groups/{group_id}/post`

```json
{
  "principal": "saanvi.mehta.personal:saanvi",
  "principal_tenant_origin": "saanvi.mehta.personal",
  "community_tenant": "wharton-r2-2027-prospective-applicants-community",
  "group_id": "main-thread-r2-2027",
  "post_type": "question",
  "post_body_ciphertext_b64": "<MLS-encrypted question body>",
  "mls_epoch_at_post": 47,
  "posted_at": "2026-12-13T11:08:00+05:30",
  "tags": ["optional-essay", "leave-disclosure"]
}
```

Audit: `EVT-J159-COMMUNITY-PARTICIPATION-008` sealed in `saanvi.mehta.personal` (membership log) and `wharton-r2-2027-prospective-applicants-community` (content store).

Note: post body is stored ONLY in the community tenant; the personal-tenant log records the membership-action event but does not duplicate the content (per ADR-0263 minimum-duplication doctrine).

### 7.2 Community replies aggregate

`[C] → community` (intra-tenant) — 11 replies between 12:18–15:48 IST Dec 13

Audit: each reply sealed within `wharton-r2-2027-prospective-applicants-community`; aggregate `EVT-J159-COMMUNITY-REPLIES-AGG-008a` summarizes the thread state.

## §8 Calibration day clean-boundary attestation

### 8.1 Personal-tenant zero-activity attestation

`[P] → audit-chain` — automatic emission at 18:00 IST Dec 14

```json
{
  "attestation_class": "personal_tenant_zero_activity_during_work_calibration",
  "principal_human": "saanvi.mehta",
  "personal_tenant": "saanvi.mehta.personal",
  "window_start": "2026-12-14T09:00:00+05:30",
  "window_end": "2026-12-14T18:00:00+05:30",
  "activity_events_in_window": 0,
  "work_tenant_calibration_meeting_concurrent": true,
  "work_tenant_id": "stripe-india-pvt-ltd"
}
```

Audit: `EVT-J159-CALIBRATION-DAY-CLEAN-BOUNDARY-009` sealed.

## §9 Booth + INSEAD submissions

### 9.1 Booth submit

`[P] → workflow-engine` — `POST /v1/workflows/mba-application/{instance_id}/submit` at 19:18 IST Dec 15 (Booth)

Audit: `EVT-J159-BOOTH-SUBMIT-006a` dual-sealed.

### 9.2 INSEAD submit

`[P] → workflow-engine` at 20:42 IST Dec 15

Audit: `EVT-J159-INSEAD-SUBMIT-006b` dual-sealed (with INSEAD Singapore + Fontainebleau cell residency considerations).

### 9.3 Stanford submit Dec 18

Audit: `EVT-J159-STANFORD-SUBMIT-006c` dual-sealed.

### 9.4 HBS submit Dec 22 22:48 IST

Audit: `EVT-J159-HBS-SUBMIT-006d` dual-sealed.

### 9.5 All-schools-submitted milestone

`[P] → workflow-engine` — automatic emission at 22:48:18 IST Dec 22

Audit: `EVT-J159-ALL-SCHOOLS-SUBMITTED-010` sealed.

## §10 Recommender withdrawal probe (test path; not invoked in baseline scenario)

### 10.1 Withdraw a recommender invitation

`[P] → tenancy` — `POST /v1/tenancy/capabilities/revoke`

```json
{
  "revoking_principal": "saanvi.mehta.personal:saanvi",
  "capability_id": "cap-priya-wharton-rec-2026-12-07",
  "revoked_at": "2026-12-08T08:00:00+05:30",
  "reason": "test_path"
}
```

Response: `{"capability_status":"revoked","propagation_lag_target_seconds":90}`.

Within 90 seconds, Priya's subsequent `cross-tenant-slot/append` attempts return 403 with `capability_revoked`. Audit: `EVT-J159-RECOMMENDER-CAP-REVOKED-013-test` dual-sealed.

## §11 Denied paths (must be tested — `⟂`)

| Probe | Cedar deny rule | Audit class |
|---|---|---|
| `⟂` HR-systems@stripe-corporate-us walks Saanvi's personal-tenant drive | FORBID-1 (ADR-0311 boundary) | `EVT-J159-CEDAR-DENY-WORK-TENANT-INTO-PERSONAL-014a` |
| `⟂` Stripe corporate Amex on personal-tenant payment | FORBID-2 (provider-credential BYOK card-tenant-mismatch, ADR-0255 §D-4) | `EVT-J159-CEDAR-DENY-CORPORATE-CARD-PERSONAL-PAYMENT-014b` |
| `⟂` Priya browses Saanvi's personal-tenant drive (broader than slot) | FORBID-3 (capability scope = write_once_no_browse) | `EVT-J159-CEDAR-DENY-RECOMMENDER-BROWSE-014c` |
| `⟂` Priya forwards Saanvi's recommendation slot to a third party | FORBID-4 (no propagation) | `EVT-J159-CEDAR-DENY-RECOMMENDER-FORWARD-014d` |
| `⟂` Arjun (spouse) tries to write to Saanvi's essay folder | FORBID-5 (spousal capability is read-only) | `EVT-J159-CEDAR-DENY-SPOUSAL-WRITE-014e` |
| `⟂` Stripe HR triggers a personal-tenant CSV export of Saanvi's account | FORBID-1 | `EVT-J159-CEDAR-DENY-HR-EXPORT-014f` |
| `⟂` Community-tenant member tries to post under Saanvi's name from work-tenant identity | FORBID-6 (community membership is personal-tenant-only) | `EVT-J159-CEDAR-DENY-COMMUNITY-WRONG-TENANT-014g` |
| `⟂` Schema-write of "Lazar" instead of "Saanvi" with attempted ASCII transliteration | FORBID-7 (legal-name diacritic strict) | `EVT-J159-CEDAR-DENY-NAME-TRANSLITERATE-014h` |

All deny paths dual-seal.

## §12 Diacritic + Devanagari fidelity invariants

| Field | Expected stored form | Forbidden form |
|---|---|---|
| Saanvi's legal name | `Saanvi Mehta` (Latin, NFC) | any transliteration |
| Saanvi's Devanagari name (rendered where she opts in) | `सान्वी मेहता` (NFC, no NFD decomposition) | NFD-decomposed form |
| Priya's name | `Priya Krishnamurthy` (Latin, NFC) | `Priya Krishna` (truncation) |
| Rajesh's name | `Rajesh Subramanian` (Latin, NFC) | none |
| Anaya's name | `Anaya Mehta` (Latin, NFC) | none |
| Arjun's name | `Arjun Mehta` (Latin, NFC) | none |
| School names | UTF-8 NFC; INSEAD's accent on the "É" never lost: `Institut Européen d'Administration des Affaires` is the underlying organization (though commonly called INSEAD in English) | dropped accent |
| Marathi spousal greeting | "आता" + "मराठीत" pass NFC | NFD-decomposed |

## §13 Performance envelope

| Operation | p50 | p95 | p99 |
|---|---|---|---|
| Notes finalize | 80 ms | 220 ms | 380 ms |
| Cross-tenant capability accept | 110 ms | 280 ms | 480 ms |
| Cross-tenant slot append | 120 ms | 320 ms | 540 ms |
| Cedar dual-tenant boundary eval | 35 ms | 95 ms | 180 ms |
| Personal-tenant payment auth + 3DS | 1.8 s | 4.2 s | 7.8 s |
| HR sweep walk (per 100 docs) | 240 ms | 580 ms | 1.1 s |
| HR sweep cross-tenant FORBID | 28 ms | 75 ms | 145 ms |
| Community post MLS encrypt + seal | 95 ms | 240 ms | 420 ms |
| All-schools-submitted attestation | 140 ms | 320 ms | 540 ms |

## §14 Tenant matrix (7 tenants involved)

| Tenant | Role | Cell residency |
|---|---|---|
| `saanvi.mehta.personal` | Saanvi's personal — home of essays, payments, GMAT, community membership | `ap-mumbai-primary` |
| `stripe-india-pvt-ltd` | Saanvi's employer (work-tenant) — home of Priya as recommender, Stripe HR's calibration | `ap-mumbai-primary` |
| `stripe-corporate-us` | Stripe global HR audit principal home | `us-east-virginia-secondary` |
| `marico-india-pvt-ltd` | Rajesh's employer (supplementary recommender) | `ap-mumbai-primary` |
| `wharton-r2-2027-prospective-applicants-community` | Peer-applicant community (third tenant) | `ap-mumbai-primary` (community-hosted) |
| `arjun.mehta.personal` | Saanvi's spouse personal tenant (spousal review) | `ap-mumbai-primary` |
| `wharton-mba-admissions-us` | Wharton's transient cross-tenant ack target | `us-east-virginia-secondary` |

Plus 4 other admissions tenants invited briefly for similar flow patterns: `stanford-gsb-admissions-us`, `hbs-admissions-us`, `chicago-booth-admissions-us`, `insead-singapore-fontainebleau`.

## §15 Stop condition

The handshake matrix is complete when all 5 schools submit-confirmed, all cross-tenant ACKs dual-seal, all 8 deny paths in §11 successfully refuse with audit, the diacritic/Devanagari fidelity invariants in §12 hold, the performance envelope in §13 is met, and Saanvi's personal-tenant journey log contains the audited proof that her Stripe work-tenant remained boundary-isolated throughout the 16-day cycle.
