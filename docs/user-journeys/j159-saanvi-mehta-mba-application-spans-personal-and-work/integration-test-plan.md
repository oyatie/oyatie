---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j159-saanvi-mehta-mba-application-spans-personal-and-work
date: 2026-05-20
authority_tier: 2
status: draft
---

# j159 — Integration test plan

Intern-buildable plan: stand up the seven-tenant seeded fixture (Saanvi's personal + Stripe-India work + Stripe-corporate-US HR + Marico-India work + Wharton-R2 community + Arjun's spousal + Wharton-admissions transient) plus mocks for HDFC payment gateway 3D-Secure SMS, Wharton/Stanford/HBS/Booth/INSEAD admissions portals, GMAC GMAT score-send service, Manhattan Prep LMS API, and Stripe HR audit principal. Walk every test in order; every test names seed values, exact API calls, expected event chain across all involved tenants (dual-seal where cross-tenant), and pass/fail criteria.

## Test environment

| Component | Source |
|---|---|
| Seed tenant — Saanvi personal | `tests/fixtures/tenants/saanvi-mehta-personal.yaml` |
| Seed tenant — Stripe India work | `tests/fixtures/tenants/stripe-india-pvt-ltd.yaml` |
| Seed tenant — Stripe corporate US (HR) | `tests/fixtures/tenants/stripe-corporate-us.yaml` |
| Seed tenant — Marico India work | `tests/fixtures/tenants/marico-india-pvt-ltd.yaml` |
| Seed tenant — Wharton R2 community | `tests/fixtures/tenants/wharton-r2-2027-prospective-applicants-community.yaml` |
| Seed tenant — Arjun spousal | `tests/fixtures/tenants/arjun-mehta-personal.yaml` |
| Seed tenant — Wharton admissions | `tests/fixtures/tenants/wharton-mba-admissions-us.yaml` |
| Seed personas | `tests/fixtures/personas/{saanvi-mehta,priya-krishnamurthy,rajesh-subramanian,arjun-mehta,mohammed-akram,hr-systems-stripe-corporate-us}.yaml` |
| Seed essays | `tests/fixtures/notes/saanvi-mehta-essays-v9-frozen.yaml` (5 essays for 5 schools) |
| Seed GMAT scores | `tests/fixtures/lms/saanvi-mehta-gmat-focus-745-2026-10-18.yaml` |
| Seed transcripts | `tests/fixtures/lms/saanvi-mehta-iit-bombay-2014.yaml`, `tests/fixtures/lms/saanvi-mehta-iim-calcutta-2019.yaml` |
| Seed provider-credential BYOK credentials (ADR-0255 §D-4) | `tests/fixtures/payments/saanvi-hdfc-millennia-7314.yaml`, `tests/fixtures/payments/saanvi-stripe-corporate-amex-4119.yaml` |
| Seed Cedar bundle | `tests/fixtures/cedar/j159/cedar-bundle-dual-tenant-mba-v1.cedar` |
| Seed marriage attestation | `tests/fixtures/identity/saanvi-arjun-marriage-attestation-2026-10-04.yaml` |
| Wire mock — Wharton admissions portal | `tests/mocks/wharton-mba-admissions-r2-2027.toml` |
| Wire mock — Stanford GSB portal | `tests/mocks/stanford-gsb-r2-2027.toml` |
| Wire mock — HBS portal | `tests/mocks/hbs-r2-2027.toml` |
| Wire mock — Chicago Booth portal | `tests/mocks/booth-r2-2027.toml` |
| Wire mock — INSEAD portal | `tests/mocks/insead-r2-2027.toml` |
| Wire mock — HDFC 3D-Secure | `tests/mocks/hdfc-3ds-sms-otp.toml` |
| Wire mock — GMAC GMAT score-send | `tests/mocks/gmac-score-send-2027.toml` |
| Wire mock — Manhattan Prep LMS | `tests/mocks/manhattan-prep-gmat-focus-quant-di.toml` |
| Frozen clock | `freeze_clock(2026-12-06T21:46:00+05:30)` then advance per test |

## Seed data summary

