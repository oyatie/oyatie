---
doc_class: User-Journey-Story
journey_id: j165-cco-naveen-iyer-board-quarterly-compliance-report
date: 2026-05-20
authority_tier: 2
status: draft
---

# j165 — Story: 06:18 EDT Thursday April 8, the Q1 compliance report begins

## §0 — Thursday April 8, 2027, 06:18 EDT — Tessellate Health AI, Boston Seaport

Early spring in Boston. 11°C and gusty. The cherry blossoms on Atlantic Wharf have opened. The Tessellate HQ occupies floors 14–17 of 100 Seaport Boulevard, the glass-and-cor-ten-steel building two blocks from the Boston Convention Center. Naveen Iyer (नवीन अय्यर) badges into the 16th-floor secure compliance wing at 06:18:14 EDT. His M3 MacBook Pro 16" is already on his standing desk; his second monitor — a 32" 4K — was off; he turns it on. The GRC console loads at 06:18:42.

Naveen sits down. He starts with a black coffee from the kitchenette + a single hard-boiled egg + the half-eaten everything bagel he stress-bought at Dunkin' on the corner of Boylston + Atlantic. He has six business days. Next Wednesday April 14 at 14:00 EDT the Tessellate board convenes. By April 13 17:00 EDT the report must be in the board's pre-read folder.

The active-tenant pill reads `tessellate-health-ai-inc · compliance · chief_compliance_officer`. He opens the **governance** µservice's quarterly-board-report board. He sees the workflow card:

```
[REPORT] Tessellate Q1-2027 Quarterly Board Compliance Report
─
state: not_started
scheduled board: 2027-04-14T14:00:00-04:00
pre-read deadline: 2027-04-13T17:00:00-04:00
days remaining: 5 business days
required artifacts: 47-page bound document + Merkle bundle + SEC trigger eval
packs active: 8
expected pack evidence count: 480-540 artifacts
```

He starts the workflow at **06:24:18 EDT**. The state advances from `not_started` to `draft`.

`EVT-J165-WORKFLOW-INITIATED-000` sealed.

## §1 — 06:24–11:48 EDT Thursday: cross-pack evidence pull

The compliance µservice's cross-pack aggregator query begins. Each of the 8 packs has its own evidence ledger; each ledger lives in the cell that hosts the pack overlay (so SOC 2 + HIPAA + PCI live in `us-east-boston-tier-1-compliance`; GDPR + EU AI Act live in `eu-frankfurt-evidence-mirror`; KR PIPA + CSAP live in `kr-seoul-evidence-mirror`; SEC pre-IPO lives in `us-east-boston-tier-1-compliance`).

The aggregator fans out 8 queries in parallel:

```
PACK QUERY DISPATCH (06:24:42 EDT)
─
[1] pack-soc2-type2-fy2026                    → us-east-boston-tier-1-compliance
[2] pack-hipaa-business-associate              → us-east-boston-tier-1-compliance
[3] pack-gdpr-controller-processor-mixed       → eu-frankfurt-evidence-mirror
[4] pack-eu-ai-act-high-risk-medical           → eu-frankfurt-evidence-mirror
[5] pack-kr-pipa-controller                    → kr-seoul-evidence-mirror
[6] pack-csap-naver-cloud-tier-3              → kr-seoul-evidence-mirror
[7] pack-pci-dss-saq-c                         → us-east-boston-tier-1-compliance
[8] pack-sec-pre-ipo-s1-active                → us-east-boston-tier-1-compliance
```

The pulls return between 06:24:48 and 06:28:14 (cross-region pulls slightly slower than local). Results:

| Pack | Evidence count | Findings | Open risks |
|---|---|---|---|
| SOC 2 Type II | 142 | 3 (1 P2, 2 P3) | 2 (control gap on user-access-review cadence; control gap on subprocessor risk assessment) |
| HIPAA BA | 87 | 1 (P3 audit log retention) | 1 (one BAA expiring May 2027; renewal in flight) |
| GDPR | 64 | 2 (P2 DSAR latency; P3 cookie banner consent matrix) | 1 (Article 30 ROPA needs annual update by June 2027) |
| EU AI Act | 71 | 0 | 4 (Article 9 risk-management system documentation gap; Article 12 logging completeness; Article 14 human-oversight evidence; Article 15 robustness testing cadence) |
| KR PIPA | 42 | 0 | 0 |
| CSAP | 38 | 1 (P3 incident-response runbook needs annual review) | 0 |
| PCI DSS | 28 | 0 | 0 |
| SEC pre-IPO | 24 | 0 | 2 (S-1 risk-factor section needs Q1 refresh; new related-party transaction with Bangalore subsidiary requires disclosure review) |
| **TOTAL** | **496** | **7** | **10** |

`EVT-J165-PACK-EVIDENCE-PULL-001` sealed at 06:28:18 EDT.

Naveen drinks his coffee. Then he begins the long part: reviewing each evidence ledger, classifying each finding, validating each remediation status. This takes from 06:30 to 11:48 EDT — about 5h18m of focused work. He uses the GRC console's per-pack pane. Each pack opens to its own canvas with the evidence + findings + remediation matrix.

Key triage decisions:

- The SOC 2 P2 finding (user-access-review cadence — Tessellate runs quarterly when the policy says monthly) needs an audit-committee-level disclosure in the report. Naveen drafts a remediation plan: change policy to quarterly (matching reality) AND deploy automated review tooling by end of Q2. Audit committee will be asked to endorse the policy change.
- The EU AI Act four open risks need careful framing in the report. Article 9 risk-management documentation is genuinely behind — Naveen marks this as `material_to_disclose`. Article 12 logging completeness is at 87%; he marks `partially_remediated`. Article 14 human-oversight evidence requires a process improvement; `in_progress_q2`. Article 15 robustness testing cadence is well-positioned; `green_with_continuous_monitoring`.
- The SEC pre-IPO related-party transaction with the Bangalore subsidiary (Tessellate Health AI India Pvt Ltd) is the most consequential item. Naveen flags it for counsel review during stage 2. It is not Form 8-K material (Tessellate is pre-IPO not post-IPO; 8-K applies post-effective-date of S-1). It IS S-1 risk-factor relevant. Naveen marks `s_1_refresh_required`.

At 11:48 EDT Naveen leaves for a quick walk + a salmon poke bowl from the food court at 50 Liberty. He returns at 12:42 EDT.

## §2 — 12:42–16:18 EDT Thursday: per-pack Merkle computation + LLM-assisted exec summary

Naveen invokes the audit-chain µservice to compute the Merkle root per pack:

```
PER-PACK MERKLE COMPUTE
─
[1] SOC 2          142 artifacts  → root 0x7a2f4b8c1e9d5f3a6b2c8e0f4d7a9b1c
[2] HIPAA BA        87 artifacts  → root 0x3e8b2f9a6c4d1e7f5a8b3c0d6e9f2a4b
[3] GDPR             64 artifacts  → root 0xb1d4f8c3a7e6b9c2d5f0a4e8b7c1d3f6
[4] EU AI Act        71 artifacts  → root 0x9c6e2a8b4d7f3c1e5a0b8d6c4f9e2b3a
[5] KR PIPA          42 artifacts  → root 0x5f8a1c7e9b3d6f2a4c8e0b5d7f3a9c1e
[6] CSAP             38 artifacts  → root 0xe2b7d4a9c6f8e1b3a5d7c0f2b9e4a8c6
[7] PCI DSS          28 artifacts  → root 0x4d8f1b6e9a3c7d2f5b8e0a1d4c7f9b3e
[8] SEC pre-IPO      24 artifacts  → root 0xa8c3e7b2f4d9a1c6e8b3f5d7a0c2e4b6
```

