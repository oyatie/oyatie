---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j152-ahmad-hassan-construction-site-incident-bilingual
date: 2026-05-20
authority_tier: 2
status: draft
---

# j152 — Integration test plan

This plan is intern-buildable: any new engineer can stand up the seeded test environment and walk through every test in the order they appear. Each test names the seed values, the exact API call invoked, the expected event chain, and the pass/fail criteria.

## Test environment

| Component | Source |
|---|---|
| Seed tenant | `tests/fixtures/tenants/halcyon_build_llc.yaml` |
| Seed personas | `tests/fixtures/personas/{ahmad-hassan,khalil-mansour,roberto-santos,bryan-oconnor,priya-mehrotra}.yaml` |
| Seed crane telemetry | `tests/fixtures/streams/crane.load_pin.sensor_v1/2026-10-14T21h36m-T21h38m.csv` (6,000 samples, deterministic) |
| Seed camera clips | `tests/fixtures/streams/cam-deck-6-{northwest,southeast}/2026-10-14T21h35m-T21h41m.h265` |
| Seed consent token | `tests/fixtures/consent/khalil-allergy-excerpt.signed.json` |
| Cedar policy bundle | `tests/fixtures/cedar/j152/*.cedar` |
| Wire mock — Paycom | `tests/mocks/paycom-eir.toml` |
| Wire mock — State Fund | `tests/mocks/state-fund-froi-1.toml` |
| Wire mock — Cal/OSHA Area Office Oakland | `tests/mocks/cal-osha-oakland-inbox.toml` |
| Wire mock — AMR ePCR bridge | `tests/mocks/amr-epcr.toml` |
| Wire mock — Oakland PSAP CAD | `tests/mocks/oakland-psap-cad.toml` |
| Frozen clock | `freeze_clock(2026-10-14T21:36:00Z)` then advance per test |

## Test catalog

### T-J152-001 — Happy path: incident-create within SLO

**Pre-conditions:** clock frozen at 21:37:11, all µservices healthy, Ahmad pre-enrolled, Khalil pre-enrolled with allergy consent token.

**Action sequence:**

1. Simulate physical SOS button press from Ahmad's DuraForce
2. POST `/v1/identity/stepup` with valid attestation
3. POST `/v1/sites/HB-OAK-4421/incidents` with body from §3.1 of `handshake.md`
4. POST `/v1/incidents/{id}/narrative/voice-note` × 2 (AR, EN)

**Expected events (in order):**

- `EVT-J152-IDENTITY-STEPUP-OK-001`
- `EVT-J152-INCIDENT-CREATE-004`
- `EVT-J152-INCIDENT-NARRATIVE-VOICE-AR-005`
- `EVT-J152-INCIDENT-NARRATIVE-VOICE-EN-006`

**Pass criteria:**

- Incident record exists with status `OPEN-EMS-DISPATCHED`
- `narrative_en` and `narrative_ar` are two distinct fields, not concatenated; both non-empty
- All 4 events sealed in `audit-chain`; `merkle_proof` validates against the day's epoch root
- p95 of `incident.create` request → response ≤ 480ms (per handshake §11)

**Fail criteria:**

- `narrative_en` or `narrative_ar` empty
- Either narrative populated by string-concatenation rather than per-field
- Any event missing the `tenant_id = halcyon_build_llc` attribute

### T-J152-002 — Stop-work fanout to 19 devices

**Pre-conditions:** T-J152-001 completed; 19 device sessions seeded across decks 4-7.

**Action sequence:**

1. POST `/v1/channels/site-hb-oak-4421-deck-6/broadcast` with the bilingual stop-work body
2. Each device session emits its ACK on the messenger streaming endpoint

**Expected events:**

- `EVT-J152-MSG-STOPWORK-FANOUT-003` (one)
- `EVT-J152-MSG-STOPWORK-ACK-001` through `EVT-J152-MSG-STOPWORK-ACK-019` (nineteen)

**Pass criteria:**

