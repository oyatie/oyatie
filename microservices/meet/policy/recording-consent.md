---
doc_class: PolicySpec
title: Recording + Transcription Consent Policy
microservice: meet
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-meet + ops-compliance
deciders: council-privacy, ops-security, axis-meet, council-architecture
related_adrs: [ADR-0008, ADR-0028, ADR-0126, ADR-0131, ADR-MEET-0002, ADR-MEET-0006]
related_artifacts:
  - microservices/meet/threat-model.md (T-L-06 unawareness)
  - microservices/meet/dpia.md (R-14)
  - microservices/meet/policy/meeting-scope.cedar
  - microservices/meet/policy/data-residency.md
review_cadence: quarterly + on every per-pack regulation change
doc_status: published
---

# Recording + Transcription Consent Policy (meet µservice)

## Purpose

Define the consent flow that satisfies KR PIPA Art. 15 + GDPR Art. 13 + EU AI Act Art. 50 + HIPAA §164.502 + state-specific recording laws (e.g., California one-party-consent; some states two-party-consent) when a meet meeting is recorded, transcribed, or processed by AI summary.

This policy is the load-bearing UX + audit-chain contract for "the attendee knew the meeting was being recorded."

## Consent Modes

### Mode A — explicit-modal-at-join (default for pack-kr, pack-eu, pack-us-healthcare, pack-eu-investment-firm)

When the host enables recording at meeting-create OR mid-meeting:

1. Every existing participant sees a modal banner: "This meeting is being recorded. By staying in the meeting, you consent to recording under KR PIPA Art. 15 / GDPR Art. 13 / applicable law."
2. Modal MUST be acknowledged (click "Continue" or "Leave Meeting") before media flows. Default-deny: until acknowledgement, the participant's mic + camera are disabled.
3. Every new participant joining after recording started sees the modal at join.
4. Acknowledgement emits `oya_meet_recording_consent_acknowledged_total` metric + audit-chain `ParticipantConsentAcknowledged` event with `(participant_ref, instance_id, ack_ts, recording_id)`.
5. Banner persists in the UI throughout the meeting (recording-indicator dot + tooltip).

### Mode B — recording-indicator-only (default for pack-us non-financial, pack-au, pack-jp, pack-sg, pack-in, pack-br, pack-ae)

For packs whose jurisdiction supports one-party consent or where notice-only is sufficient:

1. Recording-indicator dot appears in UI when recording is active.
2. Tooltip discloses "Recording is active; transcript is being generated."
3. No modal blocking media; participant can leave at any time.
4. Audit-chain `ParticipantNotified` event recorded.

### Mode C — strict-recording-disabled (pack-us-financial when recording would violate supervised-comms posture, or for E2E meetings per ADR-MEET-0003)

Recording structurally impossible:
1. UI does not present recording controls.
2. Cedar `forbid` on `Action::"start_recording"` per ADR-MEET-0003 (E2E mode) or pack-policy.
3. Audit-chain emits no recording-related events because none occur.

## Per-Pack Mode Assignment

| Pack | Default mode | Rationale |
|---|---|---|
| pack-kr | A | KR PIPA Art. 15 requires explicit consent at the moment of recording |
| pack-eu | A | GDPR Art. 13 transparency + ePrivacy Art. 5(3) confidentiality + EU AI Act Art. 50 if AI-transcribed |
| pack-us-healthcare | A | HIPAA §164.502(a)(1)(ii) requires authorization; participant attests no PHI-leak risk OR opts out |
| pack-us-financial | A (modal) + supervised-comms audit | SEC 17a-4(f) + FINRA 4511 |
| pack-us (non-financial) | B | most US states one-party consent; CA/MA/PA/WA two-party where applicable → tenant can configure mode A |
| pack-jp | B | APPI Art. 17/27 notice basis |
| pack-sg | B | PDPA notification basis |
| pack-au | B | Privacy Act APP 5 + state-specific recording laws (TIA Act + Surveillance Devices Acts) — tenant attests state legality |
| pack-in | B | DPDPA 2023 §6 lawful processing |
| pack-br | B | LGPD Art. 7(I) consent or Art. 7(IX) legitimate interest |
| pack-ae | B | UAE PDPL Art. 6 lawful processing |
| pack-ksa | B | PDPL Art. 6 lawful processing |

