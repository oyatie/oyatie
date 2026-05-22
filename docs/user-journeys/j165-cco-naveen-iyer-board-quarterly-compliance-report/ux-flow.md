---
doc_class: User-Journey-UX-Flow
journey_id: j165-cco-naveen-iyer-board-quarterly-compliance-report
date: 2026-05-20
authority_tier: 2
status: draft
---

# j165 — UX flow: GRC console, cross-pack canvas, workflow state-machine, Merkle anchor

Five primary surfaces:

- Naveen's GRC console (M3 MacBook Pro 16" + 32" 4K monitor at Boston Seaport)
- Cross-pack assembly canvas (8 panes; one per pack; per-pack findings + remediation)
- Workflow state-machine view (draft → counsel → audit-committee → board)
- LLM-assist exec summary editor (draft text + edit-distance meter + provenance metadata)
- Drive WORM archive + super-Merkle anchor confirmation screen

All screens preserve Devanagari + Tamil + Hangul + German diacritics + Korean hospital names UTF-8 NFC byte-exact.

## Screen 1 — GRC console workflow card (06:18 EDT Thursday)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Tessellate Health AI · GRC Console · CCO View                           │
│  active tenant: tessellate-health-ai-inc · role: chief_compliance_officer │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─ Q1-2027 Quarterly Board Compliance Report ─────────────────────────┐ │
│  │                                                                     │ │
│  │  state:           not_started                                       │ │
│  │  scheduled board: 2027-04-14T14:00:00-04:00                         │ │
│  │  pre-read due:    2027-04-13T17:00:00-04:00 (5 business days)       │ │
│  │  active packs:    8                                                 │ │
│  │  expected evidence:  480–540 artifacts                               │ │
│  │                                                                     │ │
│  │  required pipeline:                                                 │ │
│  │   1. cross-pack evidence pull                                       │ │
│  │   2. per-pack findings triage                                       │ │
│  │   3. LLM-assist exec summary                                        │ │
│  │   4. per-pack Merkle compute                                        │ │
│  │   5. SEC 8-K trigger evaluation                                     │ │
│  │   6. super-Merkle-of-Merkles compute                                │ │
│  │   7. workflow transition: draft → counsel review                    │ │
│  │   8. counsel review by Hampton Reese                                │ │
│  │   9. workflow transition: counsel → audit committee                 │ │
│  │  10. AC quorum sign-off (≥ 3 of 5)                                  │ │
│  │  11. workflow transition: AC → board                                │ │
│  │  12. drive WORM archive + Merkle anchor                             │ │
│  │  13. pre-read distribution                                          │ │
│  │                                                                     │ │
│  │   [INITIATE WORKFLOW]                                               │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 2 — Cross-pack assembly canvas (08:42 EDT Thursday)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  CROSS-PACK ASSEMBLY · 8 PACKS · 496 EVIDENCE · 7 FINDINGS · 10 OPEN RISK │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─ SOC 2 TYPE II ──────────────┐  ┌─ HIPAA BA ──────────────────────┐  │
│  │  evidence: 142   findings: 3 │  │  evidence: 87   findings: 1     │  │
│  │  open risk: 2                │  │  open risk: 1 (BAA renewal)     │  │
│  │  P2: user-access-review      │  │  P3: audit-log retention        │  │
│  │  P3: subprocessor risk x2    │  │                                 │  │
│  │  [open canvas]               │  │  [open canvas]                  │  │
│  └──────────────────────────────┘  └─────────────────────────────────┘  │
│                                                                          │
│  ┌─ GDPR ───────────────────────┐  ┌─ EU AI ACT ─────────────────────┐  │
│  │  evidence: 64   findings: 2  │  │  evidence: 71   findings: 0     │  │
│  │  open risk: 1 (ROPA refresh) │  │  open risk: 4 (Art 9/12/14/15)  │  │
│  │  P2: DSAR latency            │  │                                 │  │
│  │  P3: cookie banner           │  │  Article 9 documentation gap ⚠ │  │
│  │  [open canvas]               │  │  [open canvas]                  │  │
│  └──────────────────────────────┘  └─────────────────────────────────┘  │
│                                                                          │
│  ┌─ KR PIPA ────────────────────┐  ┌─ CSAP ─────────────────────────┐   │
│  │  evidence: 42   findings: 0  │  │  evidence: 38   findings: 1    │   │
│  │  open risk: 0                │  │  open risk: 0                  │   │
│  │  ✓ all green                 │  │  P3: incident-response runbook │   │
│  │  [open canvas]               │  │  [open canvas]                 │   │
│  └──────────────────────────────┘  └────────────────────────────────┘   │
│                                                                          │
│  ┌─ PCI DSS ────────────────────┐  ┌─ SEC PRE-IPO (S-1) ────────────┐   │
│  │  evidence: 28   findings: 0  │  │  evidence: 24   findings: 0    │   │
│  │  open risk: 0                │  │  open risk: 2 (S-1 + RP txn)   │   │
│  │  ✓ all green                 │  │  ⚠ Bangalore related-party     │   │
│  │  [open canvas]               │  │  [open canvas]                 │   │
│  └──────────────────────────────┘  └────────────────────────────────┘   │
│                                                                          │
│  cross-pack themes: audit-logging · access-review · subprocessor-risk    │
└──────────────────────────────────────────────────────────────────────────┘
```

UX notes:

- 8-pack grid renders at-a-glance heatmap (color: green/yellow/orange/red based on findings + open risks).
- Each pane shows pack name + evidence count + findings + open risks + top concerns.
- "open canvas" drills into per-pack canvas with full evidence ledger.
- Cross-pack themes surface at the bottom — pattern recognition across packs.

## Screen 3 — Per-pack canvas: EU AI Act (10:14 EDT Thursday)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  PACK: EU AI Act (high-risk medical) · pack-eu-ai-act-high-risk-medical  │
│  evidence: 71 · findings: 0 · open risks: 4                              │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─ OPEN RISKS ────────────────────────────────────────────────────────┐ │
│  │                                                                     │ │
│  │  ⚠ Article 9 (Risk Management System)                              │ │
│  │    status: structured documentation in progress under Q2 program    │ │
│  │    remediation owner: M. Garcia (VP Engineering)                    │ │
│  │    target: Q2 end                                                   │ │
│  │    material_to_disclose: yes                                        │ │
│  │                                                                     │ │
│  │  ⚠ Article 12 (Logging Completeness)                                │ │
│  │    status: 87% logging completeness vs target 95%                   │ │
│  │    remediation: extend OpenTelemetry coverage to 4 missed surfaces  │ │
│  │    target: Q2 mid                                                   │ │
│  │    material_to_disclose: partial                                    │ │
│  │                                                                     │ │
│  │  ⚠ Article 14 (Human Oversight)                                     │ │
│  │    status: evidence gap on real-time oversight workflow             │ │
│  │    remediation: deploy human-in-the-loop dashboard                  │ │
│  │    target: Q3 start                                                 │ │
│  │    material_to_disclose: partial                                    │ │
│  │                                                                     │ │
│  │  ✓ Article 15 (Accuracy + Robustness + Cybersecurity)               │ │
│  │    status: green with continuous monitoring                         │ │
│  │    target: continuous                                               │ │
│  │                                                                     │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  EVIDENCE LEDGER (71 artifacts)                                          │
│   • Risk assessment doc v3.2 (Article 9)                                 │
│   • OpenTelemetry config + log volume daily (Article 12)                 │
│   • Human oversight workflow doc v2.1 (Article 14)                       │
│   • Robustness testing results Q1 (Article 15)                           │
│   • Model card: TessellateCDS v4.0                                       │
│   • CE marking documentation (MDR Class IIa)                             │
│   ... 65 more                                                            │
│                                                                          │
│  [draft section for report]   [mark complete]                            │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 4 — Workflow state-machine view (17:42 EDT Friday)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  WORKFLOW STATE MACHINE · Q1-2027 board compliance report                │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   ┌────────┐        ┌──────────────┐        ┌─────────────────┐        ┌────┐  │
│   │ draft  │ ─────→ │counsel review│ ─────→ │audit committee  │ ─────→ │board│ │
│   │        │        │              │        │  sign-off        │        │presnt│ │
│   └────────┘        └──────────────┘        └─────────────────┘        └────┘  │
│      ✓                  →                          □                       □    │
│   complete           propose now                pending                 pending  │
│                                                                                 │
│   transitioning: draft → counsel_review                                         │
│   actor: naveen.iyer@tessellate-health-ai-inc                                  │
│   guard satisfaction:                                                           │
│    ✓ cco_signoff_present                                                       │
│    ✓ passkey_assertion_present                                                 │
│    ✓ super_merkle_root_present                                                 │
│    ✓ twelve_sections_complete (12/12)                                          │
│                                                                                 │
│   downstream actor on next state: hampton.reese@tessellate-health-ai-inc       │
│   notification mode: messenger + email                                          │
│                                                                                 │
│   ┌─────────────────────────────────────┐                                       │
│   │  CONFIRM TRANSITION (passkey)       │                                       │
│   └─────────────────────────────────────┘                                       │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 5 — LLM-assist exec summary editor (14:42 EDT Thursday)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  EXECUTIVE SUMMARY EDITOR · LLM-assisted · human-finalized                │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─ LLM PROVENANCE ────────────────────────────────────────────────────┐ │
│  │  model: sonnet-compliance-tuned-v3@oyatie-2027-03                   │ │
│  │  prompt template: quarterly-board-exec-summary-v4                   │ │
│  │  input tokens: 14,820   output tokens: 1,840                       │ │
│  │  latency: 4.2s                                                      │ │
│  │  EU AI Act Article 50 declaration: this is a generative AI system  │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ DRAFT TEXT (with edit tracking) ───────────────────────────────────┐ │
│  │                                                                     │ │
│  │  In Q1 2027 Tessellate's compliance posture remained strong across  │ │
│  │  ~~seven~~ eight active packs. The introduction of the EU AI Act    │ │
│  │  high-risk-medical pack ~~has been managed effectively~~ has        │ │
│  │  surfaced four documentation and evidence gaps that the team is     │ │
│  │  actively remediating against a Q2-Q3 plan. SOC 2 Type II evidence  │ │
│  │  collection found one P2 finding on user-access-review cadence...   │ │
│  │                                                                     │ │
│  │  (3 pages more)                                                     │ │
│  │                                                                     │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ EDIT METRICS ──────────────────────────────────────────────────────┐ │
│  │  edit distance from LLM output: 38% (substantial human edit)        │ │
│  │  review duration: 47 minutes                                        │ │
│  │  human signoff: naveen.iyer@tessellate-health-ai-inc               │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  [ save draft ]   [ finalize for report ]                                │
└──────────────────────────────────────────────────────────────────────────┘
```

