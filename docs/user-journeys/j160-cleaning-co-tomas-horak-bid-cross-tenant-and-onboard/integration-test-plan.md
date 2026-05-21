---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j160-cleaning-co-tomas-horak-bid-cross-tenant-and-onboard
date: 2026-05-20
authority_tier: 2
status: draft
---

# j160 — Integration test plan

Intern-buildable plan: stand up the seven-tenant seeded fixture (Úklid Horák + PolyCraft Bohemia + CZ-OSSZ + CZ-ZP + CZ-FÚ + CZ-Datová-schránka + CZ-Cleaning-Industry-Community) plus mocks for ARES (Czech business register), I.CA QES, SecuSign QES, Tennant T7AMR equipment registration, Diversey neutralizer chemistry catalog, and the PolyCraft access-control system. Walk every test in order; every test names seed values, exact API calls, expected event chain across involved tenants (dual-seal where cross-tenant), and pass/fail criteria.

## Test environment

| Component | Source |
|---|---|
| Seed tenant — Úklid Horák | `tests/fixtures/tenants/uklid-horak-sro-plzen-cz.yaml` |
| Seed tenant — PolyCraft Bohemia | `tests/fixtures/tenants/polycraft-bohemia-as-plzen-cz.yaml` |
| Seed tenant — CZ OSSZ | `tests/fixtures/tenants/cz-ossz-state-tenant.yaml` |
| Seed tenant — CZ ZP (VZP, OZP, ZPMV) | `tests/fixtures/tenants/cz-vzp-zp-tenant.yaml` |
| Seed tenant — CZ FÚ | `tests/fixtures/tenants/cz-financni-urad-tenant.yaml` |
| Seed tenant — CZ Datová schránka | `tests/fixtures/tenants/cz-datova-schranka-tenant.yaml` |
| Seed tenant — CZ cleaning community | `tests/fixtures/tenants/cz-cleaning-industry-owner-operators-community.yaml` |
| Seed personas | `tests/fixtures/personas/{tomas-horak,martina-prochazkova,pavel-novak,lenka-simkova,hoang-van-long,maria-kovacova,ivan-shevchenko,jaromir-horak,petra-horakova,anna-horakova,vavrinec-danes,marius-iancu-tennant-czech}.yaml` |
| Seed certs | `tests/fixtures/learning-management/iso-9001-uklid-horak-2025-04-18.yaml`, `tests/fixtures/learning-management/issa-cims-2024-uklid-horak.yaml` |
| Seed provider-credential BYOK card (ADR-0255 §D-4) | `tests/fixtures/payments/tomas-horak-fio-banka-business-card.yaml` |
| Seed Cedar bundle | `tests/fixtures/cedar/j160/cedar-bundle-cleaning-bid-onboard-v1.cedar` |
| Wire mock — CZ ARES | `tests/mocks/cz-ares-business-register.toml` |
| Wire mock — I.CA QES | `tests/mocks/ica-qes-cz-eidas.toml` |
| Wire mock — SecuSign QES | `tests/mocks/secusign-qes.toml` |
| Wire mock — Tennant T7AMR | `tests/mocks/tennant-t7amr-cz-distributor.toml` |
| Wire mock — Diversey chemistry | `tests/mocks/diversey-eu-chemistry-catalog.toml` |
| Wire mock — PolyCraft access control | `tests/mocks/polycraft-iso-27001-access-control-v3.toml` |
| Wire mock — Czech state systems | `tests/mocks/cz-state-bundle-ossz-zp-fu-datova.toml` |
| Frozen clock | `freeze_clock(2026-10-14T14:42:00+02:00)` then advance per test |
| Locale | `cs-CZ` primary; `de-DE`, `vi-VN`, `sk-SK`, `uk-UA`, `en-GB` available |

## Seed data summary

| Datum | Value |
|---|---|
| Úklid Horák IČ | 27488123 |
| Úklid Horák DIČ | CZ27488123 |
| Tomáš Horák birth | 1985-08-19 |
| PolyCraft IČ | 47714232 |
| PolyCraft DIČ | CZ47714232 |
| Bid total | CZK 7,940,000 excl. VAT |
| Site area | 12,400 m² total |
| Contract months | 24 |
| Crew FTE required | 5 (1 foreman + 4 cleaners) |
| ČSN-EN-13549 grade | 4 (majority); 3 (warehouse) |
| ČSN-262-2006 training hours | 8 |
| GDPR training hours | 6 |
| Tennant T7AMR training hours | 4 + cert assessment |
| PolyCraft induction days | 3 |
| First shift | Mon 2027-01-04 06:00 CET |
| QES uncertainty target | ≤ 10 ms (ADR-0252) |
| Datová schránka archival | 7 years |
| OSSZ employee registration window | within 8 days of contract start (Czech labor code) |