Per-tenant override: tenant-admin may force Mode A globally (for stricter compliance posture); cannot loosen below mode determined by pack.

## Transcription + AI Summary Consent

When transcription or AI summary is enabled, the modal in Mode A explicitly extends to:

> "...This meeting is being recorded AND transcribed AND summarized by AI. Captions and summary are AI-generated and may contain errors per EU AI Act Art. 50 / KR PIPA Art. 28."

Audit-chain `ParticipantAiConsentAcknowledged` event recorded.

## Per-Participant Opt-Out

A participant may opt out of being recorded:
- Their video + audio tracks are not captured into the recording blob.
- The composite recording shows them as "muted-not-recorded" placeholder during their speaking turns.
- Their transcript contributions are tombstoned with `«opted-out»` placeholder.
- Audit-chain `ParticipantOptedOut` event recorded.

Practical consequence: in a 10-person meeting where 1 opts out, the recording captures the other 9 normally + a placeholder card for the 1.

Tenant-admin can disable per-participant opt-out in tenant settings (e.g., pack-us-financial supervised-comms mandate); attendees must accept-or-leave.

## E2E Mode Interaction

Per ADR-MEET-0003: E2E mode disables recording + transcription + AI summary entirely. Consent flow is structurally unnecessary because there is nothing to consent to. UI shows "End-to-end encrypted" badge instead.

## Cedar Policy Wiring

`policy/meeting-scope.cedar` includes:

```cedar
// When recording starts, every active participant must have consented.
// Cedar evaluates at recording-start time; refuses if non-consented participant present.
forbid (
  principal,
  action == Action::"start_recording",
  resource in MeetingInstance::?i
)
when {
  resource has unconsented_participant_count &&
  resource.unconsented_participant_count > 0 &&
  resource has pack_consent_mode &&
  resource.pack_consent_mode == "A"
};
```

This forces the host to wait until all current participants acknowledge before recording-start succeeds.

## Audit Trail

Per-recording audit-chain log contains:

- `RecordingStarted{instance_id, started_by, pack_consent_mode, participant_count, consented_count, started_at}`.
- One `ParticipantConsentAcknowledged{participant_ref, ack_ts}` per consenting participant.
- One `ParticipantOptedOut{participant_ref, opt_out_ts}` per opting-out participant.
- `RecordingFinalized{recording_id, content_hash, finalized_at}`.

Audit-chain replay can reconstruct who consented when for legal-hold or DSR queries.

## Verification

- Integration test `tests/recording_consent_e2e.rs`: start recording with non-consenting participant → Cedar refuses; participant acknowledges → recording starts.
- Pen-test: synthetic participant joins after recording started + dismisses modal client-side → server refuses media until ack received server-side.
- DSR test: data subject erasure → opted-out placeholders + redacted body per `policy/data-residency.md` §DSR.

## References

- KR PIPA Art. 15 (collection consent) + Art. 23 (sensitive consent).
- GDPR Art. 13 (transparency) + Art. 9 (special-category) + Art. 17 (erasure).
- ePrivacy Directive 2002/58/EC Art. 5(3).
- EU AI Act Regulation 2024/1689 Art. 50.
- HIPAA 45 CFR §164.502(a) + §164.508 (authorization for use beyond TPO).
- SEC Rule 17a-4(f); FINRA Rule 4511 + 3110.
- MiFID II Art. 16(7).
- California Penal Code §632 (two-party consent).
- TIA Act 1979 (AU); Surveillance Devices Act 1998 (NSW); equivalents.
- ADR-MEET-0002; ADR-MEET-0003; ADR-MEET-0006.
- `microservices/meet/policy/meeting-scope.cedar`.
- `microservices/meet/policy/data-residency.md`.