UX notes:

- LLM provenance is foregrounded — Naveen always knows the model + version + prompt template + tokens used.
- Edit tracking shows what Naveen changed from the LLM output (visible strikethrough + insertions).
- Edit distance shown as a percentage — Naveen sees how much of the draft is his vs the LLM's.
- EU AI Act Article 50 transparency declaration inline.

## Screen 6 — SEC 8-K trigger evaluation panel (16:18 EDT Thursday)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  SEC FORM 8-K TRIGGER EVALUATION · 2027-04-08 16:18 EDT                  │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Filing obligation status: PRE-IPO (8-K not yet attached)                │
│  Post-S-1 effective date estimate: H2 2027                               │
│                                                                          │
│  ┌─ ITEMS EVALUATED ───────────────────────────────────────────────────┐ │
│  │   ✓ Item 1.01 (Material Definitive Agreement)            NO TRIGGER │ │
│  │   ✓ Item 1.02 (Termination Material Definitive Agt)      NO TRIGGER │ │
│  │   ✓ Item 2.02 (Results of Operations + Financial Cond)   NO TRIGGER │ │
│  │   ✓ Item 2.04 (Triggering Events — Financial Obligation) NO TRIGGER │ │
│  │   ✓ Item 4.02 (Non-Reliance on Financial Statements)     NO TRIGGER │ │
│  │   ✓ Item 5.02 (Departure of Directors / Officers)        NO TRIGGER │ │
│  │   ✓ Item 8.01 (Other Events — voluntary)                 NO TRIGGER │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ⓘ Note: Tessellate is pre-IPO; 8-K filing obligations do not yet attach.│
│    This evaluator runs in shadow mode during the pre-IPO period; will   │
│    flip to enforcement-mode upon S-1 effectiveness.                      │
│                                                                          │
│  Form NT (notification of inability to file): NO TRIGGER (not yet appl.) │
│                                                                          │
│  [ record evaluation in audit ]                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 7 — Super-Merkle anchor confirmation (17:24 EDT Friday)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  SUPER-MERKLE OF MERKLES · Q1-2027 board report bundle                   │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─ PER-PACK ROOTS (ordered pack_id ascending) ───────────────────────┐ │
│  │  [1] pack-sec-pre-ipo-s1-active                                    │ │
│  │      0xa8c3e7b2f4d9a1c6e8b3f5d7a0c2e4b6                            │ │
│  │  [2] pack-gdpr-controller-processor-mixed                           │ │
│  │      0xb1d4f8c3a7e6b9c2d5f0a4e8b7c1d3f6                            │ │
│  │  [3] pack-pci-dss-saq-c                                            │ │
│  │      0x4d8f1b6e9a3c7d2f5b8e0a1d4c7f9b3e                            │ │
│  │  [4] pack-csap-naver-cloud-tier-3                                  │ │
│  │      0xe2b7d4a9c6f8e1b3a5d7c0f2b9e4a8c6                            │ │
│  │  [5] pack-kr-pipa-controller                                       │ │
│  │      0x5f8a1c7e9b3d6f2a4c8e0b5d7f3a9c1e                            │ │
│  │  [6] pack-eu-ai-act-high-risk-medical                              │ │
│  │      0x9c6e2a8b4d7f3c1e5a0b8d6c4f9e2b3a                            │ │
│  │  [7] pack-hipaa-business-associate                                  │ │
│  │      0x3e8b2f9a6c4d1e7f5a8b3c0d6e9f2a4b                            │ │
│  │  [8] pack-soc2-type2-fy2026                                        │ │
│  │      0x7a2f4b8c1e9d5f3a6b2c8e0f4d7a9b1c                            │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ SUPER-MERKLE ROOT ────────────────────────────────────────────────┐ │
│  │                                                                     │ │
│  │  0xf3a8c2e7b6d9f4a1c8e3b5d7f0a2c4e6b8d1f5a3c7e9b2d4f6a8c0e2b5d7f1a4 │ │
│  │                                                                     │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  Anchored to:                                                            │
│   ✓ audit-chain-spine-tessellate-q1-2027                                │
│   ✓ external-transparency-log-batch-2027-04-11T1742 (after archive)     │
│                                                                          │
│  Independent verification: any observer can recompute the super-root     │
│  from the 8 per-pack roots + ordering convention; can verify against     │
│  the external transparency log.                                          │
│                                                                          │
│  [ proceed to workflow transition ]                                      │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 8 — Audit Committee sign-off view (Saturday April 10, evening, Jasmine's iPad)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  AUDIT COMMITTEE SIGN-OFF · Jasmine Wells-Okafor (Chair)                  │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Q1-2027 Quarterly Board Compliance Report                               │
│  drafted by: Naveen Iyer (CCO)                                           │
│  counsel reviewed by: Hampton Reese (GC) · 2027-04-10 16:32 EDT          │
│   3 counsel redlines · all resolved by Naveen                            │
│                                                                          │
│  Section summary:                                                        │
│   1. Executive Summary                       (4 pages)                   │
│   2. SOC 2 Type II status                   (5 pages)                   │
│   3. HIPAA BA + BAA renewal                  (4 pages)                   │
│   4. GDPR + EU AI Act EU status             (8 pages)                    │
│   5. KR PIPA + CSAP                         (3 pages)                   │
│   6. PCI DSS                                 (2 pages)                   │
│   7. SEC pre-IPO + S-1 refresh              (5 pages)                   │
│   8. Cross-pack themes                       (4 pages)                   │
│   9. SEC trigger evaluation                  (3 pages)                   │
│  10. Audit committee recommendations         (4 pages)                   │
│  11. Open risks register                     (3 pages)                   │
│  12. Appendices                              (2 pages)                   │
│                                                                          │
│  Audit committee sign-off status:                                        │
│   □ Jasmine Wells-Okafor (chair)         pending — you                   │
│   ✓ Tunde Akinwale (independent)         signed 2027-04-11 12:48 EDT     │
│   ✓ Lisa Cheng-Halsey (independent)      signed 2027-04-11 11:32 PDT     │
│   □ Marcus Lin (NED)                     deferred to pre-read           │
│   □ Patricia Hwong (independent)         deferred to pre-read           │
│                                                                          │
│  Quorum threshold: 3 of 5 (Cedar-enforced)                               │
│  Current: 2 / 3 → 3 / 3 if you sign                                      │
│                                                                          │
│  My questions for Naveen (sent + answered):                              │
│   ✓ Q1: Bangalore related-party — full disclosure rationale?            │
│       Answer: $84K is below $120K S-1 Item 404 threshold;                │
│       AC policy to disclose anyway. Hampton concurred.                   │
│   ✓ Q2: EU AI Act Article 9 remediation timeline confidence?            │
│       Answer: M. Garcia (VP Eng) has committed Q2 end; tracking weekly.  │
│                                                                          │
│  ┌─────────────────────────────────────┐                                 │
│  │  SIGN OFF (passkey + YubiKey)       │                                 │
│  └─────────────────────────────────────┘                                 │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 9 — Drive WORM archive confirmation (17:42 EDT Sunday)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  DRIVE WORM ARCHIVE · Q1-2027 board compliance report                    │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ✓ Final report archived                                                 │
│                                                                          │
│  drive_room:    tessellate/board/2027/q1/compliance-report               │
│  filename:      2027-q1-tessellate-board-compliance-report-final.pdf     │
│  size:          12.4 MB                                                  │
│  pages:         47                                                       │
│                                                                          │
│  Approval chain                                                          │
│   ✓ CCO sign-off:        Naveen Iyer · 2027-04-09 17:42 EDT              │
│   ✓ Counsel review:      Hampton Reese · 2027-04-10 16:32 EDT            │
│   ✓ AC chair sign-off:   Jasmine Wells-Okafor · 2027-04-11 14:18 EDT     │
│   ✓ AC independent #1:   Tunde Akinwale · 2027-04-11 12:48 EDT           │
│   ✓ AC independent #2:   Lisa Cheng-Halsey · 2027-04-11 11:32 PDT        │
│                                                                          │
│  Retention                                                               │
│   ✓ WORM lock engaged                                                    │
│   ✓ Retention authority: SEC-pre-IPO-adapted-17-CFR-240-17a-4            │
│   ✓ Retention until 2034-04-11                                           │
│                                                                          │
│  Super-Merkle root                                                       │
│   0xf3a8c2e7b6d9f4a1c8e3b5d7f0a2c4e6b8d1f5a3c7e9b2d4f6a8c0e2b5d7f1a4     │
│                                                                          │
│  Anchored to:                                                            │
│   ✓ audit-chain-spine-tessellate-q1-2027                                │
│   ✓ external-transparency-log-batch-2027-04-11T1742                     │
│                                                                          │
│  Regional evidence preservation:                                         │
│   ✓ us-east: 281 artifacts (SOC 2 + HIPAA + PCI + SEC)                  │
│   ✓ eu-frankfurt: 135 artifacts (GDPR + EU AI Act)                      │
│   ✓ kr-seoul: 80 artifacts (KR PIPA + CSAP)                             │
│   ✓ only hashes crossed regions; data stayed local                       │
│                                                                          │
│  [ proceed to pre-read distribution ]                                    │
└──────────────────────────────────────────────────────────────────────────┘
```
