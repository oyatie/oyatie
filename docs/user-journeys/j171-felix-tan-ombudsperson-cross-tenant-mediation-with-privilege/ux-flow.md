---
doc_class: User-Journey-UX-Flow
journey_id: j171-felix-tan-ombudsperson-cross-tenant-mediation-with-privilege
date: 2026-05-20
authority_tier: 2
status: draft
---

# j171 — UX flow: ombuds intake console, privileged dyad channel, WORM evidence, community-appeal handoff, latent escalation gate

Six primary surfaces:

- Priscilla's personal-tenant complaint composer (mobile, Pixel 8 Pro)
- Community-appeal handoff dialog (on the community channel side)
- Felix's ombuds intake console (desktop)
- Privileged dyad messenger channel (with redaction indicators)
- WORM evidence drive room (with privileged-content + retention indicators)
- Latent governance escalation gate (visible to Felix but only armed if complainant elects formal investigation)

All screens preserve Cantonese + Hokkien + Mandarin + Singapore-English + diacritics UTF-8 NFC byte-exact. The privilege boundary is always indicated visually with a shield + IOA mark.

## Screen 1 — Priscilla's personal-tenant complaint composer (Sunday May 2 22:18 SGT)

```
┌──────────────────────────────────────────────────────────────────────────┐
│ Pixel 8 Pro · personal tenant · priscilla-lim-personal-2018              │
├──────────────────────────────────────────────────────────────────────────┤
│  ⊕ Community appeal — #womenintech-halberd post removed                  │
│                                                                          │
│  Your post was removed by a moderator on 2027-05-02 19:48 SGT            │
│  Reason: community guideline §4.2                                        │
│                                                                          │
│  ┌─ APPEAL OPTIONS ────────────────────────────────────────────────────┐ │
│  │                                                                     │ │
│  │  ⓘ You can do BOTH of these in parallel.                            │ │
│  │                                                                     │ │
│  │  [ ] Appeal to community moderators (4 moderators · 3-5 biz days)   │ │
│  │      → outcome: post restored / amended / final removal             │ │
│  │                                                                     │ │
│  │  [ ] Forward to ombudsperson office (Felix Tan · confidential ·     │ │
│  │      ombudsperson-privileged · cross-tenant boundary preserved)     │ │
│  │      → outcome: confidential conversation; no employer notification │ │
│  │      → handoff goes from your PERSONAL tenant; your identity is     │ │
│  │        only revealed to the ombudsperson                            │ │
│  │                                                                     │ │
│  │  ⚠ ombudsperson route bypasses HR / IT / your manager / leadership. │ │
│  │     Only the ombudsperson can see your identity + narrative.        │ │
│  │                                                                     │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ ATTACH NARRATIVE (optional) ───────────────────────────────────────┐ │
│  │  [text area — 0/4000 characters]                                    │ │
│  │  language picker: zh-Hant · zh-Hans · en-SG · ms-SG · ta-SG          │ │
│  │  attachment slots: [+1] [+2] [+3] [+4] [+5] [+6]                     │ │
│  │  attachment classes: image/png · image/jpeg · application/pdf       │ │
│  │  encryption: e2ee at transit + at rest (your device key)            │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  Cedar permit: complainant_self_intake (personal_tenant + verified_face) │
│  Audit class: EVT-J171-COMPLAINANT-INTAKE-INITIATED-Δ000                 │
│                                                                          │
│  [ Cancel ]                                  [ Send to ombudsperson ]   │
└──────────────────────────────────────────────────────────────────────────┘
```

UX notes:
- The "cross-tenant boundary preserved" microcopy is mandatory: the user must understand that the personal tenant is the filing principal, not the employer principal.
- The shield-IOA mark appears on the ombudsperson route option.
- Attachment encryption uses ChaCha20-Poly1305 with the user's device-bound key (per ADR-0246 + MLS RFC 9420 + ADR-0263 emission contract).