| Datum | Value |
|---|---|
| Saanvi personal email | `saanvi@saanvi.mehta.personal` |
| Saanvi work email | `saanvi.mehta@stripe-india-pvt-ltd` |
| Priya work email | `priya.krishnamurthy@stripe-india-pvt-ltd` |
| Rajesh work email | `rajesh.subramanian@marico-india-pvt-ltd` |
| HR systems principal | `hr-systems@stripe-corporate-us` |
| Wharton essay | 650 words (final v9), UTF-8 NFC |
| GMAT 745 | Q90 V87 DI88; tested 2026-10-18 |
| HDFC Millennia card | personal provider-credential BYOK; last 4 = 7314; ADR-0255 §D-4 |
| Stripe corporate Amex | corporate; last 4 = 4119; FORBIDDEN for personal-tenant payments |
| Application fee | USD 275 × 4 + USD 250 (INSEAD) = USD 1,350 (plus GMAC score-send $35) |
| Submission deadlines | Wharton 2027-01-05 23:59 ET; Stanford 2027-01-06; HBS 2027-01-04; Booth 2027-01-05; INSEAD 2027-01-12 |
| Marriage attestation | Saanvi + Arjun, 2026-10-04 |
| Community tenant membership | 47 members, MLS group epoch 47 at Saanvi's join |

## Test catalog

### T-J159-001 — Essay finalize in personal tenant

**Pre-conditions:** Clock `2026-12-06T21:46:00+05:30`. Saanvi signed into iPad personal-tenant.

**Action sequence:**

1. Finalize essay v9 at 650 words via `notes.document_finalize`
2. Verify drive path materialized
3. Verify audit chain

**Expected events:**

- `EVT-J159-ESSAY-FINALIZED-001` sealed in `saanvi.mehta.personal`
- Drive path resolves to `personal-drive://saanvi.mehta.personal/saanvi/mba-2027/essays/wharton/essay-1-why-mba-why-wharton-why-now-final.docx`
- Merkle leaf populated

**Pass criteria:**

- Word count exactly 650
- UTF-8 NFC preserved
- No cross-tenant audit triggered (this is a single-tenant action)
- p95 finalize latency ≤ 220 ms

**Fail criteria:** word count drift; NFC normalization fail; cross-tenant audit spuriously triggered; latency >320 ms.

### T-J159-002 — Spousal capability grant + use

**Pre-conditions:** T-J159-001 passed. Marriage attestation seed loaded.

**Action sequence:**

1. Saanvi grants spousal read-only capability via `tenancy.capability_grant`
2. Arjun's tenant receives capability
3. Arjun lists folder + reads file at 13:18 IST Dec 7

**Expected events:**

- `EVT-J159-SPOUSAL-REVIEW-012` dual-sealed in both spousal tenants
- Arjun's read attempt succeeds (read-only)

**Pass criteria:**

- Grant requires Saanvi's passkey
- Arjun's read scope is bounded to `/saanvi/mba-2027/essays/wharton/`
- Arjun attempts download → 403 (FORBID-5)
- Capability auto-expires 2026-12-22 23:59 IST

**Fail criteria:** download succeeded; capability outlives expiry; passkey skipped.

### T-J159-003 — Recommender cross-tenant capability accept (Priya)

**Pre-conditions:** Saanvi has invited Priya via Wharton portal mock.

**Action sequence:**

1. Wharton mock sends recommender-invite email to Priya's work-tenant mail
2. Mail µservice renders inline card
3. Priya taps Accept (work identity)
4. Cedar evaluates cross-tenant capability

**Expected events:**

- `EVT-J159-RECOMMENDER-INVITE-ARRIVED-002a` sealed in `stripe-india-pvt-ltd`
- `EVT-J159-RECOMMENDER-ACCEPT-002` dual-sealed in `saanvi.mehta.personal` AND `stripe-india-pvt-ltd`

**Pass criteria:**

- Cedar decision: `permit`; reason contains "work_context_principal_signing_as_manager"
- Priya's capability scope = `write_once_no_browse`
- Capability auto-revoke set to 2027-01-06 23:59 ET
- Cedar evaluation latency ≤ 95 ms (p95)

**Fail criteria:** Cedar allow without write_once_no_browse; capability persists past revoke window.

### T-J159-004 — Priya browses personal-tenant drive (FORBID-3)

**Pre-conditions:** T-J159-003 passed. Priya has capability for slot.

