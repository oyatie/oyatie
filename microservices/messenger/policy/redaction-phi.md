---
doc_class: PolicySpec
title: PHI Redaction Policy (pack-us-healthcare overlay)
microservice: messenger
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-security + ops-compliance + axis-messenger
deciders: council-architecture, council-privacy, ops-security
related_adrs: [ADR-0008, ADR-0028, ADR-0126, ADR-0131]
related_artifacts:
  - microservices/messenger/threat-model.md (T-I-08)
  - microservices/messenger/dpia.md (R-08)
  - microservices/messenger/compliance.md (HIPAA)
  - microservices/messenger/policy/dual-context-isolation.md
review_cadence: quarterly + on every PHI-channel onboard
doc_status: published
---

# PHI Redaction Policy (messenger µservice — pack-us-healthcare overlay)

## Purpose

Define how Protected Health Information (PHI per HIPAA 45 CFR §160.103) is
identified, marked, and redacted in messenger before it reaches the search
index, derivation pipelines, capabilities, or any downstream consumer that
isn't BAA-covered for PHI access.

Audit posture: SOC 2 CC6.1 + ISO 27001 A.8.12 + HIPAA 45 CFR §164.502(b)
"minimum necessary" + §164.514 "Safe Harbour de-identification".

## When PHI Enters Messenger

PHI is permitted in messenger channels only when ALL of:

1. Pack is `pack-us-healthcare`.
2. Channel has `phi_handling: true` attribute set by tenant-admin entitlement.
3. Active BAA on file for the tenant (verified at channel-create time via
   `tenancy.has_baa(tenant_id) == true`).
4. All channel members have PHI-cleared roles per tenant directory.

Any message posted into such a channel is treated as PHI by default.

## PHI Markers (data classes)

Per ADR-0008 + dual-context-isolation.md:

| Marker | Applied to | Defence |
|---|---|---|
| `data_class: PHI` | Message rows in PHI-handling channels | Postgres column + RLS |
| `phi_field` (column-level) | Specific fields (e.g., body excerpt for search snippet) | type-tagged at kernel layer |
| `phi_redacted_for: <consumer-id>` | Index-time pre-redaction marker | Meilisearch document field |

## Redaction Rules

### Rule R-PHI-01 — Search index sanitisation

Before document emission to Meilisearch, run the redaction pass:

1. Detect named-entity types: PERSON, DATE_OF_BIRTH, SSN, MRN, ICD-10, CPT,
   ADDRESS, EMAIL, PHONE per a HIPAA Safe Harbour pre-classifier (open-weights;
   on-prem only).
2. Replace detected spans with `[REDACTED:<entity_type>]` token.
3. Store the **redacted** form in Meilisearch + the **original** in Postgres.
4. Search returns redacted snippet to caller; full body requires per-message
   Cedar `Action::"read_message"` allow (separate authorization).

### Rule R-PHI-02 — T0/T1/T2 capability gating

- T0 (smart-reply suggest): DISABLED for PHI channels per HIPAA Safe Harbour
  (no derived inferences over PHI permitted without per-channel BAA-extension).
- T1 (thread summary + action-item extract): DISABLED by default; per-channel
  opt-in only.
- T2 (auto-mute / auto-categorize / auto-translate): translation DISABLED;
  classification ON with on-prem-only model.

Enforcement: `policy/personal-dm-scope.cedar` + `policy/channel-scope.cedar`
declare CapabilityExecutor::?c FORBID rules; Cedar evaluator + LEAN-lane
oya-check-pack-overlay-policy verifies the pack-us-healthcare overlay enforces these.

### Rule R-PHI-03 — Export + replay

eDiscovery export bundles PHI channels with the un-redacted body for the
authorised recipient (e.g., legal counsel under engagement letter) — IF the
recipient is BAA-covered. Otherwise, export refuses with a Cedar deny + audit
log.

Event replay (per `backfill-replay.md`) preserves the PHI markers and re-emits
the same redaction posture to downstream consumers.

### Rule R-PHI-04 — Logging + telemetry sanitisation

- Application logs: PHI fields stripped at the structured-log SDK boundary
  (`tracing::field::skip(phi)`).
- Telemetry traces: span attributes scrubbed via `policy/redaction-otel.md`
  filter chain before reaching OTel collector.
- Crash dumps: PHI fields scrubbed via the panic-sanitiser; if scrubbing fails,
  dump is destroyed unprocessed.

## Verification

- Unit tests on the redaction pass cover 100 % of the 18 HIPAA Safe Harbour
  identifier categories.
- Integration test: post a PHI-containing message; verify Meilisearch returns
  `[REDACTED:DOB]` not the original DOB.
- Pen-test: annual external red-team attempt to extract PHI via search /
  capability / telemetry side-channels.
- LEAN-lane `oya-check-redaction-coverage` asserts that every PHI-handling
  channel has all rules engaged.

## Failure Mode

If the redaction pass fails (e.g., classifier crashes):

1. Document is NOT emitted to Meilisearch.
2. `oya_messenger_phi_redaction_failure_total` metric increments.
3. Alertmanager fires `MessengerPhiRedactionFailure` Sev-2 (Sev-1 if sustained).
4. Search results for the affected channels degrade to "search unavailable —
   redaction queue backed up" rather than ever leaking unredacted PHI.

## References

- HIPAA 45 CFR §160.103 (definitions); §164.502(b) (minimum necessary);
  §164.514 (Safe Harbour de-identification).
- HITECH Act breach-notification rules.
- ADR-0008 Data Use Boundary.
- ADR-0126 (Connect dual-context, parallel).
- `microservices/messenger/threat-model.md` T-I-08.
- `microservices/messenger/dpia.md` R-08.
- `microservices/messenger/compliance.md` HIPAA section.
- HIPAA Safe Harbour 18 identifiers reference:
  https://www.hhs.gov/hipaa/for-professionals/privacy/special-topics/de-identification/index.html