## Screen 2 — Community moderator handoff (visible on community-side; moderator-perspective)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  #womenintech-halberd · MOD QUEUE · Halberd-Mercer Property Sg          │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Appeal #appeal-2027-05-02-Δ09  (post-removal-appeal)                    │
│  Original post:  removed by @dei-mod-sg-4 (jacinta.wong-hervey)           │
│  Original handle: @quiet-architect-7  [identity NOT revealed to mods]    │
│                                                                          │
│  ⓘ This appeal was ALSO forwarded by the user to the ombudsperson       │
│     office. The ombudsperson handoff is ombuds-privileged; community     │
│     moderators do NOT see the ombudsperson channel content.              │
│                                                                          │
│  ┌─ COMMUNITY-SIDE APPEAL ACTIONS ────────────────────────────────────┐  │
│  │  [ ] restore post                                                  │  │
│  │  [ ] amend post (request user edit)                                │  │
│  │  [ ] uphold removal (with revised reason)                          │  │
│  │  [ ] defer to ombudsperson office (recommended for harassment      │  │
│  │      allegations naming specific incidents)                        │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: community.moderator_appeal_review                         │
│  Audit class:   EVT-J171-COMMUNITY-APPEAL-HANDOFF-001                    │
└──────────────────────────────────────────────────────────────────────────┘
```

UX notes:
- Moderators see the appeal but never the ombuds-channel content.
- The "defer to ombudsperson" option is the canonical handoff path.
- Identity is never revealed to moderators.

## Screen 3 — Felix's ombuds intake console (Monday May 3 09:14 SGT)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  OMBUDS OFFICE · Halberd-Mercer Holdings · Felix Tan (OCO 2022, IOA)     │
├──────────────────────────────────────────────────────────────────────────┤
│  active tenant:  halberd-mercer-holdings-corporate-sg                    │
│  role:           ombudsperson_certified_ioa                              │
│                                                                          │
│  ┌─ CASE QUEUE (4 active + 1 NEW) ─────────────────────────────────────┐ │
│  │                                                                     │ │
│  │  🛡 Δ47 · NEW · received 2027-05-02 22:18 SGT                       │ │
│  │     class: harassment_pattern_named_respondent · priority P2         │ │
│  │     sources: community_appeal_handoff + complainant_self_intake     │ │
│  │     cross-tenant: personal-tenant → corporate ombuds office         │ │
│  │     attachments: 3 (image/png) · narrative: zh-Hant + en-SG          │ │
│  │     [open privileged view]                                           │ │
│  │                                                                     │ │
│  │  🛡 Δ44 · active · in_mediation (day 6 of N)                        │ │
│  │     [...]                                                            │ │
│  │  🛡 Δ41 · active · resolution_pending                                │ │
│  │     [...]                                                            │ │
│  │  🛡 Δ38 · active · evidence_collection (day 21)                     │ │
│  │     [...]                                                            │ │
│  │                                                                     │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ INDEPENDENCE INDICATORS ──────────────────────────────────────────┐  │
│  │  ⓘ Your reporting line: Independent Audit & Risk Committee chair   │  │
│  │  ⓘ Your cases are NOT visible to: HR · IT · Legal · CEO · CFO ·   │  │
│  │     Aloysius Goh · Rohan Pillai · Jeremy Tan · all enumeration     │  │
│  │     attempts are Cedar-denied + logged.                            │  │
│  │  ⓘ Mandatory-reporter exception: armed but inactive                 │  │
│  │     (child safety / criminal threat / imminent harm)                │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: ombudsperson_certified_ioa × ombuds_office_intake_queue   │
│  Audit class: EVT-J171-INTAKE-INITIATED-002                              │
└──────────────────────────────────────────────────────────────────────────┘
```

UX notes:
- The shield icon (🛡) marks every ombudsperson-privileged case.
- The "independence indicators" panel is mandatory per IOA Standards §1.
- Enumeration attempts by non-permitted principals are visible to Felix as a counter.