**Action sequence:** Priya attempts to `drive.list_folder` on `/saanvi/mba-2027/essays/`.

**Expected events:**

- Cedar evaluates FORBID-3 (capability scope = write_once_no_browse, no list/read of broader drive)
- `EVT-J159-CEDAR-DENY-RECOMMENDER-BROWSE-014c` dual-sealed
- HTTP 403 with body `{"error":"capability_scope_excludes_browse"}`

**Pass criteria:**

- Browse refused with explicit doctrine
- Priya's UI shows "you cannot browse Saanvi's other personal files"
- Audit dual-sealed

**Fail criteria:** any allow path; silent fail.

### T-J159-005 — Priya forwards capability (FORBID-4)

**Pre-conditions:** T-J159-003 passed.

**Action sequence:** Priya attempts `tenancy.capability_propagate` to grant a third party (e.g., another Stripe colleague) access to the recommendation slot.

**Expected events:**

- Cedar evaluates FORBID-4 (no propagation)
- `EVT-J159-CEDAR-DENY-RECOMMENDER-FORWARD-014d` dual-sealed
- HTTP 403

**Pass criteria:** propagation refused.

**Fail criteria:** propagation allowed; silent allow.

### T-J159-006 — Priya final-submits recommendation

**Pre-conditions:** T-J159-003 passed. Priya has drafted letter via revisions 1-3.

**Action sequence:** Priya taps Final Submit at 18:42 IST Dec 7 with passkey.

**Expected events:**

- `EVT-J159-RECOMMENDER-PRIYA-FINAL-002c` dual-sealed
- Capability status transitions to `consumed`
- Subsequent write attempts return 403

**Pass criteria:**

- Final-submit requires passkey re-prompt
- Capability auto-consumes
- Saanvi's tenant log records the receipt (not the body — body stored encrypted)
- Wharton portal mock confirms receipt within 60s

**Fail criteria:** capability remains writable after final submit; Wharton portal does not confirm.

### T-J159-007 — HR audit sweep refused at personal-tenant boundary

**Pre-conditions:** HR sweep principal seeded. Saanvi has both personal + work tenants.

**Action sequence:**

1. HR sweep walks `stripe-india-pvt-ltd` for Saanvi
2. HR sweep probes `discovery.walk_all_principal_artifacts(saanvi.mehta)`
3. HR sweep emits attestation

**Expected events:**

- Work-tenant walk returns 217 documents
- Broader probe returns 403 with FORBID-1
- `EVT-J159-CEDAR-DENY-WORK-TENANT-INTO-PERSONAL-014a` dual-sealed
- `EVT-J159-HR-SWEEP-NO-PERSONAL-LEAK-004` sealed positive attestation

**Pass criteria:**

- ZERO personal-tenant artifacts in response
- ZERO references to Wharton/Stanford/HBS/Booth/INSEAD in work-tenant walk
- Refusal is explicit (not silent empty result)
- Personal-tenant transparency log records the external query attempt
- HR analyst dashboard shows the refusal

**Fail criteria:** any personal-tenant artifact leak; silent empty result; missing transparency log entry.

### T-J159-008 — Personal-tenant payment with HDFC card

**Pre-conditions:** T-J159-001–006 passed. Saanvi opens Wharton fee payment screen.

**Action sequence:**

1. Saanvi selects HDFC Millennia 7314
2. `payments.authorize` called with personal-tenant provider-credential BYOK credential (ADR-0255 §D-4)
3. HDFC 3DS mock sends SMS OTP
4. Saanvi enters OTP `184729`
5. Payment confirms

**Expected events:**

- `EVT-J159-WHARTON-FEE-PAID-005` sealed in `saanvi.mehta.personal`
- Settlement scheduled T+1
- Wharton admissions cross-tenant ack `EVT-J159-WHARTON-ACK-006` dual-sealed

**Pass criteria:**

- Authorization succeeds
- 3DS OTP flow completes ≤ 30s
- Settlement scheduled correctly
- Cross-tenant ack within 5 min
- Cell residency `ap-mumbai-primary` for the auth record

**Fail criteria:** authorization fails; 3DS times out; ack >5 min; wrong cell residency.

### T-J159-009 — Stripe corporate Amex on personal-tenant payment (FORBID-2)

