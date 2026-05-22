---
doc_class: User-Journey-Story
journey_id: j171-felix-tan-ombudsperson-cross-tenant-mediation-with-privilege
date: 2026-05-20
authority_tier: 2
status: draft
---

# j171 — Story: Felix Tan opens the harassment complaint at 09:14 SGT Monday

## §0 — Sunday May 2, 2027, 22:18 SGT — Priscilla's flat, Block 218 Toa Payoh Lorong 8

Priscilla Lim Hui-min (林慧敏) is 31. She is sitting cross-legged on her parents' couch in the HDB flat she still calls home some weekends. The aircon is set to 25°C. She has a Tetley Singapore Breakfast tea on the side table next to her phone. She has been crying intermittently since 19:00. Earlier today she posted a 380-word essay in the Halberd-Mercer **#womenintech-halberd** community channel — an oyatie corporate community channel her tenant joins her into when she onboarded in 2017 — about "what counts as a hand on a back". She did not name Aloysius. She named no one. The post was anonymous-handle-permitted (Priscilla posts as `@quiet-architect-7`).

The post was removed at **2027-05-02 19:48 SGT** by a moderator (Halberd-Mercer Property Sg DEI Council member; `@dei-mod-sg-4` — real name Jacinta Wong-Hervey, HR business partner). The moderator's note read: *"Removed per community guideline §4.2 (no allegations naming specific incidents without a formal channel). Please contact ombuds@halberd-mercer or HR."*

Priscilla cried for 90 minutes. Then she opened her **personal** oyatie tenant on her own phone (a Pixel 8 Pro, locked to her face). She did not use her work-issued laptop. She elected not to use her work-tenant identity. She tapped **Appeal** on the removal notification, and the appeal handoff flow displayed two paths:

```
APPEAL — POST REMOVAL · #womenintech-halberd
─
[ ] Appeal to community moderators (4 moderators will review; outcome 3-5 business days)
[ ] Forward to ombudsperson office (Felix Tan; confidential; ombuds-privileged)

You can do both.
```

She chose **both**. The community appeal went to the moderator team. The ombudsperson handoff went to Felix Tan. She added a short narrative — 142 Chinese characters and 88 English words — and three Whatsapp screenshots she had been keeping on her phone for 11 days. She tapped **Send**.

`EVT-J171-COMPLAINANT-INTAKE-INITIATED-Δ000` sealed at 22:18:42 SGT from her **personal tenant** `priscilla-lim-personal-2018`. The cross-tenant principal mapping was created: her personal tenant principal `priscilla.lim@priscilla-lim-personal-2018` was paired (with privileged class) to her **employer-tenant employee record** `priscilla-lim-HMP-SG-2017-3082@halberd-mercer-property-sg` via the ombuds-intake bridge. The bridge does not expose her real identity to her employer's HR or her manager Aloysius. It exposes her real identity only to **Felix Tan**, gated by the Cedar permit `ombudsperson_certified_ioa` × `ombuds-case-Δ47`.

She did not tell her parents. She finished her tea. She slept badly.

## §1 — Monday May 3, 2027, 09:14 SGT — Felix Tan's office, OUE Bayfront 50 Collyer Quay, Singapore Corporate

Singapore is wet at the start of the southwest monsoon. 28°C and 89% humidity at 09:14. Felix walks from Raffles Place MRT to OUE Bayfront through the underground link. He hates the humidity. His office is on the 22nd floor — a single window office on a low-traffic corridor, accessible by a keycard that only Felix + the building security desk hold (a deliberate IOA-independence layout — his office is not on the corporate executive floor; he sits two floors above the legal department in a deliberately quiet area).

His desk: a Herman Miller Aeron (work-issued), two monitors, a single shelf of IOA reference texts (the 4-standard plaque is on the shelf in Chinese + English), a small Yi Xing teapot a colleague brought back from Nanjing. He brews his Tieguanyin. He sits down at 09:14:08 SGT and unlocks his workstation with his YubiKey 5C NFC + passkey.

The active-tenant pill reads `halberd-mercer-holdings-corporate-sg · ombudsperson_office · ombudsperson_certified_ioa`.