## Test catalog

### T-J160-001 — Bid request render in Czech

**Pre-conditions:** Clock `2026-10-14T14:42:00+02:00`. Bid request seeded.

**Action sequence:**

1. Tomáš opens marketplace; GET bid request
2. UI renders Czech text + structured spec

**Expected events:**

- `EVT-J160-BID-REQUEST-READ-001` sealed in `uklid-horak-sro-plzen-cz`

**Pass criteria:**

- All Czech diacritics rendered correctly ("Plzeňský", "úklid", etc.)
- Structured fields populated (area, contract window, ČSN grade, ISO requirement)
- Bid deadline displayed in CET timezone
- Competitor count shown anonymously

**Fail criteria:** any diacritic loss; structured fields missing; timezone drift; competitor identity leaked.

### T-J160-002 — Site walk evidence capture

**Pre-conditions:** T-J160-001 passed. Site-walk task materialized.

**Action sequence:**

1. Tomáš walks the site Wed Oct 15
2. Captures 47 photographs + 12 voice-notes
3. Each photo + voice-note classified at capture

**Expected events:**

- `EVT-J160-SITE-WALK-002` sealed
- All evidence stored in `uklid-horak-sro-plzen-cz` tenant

**Pass criteria:**

- 47 photos with EXIF including zone-ID + timestamp + GPS
- 12 voice-notes transcribed in Czech with NFC preservation
- Each evidence item linked to specific bid line item

**Fail criteria:** EXIF missing; transcript ASCII-normalized; evidence orphaned.

### T-J160-003 — Bid submit Cedar permit chain

**Pre-conditions:** T-J160-002 passed. Tomáš has all attachments ready.

**Action sequence:**

1. POST `/v1/marketplace/bid-requests/{bid_id}/submit-bid` at 16:42:18 CET Wed Oct 15
2. Cedar evaluates with cert + ARES + diacritic context
3. Verify response + dual-seal

**Expected events:**

- Cedar `permit` on `marketplace.bid_submit`
- `EVT-J160-BID-SUBMITTED-003` dual-sealed in both tenants

**Pass criteria:**

- ISSA-CIMS-2024 + ISO-9001 certs verified unexpired
- ARES verification returns active
- Bid total CZK 7,940,000 (excl. VAT) recorded exactly
- Dual-seal latency ≤ 720 ms (p95)
- UTF-8 NFC preserved throughout

**Fail criteria:** missing cert allowed; ARES bypassed; total drift; latency >1.2 s; NFC drift.

### T-J160-004 — Bid window closed refusal (FORBID-2)

**Pre-conditions:** Variant fixture with clock advanced to 2026-10-17T17:01 CET (1 minute past window close).

**Action sequence:** Late bid submit attempt by a hypothetical 6th bidder.

**Expected events:**

- Cedar `forbid` FORBID-2
- `EVT-J160-CEDAR-DENY-BID-WINDOW-CLOSED-014b` sealed
- HTTP 403 with reason "bid_window_closed"

**Pass criteria:**

- Submit refused
- UI shows specific reason in Czech: "Termín pro podání nabídek vypršel"
- Audit dual-sealed

**Fail criteria:** late submit accepted; silent fail.

### T-J160-005 — Diacritic ASCII transliteration refusal (FORBID-3)

**Pre-conditions:** Variant fixture: Tomáš's name input as "Tomas Horak" without diacritics in legal-name field.

**Action sequence:** Attempt to write the ASCII-normalized name to OSSZ or contract field.

**Expected events:**

- Cedar `forbid` FORBID-3
- `EVT-J160-CEDAR-DENY-NAME-TRANSLITERATE-014c` sealed
- HTTP 422 with field diff

**Pass criteria:**

- Write refused
- Diff highlights "Tomas Horak" vs expected "Tomáš Horák"
- Audit dual-sealed

**Fail criteria:** ASCII write accepted; no diff shown.

### T-J160-006 — Award decision + cross-tenant notification

**Pre-conditions:** T-J160-003 passed. Procházková completes evaluation.