- All 19 ACKs received within 30s of fanout
- Per-device rendered language matches the device's locale (en-US devices render EN; ar-EG devices render AR; es-MX devices render ES)
- p95 fanout → first ACK ≤ 4.1s
- p99 fanout → last ACK ≤ 7.4s

**Fail criteria:**

- Any device receives EN when its locale is AR or ES
- Any device fails to ACK within 30s (must emit `EVT-J152-MSG-STOPWORK-ACK-TIMEOUT-NNN` and be flagged)

### T-J152-003 — Auto-attach telemetry + camera

**Pre-conditions:** T-J152-001 completed; seed streams loaded.

**Action sequence:**

(no explicit API call — this is a workflow-engine side-effect of `incident.create`)

**Expected events:**

- `EVT-J152-DRIVE-EVIDENCE-ATTACH-TEL-006a` (crane telemetry, 90s window, 4,500 samples)
- `EVT-J152-DRIVE-EVIDENCE-ATTACH-CAM-NW-006b` (camera NW, 4-minute clip)
- `EVT-J152-DRIVE-EVIDENCE-ATTACH-CAM-SE-006c` (camera SE, 4-minute clip)

**Pass criteria:**

- Each evidence file has a deterministic SHA-256 hash matching the seed fixture's expected hash
- Each evidence file is **read-only** after attach (subsequent write attempts return `409 Conflict`)
- The chain-of-custody link in the incident's evidence folder shows all 3 attachments

**Fail criteria:**

- Any attachment is missing
- Any attachment's hash deviates from the seed expectation (would indicate stream corruption)
- Any attachment is writable after attach

### T-J152-004 — ADR-0298 medical bypass — happy path

**Pre-conditions:** T-J152-001 completed; Ahmad's step-up was ≤120s ago; Khalil's allergy consent token is valid.

**Action sequence:**