His ombuds-intake console has **one new case** at the top of the queue:

```
[OMBUDS INTAKE] ombuds-case-Δ47
─
state:                        intake_received
intake_time:                  2027-05-02T22:18:42+08:00
intake_source:                community_appeal_handoff + complainant_self_intake (both)
complainant_tenant_class:     personal (cross-tenant filing)
complainant_handle:            complainant-2027-Δ47  (real identity sealed, visible-to: felix.tan only)
complainant_employer_record:   HMP-SG-2017-3082 (Halberd-Mercer Property Singapore)
allegation_class:              harassment_pattern_named_respondent
respondent_handle:            respondent-2027-Δ47-α (real identity sealed)
priority_tier:                 P2 (no imminent-harm flag; mandatory-reporter NOT triggered)
attachments:                   3 (image/png · whatsapp screenshots)
narrative_lang:                zh-Hant (Cantonese-inflected colloquial) + en-SG
privileged_class:              ombudsperson_privileged
```

`EVT-J171-COMMUNITY-APPEAL-HANDOFF-001` sealed at 09:14:18 SGT (the community-side handoff event; this was already queued from Sunday but it surfaces in Felix's UI now). Then `EVT-J171-INTAKE-INITIATED-002` sealed at 09:14:22 (Felix's UI acknowledges the intake).

Felix reads the narrative once. He stops at the line:

> 「他在 corridor 把手放在我背上，没说一声，然後 whatsapp 跟我说 『你今天看起来很性感』。我已经四个月不知道怎么开口了。」
>
> *("He put his hand on my back in the corridor without saying a word, then whatsapp'd me 'you look very sexy today.' I haven't known how to bring it up for four months.")*

He reads it again. He drinks his tea. He notes the time on a paper Hobonichi notebook he keeps for his own emotional ballast (he uses the digital privileged-notes µservice for the case substance). He writes the date `2027-05-03` and three Chinese characters: **谨慎 · 守密 · 公正** ("careful · confidential · impartial") — the IOA standards in his own hand. Then he opens the privileged channel.

## §2 — Monday May 3, 09:18–09:42 SGT: Felix opens the NEED-TO-KNOW privileged dyad channel

The messenger µservice's privileged-channel-open flow:

```
[OMBUDS PRIVILEGED CHANNEL OPEN]
─
case_id:                       ombuds-case-Δ47
channel_id:                    privileged-dyad-Δ47-felix-priscilla
channel_class:                  ombudsperson_privileged_dyad
member_count:                  2
permitted_principals:          [felix.tan@halberd-mercer-holdings-corporate-sg,
                                priscilla.lim@priscilla-lim-personal-2018]
enumerable_to:                 [permitted_principals] only
mls_e2ee_group_id:             mls-priv-Δ47-2027-05-03
mls_group_epoch:               0
retention_class:                ombudsperson_privileged_7y
cell:                           eu-frankfurt-tier-1-privileged-worm (with sg-singapore live-tier-2 mirror)
metadata_visibility:           redacted (audit metadata only; no payload class in metrics)
mandatory_reporter_exception:   armed_but_not_triggered
```

Felix's first message — sent 09:42:18 SGT, payload class `ombudsperson_clarification_question`:

> 「Priscilla 你好。我是 Felix。我看到你的内容。多谢你愿意走出这一步。我们这个 channel 只有你和我能看到 — 公司其他人，包括 HR、 IT、你的主管，他们都看不到、enumerate 不了、metadata 也是 redacted 的。如果你愿意，我们可以一步一步来。我们今天没有时间限制 — 你想停就停。」
>
> *(Hi Priscilla, I'm Felix. I read what you sent. Thank you for taking this step. Only you and I can see this channel — no one else at the company, not HR, not IT, not your manager, no one. They can't enumerate it and the metadata is redacted. If you're willing, we can take this step by step. No time pressure today — you can pause whenever.)*

`EVT-J171-PRIVILEGED-CHANNEL-OPENED-003` sealed at 09:42:42 SGT.

She does not reply until 11:18 SGT. Then:

> "thanks felix. i'm at lunch. i can talk now if u r free. i can switch to mandarin or cantonese or english whichever is easier for u"

Felix replies in **Singapore-English** (he reads her hint that English is easier):

> "Thanks Priscilla. English is fine. We can move between languages whenever you need. A few starter questions, only if you're ready: (1) the four-month window — when did it begin, and was there a single trigger event or a slow change? (2) the Capella Sentosa offsite incident — would you like to upload the screenshots + your contemporaneous notes to a sealed evidence room I'll create, or hold off? (3) what outcome would feel right to you — and 'I don't know yet' is a complete answer."

She answers within 12 minutes (her lunch break runs to 12:30 SGT):

- (1) "early jan 2027 — first comment was at the q4 review dinner Jan 8. it was a 'wow you wore a dress'. before jan he was very normal."
- (2) "yes i want them in the evidence room. i have 6 screenshots not 3 — i can send 3 more now."
- (3) "i don't want him fired. i don't want a court case. i want him to stop and i don't want to be on his team anymore. and i want him to know what he did wrong."

Felix's note (privileged):

> *Outcome preference indicates ombuds-mediation pathway, NOT formal-escalation-to-investigation. Confirms intake at P2 not P1. Mandatory-reporter exception remains inactive.*

`EVT-J171-COMPLAINANT-OUTCOME-PREFERENCE-RECORDED-Δ003a` sealed at 12:18 SGT.

## §3 — Monday May 3, 12:42–15:18 SGT: WORM evidence room creation + Whatsapp screenshot intake

Felix creates the WORM drive room `halberd-mercer-holdings-corporate-sg/ombuds/cases/Δ47/evidence/`:

```
[DRIVE WORM EVIDENCE ROOM]
─
room_id:                       drive-ombuds-Δ47-evidence
privilege_class:               ombudsperson_privileged
retention_class:                7y_from_case_close (or 25y_if_minor_complainant — N/A)
retention_basis:                halberd-mercer-ombuds-office-records-retention-rule-2024-v3
cell:                           eu-frankfurt-tier-1-privileged-worm
mirror:                         sg-singapore-tier-2-corporate (live read; WORM write at EU)
worm_seal_class:                halberd-mercer-ombuds-worm-class-2
write_principals:               [felix.tan, priscilla.lim] (only)
read_principals:                [felix.tan, priscilla.lim] (only)
regulator_compulsion_path:     enabled (proof only; payload disclosure requires court order)
e2ee_at_rest:                   ChaCha20-Poly1305 with HKDF tenant key
```

Priscilla uploads the 6 Whatsapp screenshots:

```
EVIDENCE INTAKE · ombuds-case-Δ47
─
[01] whatsapp-jan-12-2027-21-48-quotemention-dress.png        842 KB  uploaded 13:02 SGT
[02] whatsapp-feb-03-2027-22-14-quotemention-perfume.png      1.1 MB  uploaded 13:04 SGT
[03] whatsapp-feb-28-2027-19-22-quotemention-bodily.png       956 KB  uploaded 13:06 SGT
[04] whatsapp-apr-22-2027-23-14-quotemention-sexy.png         1.2 MB  uploaded 13:08 SGT
[05] whatsapp-apr-23-2027-08-12-apology-deflection.png        788 KB  uploaded 13:11 SGT
[06] whatsapp-apr-30-2027-19-48-pressure-keep-quiet.png       1.4 MB  uploaded 13:14 SGT
```

She also uploads 3 contemporaneous notes (Obsidian markdown exports she keeps on her personal phone):

```
[07] contemporaneous-note-2027-04-22-capella-sentosa.md       4.2 KB  uploaded 13:18 SGT
[08] contemporaneous-note-2027-04-23-morning-shower.md         3.1 KB  uploaded 13:22 SGT
[09] contemporaneous-note-2027-04-30-decision-to-file.md       6.8 KB  uploaded 13:24 SGT
```

Felix uploads his own corridor-incident reconstruction (a synthetic from the offsite agenda, her note, and an open-source corridor floorplan of Capella Sentosa — NOT camera footage, which would require a separate Cedar-permitted retrieval):

```
[10] corridor-incident-reconstruction-2027-04-22-21-18.md     8.4 KB  uploaded 14:48 SGT (felix-authored)
```

`EVT-J171-EVIDENCE-WORM-WRITTEN-004` sealed at 15:18:08 SGT.

Each upload triggers a per-item Merkle leaf insertion. The Merkle anchor for the case is updated:

```
[MERKLE PRIVILEGED ANCHOR]
─
anchor_id:                     anchor-ombuds-Δ47-2027-05-03
case_id:                       ombuds-case-Δ47
leaf_count:                    10
root_hash:                      sha256:7f3c…9a14
privileged_content_tag:        ombudsperson_privileged_no_payload_disclosure
external_transparency_log:     external-transparency-log-batch-2027-05-03 (queued for batch at 18:00 SGT)
proof_class:                    inclusion_proof_only_without_payload
regulator_compulsion_path:      eu_wd_article_22 | sox_806_subpoena | kr_acrc_article_13_demand | sg_court_order
```

`EVT-J171-MERKLE-PRIVILEGED-ANCHOR-005` sealed at 15:18:42 SGT.

## §4 — May 4–7 (Tue–Fri week 1): clarification dialogues + corroboration probe

Over Tuesday May 4 through Friday May 7, Felix and Priscilla exchange 24 messages on the privileged channel. Sample:

**May 4 10:18 SGT — Felix → Priscilla** (clarification):
> "Did anyone else witness any of the corridor moment on Apr 22? I'm not asking you to name them — only the count and whether you trust any of them enough to consider asking. Your call entirely."

**May 4 14:42 SGT — Priscilla → Felix**:
> "two people walked past — i think it was Joon-Ho from the KR team and someone from leasing whose name i don't know. Joon-Ho i trust but i don't want to drag him in. the other person i don't know."

**May 5 08:48 SGT — Felix → Priscilla**:
> "Understood, Joon-Ho stays off-list unless you change your mind. I want to think out loud about three outcome shapes — not commitments, just options — and you tell me which is closer to what you want. (i) ombudsperson-mediated outcome: I write to the CEO and the Audit & Risk Committee chair on your behalf with the case substance held in privilege; we negotiate a reassignment + written reprimand + you receive an apology + 6-month workplace-transfer support; no formal investigation; nothing in his employment file beyond a confidential mediation entry that does not name you. (ii) formal investigation: HR-led; he and you are interviewed; outcome is disciplinary action up to termination; appears in his file; you have anti-retaliation protection under SG-WFA + EU-WD + SOX-806 + KR-ACRC + EEO. (iii) law-enforcement referral: only if there is a criminal threshold met; the Apr 22 corridor incident may meet SG Protection from Harassment Act § 4 (sexual harassment in workplace setting), but that determination is for SPF not for me."

**May 5 22:18 SGT — Priscilla → Felix**:
> "(i). definitely (i). i don't want a court case and i don't want him to lose his job and his kids. i just want him to know and i want to be off his team."

**May 6 09:14 SGT — Felix → Priscilla**:
> "Hearing you. (i) it is. I will not start anything outside this channel today. I will draft what I'd say to the CEO + ARC chair and send you the draft for review before anything leaves this channel. You retain veto at every step."

**May 6 11:42 SGT — Felix sends the draft** (payload class `ombudsperson_mediation_option`):

```
DRAFT — ombudsperson recommendation to CEO + ARC chair
─
to:                            adrian.cheng-whitford@halberd-mercer-holdings-corporate-sg (CEO)
                                sarojini.iyer-krishnan@halberd-mercer-holdings-corporate-sg (ARC chair, INED)
from:                           felix.tan@halberd-mercer-holdings-corporate-sg (ombudsperson, IOA OCO 2022)
subject:                        Confidential ombuds matter Δ47 — recommendation under privilege

Dear Mr. Cheng-Whitford and Mrs. Iyer-Krishnan,

Under IOA Standards §3 (confidentiality) and Singapore Evidence Act §128 (privileged communication), I am writing in my ombudsperson capacity on a confidential matter Δ47.

The complainant — whose identity remains under privilege and is not disclosed in this communication — has alleged a pattern of harassment by a named senior executive within Halberd-Mercer Property Singapore. The pattern, documented in contemporaneous notes + Whatsapp evidence + a corridor incident at the Capella Sentosa offsite of 2027-04-22, includes sexualized verbal commentary across a four-month window and one instance of unwelcome physical contact.

The complainant has elected ombudsperson-mediated resolution over formal HR investigation. The complainant has declined law-enforcement referral. The complainant has explicitly stated she does not seek the named executive's termination. The complainant requests:

(1) the named executive be informed, in writing, that his pattern of behaviour violated the Halberd-Mercer Code of Conduct §11.3 and Singapore Workplace Fairness Act 2024 § 12;
(2) the named executive be reassigned to a team that does not overlap with the complainant's reporting line or her physical office location;
(3) the named executive provide a written apology to the complainant, drafted under your review;
(4) the complainant receive a 6-month workplace-transfer support package (relocation if she chooses, with no change to her seniority + salary + bonus eligibility);
(5) a confidential mediation entry be placed in the named executive's file, NOT naming the complainant, NOT publicly disclosed, retained 7 years.

I am available to meet on Friday May 7 at 14:00 SGT in your office, in person, no aides present. I will not bring the case file. I will hold the substance under privilege per IOA Standards. The complainant has reviewed this draft and approved it for transmission.

— Felix Tan, OCO 2022, IOA-certified Ombudsperson
   2027-05-06 11:42 SGT
```

**May 6 12:18 SGT — Priscilla → Felix**:
> "yes. send."

**May 6 12:42 SGT — Felix transmits** the recommendation outside the privileged channel — but **only to CEO + ARC chair**, and the recommendation does NOT name Priscilla and contains only the case summary, not the evidence. The evidence remains in the WORM drive room under privilege. The CEO + ARC chair principals receive the recommendation via the corporate messenger µservice in a `confidential_executive` channel class with audit-class `EVT-J171-OMBUDS-RECOMMENDATION-TRANSMITTED-Δ006`.

`EVT-J171-MEDIATION-OPTIONS-006` sealed at 12:42:18 SGT.

## §5 — Friday May 7, 14:00 SGT: in-person meeting with CEO + ARC chair

Felix walks to the executive floor (24th floor) at 13:48 SGT. The CEO's office is the corner overlooking Marina Bay. Mrs. Iyer-Krishnan is already there — she flew up from her usual office at the Halberd-Mercer Property KL operation specifically for this meeting. Adrian Cheng-Whitford is in his usual Saturday-but-it's-Friday outfit (open collar, no tie). They greet Felix. They have water on the table; he declines coffee.

The meeting runs 78 minutes. Felix does not bring the case file. He does not name Priscilla. He does not show the Whatsapp screenshots. He holds the substance under privilege. He explains:

- the **substance** of the allegation (without identity);
- the **complainant's preferred outcome** (5 items);
- the **legal posture** (SG Workplace Fairness Act + EU-WD + IOA standards; no criminal referral; complainant veto on escalation);
- the **named respondent** — Aloysius Goh Kheng-Soon, MD-level, Halberd-Mercer Property Sg.

Adrian asks one question: *"Felix, is the evidence strong enough that if this went to a formal investigation we would be inside our duty to act?"* Felix answers: *"In my professional ombudsperson assessment based on what I have reviewed, the evidence would meet a balance-of-probabilities threshold; I cannot speak to criminal standards because that is not within my mandate."*

Sarojini asks: *"What does the complainant lose if we treat this as ombuds-mediated and not investigated?"* Felix answers: *"She loses the chance for a finding entered on Mr. Goh's record; she loses the chance for a disciplinary action up to termination; she retains anti-retaliation protection under SG-WFA + Halberd-Mercer policy; she retains the option to escalate later within the statute of limitations. She has weighed these and chosen mediation."*

Adrian: *"And if I, as CEO, judge that the company needs a finding on Mr. Goh's record beyond mediation — what then?"* Felix: *"That decision is yours. I will respect it. But I must inform the complainant that the recommendation she approved has been escalated, and she may withdraw from the privileged channel and elect formal investigation or no action."*

Sarojini, after a pause: *"Adrian, on the ARC's behalf, the mediation pathway with the 5 elements as drafted is acceptable to the Committee. We will ratify at the next meeting. I would add one item: a confidential coaching engagement for Mr. Goh with the external EAP, 12 sessions, monitored by ombuds office for completion only."*

Felix returns to his office at 15:42 SGT. He writes to Priscilla via the privileged channel:

> "Meeting completed. CEO + ARC chair accept the 5 elements. ARC chair adds (6) — a confidential 12-session coaching engagement for Mr. Goh with the external EAP, completion-only monitoring by ombuds office. The substance of your case was held under privilege throughout. Your identity was not disclosed. Mr. Goh has not yet been informed. I propose to inform him next week with the CEO present. You will not be required to be present, and your name will not be in the room. Do you accept the 6th element, and do you accept the timeline?"

Priscilla, at 16:08 SGT: "yes. and thank you Felix. i can breathe."

`EVT-J171-MEDIATION-OPTIONS-ACCEPTED-Δ006a` sealed.

## §6 — Tuesday May 11, 10:00 SGT: respondent notification

Felix sits with Adrian in the CEO's office. Aloysius is summoned in. He is told (paraphrased by Felix later in the privileged note):

> *Mr. Goh, this is Felix Tan, our ombudsperson. We have received a confidential complaint that your conduct over the past four months — including a corridor incident at Capella Sentosa on April 22 and a series of Whatsapp messages — violated our Code of Conduct §11.3 and Singapore Workplace Fairness Act 2024 §12. The complainant's identity is held under ombudsperson privilege and will not be disclosed. The complainant has elected ombudsperson-mediated resolution. As CEO, I am informing you of the company's response: a written reprimand which I will draft today, reassignment effective Monday May 17 to a non-overlapping team and physical floor, a written apology you will draft under my review, a confidential 12-session coaching engagement with the external EAP starting next week, and a confidential mediation entry on your file retained 7 years. This is not a formal disciplinary investigation; if you contest the response, a formal investigation will be opened, at which point your protection under the mediation framework lapses. Do you understand?*

Aloysius accepts. He requests one clarification: *"May I know who?"* Adrian answers: *"No. The complainant's identity is held under privilege. You will not be told."* Aloysius signs the acknowledgment.

`EVT-J171-RESPONDENT-NOTIFIED-Δ006b` sealed at 10:42 SGT.

## §7 — Wednesday May 12 through Friday May 16: written apology + workplace transfer support

Aloysius drafts the written apology over May 12. Adrian + Felix review it on May 13 morning. They redraft (Aloysius's first draft minimized; the redraft owns the conduct and apologizes without conditional language). The apology is sent to Priscilla via the privileged channel on **May 13 14:18 SGT**.

Sample (translated to English; original in Singapore-English with Hokkien expressions):

> "Priscilla, I am writing this knowing that I was wrong. The comments I made over the past four months were not acceptable. The corridor incident on April 22 was wrong. I am sorry. There is no excuse and I will not make one. I am undertaking the coaching engagement and the reassignment immediately. I will not contact you outside formal work channels. I understand you may not respond and I do not expect you to. — Aloysius"

Priscilla reads it. She does not reply for 18 hours. On May 14 at 09:14 SGT she writes to Felix:

> "felix — i read it. it's enough. i don't need to respond. can we close the case?"

Felix: "Yes. I will move us to resolution + archive over the next 72 hours. The case stays in the privileged WORM room for 7 years. You retain the right to reopen within the statute of limitations under SG-WFA. The community appeal will resolve in parallel — I'll handle the handoff back to the community moderators with the case-substance redacted."

`EVT-J171-MEDIATION-OUTCOME-007` sealed at 09:42 SGT on May 14.

The 6-month workplace transfer support package is activated: Priscilla is moved to the Halberd-Mercer Property Sg **leasing-Bishan** team (different building, different reporting line) effective Monday May 17. Her salary + seniority + bonus eligibility are preserved. She receives a one-time SGD 6,200 transfer support allowance.

## §8 — Saturday May 17, 18:48 SGT: case archive + community moderator handoff

Felix issues the archive command from his home laptop (he works Saturday afternoons sometimes from a quiet co-working in Tiong Bahru). The case state moves from `mediation` → `resolution` → `archive`.

```
[OMBUDS CASE ARCHIVE]
─
case_id:                       ombuds-case-Δ47
final_state:                   archive
case_close_time:                2027-05-17T18:48:08+08:00
total_duration_days:           14.86
privileged_channel_messages:   8 substantive exchanges (excluding system messages)
evidence_count:                10
mediation_outcome_class:        ombudsperson_mediated_resolution_no_investigation
respondent_action_class:        reprimand_reassignment_apology_coaching_filenote
complainant_action_class:       transfer_support_no_retaliation_protection_active
retention_class:                ombudsperson_privileged_7y
retention_end_date:              2034-05-17
audit_chain_anchor_count:      14 (per-day anchors over the journey)
external_transparency_log:     external-transparency-log-batch-2027-05-17 (final anchor batched)
mandatory_reporter_exception:   NOT_TRIGGERED (deny-test exercised; no real trigger)
regulator_compulsion_state:     dormant (proof-on-demand armed)
community_appeal_handoff:        completed_to_moderators_redacted (case closed at community side; moderator note rewritten under ombuds direction)
```

Felix sends a final message to Priscilla:

> "Case closed. You are protected against retaliation for 24 months under SG-WFA + 18 months under EU-WD + 36 months under SOX-806 + KR-ACRC. If anything changes — a comment in the elevator, a removed scheduling invite, anything — reach me. The privileged channel stays open one-way for you to reach me, but stops being live two-way after 30 days. The drive room stays accessible to you for 90 days, then read-only to me only, for 7 years. I won't message you again unless you message me. Take care."

She does not reply for two days. On Monday May 19 at 22:18 SGT she writes:

> "thank you felix. i'm at peace."

He does not reply. He logs it. `EVT-J171-CASE-ARCHIVED-Δ007` sealed at 22:18:42 SGT.

## §9 — Tuesday May 18, 11:18 SGT: Cedar deny coverage + observability redaction verification

Through the journey, the ombuds-office monitoring sub-team (Sarah Wong-Henderson, ombuds office paralegal; Felix's only direct report) runs the Cedar deny-coverage report:

```
[CEDAR DENY COVERAGE · ombuds-case-Δ47]
─
denied_enumeration_attempts:    6 total
  - aloysius.goh @ HMP-SG · 3 attempts (privileged-channel-enumerate denied)
  - HR-director-rohan-pillai @ HMP-SG · 2 attempts (case-metadata-read denied)
  - IT-admin-jeremy-tan @ HMP-SG · 1 attempt (drive-room-list denied)
denied_payload_class_attempts:  0 (no in-channel principal attempted out-of-class payload)
mandatory_reporter_exception:    armed but not triggered (deny-test row: child_safety=false, criminal_threat=false, imminent_harm=false)
regulator_compulsion_state:     dormant (no order received; proof-on-demand armed)
observability_redaction:        100% (no payload-class leaked into metrics; only counters)
```

`EVT-J171-CEDAR-DENY-COVERAGE-008` sealed at 11:42 SGT.

`EVT-J171-PACK-MANIFEST-009` confirms 6 packs active + cross-validated.

`EVT-J171-MANDATORY-REPORTER-NOT-TRIGGERED-010` confirms the exception path is exercised in deny-test row + not in happy path.

`EVT-J171-OBSERVABILITY-REDACTED-011` confirms 100% redaction.

Felix closes the system view. He pours another Tieguanyin. He writes in his paper Hobonichi: *谨慎 · 守密 · 公正 — done.*

## §10 — Stop condition

All 12 AC pass on the seeded fixture. The privileged channel remains enumerable only by Felix + Priscilla. The 10 evidence items are WORM-sealed in EU privileged retention. The Merkle anchor proves inclusion without disclosing payload. The community-appeal handoff is recorded. The mediation outcome is sealed. The mandatory-reporter exception is exercised in deny-tests without being triggered in the happy path. Cantonese + Hokkien + Mandarin + Singapore-English + diacritics preserved UTF-8 NFC byte-exact.