**Pre-conditions:** Same as T-J159-008 but Saanvi attempts to use the Stripe corporate Amex 4119.

**Action sequence:** `payments.authorize` with corporate Amex credential.

**Expected events:**

- Cedar evaluates FORBID-2 (corporate_card_not_eligible_for_personal_tenant_payment)
- `EVT-J159-CEDAR-DENY-CORPORATE-CARD-PERSONAL-PAYMENT-014b` sealed
- HTTP 403 with explicit doctrine anchor in response body

**Pass criteria:**

- Payment refused
- UI shows the corporate Amex grayed out with doctrine reason
- No authorization sent to network

**Fail criteria:** auth sent; UI omits doctrine reason; silent fail.

### T-J159-010 — Wharton application submit + cross-tenant ack

**Pre-conditions:** T-J159-001–008 passed.

**Action sequence:**

1. Saanvi submits via `workflow-engine.mba-application.submit` at 22:14:34 IST Dec 11
2. Wharton portal mock processes
3. Cross-tenant ack returns

**Expected events:**

- `EVT-J159-WHARTON-ACK-006` dual-sealed
- Application state transitions to `submitted`

**Pass criteria:**

- All required refs present (essay, recommender finals, fee, GMAT, transcripts)
- Cross-tenant ack arrives within 90s
- State machine reaches `submitted`

**Fail criteria:** any required ref missing; ack >5 min; state machine stuck.

### T-J159-011 — GMAT score-send to all 5 schools

**Pre-conditions:** Saanvi has GMAT 745 official record.

**Action sequence:** `learning-management.credentials.gmat.score-send` to all 5 schools.

**Expected events:**

- `EVT-J159-GMAT-SCORE-SEND-007` sealed
- GMAC mock confirms each delivery within 24h

**Pass criteria:**

- All 5 schools confirm receipt
- Score (745) and breakdown preserved at byte level
- Saanvi's name on the score report matches her IIT/IIM/passport ASCII spelling (per ICAO 9303 exception)
- Fee $35 charged to HDFC card

**Fail criteria:** any school does not confirm; score data drift; fee charged to corporate Amex.

### T-J159-012 — Community participation isolated

**Pre-conditions:** Saanvi is a member of `wharton-r2-2027-prospective-applicants-community`.

**Action sequence:**

1. Saanvi posts question at 11:08 IST Dec 13
2. 11 members reply over 4 hours
3. Probe whether HR sweep (T-J159-007 redux) sees this content

**Expected events:**

- `EVT-J159-COMMUNITY-PARTICIPATION-008` sealed in personal-tenant (membership-link) AND community-tenant (content)
- Post body stored ONLY in community tenant

**Pass criteria:**

- Saanvi's work-tenant has zero references to this community post
- Saanvi's personal-tenant log records membership-action; body is not duplicated
- HR sweep probe finds nothing
- Community thread is MLS-encrypted at epoch 47

**Fail criteria:** post body in work-tenant; post body duplicated in personal-tenant drive without explicit save; HR sweep finds content.

### T-J159-013 — Calibration day clean-boundary attestation

**Pre-conditions:** Dec 14, 09:00–18:00 IST.

**Action sequence:**

1. Saanvi works on work-tenant (Stripe MacBook) only
2. No personal-tenant access from any device during window
3. Stripe calibration meeting concludes at 17:42 IST
4. Auto-attestation emits at 18:00 IST

**Expected events:**

- `EVT-J159-CALIBRATION-DAY-CLEAN-BOUNDARY-009` sealed

**Pass criteria:**

- Personal-tenant activity events in window: 0
- Work-tenant calibration outcome: Saanvi rated `Exceeds`
- Attestation is auto-generated, not user-action
- Attestation references the work-tenant calibration meeting by ID

**Fail criteria:** any personal-tenant activity event in window; attestation missing.

### T-J159-014 — Marico recommender accept + final-submit

**Pre-conditions:** Saanvi has invited Rajesh.

**Action sequence:** Similar to T-J159-003/006 but for Rajesh in Marico tenant.

**Expected events:**

- `EVT-J159-RECOMMENDER-MARICO-ACCEPT-003` dual-sealed
- `EVT-J159-RECOMMENDER-MARICO-FINAL-003a` dual-sealed