1. Tap "Pull worker medical (acute window)" on the incident form
2. POST `/v1/cedar/decide` (via incident-management's sidecar)
3. gRPC `drive.ProjectMedicalExcerpt` for fields `["allergies", "current_medications"]`

**Expected events:**

- `EVT-J152-CEDAR-DECIDE-PERMIT-MED-BYPASS-006c`
- `EVT-J152-DRIVE-MED-EMRG-DISCLOSE-007`

**Pass criteria:**

- Cedar returns `permit` with `bypass_window_expires_at = T+60min`
- Drive returns exactly the two requested fields, no extras (no DOB, no weight, no SSN, no insurance, no full medical history)
- The excerpt is encrypted with the incident's per-record key
- The disclosure event includes the bypass justification + the 60-minute expiry

**Fail criteria:**

- More than the two requested fields are returned (data over-projection)
- Cedar returns `permit` when consent token is missing
- The 60-minute window is shorter or longer than 60 minutes
- The excerpt is stored unencrypted

### T-J152-005 — ADR-0298 medical bypass — denied (stale step-up)

**Pre-conditions:** Ahmad's step-up was >120s ago.

**Action sequence:** same as T-J152-004.

**Expected events:**

- `EVT-J152-CEDAR-DENY-STEPUP-STALE-NNN`

**Pass criteria:**

- Cedar returns `deny` with reason `step_up_seconds_ago > 120`
- No drive projection occurs
- The user is prompted to re-step-up via biometric
- The audit event is sealed

**Fail criteria:**

- Cedar returns `permit` despite stale step-up
- Any drive projection occurs

### T-J152-006 — ADR-0298 medical bypass — denied (consent revoked)

**Pre-conditions:** Khalil's consent token has been revoked (simulate via `tests/fixtures/consent/khalil-allergy-excerpt.revoked.json`).

**Action sequence:** same as T-J152-004.

**Expected events:**

- `EVT-J152-CEDAR-DENY-CONSENT-REVOKED-NNN`

**Pass criteria:**

- Cedar returns `deny`
- The audit event names the revoked consent token's ID

**Fail criteria:**

- Cedar permits the access

### T-J152-007 — Non-site-lead invocation denied

**Pre-conditions:** A crew member (not Ahmad) attempts to pull medical via the incident UI.

**Action sequence:**

1. POST `/v1/cedar/decide` with `principal = manuel.reyes@halcyon-build.com` (a fitter)

**Expected events:**

- `EVT-J152-CEDAR-DENY-NOT-SITE-LEAD-NNN`

**Pass criteria:**

- Cedar denies; the deny reason is `principal.role_on_site != site_lead`
- The crew member can still create a basic incident (a non-privileged path) but cannot pull medical

**Fail criteria:**

- Cedar permits

### T-J152-008 — Cross-tenant drive attachment refused

**Pre-conditions:** A photo from Manuel Reyes's personal-tenant drive is offered for attachment to the Halcyon-tenant incident.

**Action sequence:**

1. gRPC `drive.AttachEvidence` with `source_tenant = manuel-personal` (not `halcyon_build_llc`)

**Expected events:**

- `EVT-J152-CEDAR-DENY-CROSS-TENANT-DRIVE-NNN`

**Pass criteria:**

- Cedar denies; the deny reason is `source_tenant != incident.tenant_id`
- The personal-tenant drive is untouched

**Fail criteria:**

- Any cross-tenant access succeeds

### T-J152-009 — Paycom workplace-integration sync

**Pre-conditions:** T-J152-001 + T-J152-002 + T-J152-003 + T-J152-004 completed.

**Action sequence:**

1. workflow-engine step-7 fires; workplace-integration POSTs to the Paycom mock
2. Mock returns `201` with `paycom_injury_report_id = PCM-EIR-49217`

**Expected events:**

- `EVT-J152-WORKPLACE-PAYCOM-WRITE-011`

**Pass criteria:**

- All 7 mapped fields match the spec in `schemas/workplace-integration-paycom-map.yaml`
- The Paycom ACK is sealed in audit-chain with the Paycom report ID
- Paycom mock receives the bilingual narrative in two fields (`IncidentDescription` + `IncidentDescriptionAlt`), not concatenated

**Fail criteria:**

- Any required Paycom field is missing or mapped wrong
- The bilingual narrative is concatenated

### T-J152-010 — Paycom retry on transient failure

**Pre-conditions:** Paycom mock returns `503` for the first 3 attempts then `201`.

**Expected events:**

- `EVT-J152-WORKPLACE-PAYCOM-RETRY-001`
- `EVT-J152-WORKPLACE-PAYCOM-RETRY-002`
- `EVT-J152-WORKPLACE-PAYCOM-RETRY-003`
- `EVT-J152-WORKPLACE-PAYCOM-WRITE-011`

**Pass criteria:**

- The first 3 retries happen with exponential backoff (base 2s, jitter ±20%; expected delays ~2s, ~4s, ~8s)
- The 4th attempt succeeds
- The final state mirrors T-J152-009

**Fail criteria:**

- The retry sequence stops before 5 attempts on transient failure
- Backoff is constant rather than exponential

### T-J152-011 — State Fund FROI-1 derivation + submit

**Pre-conditions:** T-J152-009 completed.

**Action sequence:**

1. workflow-engine step-8 fires; workplace-integration derives the FROI-1 from incident + Paycom record
2. POSTs to State Fund mock via EDI 148
3. Mock returns `SF-FROI-ACK-2026-10-14-49217`

**Expected events:**

- `EVT-J152-WORKPLACE-STATEFUND-FROI-012`

**Pass criteria:**

- The FROI-1 includes all 14 mandatory fields per DWC Form 5020
- Khalil's DOB is **tokenized** (not raw) on the wire
- The State Fund ACK is sealed

**Fail criteria:**

- Any DWC mandatory field is missing
- Khalil's raw DOB appears on the wire (tokenization failure)

### T-J152-012 — Cal/OSHA §342 timer arming + reminder + escalation

**Pre-conditions:** T-J152-009 + T-J152-011 completed. Clock at 14:50 PDT.

**Action sequence:**

1. workflow-engine arms timer `tmr-cal-osha-342-INC-49217` to fire at 22:37:11 PDT (T+8h from incident)
2. Advance clock to 20:37:11 PDT (T+6h)
3. Reminder fires to Priya
4. Advance clock to 22:37:11 PDT (T+8h)
5. Escalation fires to safety-officer pager (since no ruling has been filed)

**Expected events:**

- `EVT-J152-WORKFLOW-TIMER-SET-010`
- `EVT-J152-WORKFLOW-TIMER-REMINDER-T6H-NNN`
- `EVT-J152-WORKFLOW-TIMER-ESCALATION-T8H-NNN`

**Pass criteria:**

- Timer fires exactly at the scheduled time (±2s)
- Reminder is sent to Priya's pager
- If no ruling is filed by T+8h, escalation pages the safety officer

**Fail criteria:**

- Timer drifts >2s
- Escalation fires when a ruling was already filed
- Reminder/escalation routing is wrong

### T-J152-013 — Priya's §342 ruling cancels escalation

**Pre-conditions:** Clock at 21:00 PDT (between T+6h and T+8h). Priya is logged into HSE dashboard.

**Action sequence:**

1. Priya clicks "Review §342 reportability"
2. She selects "Not reportable — but courtesy filing recommended"
3. She types a ≥200-char rationale
4. She passkey-signs
5. She files the ruling
6. workflow-engine cancels the T+8h escalation timer

**Expected events:**

- `EVT-J152-COMPLIANCE-CALOSHA-RULING-FILED-NNN`
- `EVT-J152-WORKFLOW-TIMER-CANCEL-T8H-NNN`
- `EVT-J152-COMPLIANCE-CALOSHA-COURTESY-FILED-016`

**Pass criteria:**

- Ruling sealed in audit-chain
- T+8h escalation timer cancelled cleanly
- Courtesy filing reaches the Cal/OSHA Area Office Oakland mock inbox

**Fail criteria:**

- T+8h escalation fires despite ruling
- Courtesy filing missing or malformed

### T-J152-014 — Bilingual narrative survives OSHA-301 PDF export

**Pre-conditions:** T-J152-001 completed.

**Action sequence:**

1. GET `/v1/incidents/{id}/exports/osha-301.pdf`

**Pass criteria:**

- The PDF contains both `narrative_en` and `narrative_ar` in distinct sections
- Arabic text is rendered right-to-left with correct shaping (final/medial/initial Arabic letter forms)
- The PDF includes the audit-chain merkle root hash on the last page

**Fail criteria:**

- Arabic letters appear in left-to-right order or in disconnected form (rendering failure)
- Arabic narrative is missing
- PDF lacks the audit-chain hash footer

### T-J152-015 — Bypass-window expiry

**Pre-conditions:** T-J152-004 completed. Clock advances to 15:37:14 PDT (T+60min from disclosure).

**Action sequence:**

1. A first responder attempts to view the share link after expiry
2. The incident-management UI attempts to re-pull the medical excerpt

**Expected events:**

- `EVT-J152-CONNECT-EMS-EXCERPT-TTL-EXPIRED-NNN` (on share-link access)
- `EVT-J152-CEDAR-DENY-STALE-BYPASS-NNN` (on re-pull attempt)

**Pass criteria:**

- Share link returns `410 Gone`
- The displayed excerpt in the incident UI becomes unreadable (replaced with "Window expired — re-invoke bypass to view")
- Re-invoking the bypass requires a NEW step-up + a NEW consent-token check

**Fail criteria:**

- Share link continues to work after T+60min
- The excerpt remains visible after expiry

### T-J152-016 — Role-projection cross-check

**Pre-conditions:** T-J152-013 completed.

**Action sequence:**

1. Ahmad opens the incident from his DuraForce
2. The role-projection layer (ADR-0317) renders the `site_lead_incident_v1` view

**Pass criteria:**

- Ahmad sees: narrative pair, camera clips, his own actions log, §342 timer status, broadcast log
- Ahmad does NOT see: Khalil's full medical record, salary, SSN, the Paycom EIR internal fields, the State Fund FROI-1 PII, Priya's rationale text
- The "lock pill" at the top is visible and accurate

**Fail criteria:**

- Any of the forbidden fields is visible
- The role-projection silently shows more than the site-lead view

### T-J152-017 — Audit-chain merkle proof verification

**Pre-conditions:** All preceding tests complete.

**Action sequence:**

1. GET `/v1/audit-chain/journeys/j152/proofs`

**Pass criteria:**

- The bundle includes all events from EVT-J152-IDENTITY-STEPUP-OK-001 through EVT-J152-COMPLIANCE-CALOSHA-COURTESY-FILED-016
- Each event's merkle proof validates against the day's epoch root
- The day's epoch root is published to the public-key-pinned read endpoint
- No event is missing from the chain

**Fail criteria:**

- Any event missing from the chain
- Any merkle proof fails to validate
- The epoch root is unsigned or pinned to a different key

### T-J152-018 — Tenant scoping invariant (ADR-0244)

**Pre-conditions:** All preceding tests complete.

**Action sequence:** Query each microservice's audit log for all events emitted in this journey.

**Pass criteria:**

- Every single event carries `tenant_id = halcyon_build_llc`
- No event carries a different `tenant_id` or null
- The Paycom/State Fund/Cal/OSHA bridge events carry the source `tenant_id` even though the destination is external

**Fail criteria:**

- Any event missing `tenant_id`
- Any event with a different `tenant_id`

### T-J152-019 — Locale-pack EEOC overlay applied

**Pre-conditions:** Halcyon Build is provisioned with the `us-eeoc-language-access` pack overlay.

**Action sequence:**

1. Open the stop-work composer
2. Try to send with only `en-US` selected

**Pass criteria:**

- The composer rejects the send with the message "Site requires all configured languages: en-US, ar-EG, es-MX"
- A re-attempt with all three languages succeeds

**Fail criteria:**

- The composer allows English-only fanout on a multilingual site

### T-J152-020 — Soak test: 100 incidents in 24 hours

**Pre-conditions:** Synthetic load: 100 incident-create events spread across decks 4-7 over a simulated 24-hour shift.

**Pass criteria:**

- All 100 incidents reach status RULING-FILED within 24 simulated hours
- p95 incident-create → workplace-integration ack ≤ 90s
- p99 stop-work broadcast fanout to all decks ≤ 7.4s
- Zero audit-chain gaps
- Zero Cedar permit decisions on actions that should have been denied

**Fail criteria:**

- Any of the above fails
- Soak surfaces an event-ordering anomaly (out-of-sequence sealing)

## Pass/fail summary

| Test | Type | Maps to AC |
|---|---|---|
| T-J152-001 | Happy path | AC-J152-001 |
| T-J152-002 | Happy path | AC-J152-002 |
| T-J152-003 | Happy path | AC-J152-003 |
| T-J152-004 | Happy path | AC-J152-004 |
| T-J152-005 | Negative | AC-J152-004 |
| T-J152-006 | Negative | AC-J152-004 |
| T-J152-007 | Negative | AC-J152-008 |
| T-J152-008 | Negative | AC-J152-008 |
| T-J152-009 | Happy path | AC-J152-005 |
| T-J152-010 | Resilience | AC-J152-005 |
| T-J152-011 | Happy path | AC-J152-005 |
| T-J152-012 | Timer | AC-J152-006 |
| T-J152-013 | Happy path | AC-J152-006 |
| T-J152-014 | Output | AC-J152-007 |
| T-J152-015 | Lifecycle | AC-J152-004 |
| T-J152-016 | Authorization | AC-J152-008 |
| T-J152-017 | Audit | ALL |
| T-J152-018 | Invariant | ALL |
| T-J152-019 | Compliance | AC-J152-002 |
| T-J152-020 | Soak | ALL |

## Out of scope

- The crane-sling root-cause analysis tests (separate journey under quality-management)
- The OSHA Area Office investigation interview workflow tests
- Khalil's workers'-comp benefit calculation tests (State Fund's downstream system)
- Long-term retention tests (ECOWAS-Maritime is for j151; ISO-45001 7-year retention test belongs to a dedicated compliance journey)