**Action sequence:**

1. PolyCraft side: bid evaluated → award decision recorded
2. Cross-tenant messenger post at 14:00:12 CET Mon Oct 27
3. Úklid Horák receives notification

**Expected events:**

- `EVT-J160-BID-EVALUATED-004` sealed in PolyCraft
- `EVT-J160-AWARD-RECEIVED-005` dual-sealed
- Workflow advances `bid_evaluated → award_received`

**Pass criteria:**

- MLS encryption preserved across messenger group
- Tomáš's iPhone receives push within 60s
- State machine transition correct
- p95 dual-seal latency ≤ 420 ms

**Fail criteria:** push >2 min; MLS break; state machine drift.

### T-J160-007 — Contract negotiation + 4 draft iterations

**Pre-conditions:** T-J160-006 passed.

**Action sequence:**

1. Tomáš + Procházková exchange 4 draft revisions over 11 days (Oct 28 – Nov 6)
2. Each draft revision dual-sealed
3. Final draft locked Thu Nov 6

**Expected events:**

- 4 × `EVT-J160-CONTRACT-DRAFT-REVISION-006-{n}` dual-sealed

**Pass criteria:**

- Each revision preserves prior + delta
- Cedar permits draft revision only for principals on contract
- Diacritic preserved in revision text
- p95 revision latency ≤ 480 ms

**Fail criteria:** revision history loss; unauthorized principal; diacritic drift.

### T-J160-008 — QES dual-tenant sign under TrueTime fence

**Pre-conditions:** T-J160-007 passed.

**Action sequence:**

1. Tomáš signs via I.CA QES at 11:18:18 CET Fri Nov 7
2. Procházková signs via SecuSign QES at 11:18:38 CET
3. TrueTime fence holds uncertainty ≤ 10 ms

**Expected events:**

- `EVT-J160-CONTRACT-SIGNED-006` dual-sealed under TrueTime
- Contract state machine `contract_in_negotiation → contract_signed`

**Pass criteria:**

- Both QES certificates verified valid (not expired, not revoked)
- eIDAS qualified
- TrueTime uncertainty ≤ 10 ms
- Audit dual-sealed
- p95 sign latency ≤ 3.2 s

**Fail criteria:** any cert invalid; TrueTime breach; single-seal; latency >5 s.

### T-J160-009 — Datová schránka notification

**Pre-conditions:** T-J160-008 passed.

**Action sequence:** Auto-emission of contract metadata to Datová schránka.

**Expected events:**

- `EVT-J160-DATOVA-SCHRANKA-NOTIFIED-006a` sealed
- Czech state record-keeping populated

**Pass criteria:**

- IČ 27488123 + 47714232 both included
- Contract hash SHA-256 sent
- Notification arrives in Tomáš's Datová schránka mailbox within 5 min
- Czech legal-effect timer starts

**Fail criteria:** missing IČ; hash drift; >10 min delivery.

### T-J160-010 — Crew hiring + offer/accept

**Pre-conditions:** Contract signed. Labor-pool positions published Nov 10.

**Action sequence:**

1. Applicants intake over 2 weeks
2. Tomáš selects 3 hires + confirms 2 internal rotations
3. Offer letters signed via QES
4. Acceptances recorded

**Expected events:**

- `EVT-J160-CREW-SELECTED-008-prep` sealed Wed Nov 26
- Each hire has offer-accept audit

**Pass criteria:**

- All 3 new hires have valid Czech work-permit status
- Hoàng Văn Long Vietnamese permanent residence verified
- Mária Kováčová Slovak EU citizenship verified
- Іван Шевченко Ukrainian temporary protection status verified
- All names preserve diacritics + Cyrillic where applicable

**Fail criteria:** permit status not verified; diacritic loss; name normalization.

### T-J160-011 — OSSZ + ZP + FÚ registration cascade

**Pre-conditions:** T-J160-010 passed.

**Action sequence:**

1. POST OSSZ employee registration for 3 new hires
2. POST ZP registration (VZP for Hoàng, OZP for Mária, ZPMV for Іван)
3. POST FÚ tax registration for each

**Expected events:**

- 3 × `EVT-J160-OSSZ-EMPLOYEE-REGISTERED-008-{name}` sealed
- 3 × `EVT-J160-ZP-REGISTERED-008-{name}` sealed
- 3 × `EVT-J160-FU-TAX-REGISTERED-008-{name}` sealed

