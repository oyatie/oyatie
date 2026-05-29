---
doc_class: FAQ
microservice: recordings
persona: compliance-officer + records-management
date: 2026-05-20
doc_status: published
---

# Compliance Officer FAQ — recordings

## A user filed a GDPR Art. 17 right-to-erasure on a recording that's on legal hold. What do I do?

Per ADR-RECORDINGS-0002 § "Conflict resolution" + GDPR Art. 17(3)(b). The legal hold legitimately overrides the erasure request when the recording is "for establishment, exercise or defence of legal claims". You:

1. Emit `dsar_received` and `dsar_blocked_by_legal_hold` audit events.
2. Notify the data subject IN WRITING with the specific basis (Art. 17(3)(b)) + the expected hold duration.
3. Queue the DSAR for re-evaluation when the hold releases.
4. Retain the notification + the legal-hold order + the audit chain for ≥ 7 y as defense evidence.

You CANNOT silently honor the DSAR by deleting the recording — that breaks the legal hold + exposes the tenant to litigation-spoliation sanctions. You CANNOT silently ignore the DSAR — that's a GDPR Art. 17 violation. The escape is the Art. 17(3)(b) clause; document it explicitly.

## What's the difference between hot tier (30 d default) and cold tier (Glacier)?

- Hot tier: SeaweedFS-S3 on local NVMe; sub-second read latency; transcript + redaction overlay generated synchronously on ingest.
- Cold tier: SeaweedFS-S3 Glacier-class storage; 3-5 hour restore time on read; transcript + redaction overlay persisted alongside the recording.

When a tenant queries a cold-tier recording's transcript, we serve the transcript immediately (it's persisted in the analytics MV); we initiate the cold-tier restore in parallel for the playback bytes. The user sees the transcript within 200 ms; the play button is grayed-out with "preparing playback" until restore completes.

## Why is the retention default 90 d hot + 7 y cold instead of forever-keep?

Per ADR-RECORDINGS-0002 § "Retention defaults". Forever-keep is operationally suspect (storage cost; DSAR exposure; data-class drift over time). The 90 d / 7 y default balances:

- Most playback happens within 30 d of recording (per the meet + messenger usage telemetry).
- Compliance-driven retention typically caps at 7 y (HIPAA 6 y + buffer; SEC 17a-4(f) 7 y).
- After 7 y, the right-to-erasure default kicks in unless explicit pack-override extends.

Tenants can extend to 10 y, 20 y, or 30 y per their retention policy. We don't offer forever-keep because it's not policy-justifiable for most data classes.

## SEC 17a-4(f) requires WORM — what's our WORM substrate?

Per 17 CFR § 240.17a-4(f) + ADR-RECORDINGS-0002 § "SEC 17a-4(f) overlay". WORM = Write-Once-Read-Many. Our substrate: SeaweedFS Glacier-Vault-Lock with compliance-mode immutability — the lock policy denies all deletion + modification operations until the lock-expiration date, even by the bucket owner. The immutability is enforced at the storage layer, not the application layer.

We pass 17 CFR § 240.17a-4(f)(2)(ii)(A) "preserves the records exclusively in a non-rewritable, non-erasable format" via the storage layer's compliance-mode lock; we pass § 240.17a-4(f)(3)(vi) "third-party-readability" via the SHA-256 hash-chain that's exportable + verifiable by a non-affiliated auditor.

## A user says their transcript has the wrong word in it. Can I edit the transcript?

You can OVERRIDE the auto-generated transcript with a manual edit. The override:

1. Is stored as a SEPARATE artifact alongside the original auto-transcript.
2. Does NOT replace the original — the original is preserved (for audit + evidence).
3. Emits `transcript_overridden_by_user` audit event with the principal + the edit-diff.
4. The override becomes the playback-displayed transcript by default; the original is available via `--show-original`.

You CANNOT silently edit the original. The defense-of-evidence invariant requires both versions to coexist.

## When does the redaction overlay vs the actual-redaction apply?

Per ADR-RECORDINGS-0003 § "Overlay-not-mutation". The redaction overlay is the DEFAULT — we apply at playback-time + transcript-display-time; the underlying audio + video bytes are NEVER modified. This:

- Preserves the original for legal hold + eDiscovery (full-fidelity bytes available).
- Allows the overlay to be revised (e.g., a new PII pattern discovered post-hoc).
- Allows per-principal redaction (a senior reviewer sees less redaction than an intern).

Actual-redaction (modifying the bytes) is reserved for the rare case where the tenant explicitly invokes `recordings::redaction::burn-in` with justification + an ADR-RECORDINGS-0003 § "burn-in approval" review. Burn-in is irreversible; once applied, the original is gone.

## EDRM-XML — why that format for eDiscovery?

Per IP-012 + IP-CLUSTERAPI-001-EDRM-XML 1.2. EDRM (Electronic Discovery Reference Model) XML is the de-facto exchange format for litigation discovery; major review platforms (Relativity, Concordance, Ipro, Disco) all ingest EDRM-XML. Exporting in this format means the tenant's outside counsel can hand the export directly to opposing counsel + their review platform handles it.

Alternatives we considered: Concordance DAT/OPT (older; less rich), Relativity Native (proprietary; vendor-lock), DocuSign Files. EDRM-XML is the cross-vendor canonical format.

## What's the boundary between recordings vs meet/messenger huddles?

Per ADR-RECORDINGS-0007. `meet` and `messenger` are PRODUCERS — they record a live session as a side-effect of an active meeting. The recording lifecycle (durable storage, transcript, redaction, retention, legal-hold, export, eDiscovery) is owned HERE, not in the producing µservice. The producer emits via the `recording.ingest.v1` durable contract and stops caring.

Conversely: starting/stopping a recording, controlling the recording UI, deciding "this audio is recording-eligible" — those belong in the producer (meet/messenger). The handoff happens at the `recording.ingest.v1` contract.

## A tenant requested all recordings from a specific user be permanently deleted (right-to-be-forgotten). Some are on legal hold. Can I delete the others?

Yes; the ones NOT on legal hold are deletable. Per ADR-RECORDINGS-0002 § "Per-recording right-to-erasure":

1. Identify all recordings for the user via the `recording.metadata` index.
2. For each recording, check legal-hold status.
3. Delete the un-held recordings; emit `recording_deleted_per_dsar` audit event per recording.
4. For the held recordings, emit `dsar_blocked_by_legal_hold` per recording + notify the user with the per-recording basis.
5. Submit the consolidated DSAR response to the user within the GDPR 30-d window (or KR-PIPA 10-d; or CA-CCPA 45-d; per pack).

The audit-chain has per-recording attribution; the response can cite the specific reason per recording.

## When does the transcription auto-detect language vs require the user to specify?

Whisper-large-v3 auto-detects language from the first ~ 30 s of audio with > 95 % accuracy across the 99 supported languages. We use auto-detect by default; the user can override via `recordings::transcript::set-language`.

The exception: pack-bound model overlays. Per compliance_pack-bound paid tier, the pack's pack-bound model may be Korean-only (KR-PIPA), Mandarin-only (CN-PIPL), etc.; in those cases the language is implicit + the auto-detect is disabled.

## Why don't we ship Zoom's transcript format directly?

Per ADR-RECORDINGS-0001 + ADR-RECORDINGS-0007. Zoom's transcript is VTT-ish but has Zoom-specific extensions (speaker-id is "Speaker 1" / "Speaker 2" without identity binding; timestamps are formatted differently). We canonicalise to standard WebVTT 1.2 + a per-segment speaker_id binding to the tenant's user directory. The Zoom-format export is a separate output (per the export-format options).

## What's our position on "automatic transcription disclosure"?

Per ADR-RECORDINGS-0003 + pack-specific rules. In packs where two-party consent is required (CA, FL, IL, MD, MT, NH, PA, WA — US two-party-consent states; EU under ePrivacy + GDPR; KR under 통신비밀보호법), the producer (meet/messenger) MUST display a notification to all participants that the session is being recorded BEFORE the recording starts. The notification + acknowledgments are emitted to the audit chain (`recording_consent_obtained`).

In other packs (single-party-consent states; some Asia-Pacific), the producer's notification is best-practice but not statutorily required.

If a participant DECLINES, the producer must (a) not start the recording, OR (b) start it without that participant's stream, OR (c) end the meeting. The producer makes this choice; recordings just emits/stores what arrives.