## Screen 4 — Privileged dyad channel (Monday May 3 09:42 SGT)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  🛡 PRIVILEGED CHANNEL · Δ47 · dyad (2 members)                          │
│  members: felix.tan@HALBERD-MERCER + priscilla.lim@PERSONAL              │
│  class: ombudsperson_privileged_dyad                                     │
│  e2ee: MLS RFC 9420 · group mls-priv-Δ47-2027-05-03 · epoch 0            │
│  retention: 7y_from_case_close · cell: eu-frankfurt-tier-1-privileged    │
│  metadata visibility: REDACTED in metrics                                │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  09:42 felix.tan: 「Priscilla 你好。我是 Felix。我看到你的内容...」       │
│                   (full text in story.md §2)                              │
│                                                                          │
│  11:18 priscilla.lim: "thanks felix. i'm at lunch..."                    │
│                                                                          │
│  11:24 felix.tan: "Thanks Priscilla. English is fine..."                 │
│                                                                          │
│  11:42 priscilla.lim: "(1) early jan 2027 — first comment was at the    │
│                       q4 review dinner Jan 8..."                          │
│                                                                          │
│  12:18 system: 6 screenshots queued for WORM upload to drive room         │
│                                                                          │
│  [...subsequent 14-day exchanges...]                                     │
│                                                                          │
│  ┌─ COMPOSE ──────────────────────────────────────────────────────────┐  │
│  │  payload class:  ◉ ombudsperson_clarification_question             │  │
│  │                  ○ ombudsperson_mediation_option                   │  │
│  │                  ○ ombudsperson_resolution_proposal                │  │
│  │  language:        en-SG  [zh-Hant · zh-Hans · ms-SG · ta-SG ▾]      │  │
│  │  attachment:      [ + attach to WORM evidence room ]                │  │
│  │                                                                    │  │
│  │  [text area]                                                       │  │
│  │                                                                    │  │
│  │  ⚠ payload-class outside whitelist will be Cedar-denied             │  │
│  │  ⚠ this channel is NOT enumerable by anyone outside the dyad        │  │
│  │  ⚠ exit channel only via case archive (preserves WORM seal)         │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: messenger.privileged_channel_send (payload_class in       │
│                allowlist + dyad membership + MLS envelope intact)        │
│  Audit class: EVT-J171-PRIVILEGED-CHANNEL-OPENED-003 +                   │
│               EVT-J171-PRIVILEGED-MESSAGE-Δ{n}                           │
└──────────────────────────────────────────────────────────────────────────┘
```

UX notes:
- Payload class radio buttons enforce the Cedar allowlist client-side.
- Language picker preserves UTF-8 NFC byte-exact.
- The "channel not enumerable" warning is permanent + non-dismissible.

## Screen 5 — WORM evidence drive room (Monday May 3 12:42 SGT)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  🛡 EVIDENCE ROOM · Δ47 · ombudsperson-privileged                         │
│  room_id: drive-ombuds-Δ47-evidence                                      │
│  retention: 7y_from_case_close · seal class: halberd-mercer-ombuds-2     │
│  cell: eu-frankfurt-tier-1-privileged-worm · mirror: sg-tier-2-corp       │
│  write principals: [felix.tan, priscilla.lim] · read principals: same    │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─ EVIDENCE ITEMS (10) ───────────────────────────────────────────────┐ │
│  │  ─ Whatsapp screenshots (6, uploaded 13:02–13:14 SGT) ───────────── │ │
│  │  📷 [01] whatsapp-jan-12-2027-21-48-dress.png       842 KB    🔒    │ │
│  │     sha256: a1b3…ef21 · merkle-leaf: leaf-Δ47-001                   │ │
│  │  📷 [02] whatsapp-feb-03-2027-22-14-perfume.png    1.1 MB     🔒    │ │
│  │  📷 [03] whatsapp-feb-28-2027-19-22-bodily.png      956 KB    🔒    │ │
│  │  📷 [04] whatsapp-apr-22-2027-23-14-sexy.png       1.2 MB     🔒    │ │
│  │  📷 [05] whatsapp-apr-23-2027-08-12-apology.png     788 KB    🔒    │ │
│  │  📷 [06] whatsapp-apr-30-2027-19-48-pressure.png   1.4 MB     🔒    │ │
│  │                                                                     │ │
│  │  ─ Contemporaneous notes (3, uploaded 13:18–13:24 SGT) ──────────── │ │
│  │  📝 [07] note-2027-04-22-capella-sentosa.md         4.2 KB    🔒    │ │
│  │  📝 [08] note-2027-04-23-morning-shower.md          3.1 KB    🔒    │ │
│  │  📝 [09] note-2027-04-30-decision-to-file.md        6.8 KB    🔒    │ │
│  │                                                                     │ │
│  │  ─ Ombudsperson reconstruction (1, uploaded 14:48 SGT) ──────────── │ │
│  │  📝 [10] corridor-incident-reconstruction.md        8.4 KB    🔒    │ │
│  │     felix-authored · open-source corridor floorplan + narrative    │ │
│  │     fusion · NO camera footage retrieved (separate Cedar)           │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ INTEGRITY ATTESTATION ────────────────────────────────────────────┐  │
│  │  merkle root (case): sha256:7f3c…9a14                              │  │
│  │  anchor count:        14 (per-day, batched at 18:00 SGT)           │  │
│  │  privileged-content tag: ombudsperson_privileged_no_payload_disc.   │  │
│  │  proof class: inclusion_proof_only_without_payload                   │  │
│  │  external transparency log: external-transparency-log-batch-2027-… │  │
│  │  regulator compulsion path: armed (proof-on-demand)                 │  │
│  │     · EU-WD Art 22 · SOX 806 subpoena · KR-ACRC Art 13 · SG Court  │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: drive.privileged_worm_read (dyad membership + privilege)  │
│  Audit class: EVT-J171-EVIDENCE-WORM-WRITTEN-004 +                       │
│               EVT-J171-MERKLE-PRIVILEGED-ANCHOR-005                      │
└──────────────────────────────────────────────────────────────────────────┘
```