**Pass criteria:**

- All names UTF-8 NFC preserved at OSSZ + ZP + FÚ
- Rodné číslo validated correctly
- Within Czech-state-mandated 8-day window
- Each ZP receives correct employee per choice
- p95 OSSZ registration latency ≤ 780 ms

**Fail criteria:** any name normalized; rodné číslo malformed; outside window; wrong ZP routed.

### T-J160-012 — ČSN-262-2006 safety training completion

**Pre-conditions:** T-J160-011 passed.

**Action sequence:**

1. 3 new hires enroll in ČSN-262-2006 course Dec 1-5
2. Tomáš (firm-specific) + BOZP-info instructor (general) deliver 8 hours
3. Assessment passed by each
4. Completion certs issued

**Expected events:**

- 3 × `EVT-J160-CSN-262-TRAINING-COMPLETE-008-{name}` sealed
- Czech-labor-code §103 compliance attested

**Pass criteria:**

- 8 training hours logged per hire
- Assessment score ≥ 70% per hire
- Cert issued with diacritic-preserved name
- LMS retention 5 years

**Fail criteria:** training hours short; assessment skipped; cert ASCII-normalized.

### T-J160-013 — GDPR + CZ-110/2019 training

**Pre-conditions:** T-J160-012 passed.

**Action sequence:** 6-hour GDPR training Dec 8-10.

**Expected events:**

- 3 × `EVT-J160-GDPR-TRAINING-COMPLETE-008-{name}` sealed

**Pass criteria:**

- Special category data handling covered (relevant for Іван's temp protection status)
- Cross-tenant data processor obligations covered
- Assessment passed

**Fail criteria:** training short; special-category section omitted; assessment skipped.

### T-J160-014 — Biometric badge cross-tenant enrollment

**Pre-conditions:** T-J160-012 + T-J160-013 + PolyCraft induction passed.

**Action sequence:**

1. POST biometric badge enroll for each of 5 crew at PolyCraft access control
2. Cedar evaluates training prereqs + contract-assignment

**Expected events:**

- 5 × `EVT-J160-BIOMETRIC-BADGE-ENROLLED-009-{name}` dual-sealed

**Pass criteria:**

- All 5 crew have Cedar permit (training prereqs met)
- Biometric template hash stored in PolyCraft access system
- Auto-revoke on contract end is set
- Scope limited to Plzeň site only (NOT MB, NOT Ostrava)
- p95 enrollment latency ≤ 1.1 s

**Fail criteria:** any crew enroll without training prereq; scope broader than Plzeň; missing auto-revoke; latency >2 s.

### T-J160-015 — Biometric badge enroll without training prereq (FORBID-4)

**Pre-conditions:** Variant fixture: Hoàng Văn Long has not completed ČSN-262 training.

**Action sequence:** Attempt biometric badge enrollment.

**Expected events:**

- Cedar `forbid` FORBID-4
- `EVT-J160-CEDAR-DENY-BADGE-TRAINING-MISSING-014d` sealed
- HTTP 403

**Pass criteria:**

- Refused
- UI shows which training is missing
- Audit dual-sealed

**Fail criteria:** allowed; silent fail.

### T-J160-016 — Cross-tenant payroll read attempt (FORBID-5)

**Pre-conditions:** All prior passed.

**Action sequence:** PolyCraft attempts to read Úklid Horák's payroll for the 5 crew assigned.

**Expected events:**

- Cedar `forbid` FORBID-5
- `EVT-J160-CEDAR-DENY-PAYROLL-CROSS-TENANT-014e` dual-sealed

**Pass criteria:** Refused. PolyCraft cannot see what Úklid Horák pays their crew. This is the canonical client-vs-vendor data isolation.

**Fail criteria:** payroll data leaked.

### T-J160-017 — Cross-tenant customer list read attempt (FORBID-6)

**Pre-conditions:** All prior passed.

**Action sequence:** Úklid Horák attempts to read PolyCraft's customer list (PolyCraft's sales pipeline).

**Expected events:**

- Cedar `forbid` FORBID-6
- `EVT-J160-CEDAR-DENY-CUSTOMER-CROSS-TENANT-014f` dual-sealed

**Pass criteria:** Refused. Úklid Horák cannot see PolyCraft's sales data. The vendor-client relationship is one-way scoped.

**Fail criteria:** customer list leaked.

### T-J160-018 — Community participation