**Pass criteria:** Same as T-J159-003/006, applied to Marico tenant.

**Fail criteria:** same failure modes.

### T-J159-015 — All-schools-submitted milestone

**Pre-conditions:** All previous tests passed; 5 schools submitted by Dec 22 22:48 IST.

**Action sequence:** Auto-emission of milestone audit at 22:48:18 IST Dec 22.

**Expected events:**

- `EVT-J159-ALL-SCHOOLS-SUBMITTED-010` sealed
- Application tracker dashboard shows 5/5 SUBMITTED

**Pass criteria:**

- All 5 cross-tenant acks dual-sealed
- Total fees paid: USD 1,400
- Dashboard shows decision-window dates

**Fail criteria:** any school in non-`submitted` state; total fee mismatch.

### T-J159-016 — Recommender withdrawal probe

**Pre-conditions:** Variant fixture where Saanvi withdraws Priya's invitation at 08:00 IST Dec 8.

**Action sequence:**

1. Saanvi revokes capability `cap-priya-wharton-rec-2026-12-07`
2. Wait 90 seconds
3. Priya attempts `cross-tenant-slot/append` at 08:01:30 IST

**Expected events:**

- `EVT-J159-RECOMMENDER-CAP-REVOKED-013-test` dual-sealed
- Priya's append attempt → 403 with `capability_revoked`

**Pass criteria:**

- Revocation propagates ≤ 90s
- Priya's UI shows "this capability has been revoked"
- Wharton portal updates recommender slot to "withdrawn"

**Fail criteria:** revocation propagation >90s; Priya can still write; portal does not update.

### T-J159-017 — Diacritic + Devanagari fidelity

**Pre-conditions:** Seeded names include Devanagari forms.

**Action sequence:**

1. Read names from `identity` µservice
2. Write to essay metadata, recommender slot metadata, payment receipt, audit chain
3. Query persisted forms

**Expected behavior:** All names preserve UTF-8 NFC; Devanagari forms preserve canonical composition.

**Pass criteria:**

- "Saanvi Mehta" (Latin) intact
- "सान्वी मेहता" (Devanagari) intact, NFC, no NFD decomposition
- "Priya Krishnamurthy", "Rajesh Subramanian", "Anaya Mehta", "Arjun Mehta" intact
- INSEAD's "Institut Européen d'Administration des Affaires" preserves the "É"
- Marathi greetings in spousal messages preserve diacritics + matras
- Search "Saanvi" (Latin) returns Saanvi; "सान्वी" (Devanagari) also returns Saanvi
- Search "Lazar" returns NOTHING (different person; Romanian j157 fixture)

**Fail criteria:** any normalization; any transliteration without explicit user request.

### T-J159-018 — Cell residency invariants

**Pre-conditions:** Multi-cell fixture.

**Action sequence:** Verify each tenant + audit + payment is in its correct cell.

**Pass criteria:**

- `saanvi.mehta.personal` → `ap-mumbai-primary`
- `stripe-india-pvt-ltd` → `ap-mumbai-primary`
- `stripe-corporate-us` → `us-east-virginia-secondary`
- `marico-india-pvt-ltd` → `ap-mumbai-primary`
- `wharton-mba-admissions-us` → `us-east-virginia-secondary`
- INSEAD has multi-cell SG + FR residency; Saanvi's INSEAD interaction touches `sg-singapore-secondary` + `eu-paris-tertiary`
- HDFC payment auth resident in `ap-mumbai-primary`
- Cross-cell audit dual-seal works across residency

**Fail criteria:** any tenant in wrong cell; audit dual-seal fails across cells.

### T-J159-019 — Cross-tenant audit dual-seal fuzz

**Pre-conditions:** All prior tests pass.

**Action sequence:** 750 generated cross-tenant operations from the closed set `{recommender.accept, recommender.write, recommender.final_submit, spousal.read, hr_sweep.probe, payment.auth, gmat.score_send, community.post, admissions.ack}` × {all 7 tenants}.

**Expected behavior:** Every permitted op dual-seals; every denied op dual-seals deny.

**Pass criteria:**

- 0 single-seal events on cross-tenant transitions
- 0 silent passes
- p99 dual-seal ≤ 320 ms
- Merkle chain validates across all 750 events

