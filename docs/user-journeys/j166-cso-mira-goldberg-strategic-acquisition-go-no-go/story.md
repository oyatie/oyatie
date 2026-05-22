---
doc_class: User-Journey-Story
journey_id: j166-cso-mira-goldberg-strategic-acquisition-go-no-go
date: 2026-05-20
authority_tier: 2
status: draft
---

# j166 — Story: May 15 07:42 EDT Friday, the 9-day countdown begins

## §0 — Friday May 15, 2027, 07:42 EDT — Skylark Logistics, 50 Liberty Drive, Boston Seaport

Spring is on. 14°C. The Seaport harbor is calm under thin overcast. The willows along Pier 4 have leafed out. Mira Goldberg badges into the 22nd-floor Skylark executive suite at 07:42:14 EDT. She is wearing a navy blazer + slim charcoal trousers + the cobalt fountain pen she uses for paper signatures (decreasingly relevant in a passkey-first workplace; she keeps it for emotional ballast).

Her office faces east + has a view across the Fort Point Channel toward South Boston. She turns on her two monitors. The right monitor is the **M&A console** (a focused workspace on the financial-planning + intelligence + connect µservices). The left monitor is for everything else — email, messenger, the GRC console (when she occasionally cameos in CCO Naveen Iyer's compliance work).

The active-tenant pill reads `skylark-logistics-solutions-inc · executive · chief_strategy_officer`. She opens the **MRT acquisition workspace**:

```
[ACQUISITION] Mendelsohn Routing Technologies — Strategic Acquisition Q2-2027
─
state:                    due_diligence (week 9 of 9)
board_decision_target:    2027-05-25T09:00:00-04:00 (7 business days)
working_price:            $186M
price_range:               $172M – $202M
structure:                 60% cash + 40% Skylark stock
earnout:                   $30M (Bjorn Mendelsohn 24-month vest)
deal_partner:              mendelsohn-routing-technologies-inc-de
counsel:                   Daphne Harrowgate (GC)
financial_lead:            Reginald Otis (CFO)
board_chair:               Adrian Cheng-Whitford (CEO + board chair)
audit_committee_chair:     Hannah Beauregard
strategy_committee_chair:  Margarita Velasco-Heim
filings_required:          HSR + EU-MR + UK-CMA + Israeli-IMC
nda:                       nda-skylark-mrt-2027-03-08 ACTIVE
```

`EVT-J166-WORKSPACE-OPEN-000` sealed at 07:42:48.

## §1 — May 15 07:42–11:48 EDT: cross-tenant NDA channel + diligence inbox

The cross-tenant NDA-bound channel `cross-tenant-channel-skylark-mrt-2027-q2` was opened on 2027-03-08 when the NDA was signed. Today is Day 68 of the channel's life. It has carried 23 documents from MRT → Skylark so far + 14 documents from Skylark → MRT (Skylark-side diligence on its own posture for the stock-component of the deal).

Mira opens the diligence inbox. There are 4 new documents from MRT that arrived overnight (Berlin is +6 hours; MRT's deal team works late their time):

```
DILIGENCE INBOX · MRT → Skylark · 2027-05-15
─
[NEW] doc-mrt-q1-2027-cohort-churn-anonymized.csv          1.2 MB  arrived 02:14 EDT
[NEW] doc-mrt-2026-customer-concentration-named.pdf       4.8 MB  arrived 02:48 EDT
[NEW] doc-mrt-integration-architecture-overview-v2.pdf    8.4 MB  arrived 04:22 EDT
[NEW] doc-mrt-data-residency-attestation-eu.pdf            612 KB  arrived 06:18 EDT
```

Each document is wrapped in the NDA-channel envelope:

```
ENVELOPE (sample, doc-mrt-q1-2027-cohort-churn-anonymized.csv)
─
from_tenant:               mendelsohn-routing-technologies-inc-de
from_principal:             bjorn.mendelsohn@mendelsohn-routing-technologies-inc-de
to_tenant:                 skylark-logistics-solutions-inc
to_principals:              ["mira.goldberg@skylark-logistics-solutions-inc",
                            "reginald.otis@skylark-logistics-solutions-inc",
                            "daphne.harrowgate@skylark-logistics-solutions-inc"]
nda_record_id:              nda-skylark-mrt-2027-03-08
payload_class:              diligence_response_anonymized
payload_size_bytes:        1,221,408
e2ee_envelope:              true (MLS group cross-tenant-channel-skylark-mrt-2027-q2)
sent_at:                    2027-05-15T08:14:18+02:00 (Berlin) = 02:14:18 EDT
audit_event_id:             EVT-J166-DOC-ARRIVED-mrt-cohort-churn-Δ001
```

The cohort churn CSV is the document Mira has been waiting for. It contains 142 customer cohorts × 36 monthly retention data points = ~5,100 cells. The data is anonymized (customer IDs are hashed; no PII; no contract values; only retention rates per cohort). This is the ML-input grade data she needs to drive the cohort-churn forecast.

She runs the document through the compliance µservice's NDA-payload validator:

```
NDA-PAYLOAD VALIDATION
─
✓ payload_class "diligence_response_anonymized" matches NDA scope
✓ no PII detected (run through PII scanner — 0 hits)
✓ no contract values present (financial-value scanner — 0 hits)
✓ payload size within NDA-allowed envelope (≤ 50 MB per doc)
✓ e2ee envelope intact
✓ sender authorization: Bjorn Mendelsohn is on MRT's authorized signer list
```

`EVT-J166-NDA-PAYLOAD-VALIDATED-Δ001a` sealed.

Mira archives the documents to the drive room `skylark/m-a/2027-q2/mrt/diligence-inbox/` with the cross-tenant evidence flag (these are MRT's data; Skylark holds them as a temporary recipient under the NDA's data-handling terms; they will be returned/destroyed per NDA Section 7.2 after the deal closes or terminates).

## §2 — May 15 11:48–17:32 EDT: financial model first pass

Mira opens the **financial-planning** µservice's M&A model canvas. The model template `m-and-a-acquisition-saas-mid-market-v4` has been her working model for 6 weeks. She enters the latest MRT inputs:

```
M&A MODEL INPUTS · MRT · 2027-05-15
─
target ARR (audited TTM):          $42.0M
target ARR growth YoY:              31%
target gross margin:                 78%
target customer count:                340
target avg ARR per customer:       $123,500
target CAC (LTM):                  $48,000 per new logo
target LTV/CAC (LTM):                  4.2x
target net dollar retention:         117%
target gross dollar retention:        94%
target customer concentration:      top-10 = 31% of ARR
target cohort churn (CSV):           [loaded from new doc]

skylark ARR (TTM):                  $148M
skylark customer count:               1,820
skylark avg ARR per customer:       $81,300
skylark net dollar retention:        128%

deal_terms:
  price_range_low:                  $172M
  price_working:                     $186M
  price_high:                        $202M
  structure_cash_pct:                 60%
  structure_stock_pct:                 40%
  earnout:                            $30M
  earnout_condition:                  Bjorn Mendelsohn 24-month vest
```

She runs the model at all three price points. Output:

```
SCENARIO          $172M               $186M               $202M
─
revenue multiple   4.1x ARR            4.4x ARR            4.8x ARR
NTM accretive at   18 months           23 months           34 months
year-3 IRR          22%                  18%                  12%
year-5 IRR          27%                  22%                  16%
year-3 cumulative
  cash impact      -$48M              -$62M               -$78M
year-5 cumulative
  cash impact      +$104M              +$84M              +$54M

dilution to skylark stock:
  $172M:             4.2%
  $186M:             4.6%
  $202M:             5.0%
```

`EVT-J166-M-A-MODEL-COMPUTED-003` sealed at 14:42 EDT.

Mira saves the model + adds her notes to the notebook `m-a-mrt-2027-q2` in the notes µservice:

> "$186M working price gives 23-month accretion; year-3 IRR 18%. Below the 20% IRR threshold the board approved as the minimum yield criterion. Either we negotiate down toward $176-180M, or we make the case that the strategic value justifies sub-threshold IRR. CFO needs to weigh in."

She messages CFO Reginald Otis on the messenger executive-only channel:

> "Reg — first-pass model is at $186M = 23-month accretion + 18% year-3 IRR. Below the 20% threshold. Want to talk this evening or Monday?"

Reginald replies 11 minutes later: "Monday morning 09:00. I'll review the model over the weekend."

## §3 — Monday May 18 06:18–17:48 EDT: ML scenario modeling

Mira is back in the office Monday May 18 at 06:18 EDT. She invokes the **intelligence** µservice's ML-scenario-modeling endpoint:

```
ML SCENARIO MODELING REQUEST · 2027-05-18T06:42:08-04:00
─
deal_id:                 mrt-acquisition-2027-q2
ml_models:
  - model: "monte-carlo-mid-market-saas-v7@oyatie-2027-02"
    iterations: 10000
    scenarios: ["recession", "neutral", "tailwind"]
  - model: "cohort-churn-forecast-saas-v5@oyatie-2027-04"
    horizon_years: 5
    inputs: "MRT cohort-churn CSV (5100 cells)"
  - model: "integration-cost-forecast-cross-stack-v3@oyatie-2027-01"
    integration_complexity_score: 0.62
    skylark_stack_inventory: <internal>
    mrt_stack_inventory: <from diligence>
```

The intelligence µservice runs the inference. Results return at 06:54 EDT (~12 minutes for the full battery):

```
ML SCENARIO RESULTS
─
Monte-Carlo 10K iterations × 3 scenarios:

  Recession (P10):
    ARR @ year-5: $58M (target $58M base)
    IRR year-5:   8% at $186M price
    Probability of negative cash flow year-3+: 18%

  Neutral (P50):
    ARR @ year-5: $98M
    IRR year-5:   24% at $186M
    Probability of negative cash flow year-3+: 3%

  Tailwind (P90):
    ARR @ year-5: $148M
    IRR year-5:   38% at $186M
    Probability of negative cash flow: 0%

  Expected (probability-weighted):
    ARR @ year-5: $94M
    IRR year-5:   23% at $186M  ← above the 20% threshold

Cohort churn forecast (5-year horizon):
  Year-1: 6.2% gross / 88% net dollar retention from existing cohorts
  Year-2: 6.8% gross / 87% net
  Year-3: 7.1% gross / 86% net
  Year-4: 7.4% gross / 86% net
  Year-5: 7.6% gross / 85% net
  (NDR trend declining; consistent with mid-market SaaS norms;
   MRT's current 117% NDR is unsustainable at the year-5 horizon)

Integration cost forecast:
  point estimate: $14.2M
  95% CI:        $9.4M – $19.0M
  primary drivers:
    - data migration from PostgreSQL 14 → 16 (MRT) + alignment to
      Skylark's Postgres 16 sharded fleet
    - identity unification (Skylark uses oyatie identity; MRT uses
      Auth0 + custom)
    - route-optimization engine integration (deep work; 4–6 senior
      engineers × 9 months)
```

`EVT-J166-ML-SCENARIOS-004` sealed at 06:54 EDT.

Mira reviews the results carefully. The probability-weighted 5-year IRR of 23% is above the board's 20% threshold. The cohort churn forecast confirms MRT's exceptional NDR is mean-reverting but stays healthy. The integration cost $14.2M ± $4.8M is on the higher end of what she had budgeted ($12M working assumption) but within tolerable range.

She updates the M&A model with the ML inputs at 08:18 EDT.

## §4 — May 18 09:00–10:42 EDT: CFO meeting

Reginald Otis arrives in Mira's office at 09:00 EDT sharp. He has the M&A model open on his laptop. They go through the model section by section. Reginald's concerns:

- The $14.2M integration cost is higher than the $12M working assumption — would push the year-1 cash impact more negative
- The ML cohort churn forecast (gross 6.2% → 7.6% over 5 years) is more conservative than MRT's own historical 5.8% — Mira agrees the ML model is appropriately skeptical
- The earnout structure ($30M conditional on Bjorn's 24-month vest) is unusual and creates a key-person dependency

They agree:

- Working price stays $186M
- Mira will probe MRT on whether the earnout could be unconditional or structured around team retention more broadly
- They'll bring an explicit "key-person risk" framing to counsel review

Reginald signs off on the financial model at 10:32 EDT.

`EVT-J166-CFO-SIGNOFF-008` sealed at 10:42 EDT (after 10-minute formal review checklist).

## §5 — May 18 14:18–17:48 EDT: pack-manifest cross-check

Mira invokes the **compliance** µservice's pack-manifest cross-check between Skylark and MRT:

```
PACK MANIFEST CROSS-CHECK · skylark + mrt
─
Skylark active packs (6):
  - pack-soc2-type2-fy2026 (audit: PwC)
  - pack-gdpr-controller (EU customers ~22% ARR)
  - pack-ccpa-controller (CA customers ~14% ARR)
  - pack-hipaa-business-associate (limited; 8 BAAs)
  - pack-pci-dss-saq-c (billing)
  - pack-iso-27001-active

MRT active packs (5):
  - pack-soc2-type1-fy2026 (audit: KPMG) ← Type 1 not Type 2
  - pack-gdpr-controller (EU customers ~78% ARR)
  - pack-iso-27001-active
  - pack-german-bdsg (German federal data protection)
  - pack-tisax-vda (German automotive industry standard)

Overlap analysis:
  Common: SOC 2 (but different Type), GDPR, ISO 27001
  Skylark-only: CCPA, HIPAA BA, PCI DSS
  MRT-only: German BDSG, TISAX/VDA

Blockers identified:
  ⚠ SOC 2 Type 1 vs Type 2 — MRT's Type 1 is less rigorous than
    Skylark's existing posture. Post-close, MRT operations must
    upgrade to Type 2 within 18 months to meet Skylark's
    customer-facing baseline. Remediation cost estimated $480K.

Open considerations:
  - TISAX/VDA pack on MRT is positive — opens automotive vertical
    integration. Should be highlighted in the strategic rationale.
  - German BDSG overlay is essentially subsumed by GDPR for Skylark's
    operating model; no incremental burden.
  - HIPAA BA — MRT has no healthcare customers; Skylark's BAA portfolio
    stays unchanged.

Compatibility score: 84% (high)
```

`EVT-J166-PACK-CROSS-CHECK-005` sealed at 16:42 EDT.

Then the merger filing requirements computation:

```
MERGER FILING REQUIREMENTS
─
HSR (US):
  threshold: $111.4M size-of-transaction (2027)
  deal_size: $186M
  filing_required: YES
  filing_fee: $280,000 (size class $161.5M-$268.5M)
  waiting_period: 30 days standard
  early_termination_likely: YES

EU Merger Control:
  threshold:  global turnover > €5B + community turnover > €250M per party
  skylark_turnover: $148M (€135M ≈)  ← below threshold
  mrt_turnover: $42M (€38M ≈)  ← below threshold
  filing_required_threshold_basis: NO

  alternative: national notification in member states
  national_required_in: Germany (BWB/Bundeskartellamt) — required because
    MRT is German-domiciled and the combined turnover threshold in DE
    is met (€50M domestic + €17.5M-€500K thresholds)
  german_bwb_filing_required: YES
  filing_window: 1 month
  
UK CMA:
  threshold: target UK turnover > £70M OR share of supply > 25%
  mrt_uk_turnover: $4.2M (£3.4M ≈) ← below £70M threshold
  filing_required_threshold_basis: NO
  alternative: voluntary notification (recommended for transparency)
  voluntary_filing_recommended: YES
  
Israeli IMC:
  threshold: combined turnover > NIS 360M AND each party > NIS 20M
  skylark_il_turnover: minimal (no IL operations beyond CSO citizenship)
  mrt_il_turnover: minimal
  filing_required: NO (below thresholds)
  parties_review_recommended: NO

Total filing burden:
  - HSR US — required, $280K fee, 30-day waiting
  - German BWB — required (NOT EU-level, NATIONAL), 1-month window
  - UK CMA — voluntary, recommended
  - Israeli IMC — none required

Estimated regulatory clearance window: 30–45 days post-signing
```

`EVT-J166-MERGER-FILINGS-006` sealed at 17:32 EDT.

Mira closes the day with a working brief: the deal is regulatory-feasible. HSR is the binding constraint at 30 days; German BWB runs in parallel; the deal can close ~45 days post-signing.

## §6 — Tuesday May 19 07:12–17:48 EDT: counsel review (Daphne Harrowgate)

GC Daphne Harrowgate (in Boston) opens the deal package Tuesday morning. She reviews the financial model + ML scenarios + pack-cross-check + filing requirements + diligence inbox over the day. At 16:48 EDT she submits her counsel review:

```
COUNSEL REVIEW · DAPHNE HARROWGATE · 2027-05-19
─
redlines:
  R1. Earnout structure: recommend converting to "team retention pool"
      rather than single-person key-person dependency. Bjorn vests his
      portion; remaining ~40% of earnout vests on broader leadership team
      retention.
  R2. SOC 2 Type 1→2 remediation: write into LOI as condition precedent
      to close-out compensation pool (i.e., earnout deferred 6 months until
      Type 2 readiness audit complete).
  R3. German BWB filing: counsel recommends Tessellate-class outside counsel
      (Hengeler Mueller in Frankfurt). Skylark's standard outside counsel
      Cooley LLP has SF + DC presence; need German co-counsel.
  R4. NDA Section 7.2 (return/destroy obligation): clarify return-OR-destroy
      election timeline; current NDA gives 30 days but the data-residency
      attestation from MRT references 60 days standard.

deal-term clarification:
  - Founder Bjorn Mendelsohn — is he expected to relocate to Boston post-close,
    or remain in Berlin? This affects integration cost + retention probability.
    Recommend deal sheet clarify before board package.
```

`EVT-J166-COUNSEL-REVIEW-007` sealed at 17:42 EDT.

Mira reads Daphne's review at 18:12 EDT. She agrees with all 4 redlines + the deal-term clarification. She updates the deal package.

She also messages Bjorn via the cross-tenant channel asking the relocation question. Bjorn responds Wednesday 09:14 CET (= 04:14 EDT): "I will remain Berlin-based. Skylark Berlin office to be established for MRT integration." Mira archives this; this is material context for the board.

## §7 — May 20 09:00–11:48 EDT: Wednesday — strategy + audit committee endorsement

Wednesday at 09:00 EDT Mira presents the deal package to the Strategy + M&A committee (5 members chaired by Margarita Velasco-Heim). The meeting runs 90 minutes. Discussion focuses on:

- The 23% probability-weighted year-5 IRR (above 20% threshold; supports go)
- Key-person earnout — committee agrees with Daphne's redline R1
- Strategic fit — MRT's route-optimization engine is complementary; the TISAX automotive overlay is genuinely additive
- Integration risk — $14.2M ± $4.8M is acknowledged as high; committee asks for the integration playbook

By 10:32 EDT 4 of 5 strategy committee members endorse (one — Director Kenji Park-Holloway — has reservations about the integration cost variance). Mira commits to including an integration playbook annex in the board pack.

In parallel, the Audit committee (chaired by Hannah Beauregard) reviews the regulatory filings + pack-cross-check materials. By 11:48 EDT 3 of 5 audit committee members endorse.

`EVT-J166-COMMITTEE-ENDORSEMENT-009` sealed at 11:48 EDT Wednesday.

## §8 — May 21–22 (Thursday–Friday): final deal package + integration playbook

Mira spends Thursday + Friday assembling the final 84-page board package:

- Cover letter from CSO + CEO (3 pages)
- Executive summary (4 pages)
- Strategic rationale (8 pages)
- Financial model + scenarios (16 pages)
- ML scenario modeling results (10 pages)
- Pack-manifest cross-check + compatibility analysis (6 pages)
- Merger filing requirements + timeline (5 pages)
- Counsel review summary + deal terms (8 pages)
- Integration playbook annex (12 pages — NEW)
- Audit + Strategy committee endorsement letters (4 pages)
- Risk register (4 pages)
- Appendices (4 pages — references to diligence document Merkle roots)

The intelligence µservice's LLM (Sonnet-Strategy-Tuned-v2) assists with the executive summary + strategic rationale drafts. Mira's edit distance from the LLM output is 44% (substantial human authorship).

Friday May 22 17:18 EDT she finalizes the package. The drive µservice writes to `skylark/board/2027/q2/mrt-acquisition/` with WORM 7-year retention. The governance µservice computes the per-component Merkle tree + super-Merkle root:

```
super_merkle_root:
  0xc2f8a4b7e1d6f3a9c4e7b2d5f8a1c6e9b3d7f0a4c8e2b5d9f3a7c0e4b8d2f6a1
```

The package is distributed to board pre-read Friday 17:42 EDT.

## §9 — Monday May 25 09:00 EDT: board vote

The Skylark board convenes Monday May 25 at 09:00 EDT in the 22nd-floor boardroom. Adrian Cheng-Whitford chairs. Mira presents for 22 minutes. Counsel Daphne presents for 8 minutes. CFO Reginald for 6 minutes. Q&A runs 38 minutes.

The vote at 10:54 EDT:

```
BOARD GO/NO-GO VOTE · MRT ACQUISITION
─
  Adrian Cheng-Whitford (CEO, chair)         YES
  Hannah Beauregard (audit chair)             YES
  Margarita Velasco-Heim (strategy chair)     YES
  Kenji Park-Holloway (independent)            ABSTAIN (integration cost concern)
  Christine Adebayo-Lin (independent)          YES
  Anil Subramaniam (independent)               YES
  David Hofmann-Reyes (independent)            YES
  Joon-Ho Park (independent, Korean)           YES
  Patricia Wells-Okonkwo (NED)                 NO (concern: deal size relative to skylark ARR)

Result: 7 YES + 1 ABSTAIN + 1 NO = 7/9 affirmative = GO
```

`EVT-J166-BOARD-VOTE-010` sealed at 10:54:18 EDT.

The decision is recorded. The Cedar evaluation:

```
principal: Group::"skylark_board_voting_members"
action: Action::"governance.acquisition_go_no_go_vote"
resource: AcquisitionDecision::"mrt-acquisition-2027-q2"
context: {
  audit_committee_endorsement_present: true,
  counsel_review_present: true,
  financial_model_signoff_cfo_present: true,
  passkey_assertion_present: true (each board member),
  affirmative_vote_count: 7,
  total_vote_count: 9,
  majority_threshold_5_of_9: reached
}
decision: permit
```

The decision record + super-Merkle root + board roll-call + counsel + CFO + committee endorsements are bundled. The governance µservice anchors to the audit-chain spine + external transparency log at 11:18 EDT.

`EVT-J166-DECISION-RECORDED-011` sealed at 11:18:14 EDT.

## §10 — 11:18 EDT Monday: post-vote + next steps

Adrian, Mira, Daphne, and Reginald meet in Adrian's office at 11:30 EDT to discuss next steps. Daphne will brief Cooley LLP + retain Hengeler Mueller in Frankfurt today. The signing target is May 30 EDT (5 business days). HSR filing within 2 days of signing. German BWB filing within 5 days. Target close: ~July 14 EDT (45 days post-signing).

Mira messages Bjorn via the cross-tenant channel at 11:42 EDT: "Bjorn — board approved. Adrian and I would like to meet you in Berlin next Thursday May 28 to confirm signing logistics." Bjorn replies 12 minutes later: "Yes. Will arrange. Looking forward."

Mira closes her laptop at 12:18 EDT. She walks to lunch at the Liberty Hotel café down the block. She orders a chopped salad + sparkling water. She thinks about the 9 days that just ended. The Monte-Carlo modeling, the cohort churn forecast, the pack-cross-check, the regulatory filing calculus — all of it served the actual moment of decision. The substrate did its job.

## §11 — Beats not on the wire (the human texture)

- At 07:42 Friday May 15 Mira's first action was to check the diligence inbox before doing anything else. She was nervous about the cohort churn data because if MRT's churn was meaningfully worse than reported, the deal would die at that document. It wasn't. She exhaled audibly in her office, alone.
- At 14:42 Friday May 15 the M&A model output showing 18% year-3 IRR (below 20% threshold), Mira's first thought was "Reg will say no." Her second thought was "but the strategic case might justify it." She was right on both counts.
- At 06:42 Monday May 18 the ML scenario inference, when the probability-weighted 5-year IRR came back at 23% (above threshold) Mira felt the deal becoming more real. She started thinking about how to explain Monte-Carlo to Kenji Park-Holloway who, she knew, was skeptical of ML inputs to M&A decisions.
- At 10:32 Monday May 18 Reginald's sign-off, he said "I want this on the record: I'm comfortable with the model + the threshold. The strategic case is separate. That's your case to make, Mira." Mira appreciated the clarity.
- At 09:14 CET Wednesday May 20 Bjorn's reply about staying in Berlin, Mira had been hoping he would say that — Berlin-based Bjorn means MRT's culture stays continuous + Skylark gets a Berlin office it has wanted for 18 months but couldn't justify on its own. She didn't mention this to anyone in the readout.
- At 09:00 Monday May 25 the board meeting, when Patricia Wells-Okonkwo cast the no vote, her stated reason was deal-size-relative-to-Skylark-ARR (~125% of TTM ARR). Privately Mira thought the real reason was Patricia's general M&A skepticism after a bad deal she had been part of in 2019. Patricia is a thoughtful director; the no vote was substantive.
- At 12:18 EDT Monday at the Liberty Hotel café, Mira thought about her mother in Tel Aviv. Her mother is 71 and watches Skylark closely because Mira's professional success matters to her. Mira had not told her about MRT (deal was confidential). She would call her tonight.

## §12 — Stop condition for this story

This story documents the 9-day arc from May 15 07:42 EDT through May 25 11:18 EDT — Mira's full work-cycle to assemble, analyze, counsel-review, committee-endorse, board-decide, and record the MRT acquisition go/no-go. The substrate held throughout: the NDA-bound cross-tenant channel preserved boundary integrity, the ML scenarios were reproducible + provenance-preserved, the pack-cross-check identified the SOC 2 Type 1→2 remediation cleanly, the Cedar gate at the board vote was Cedar-validated for each of 9 board members, the super-Merkle root + external anchor made the decision provenance verifiable, and the regional data residency invariant held (MRT's data stayed in EU; only hashes + diligence-permitted document subsets crossed). The story exists so the next reader understands WHY the M&A workflow needs cross-tenant NDA channels with Cedar permits at the document-class level, WHY ML scenario modeling has to expose probability-weighted IRR rather than a single point estimate, and WHY the pack-manifest cross-check is computed automatically as a blocker-detection surface rather than as a manual due-diligence checklist.