**Pre-conditions:** Tomáš is community member.

**Action sequence:** 4 posts during the journey.

**Expected events:**

- 4 × `EVT-J160-COMMUNITY-PARTICIPATION-CSAR-{1..4}` sealed in community tenant + membership-attestation in Úklid Horák tenant

**Pass criteria:**

- Post bodies stored ONLY in community tenant
- PolyCraft has zero visibility
- MLS encryption with epoch progression
- Czech-language posts preserve diacritics

**Fail criteria:** body in client tenant; MLS break; diacritic drift.

### T-J160-019 — Pre-go-live readiness check

**Pre-conditions:** Crew onboarded; biometric badges enrolled.

**Action sequence:** Readiness check Sat Jan 3 23:42 CET.

**Expected events:**

- `EVT-J160-READINESS-CONFIRMED-009a` sealed
- All checklist items green

**Pass criteria:**

- All 5 crew confirmed
- Equipment loaded (Tennant T7AMR + Vileda carts + Diversey neutralizer)
- Supplies inventoried
- Cedar-prereq matrix all green

**Fail criteria:** any checklist red; missing equipment; missing supplies.

### T-J160-020 — First shift gate scan + contract live

**Pre-conditions:** T-J160-019 passed.

**Action sequence:**

1. Pavel + Lenka + Hoàng + Mária + Іван arrive 05:48 Mon Jan 4
2. Each biometric scan
3. Contract-live emission at 06:00:18

**Expected events:**

- 5 × `EVT-J160-FIRST-SHIFT-GATE-SCAN-010-prep-{name}` sealed
- `EVT-J160-CONTRACT-LIVE-007` dual-sealed at 06:00:18
- `EVT-J160-FIRST-SHIFT-COMPLETE-010` dual-sealed at 14:18:42

**Pass criteria:**

- All 5 biometric scans complete green
- Contract state transitions correctly to `contract_live`
- First shift completes within ±15 min of expected
- ČSN-EN-13549 visual check zones pass

**Fail criteria:** any biometric scan rejected; state machine drift; visual check fails.

### T-J160-021 — Multi-script diacritic fidelity

**Pre-conditions:** All persona seeds loaded.

**Action sequence:**

1. Read each name from `identity` µservice
2. Write into contracts, payslips, OSSZ, ZP, FÚ, Datová schránka, biometric badge metadata
3. Query each persisted field

**Pass criteria:**

- "Tomáš Horák" + "Martina Procházková" + "Pavel Novák" + "Lenka Šimková": Czech diacritics intact
- "Hoàng Văn Long": Vietnamese tone marks intact (4 distinct glyphs)
- "Mária Kováčová": Slovak diacritics intact
- "Іван Шевченко": Cyrillic intact (alternative Latin "Ivan Shevchenko" also stored per user preference)
- No transliteration in any legal field
- Diacritic-strict search returns exact matches
- Diacritic-insensitive search opt-in only

**Fail criteria:** any normalization; any forced transliteration; search returns spurious matches.

### T-J160-022 — Czech state systems integration end-to-end

**Pre-conditions:** All prior tests passed.

**Action sequence:** Walk ARES → OSSZ → ZP → FÚ → Datová schránka chain.

**Pass criteria:**

- ARES auto-fill returns Úklid Horák s.r.o. + IČ + DIČ correctly
- OSSZ accepts all 3 new-hire registrations within Czech-mandated 8-day window
- ZP accepts each employee's chosen ZP route
- FÚ accepts tax-withholding setup
- Datová schránka receives all legally-required notifications
- All Czech-state systems return success codes

**Fail criteria:** any state-system error; window breach; notification lost.

### T-J160-023 — Cross-tenant audit dual-seal fuzz

**Pre-conditions:** All prior tests passed.

**Action sequence:** 850 generated cross-tenant operations from the closed set `{bid_submit, bid_evaluate, award_notify, contract_draft, contract_sign, biometric_enroll, crew_hire, training_complete, first_shift_scan, payroll_isolate, customer_isolate}` × {Úklid Horák, PolyCraft, CZ state systems, community}.

**Expected behavior:** Every permitted op dual-seals; every denied op dual-seals deny.

**Pass criteria:**

- 0 single-seal events on cross-tenant transitions
- 0 silent passes
- p99 dual-seal ≤ 720 ms
- Merkle chain validates across all 850 events

**Fail criteria:** any single-seal; silent pass; merkle break.