**Fail criteria:** any single-seal; any silent pass; merkle break.

## Performance gates

| Operation | p50 | p95 | p99 |
|---|---|---|---|
| Notes finalize | 80 ms | 220 ms | 380 ms |
| Cross-tenant capability accept | 110 ms | 280 ms | 480 ms |
| Cross-tenant slot append | 120 ms | 320 ms | 540 ms |
| Cedar dual-tenant boundary eval | 35 ms | 95 ms | 180 ms |
| Personal-tenant payment auth + 3DS | 1.8 s | 4.2 s | 7.8 s |
| HR sweep walk per 100 docs | 240 ms | 580 ms | 1.1 s |
| HR sweep cross-tenant FORBID | 28 ms | 75 ms | 145 ms |
| Community post MLS encrypt + seal | 95 ms | 240 ms | 420 ms |
| All-schools-submitted attestation | 140 ms | 320 ms | 540 ms |
| Diacritic-aware search | 40 ms | 110 ms | 220 ms |

## Cross-tenant invariant tests

| Invariant | Probe | Pass condition |
|---|---|---|
| Work-tenant ambient read into personal | `hr-systems → drive.read(saanvi personal)` | 403 + dual-seal |
| Corporate card on personal payment | `payments.auth(corporate amex, personal tenant)` | 403 + dual-seal |
| Recommender browse beyond slot | `priya → drive.list(saanvi mba-2027)` | 403 + dual-seal |
| Recommender forward capability | `priya → tenancy.propagate(cap)` | 403 + dual-seal |
| Spousal write attempt | `arjun → drive.write(saanvi essay)` | 403 + dual-seal |
| Community-tenant post under work identity | `priya → community.post` | 403 + dual-seal |
| Diacritic search in legal mode normalized | `search(legal, "Saanvi") returns Sanskrit-flavored match` | 0 hits |
| Personal-tenant payment cross-cell fence | `auth in ap-mumbai for INSEAD eu-paris cell` | dual-seal with cross-cell trace |

## Chaos scenarios

1. **HDFC 3D-Secure SMS gateway unreachable** — Payments retries with backoff; alternate method offered (HDFC app push approval); audit records the retry path
2. **Wharton portal returns 502 on submit** — Workflow-engine retries with idempotency key; eventual ack within 24h; intermediate state `submit_pending_ack`
3. **MLS DS partition between personal-tenant and community-tenant for 5 min** — Posts queue locally; deliver on recovery; epoch correctness preserved
4. **Cedar service degraded** — Personal-tenant payments fail-safe (reject); work-tenant ambient access still forbidden (fail-closed)
5. **Stripe HR sweep runs during community post** — Both operations run concurrently; community post invisible to HR sweep; both audits independent
6. **Diacritic loss in upstream school portal API** — Schema validation catches; submission rejected; user notified; alternate path uses pre-stored official transcript PDF
7. **Personal-tenant data residency drift attempt** — Cedar refuses any operation that would write personal-tenant data outside `ap-mumbai-primary` without explicit consent

## Sign-off checklist

- [ ] All 19 tests pass
- [ ] All 8 cross-tenant invariant probes return expected dual-seal
- [ ] Performance gates met
- [ ] Chaos scenarios complete without data loss
- [ ] All 6 µservices in `/microservices/` resolve: identity, mail, drive, payments, community, learning-management
- [ ] All 10 ADRs cited resolve
- [ ] Diacritic + Devanagari preservation invariant: 0 normalization events
- [ ] HR sweep refusal explicit (not silent empty)
- [ ] Corporate card on personal payment ALWAYS refused
- [ ] DPO sign-off all 7 tenants
- [ ] AACSB-EDU pack compliance: 15-year retention attested
- [ ] IN-DPDP, EU-GDPR, US-FERPA, UK-DPA-2018 pack activations attested

## Stop condition

Plan complete when all 19 tests pass, the dual-tenant ADR-0311 boundary holds against all 8 stressor probes, all 5 schools reach `submitted` state by Dec 22 with cross-tenant acks dual-sealed, the diacritic + Devanagari fidelity invariant holds across every persisted field, the personal-tenant payment routing is the only allowed path for application fees, and the HR audit sweep refusal is explicit at the personal-tenant boundary with positive attestation in Stripe's corporate audit log.