Each per-pack root is computed deterministically. The audit-chain logs `EVT-J165-PER-PACK-MERKLE-002` × 8 (one per pack).

Naveen now invokes the intelligence µservice for an LLM-assisted executive summary draft. The LLM is **Sonnet-Compliance-Tuned-v3** (oyatie's compliance-domain-adapted variant; the model card declares: trained on public-domain regulatory text + Tessellate's prior 6 quarters of compliance reports for in-context learning at inference time; never trained on Tessellate's evidence artifacts themselves).

The LLM produces a 4-page executive summary draft. Naveen reads it carefully. The LLM caught the SOC 2 P2 framing well. It under-stated the EU AI Act risk (Naveen tightens the language manually). It correctly framed the Bangalore related-party item. The LLM provenance metadata is preserved in the bundle:

```
LLM provenance
─
model: sonnet-compliance-tuned-v3@oyatie-2027-03
invocation_id: llm-naveen-exec-summary-2027-04-08-1418
prompt_template: quarterly-board-exec-summary-v4
input_tokens: 14,820
output_tokens: 1,840
naveen_edit_distance_from_llm_output: 38% (substantial human edit)
naveen_review_duration_minutes: 47
human_signoff: naveen.iyer@tessellate-health-ai-inc
```

`EVT-J165-LLM-DRAFT-ASSIST-004` sealed at 14:42 EDT.

Naveen continues drafting the body of the report — 12 sections, one per pack + cross-pack themes + the SEC trigger eval + the audit committee recommendations. He breaks at 16:18 EDT.

## §3 — 16:18 EDT Thursday: SEC 8-K trigger evaluation

The compliance µservice runs the Form 8-K trigger evaluation inline as Naveen drafts:

```
SEC 8-K TRIGGER EVALUATION (2027-04-08T16:18:42-04:00)
─
Item 1.01 (Entry into material definitive agreement)    NO triggers
Item 1.02 (Termination of material definitive agreement) NO triggers
Item 2.02 (Results of operations and financial condition) NO triggers
Item 2.04 (Triggering events accelerating direct financial obligation) NO triggers
Item 4.02 (Non-reliance on previously issued financial statements) NO triggers
Item 5.02 (Departure of directors/principal officers) NO triggers
Item 8.01 (Other events — voluntary)                    NO triggers
─
NOTE: Tessellate is pre-IPO; 8-K filing obligations do not yet attach.
      Post S-1 effectiveness (currently estimated H2 2027), this
      evaluation will become a binding regulatory check.
```

`EVT-J165-SEC-8K-EVAL-005` sealed.

Form NT evaluation similarly: NO triggers (Form NT applies to inability to timely file periodic reports; not yet applicable).

Naveen documents this in the report's Section 9 (SEC compliance status).

## §4 — 17:48 EDT Thursday → 09:14 EDT Friday April 9: continued drafting

Naveen leaves at 17:48 EDT Thursday. He's home by 18:42 EDT (lives in Brookline; takes the Green Line). He has dinner with his husband Marcus (an architect) at home. They watch one episode of a Tamil-language film he's been showing Marcus.

Friday April 9 he is in by 06:42 EDT. He works through the report body 06:42–11:18 + 12:14–17:24. He completes:

- Section 1: Executive Summary (4 pages; LLM-assisted + Naveen-edited)
- Section 2: SOC 2 Type II status (5 pages)
- Section 3: HIPAA BA status + BAA renewal timeline (4 pages)
- Section 4: GDPR + EU AI Act consolidated EU status (8 pages — EU AI Act gets most of the space)
- Section 5: KR PIPA + CSAP consolidated KR status (3 pages)
- Section 6: PCI DSS status (2 pages)
- Section 7: SEC pre-IPO status + S-1 risk-factor refresh recommendation (5 pages)
- Section 8: Cross-pack themes (audit logging + access review + subprocessor risk) (4 pages)
- Section 9: SEC trigger evaluation (3 pages)
- Section 10: Audit committee recommendations (4 pages)
- Section 11: Open risks register + remediation timeline (3 pages)
- Section 12: Appendices (2 pages — pointer to evidence Merkle roots + drive locations)

Total: 47 pages.

At 17:24 EDT Friday Naveen produces the final draft. He runs the per-pack Merkle recompute (some evidence ledgers had updates during Friday from the SOC 2 control owner; new per-pack roots are computed; roots changed for SOC 2 + HIPAA only).

## §5 — 17:24–17:48 EDT Friday: super-Merkle of Merkles

The audit-chain µservice computes the super-Merkle root from the 8 per-pack roots:

```
SUPER-MERKLE OF MERKLES (2027-04-09T17:24:42-04:00)
─
inputs (ordered by pack_id ascending):
  [1] 0xa8c3e7b2f4d9a1c6e8b3f5d7a0c2e4b6  (pack-sec-pre-ipo)
  [2] 0xb1d4f8c3a7e6b9c2d5f0a4e8b7c1d3f6  (gdpr)
  [3] 0x4d8f1b6e9a3c7d2f5b8e0a1d4c7f9b3e  (pci-dss)
  [4] 0xe2b7d4a9c6f8e1b3a5d7c0f2b9e4a8c6  (csap)
  [5] 0x5f8a1c7e9b3d6f2a4c8e0b5d7f3a9c1e  (kr-pipa)
  [6] 0x9c6e2a8b4d7f3c1e5a0b8d6c4f9e2b3a  (eu-ai-act)
  [7] 0x3e8b2f9a6c4d1e7f5a8b3c0d6e9f2a4b  (hipaa)
  [8] 0x7a2f4b8c1e9d5f3a6b2c8e0f4d7a9b1c  (soc2)

super-merkle root:
  0xf3a8c2e7b6d9f4a1c8e3b5d7f0a2c4e6b8d1f5a3c7e9b2d4f6a8c0e2b5d7f1a4
```

`EVT-J165-SUPER-MERKLE-003` sealed at 17:24:48 EDT Friday.

Naveen transitions the workflow from `draft` to `counsel_review` at 17:42 EDT Friday. Cedar evaluates:

```
principal: User::"naveen.iyer@tessellate-health-ai-inc"
action: Action::"workflow_engine.transition_propose"
resource: BoardComplianceReport::"q1-2027-quarterly"
context: {
  cco_signoff_present: true,
  passkey_assertion_present: true,
  super_merkle_root_present: true,
  ten_sections_complete: 12
}
decision: permit
```

The workflow advances. Hampton Reese (the General Counsel, in San Francisco) gets a notification. `EVT-J165-TRANSITION-DRAFT-TO-COUNSEL-006` sealed.

## §6 — Saturday April 10: counsel review (Hampton Reese, SF)

Hampton Reese works Saturday on the report. He is in his apartment in Russian Hill, San Francisco. It's 09:24 PDT (= 12:24 EDT) when he opens it. He reads cover-to-cover over 4h08m. He produces 3 redline edits:

- Section 4 (EU AI Act): tighten the language on Article 9 risk-management documentation; he changes "documentation gap" to "structured documentation in progress under Q2 program plan"
- Section 7 (SEC): on the Bangalore related-party item — adds a paragraph clarifying that the related-party threshold under S-1 Item 404 is $120,000 annually; the transaction in question is $84,000; below threshold but Tessellate's audit committee policy is to disclose anyway
- Section 10 (audit committee recommendations): changes the recommendation order — moves the SOC 2 policy change ahead of the EU AI Act Article 9 plan as the higher-priority item this quarter

Hampton finalizes counsel review at 13:32 PDT (16:32 EDT). He transitions the workflow from `counsel_review` to `audit_committee_sign_off`.

`EVT-J165-COUNSEL-REVIEW-007` sealed.
`EVT-J165-TRANSITION-COUNSEL-TO-AC-008` sealed at 16:32 EDT Saturday.

## §7 — Saturday April 10 16:32 → Sunday April 11 21:42 EDT: audit committee sign-off

The audit committee chair Jasmine Wells-Okafor (in Sausalito) gets notified Saturday afternoon. She reviews Saturday evening 19:18–21:42 PDT. She has two questions for Naveen which she sends via the workflow's structured-question mechanism. Naveen responds Saturday late (his Sunday morning) at 06:42 EDT Sunday.

The two other independent audit committee members — Dr. Tunde Akinwale (Boston) and Lisa Cheng-Halsey (San Diego) — review on Sunday morning. By 14:18 EDT Sunday all 3 of 5 committee members have signed off. The quorum threshold is 3; the other 2 (Marcus Lin + Patricia Hwong) will see the report at the board pre-read.

`EVT-J165-AUDIT-COMMITTEE-SIGNOFF-009` sealed at 14:18 EDT Sunday.

Jasmine transitions the workflow from `audit_committee_sign_off` to `board_presentation` at 17:42 EDT Sunday. Cedar evaluates:

```
principal: User::"jasmine.wells-okafor@tessellate-health-ai-inc"
action: Action::"workflow_engine.transition_audit_committee_to_board"
context: {
  counsel_review_present: true,
  audit_committee_quorum_reached: true,
  audit_committee_quorum_count: 3
}
decision: permit
```

`EVT-J165-TRANSITION-AC-TO-BOARD-010` sealed at 17:42 EDT Sunday.

## §8 — Sunday April 11 17:42 EDT: drive archive + external transparency log

The drive µservice receives the final bound 47-page PDF (12.4 MB). It writes to `tessellate/board/2027/q1/compliance-report/` with WORM 7-year retention engaged. The retention authority is "SEC-pre-IPO-adapted-from-17-CFR-240-17a-4-7-year" — this is Tessellate's audit-committee-approved policy for pre-IPO compliance records.

```
DRIVE ARCHIVE (2027-04-11T17:42:08-04:00)
─
drive_room:      tessellate/board/2027/q1/compliance-report
filename:        2027-q1-tessellate-board-compliance-report-final.pdf
size_bytes:      12,408,716
content_type:    application/pdf
sha256:          0x8c4e2a7b5f3d9a1c6e8b3f5d7a0c2e4b6c8e1d3f5a7c9e2b4d6f8a0c2e4b6d8e
worm:            true
worm_until:      2034-04-11T17:42:08-04:00
signed_by:       naveen.iyer@tessellate-health-ai-inc
counsel_review:  hampton.reese@tessellate-health-ai-inc
ac_signoff:      jasmine.wells-okafor + tunde.akinwale + lisa.cheng-halsey
super_merkle_root: 0xf3a8c2e7b6d9f4a1c8e3b5d7f0a2c4e6b8d1f5a3c7e9b2d4f6a8c0e2b5d7f1a4
retention_authority: SEC-pre-IPO-adapted-from-17-CFR-240-17a-4-7-year
```

The super-Merkle root is anchored to the external transparency log batch `external-transparency-log-batch-2027-04-11T1742`. Any independent observer can verify the root from the external log + verify each per-pack root from the super-Merkle.

`EVT-J165-REPORT-ARCHIVED-011` sealed at 17:42:18 EDT Sunday.
`EVT-J165-EXTERNAL-ANCHOR-013` sealed at 17:42:42 EDT Sunday (after the 24-second external batch publication).

`EVT-J165-REGIONAL-EVIDENCE-PRESERVED-012` sealed: confirming each region's pack evidence stayed local + only the per-pack Merkle root crossed regions (the hash crossed; the data did not).

## §9 — Monday April 12 06:42–11:18 EDT: distribution to board pre-read

Monday morning Naveen + Hampton + Jasmine meet via meet µservice at 09:00 EDT for a 22-minute walkthrough. Naveen walks them through the final report one more time. They all confirm the report is ready for the Wednesday board meeting.

Naveen uploads the report PDF to the board pre-read distribution folder at 11:18 EDT Monday. The 8 board members each get a notification + a link. Pre-read is open from Monday 11:18 EDT through Wednesday 13:00 EDT. The Wednesday 14:00 EDT board meeting will discuss the report substantively for ~75 minutes.

Naveen closes the GRC console and walks to the kitchen for water at 11:24 EDT Monday. He thinks about Wednesday. The board will likely approve all three audit-committee recommendations. The Bangalore related-party item will likely generate one board-level clarifying question (Naveen guesses it will come from Director Patricia Hwong, who is a former tax partner). The S-1 risk-factor refresh will likely generate a counsel-led discussion. The EU AI Act Article 9 documentation gap is the most likely to generate a board-level question — Naveen mentally rehearses the answer one more time.

## §10 — Beats not on the wire (the human texture)

- At 06:24 Thursday morning Naveen had a brief moment of imposter syndrome before he started the workflow. He has been at Tessellate 19 months; this is his 7th quarterly board report. The first one (Q3-2025) was rough. The next two were stable. By Q1-2026 he had a rhythm. This Q1-2027 is the most complex he has done — 8 active packs instead of 5 — and the EU AI Act is a regulation he is still learning the operational implications of.
- At 14:42 Thursday afternoon when Naveen reviewed the LLM-assisted exec summary, the under-statement of EU AI Act risk made him think about how he has been training the LLM with prior quarterly reports that mostly didn't have EU AI Act on them. He noted this for the model card maintenance team. The LLM is a tool; the human is the editorial conscience.
- At 17:24 Friday when the super-Merkle root was computed, Naveen took a screenshot for his own records. He keeps a personal log of every quarterly Merkle root in his notes µservice. He has 7 roots now (one per quarter since Q3-2025).
- At 12:24 PDT Saturday when Hampton started counsel review, Hampton's husband had just left for a tennis match. Hampton works Saturdays sometimes; he's deliberate about not making it a habit; this one was unavoidable given the board calendar.
- At 19:18 PDT Saturday Jasmine's Saturday evening review, she was in her Sausalito home office with a glass of wine. Her two daughters were at a friend's house. She has done board work for 27 years; she finds it grounding.
- At 06:42 Sunday morning Naveen's response to Jasmine's two questions, he was in his Brookline kitchen in pajamas. Marcus was still asleep. He drank coffee + typed responses + felt the very specific Sunday-morning compliance-officer satisfaction of having anticipated 2 of the 3 questions Jasmine would ask.
- At 11:18 Monday morning when the report uploaded to board pre-read, Naveen also pinged the board chair Margaret Donovan-Walsh privately with a one-sentence summary: "Q1 report is in pre-read; 0 SEC triggers; 8 packs healthy; one substantive EU AI Act remediation gap to discuss; recommend full board approval of audit committee package." Margaret replied 18 minutes later: "Thank you, Naveen. See you Wednesday." That single thank-you was the moment Naveen actually relaxed.

## §11 — Stop condition for this story

This story documents the 3-day arc from 06:18 EDT Thursday April 8 through 11:18 EDT Monday April 12 — Naveen's full work-cycle to assemble, counsel-review, audit-committee-sign-off, archive, and pre-read-distribute the Q1-2027 Tessellate Health AI quarterly board compliance report. The acceptance criteria in `README.md`, API shapes in `handshake.md`, test cases in `integration-test-plan.md`, and schemas together encode the machine semantics. The story exists so the next reader understands WHY the super-Merkle-of-Merkles is the right primitive (each pack stays sovereign; only hashes cross regions; the bundle is independently verifiable), WHY the workflow-engine state-machine is Cedar-gated at every transition (audit-committee work product cannot reach the board without quorum-validated sign-off), and WHY the SEC 8-K trigger evaluation runs inline during drafting (pre-IPO Tessellate isn't yet under 8-K obligation but the evaluator surface is exactly what they will need from day one after S-1 effectiveness).