## Performance gates

| Operation | p50 | p95 | p99 |
|---|---|---|---|
| Bid submit | 280 ms | 680 ms | 1.4 s |
| Bid evaluation per criterion | 140 ms | 380 ms | 640 ms |
| Award notification dual-seal | 180 ms | 420 ms | 720 ms |
| QES dual-tenant sign + TrueTime | 1.8 s | 3.2 s | 5.4 s |
| Datová schránka notification | 480 ms | 1.2 s | 2.4 s |
| OSSZ employee registration | 320 ms | 780 ms | 1.6 s |
| ZP registration | 240 ms | 580 ms | 1.2 s |
| Biometric badge enroll | 480 ms | 1.1 s | 2.2 s |
| Community post MLS encrypt | 120 ms | 280 ms | 480 ms |
| Cedar evaluation cross-tenant | 35 ms | 95 ms | 180 ms |
| First-shift gate scan | 240 ms | 580 ms | 1.0 s |

## Cross-tenant invariant tests

| Invariant | Probe | Pass condition |
|---|---|---|
| PolyCraft reads Úklid payroll | `polycraft → payroll.read(uklid horak)` | 403 + dual-seal |
| Úklid reads PolyCraft customer list | `uklid → crm.list(polycraft customers)` | 403 + dual-seal |
| Late bid attempt | `bid_submit` after window | 403 + dual-seal |
| Diacritic ASCII transliteration legal field | `OSSZ name = "Tomas Horak"` | 422 + diff |
| Biometric badge without training | `enroll(hoang) without csn-262` | 403 + dual-seal |
| QES expired cert sign | `sign with expired I.CA cert` | refused; alternate-CA fallback |
| Cross-cell drift (Tomáš's data to US cell) | `write to cell us-east` without override | refused |
| Community tenant from work identity | post under work tenant principal | 403 |

## Chaos scenarios

1. **ARES unreachable during bid prep** — Bid prep queues local cache; submits when ARES recovers; deadline-aware
2. **I.CA QES service degraded during contract sign** — Alternate CA (PostSignum) offered; user re-affirms; sign completes
3. **OSSZ system outage during employee registration** — Registration queues with Czech-state-recognized timestamp; processes when OSSZ recovers; 8-day window paused
4. **PolyCraft access control mock returns 502 on biometric enrollment** — Retry with backoff; alternate-day enrollment scheduled; first-shift contingency
5. **MLS DS partition Úklid ↔ community for 12 min** — Posts queue locally; deliver on recovery
6. **Datová schránka unreachable** — Czech-state-compliant queue + retry; legal-effect timer paused until delivery
7. **Diacritic loss attempted by upstream system** — Schema validation catches; field write rejected; user notified
8. **TrueTime uncertainty spike (>10 ms)** — Sign deferred; retry with backoff; manual escalation path

## Sign-off checklist

- [ ] All 23 tests pass
- [ ] All 8 cross-tenant invariant probes return expected dual-seal
- [ ] Performance gates met
- [ ] Chaos scenarios complete without data loss
- [ ] All 5 µservices in `/microservices/` resolve: marketplace, workflow-engine, payments, tenancy, community
- [ ] All 9 ADRs cited resolve
- [ ] Diacritic + multi-script preservation invariant: 0 normalization events across Czech/Vietnamese/Slovak/Ukrainian
- [ ] Czech state-system integrations (ARES + OSSZ + ZP + FÚ + Datová schránka) all green
- [ ] DPO sign-off both Úklid Horák + PolyCraft tenants
- [ ] ČSN-262-2006 + GDPR + CZ-110/2019 + ČSN-EN-13549 + ČSN-EN-ISO-9001 pack activations attested
- [ ] Contract reaches `contract_live` state at 06:00:18 CET Mon Jan 4 2027

## Stop condition

Plan complete when all 23 tests pass, the multi-script diacritic fidelity invariant holds across Czech + Vietnamese + Slovak + Ukrainian + Cyrillic, the Czech state-system integration chain executes end-to-end without manual intervention, the cross-tenant Cedar-gated bid → contract → biometric-badge pipeline functions without ambient access leaks, the QES dual-tenant signing under TrueTime fence is reliable within ≤ 10 ms uncertainty, and the contract reaches `contract_live` state at 06:00:18 CET Mon Jan 4 2027 with the first shift completing cleanly at 14:18:42 CET.