UX notes:
- The lock 🔒 indicates WORM seal (write-once-read-many; immutable after seal).
- The "no payload disclosure" tag is mandatory + visible.
- The regulator compulsion path indicator is for Felix's awareness only.

## Screen 6 — Latent governance escalation gate (visible to Felix; dormant)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  ⚠ ESCALATION GATE · Δ47 · DORMANT                                       │
│  state: armed_but_not_invoked                                            │
│  trigger: requires complainant explicit escalation OR mandatory-reporter │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─ ESCALATION PATHS (all dormant in this journey) ───────────────────┐  │
│  │                                                                    │  │
│  │  ◯ (i) Complainant elects formal HR-led investigation              │  │
│  │       requires: complainant explicit consent + Cedar permit         │  │
│  │       triggers: ARC chair notification + HR director engagement    │  │
│  │       effect: privileged channel becomes investigation-bound;      │  │
│  │               privilege scope narrows; anti-retaliation activates  │  │
│  │                                                                    │  │
│  │  ◯ (ii) Mandatory-reporter exception (child safety / criminal      │  │
│  │       threat / imminent harm)                                       │  │
│  │       requires: ombudsperson professional judgement + secondary    │  │
│  │                 ombudsperson concurrence                            │  │
│  │       triggers: law enforcement referral within statutory window   │  │
│  │       effect: privilege pierced for the specific information       │  │
│  │               necessary to avert harm                               │  │
│  │                                                                    │  │
│  │  ◯ (iii) Regulator compulsion (EU-WD Art 22 / SOX 806 subpoena /   │  │
│  │       KR-ACRC Art 13 / SG court order)                              │  │
│  │       requires: lawful order received + corporate counsel review   │  │
│  │       triggers: inclusion-proof disclosure (proof only, not        │  │
│  │                 payload) by default; payload disclosure only on    │  │
│  │                 specific court order                                │  │
│  │                                                                    │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ⓘ This screen is visible only to Felix. Activation requires             │
│     additional Cedar permits + ARC chair notification.                   │
│                                                                          │
│  Cedar permit: governance.escalation_gate_evaluate (ombudsperson only)   │
│  Audit class: EVT-J171-ESCALATION-GATE-EVALUATED-Δ{n}                    │
└──────────────────────────────────────────────────────────────────────────┘
```

UX notes:
- This screen is visible only to ombudsperson principals.
- Activation requires explicit complainant consent OR a defined exception.
- All evaluations are audit-logged even when not invoked (compliance signal).

## Screen 7 — Mediation outcome confirmation (Friday May 14 09:14 SGT)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  🛡 MEDIATION OUTCOME · Δ47 · awaiting complainant confirmation          │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Proposed mediation outcome (6 elements):                                │
│   (1) Respondent: written reprimand from CEO (drafted, awaiting send)   │
│   (2) Respondent: reassignment to non-overlapping team + floor          │
│       (effective Monday May 17)                                          │
│   (3) Respondent: written apology to complainant (sent May 13 14:18)    │
│   (4) Complainant: 6-month transfer support package (SGD 6,200          │
│       allowance; leasing-Bishan team; salary + seniority preserved)     │
│   (5) Confidential mediation entry on respondent file (7y retention,    │
│       no complainant name, no public disclosure)                         │
│   (6) Respondent: 12-session external EAP coaching, ombuds-monitored    │
│       completion only (sessions content NOT disclosed)                   │
│                                                                          │
│  Confirmation:  ◉ Accept and close case                                  │
│                  ○ Hold for further mediation                            │
│                  ○ Escalate to formal investigation (privilege narrows) │
│                                                                          │
│  ⓘ You retain the right to reopen within statute of limitations under   │
│     SG Workplace Fairness Act 2024 § 14 (24 months from incident)       │
│  ⓘ Anti-retaliation protection active under SG-WFA + EU-WD + SOX-806 +  │
│     KR-ACRC + Halberd-Mercer policy                                     │
│                                                                          │
│  Cedar permit: complainant.mediation_outcome_confirm                     │
│  Audit class: EVT-J171-MEDIATION-OUTCOME-007                             │
└──────────────────────────────────────────────────────────────────────────┘
```

