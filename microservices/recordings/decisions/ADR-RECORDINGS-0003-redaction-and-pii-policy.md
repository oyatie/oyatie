---
id: ADR-RECORDINGS-0003
status: Accepted
date: 2026-05-17
microservice: recordings
deciders: council-privacy, axis-recordings, ops-compliance, ops-security
owner: council-privacy
supersedes: []
superseded_by: []
related: [ADR-RECORDINGS-0001, ADR-RECORDINGS-0002, ADR-RECORDINGS-0006]
related_artifacts:
  - microservices/recordings/PRD.md (FR-09 redaction)
  - microservices/recordings/policy/cedar/legal-hold.cedar
  - microservices/recordings/runbooks/redaction-overlay-corruption.md
  - microservices/recordings/slos/redaction-render-latency.openslo.yaml
purpose: |
  Fix the redaction model — overlay-only; source media is immutable post-
  ingest. Auto-PII redaction runs at transcription time; manual redaction
  is a compliance-officer compensating-overlay path. Conforms to GDPR
  Art. 25 (data-protection by design) + HIPAA Safe Harbor §164.514 + NIST
  SP 800-86 forensic-integrity.
---

# ADR-RECORDINGS-0003: Redaction + PII policy — overlay-only model; immutable source media

## Status

Accepted — 2026-05-17.

## Context

PRD-recordings FR-09 mandates compliance-officer redaction of recording
segments — visually + audibly hidden at playback — without mutating source
media. The legacy `oya-connect-recordings-domain` had no redaction model
(source media + WORM tier was the only protection); the new µservice
introduces redaction as a first-class capability.

Two redaction surfaces:

1. **Auto-PII redaction at transcription time** — Whisper output is post-
   processed with a PII detector (regex + lightweight NER); detected spans
   become redaction overlay rows.
2. **Manual redaction by compliance-officer** — court-order-driven or DSR-
   driven; insert-only overlay rows with reason code.

Three candidate architectures:

- **Destructive rewrite of source media** — strip the offending audio /
  video segment from the source file in S3.
- **Overlay model** — source media unchanged; overlay rows store the
  redaction span + reason; playback / transcript-render / export apply
  overlays at render time.
- **Encrypted-segment model** — source media unchanged; redacted segments
  encrypted under a separate key; un-redact requires KMS access.

Forensic-integrity expectation (NIST SP 800-86 §4.3): source media must be
preserved verbatim for chain-of-custody. SEC 17a-4(f) WORM + HIPAA Safe
Harbor + KR 전자문서법 all require source preservation.

## Decision

oyatie recordings ships a **redaction-overlay model that does NOT mutate
source media**:

1. **Source media is immutable post-ingest.** WORM for pack-us-financial
   first 2y; S3 object-lock + versioning everywhere else for the retention
   floor.
2. **Redaction overlay rows live in Postgres** under
   `oya-recordings-redaction-adapter-postgres`. Schema:
   ```
   redaction_id, recording_id, span_start_ms, span_end_ms,
   reason ∈ {pii_auto, manual_compliance, court_order, dsr_erasure},
   applied_at, applied_by, paired_approver (nullable),
   compensated_at (nullable), audit_chain_seal_ref
   ```
3. **Overlay rows are INSERT-ONLY.** No UPDATE statements exist in the
   crate's SQL surface (enforced by CI lane
   `recordings-redaction-overlay-immutability`).
4. **Un-redact** (compensating overlay) requires Cedar PERMIT + paired-
   approver + reason; emits a new row that re-applies the original span
   without the original redaction. Both rows remain in history.
5. **Half-open span semantics**: `[span_start_ms, span_end_ms)` per
   PRD-recordings FR-04 + Hyrum's-Law callout in
   `migration-from-connect.md`.
6. **Auto-PII redaction at transcription time** (per ADR-RECORDINGS-0001
   pipeline): Whisper output → PII detector → overlay row insert. Detector
   categories per pack:
   - pack-eu / pack-us / pack-jp / etc.: GDPR-class PII (names, emails,
     phone numbers, SSN-equivalents, payment-card patterns).
   - pack-us-healthcare: additional PHI categories per HIPAA Safe Harbor
     §164.514(b)(2) 18 identifiers (DOB, MRN, IP addresses, etc.).
   - pack-us-financial: additional MNPI category (material non-public
     information patterns).
7. **Manual redaction** by compliance-officer per `policy/cedar/tenant-scope.cedar`
   PERMIT 6; paired-approver required for un-redact per
   `policy/cedar/legal-hold.cedar`.
8. **DSR Art. 17 erasure overlays** behave identically; redact identifiers
   first, body remains under audit-chain protection per
   ADR-RECORDINGS-0002.
