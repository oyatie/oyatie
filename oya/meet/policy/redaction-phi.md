---
doc_class: PolicySpec
title: PHI Redaction Policy (pack-us-healthcare overlay)
microservice: meet
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-security + ops-compliance + axis-meet
deciders: council-architecture, council-privacy, ops-security
related_adrs: [ADR-0008, ADR-0028, ADR-0135, ADR-0131, ADR-MEET-0002, ADR-MEET-0006]
related_artifacts:
  - microservices/meet/threat-model.md (T-I-02)
  - microservices/meet/dpia.md (R-02)
  - microservices/meet/compliance.md (HIPAA)
  - microservices/meet/policy/data-residency.md
review_cadence: quarterly + on every PHI-channel onboard
doc_status: published
---

# PHI Redaction Policy (meet µservice — pack-us-healthcare overlay)

## Purpose

Define how Protected Health Information (PHI per HIPAA 45 CFR §160.103) is identified, marked, and redacted in meet recordings + transcripts + summaries before they reach the search index, derivation pipelines, capabilities, or any downstream consumer that isn't BAA-covered for PHI access.

Audit posture: SOC 2 CC6.1 + ISO 27001 A.8.12 + HIPAA 45 CFR §164.502(b) "minimum necessary" + §164.514 "Safe Harbour de-identification".

## When PHI Enters Meet

PHI is permitted in meet recordings + transcripts only when ALL of:

1. Pack is `pack-us-healthcare`.
2. Meeting room has `phi_handling: true` attribute set by tenant-admin entitlement.
3. Active BAA on file for the tenant (verified at meeting-create time via `tenancy.has_baa(tenant_id) == true`).
4. All meeting participants have PHI-cleared roles per tenant directory.

Any recording or transcript from such a meeting is treated as PHI by default.

## PHI Markers (data classes)

Per ADR-0008 + dual-context-isolation contract (meet inherits messenger's dual-context invariants for tenant-side roles):

| Marker | Applied to | Defence |
|---|---|---|
| `data_class: PHI` | Recording manifest + transcript rows in PHI-handling meetings | Postgres column + RLS |
| `phi_field` (column-level) | Specific transcript fields (e.g., named-entity spans) | type-tagged at kernel layer |
| `phi_redacted_for: <consumer-id>` | Index-time pre-redaction marker | Meilisearch document field |

## Redaction Rules

### Rule R-PHI-01 — Transcript snippet sanitisation

Before transcript snippet emission to Meilisearch (snippets used in search results):

1. Detect named-entity types from HIPAA Safe Harbour 18 identifiers: NAME, GEO < state, DATE_OF_BIRTH, AGE_OVER_89, TELEPHONE, FAX, EMAIL, SSN, MRN (medical record number), HEALTH_PLAN_ID, ACCOUNT_NUMBER, CERTIFICATE_NUMBER, VEHICLE_ID, DEVICE_ID, URL, IP, BIOMETRIC, FACE_PHOTO, OTHER_UNIQUE_ID.
2. Replace detected spans with `[REDACTED:<entity_type>]` token in the search snippet.
3. Store the **redacted** snippet in Meilisearch + the **original transcript** in S3 (tenant-DEK envelope).
4. Search returns redacted snippet to caller; full transcript body requires per-meeting Cedar `Action::"read_transcript"` allow (separate authorization with BAA-covered recipient).

### Rule R-PHI-02 — T0/T1/T2 capability gating

- **T0 (meeting-topic hint suggestions)**: DISABLED for PHI meetings per HIPAA Safe Harbour (no derived inferences over PHI permitted without per-meeting BAA-extension).
- **T1 (transcript + AI summary + action-item extraction)**: transcript ENABLED (it IS the PHI-bearing record); summary + action-item DISABLED by default; per-meeting opt-in only with explicit BAA-extension.
- **T2 (auto-mute on noise; auto-translate)**: auto-mute ENABLED; auto-translate DISABLED (third-party translation providers not BAA-covered; only on-prem local-mbart permitted).

Enforcement: `policy/meeting-scope.cedar` declares CapabilityExecutor::?c FORBID rules; Cedar evaluator + LEAN-lane `oya-check-pack-overlay-policy` verifies the pack-us-healthcare overlay enforces these.

### Rule R-PHI-03 — Recording video face-blur (post-DSR or per-policy)

For meetings whose recording is requested for face-blur (e.g., DSR per `policy/data-residency.md` §DSR, or per-policy face-blur for non-BAA-recipient audit access):

1. Open-weights face-detection model (on-prem GPU; not cloud-API) detects face regions per frame.
2. Gaussian-blur applied with σ ≥ 25 px (irreversible at typical resolutions).
3. Voice-mask: pitch shift + spectral inversion on audio segments where the subject speaks.
4. Re-encoded variant stored alongside original; access-controlled separately.

### Rule R-PHI-04 — Export + replay

eDiscovery export bundles PHI meetings with the un-redacted recording + transcript for the authorised recipient (e.g., legal counsel under engagement letter) — IF the recipient is BAA-covered. Otherwise, export refuses with a Cedar deny + audit log.

Event replay (per `backfill-replay.md`) preserves the PHI markers and re-emits the same redaction posture to downstream consumers.

### Rule R-PHI-05 — Logging + telemetry sanitisation

- Application logs: PHI fields stripped at the structured-log SDK boundary (`tracing::field::skip(phi)`).
- Telemetry traces: span attributes scrubbed via OTel redactor filter chain before reaching collector.
- Crash dumps: PHI fields scrubbed via the panic-sanitiser; if scrubbing fails, dump is destroyed unprocessed.

## Verification

- Unit tests on the redaction pass cover 100 % of the 18 HIPAA Safe Harbour identifier categories.
- Integration test: post a PHI-containing meeting → transcript → search → Meilisearch returns `[REDACTED:DOB]` not the original DOB.
- Pen-test: annual external red-team attempt to extract PHI via search / capability / telemetry side-channels.
- LEAN-lane `oya-check-redaction-coverage` asserts that every PHI-handling meeting has all rules engaged.

## Failure Mode

If the redaction pass fails (e.g., classifier crashes):

1. Transcript snippet NOT emitted to Meilisearch.
2. `oya_meet_phi_redaction_failure_total` metric increments.
3. Alertmanager fires `MeetPhiRedactionFailure` Sev-2 (Sev-1 if sustained).
4. Search results for the affected meetings degrade to "search unavailable — redaction queue backed up" rather than ever leaking unredacted PHI.

## References

- HIPAA 45 CFR §160.103 (definitions); §164.502(b) (minimum necessary); §164.514 (Safe Harbour de-identification).
- HITECH Act breach-notification rules.
- ADR-0008 Data Use Boundary.
- ADR-MEET-0002; ADR-MEET-0006.
- `microservices/meet/threat-model.md` T-I-02.
- `microservices/meet/dpia.md` R-02.
- `microservices/meet/compliance.md` HIPAA section.
- HIPAA Safe Harbour 18 identifiers reference: `hhs.gov/hipaa/for-professionals/privacy/special-topics/de-identification/`.
- `microservices/messenger/policy/redaction-phi.md` (shape reference; meet inherits the pattern).