## Cross-screen rules

1. **Privilege boundary indicator**: every screen that touches ombuds-privileged content displays the shield 🛡 marker + IOA reference text.
2. **Enumeration denial telemetry**: Felix's intake console surfaces enumeration-attempt counters per case (deny coverage report row).
3. **Language preservation**: every screen preserves Cantonese + Hokkien + Mandarin + Singapore-English + diacritics UTF-8 NFC byte-exact; language picker offers zh-Hant + zh-Hans + en-SG + ms-SG + ta-SG.
4. **Mandatory-reporter exception copy**: present on the escalation gate screen + on Felix's intake review screen; the copy is reviewed annually by IOA-certified ombudsperson + Halberd-Mercer GC.
5. **Cross-tenant principal mapping**: Priscilla's personal-tenant principal + employer-tenant employee record are mapped only inside the privileged context; the mapping is not visible outside the dyad.
6. **Retention indicator**: WORM evidence room displays 7-year retention countdown + retention basis (Halberd-Mercer ombuds office records-retention rule).
7. **Regulator compulsion path**: dormant indicator; visible only to Felix; activation requires court order + GC review.
8. **Observability redaction**: all metrics emissions from these screens carry redaction tag per ADR-0263; no payload class in metrics.
9. **Cell + mirror**: WORM writes to EU-Frankfurt-tier-1-privileged cell; live mirror to SG-tier-2 for read latency; reads honor privilege class.
10. **Audit class binding**: every screen has a specific Cedar permit row + a specific audit-event class; the binding is enforced by the messenger + drive + community + governance µservices.

## Accessibility + i18n

- Screen reader: every shield icon has alt-text "ombudsperson-privileged content".
- Color: shield + lock indicators use 4.5:1 contrast (WCAG AA).
- Language picker preserves NFC normalization across all CJK + Tamil + Malay.
- Mobile (Pixel 8 Pro): the composer + composer-keyboard supports Bopomofo + Jyutping + Pinyin + IPA + Tamil-99 keyboards.