9. **Render-time application** of overlays:
   - playback: ffmpeg overlay filter blackens the visual + silences the
     audio for the redacted span (no re-encode of source; runtime applied).
   - transcript render: overlay rows masked at the render layer; raw
     transcript JSON remains with `redacted_spans` annotation.
   - export: redacted spans are applied permanently on the export artifact
     (which is a derivative; export is mutable; source is not).
10. **Audit-chain seal on every overlay row** (insert + compensation);
    seal verified daily by reconciliation worker.

## Alternatives Considered

### A. Destructive rewrite of source media

- Pros: simpler; redaction is "obvious" because the audio simply isn't
  there.
- Cons: violates SEC 17a-4(f) WORM; violates NIST SP 800-86 forensic
  preservation; violates ISO 27037:2012; chain-of-custody breaks; un-
  redact impossible by design (which sounds positive but breaks DSR Art. 17
  conflict-resolution where redact must be reversible under court order).
- Rejected: regulatory + forensic-integrity blockers.

### B. Encrypted-segment model

- Pros: source bytes unchanged; redaction is "real" cryptographically.
- Cons: changes the manifest byte ordering (Hyrum's-Law risk); restoring
  requires KMS access; per-segment key management complexity; un-redact
  loses simplicity of overlay-only.
- Rejected: complexity vs. benefit unfavourable.

### C. Hybrid — overlay for visual, destructive for audio

- Pros: audio "redaction" is more obvious to end-users.
- Cons: WORM still violated for audio; chain-of-custody still broken.
- Rejected.

### D. Client-side redaction (overlay applied on client only)

- Pros: zero server-side cost.
- Cons: client-trust violation; redacted content visible to anyone who
  bypasses the client; forbidden by `feedback_no_silent_regression` server-
  side defence-in-depth posture.
- Rejected: client-trust forbidden.

## Consequences

### Positive

- Source media preserved verbatim — SEC 17a-4 + HIPAA + KR 전자문서법 + ISO
  27037 all satisfied.
- Un-redact reversible by compensating overlay (with paired-approver).
- Audit-chain seal on every overlay row gives perfect forensic timeline.
- Auto-PII at transcription time + manual-overlay at compliance gives both
  layers of defence.

### Negative

- Render-time application means a small latency penalty at playback
  (mitigated by `redaction-render-latency.openslo.yaml` SLO and CDN-cached
  pre-rendered variants per popular recording).
- Postgres overlay-row count can grow large for long retention periods;
  partitioned by `(tenant_id, recording_id)` per `capacity-model.md`.
- DSR Art. 17 erasure that needs to remove identifiers from search-index
  requires re-emit (mitigated by search-index re-emit worker on every
  overlay insert).

### Operational

- Cargo workspace adds `oya-recordings-redaction-*` (9 crates).
- CI lane `recordings-redaction-overlay-immutability` asserts no UPDATE
  statements on the overlay table.
- Reconciliation worker re-verifies audit-chain seals daily; mismatch is
  Sev-1 per `runbooks/redaction-overlay-corruption.md`.

### Regulatory

- **GDPR Art. 25** (data-protection by design): overlay model is the canonical
  pattern.
- **GDPR Art. 17** (right-to-erasure): handled via DSR cascade overlays.
- **HIPAA Safe Harbor §164.514(b)(2)**: 18-identifier auto-detection at
  transcription time on pack-us-healthcare.
- **NIST SP 800-86 §4.3** (forensic preservation): satisfied by source
  immutability.
- **ISO 27037:2012 §5.4** (preservation): satisfied.
- **SEC 17a-4(f)**: source preservation satisfied.
- **KR 전자문서법 Art. 5**: integrity-attestation via audit-chain Merkle seal.

## References

- GDPR Arts. 17, 25.
- HIPAA 45 CFR §164.514 Safe Harbor 18-identifier list.
- NIST SP 800-86 (forensic-integrity).
- ISO 27037:2012.
- SEC Rule 17a-4(f).
- KR 전자문서법 Arts. 5, 6.
- KR PIPA Arts. 21, 22-2, 28, 29.
- ADR-RECORDINGS-0001 (transcription pipeline).
- ADR-RECORDINGS-0002 (retention + legal hold).
- ADR-RECORDINGS-0006 (AI feature bounds).
- microservices/recordings/PRD.md FR-09.
- microservices/recordings/policy/cedar/tenant-scope.cedar.
- microservices/recordings/policy/cedar/legal-hold.cedar.
- microservices/recordings/runbooks/redaction-overlay-corruption.md.
- microservices/recordings/slos/redaction-render-latency.openslo.yaml.
- microservices/recordings/capacity-model.md.
