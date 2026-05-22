---
doc_class: User-Journey-Story
journey_id: j159-saanvi-mehta-mba-application-spans-personal-and-work
date: 2026-05-20
authority_tier: 2
status: draft
---

# j159 — Story: 21:47 IST in Indiranagar, the cursor blinks on essay 1 of 5

## §0 — Sunday December 6, 2026, 21:47 IST — Indiranagar 3rd Cross, Bangalore

The 3BHK rental on Indiranagar 3rd Cross is quiet. Anaya fell asleep at 21:18 after the third reading of *The Gruffalo*. Arjun is in the kitchen making chai from her mother-in-law's recipe (cardamom + clove + a single black peppercorn — the Maharashtrian way, not the Bombay-cafe sweet way). The monsoon-conditioned air is dry-cool — Bangalore's December evenings settle into the high teens Celsius after the early-morning fog burns off. Through the open balcony door, traffic on 100 Feet Road is light; the Konditori bakery across the street has shuttered its grill but the LED sign still glows. Somewhere two flats down, a neighbor's dog barks twice and stops.

Saanvi Mehta sits cross-legged on the living-room rug with her iPad Pro M4 (12.9" cellular, Apple Pencil 2 next to it), wearing a navy kurta and tracksuit pants, hair in a loose bun, a mug of green tea on the floor next to her. The iPad is signed into her personal tenant — `saanvi.mehta.personal` — and ONLY that tenant. The tenant chip at the top of the screen reads:

> **🏠 saanvi.mehta.personal · personal · 1 tenant active**

She has Wharton's Round 2 application portal open in a Safari tab and her oyatie `notes` µservice open in the foreground. The notes document is `essay-wharton-r2-2027-why-mba-why-wharton-why-now-v9` — the ninth full draft. The current word count chip in the corner reads **647 / 650**. The cursor blinks at the end of the second-to-last sentence.

Her current paragraph reads:

> When I joined Stripe Connect APAC in 2022, I inherited a platform that had grown product-led without an explicit thesis for the Indonesia and the Philippines markets. Over the following two years I co-authored the Asia-Pacific go-to-market rewrite that turned Connect from a flat fee schedule into a region-specific elasticity model — and watched our Indonesia GMV grow from $14m to $112m annualized. I did not, at any point during that work, feel that I had reached the ceiling of what I could learn at my desk. What I did feel was that the next ten years of my career, if I want them to be the ten years I built something rather than improved something, need a different operating system.

She reads it once. Then again. She nods once. She types the last sentence:

> Wharton's Asia Lauder track, the Mack Institute's translational research model, and the proximity to the Penn Center for Innovation are not the only reason I am applying — they are the reason I am applying *now*, this year, before the operating system I want to build runs out of patience.

Word count: **650 / 650**. Final.

She taps the **Auto-save · sealed · ✓** indicator. The notes µservice flushes the draft to her personal-tenant `drive` at path `/saanvi/mba-2027/essays/wharton/essay-1-why-mba-why-wharton-why-now-final.docx`. The audit-chain seals `EVT-J159-ESSAY-FINALIZED-001` at 21:47:14 IST.

She walks to the kitchen. Arjun has poured the chai into the dark-green ceramic mugs his grandmother gave them at their wedding. He looks up.

**Arjun 21:48 IST** (Marathi, the language they speak at home): *"Aata kiti urli aahe — Wharton submit?"* (Hindi/English gloss: "How much is left — Wharton submit?")

**Saanvi 21:48 IST**: *"Essay finalized aahe. Aata recommender aani fee. Recommender ne accept kelay का nahi te tar tya udhar baghu. Fee Friday la pay karen."* ("Essay is final. Now recommender and fee. Whether the recommender has accepted or not we'll see in the morning. Fee I'll pay Friday.")

**Arjun**: *"Coffee नका — green tea?"* ("Not coffee — green tea?")

**Saanvi**: *"Green tea kele aahe. Tum chai ghya."* ("I've made green tea. You have the chai.")

He hands her the mug. They sit at the dining table. She opens the essay one more time on the iPad and reads it aloud to him in English — he is the third pair of eyes after her IIM cohort writing group and the McKinsey-alum essay-coach she's been working with since September.

**Arjun 21:54 IST** (English, his work-register for this kind of review): "The last sentence runs long. 'Operating system runs out of patience' — strong. Keep it. The middle paragraph about Indonesia — make it 'Indonesia GMV' the first time, not 'our Indonesia GMV'. We don't possess it; Stripe possesses it."

She makes the edit. 649 / 650 now. She re-saves. `EVT-J159-ESSAY-FINALIZED-001` advances epoch.

**Arjun 21:56 IST**: *"Aata jhop. Salu hota udya nighto. Tu bhi udhya kaam aahe."* ("Now sleep. Long day tomorrow. You also have work tomorrow.")

She nods. Wharton can wait until Tuesday for the cover-page upload + the optional disclosure. The essay is done.

## §1 — Sunday Dec 6 22:18 IST — spousal review handshake

Before she sleeps she does one more thing: she grants Arjun read-only access to the essay folder for second-pair-of-eyes review tomorrow during his lunch break. The spousal cross-tenant capability is a built-in pattern in oyatie's `tenancy` µservice — both Saanvi and Arjun confirmed their relationship at oyatie onboarding (joint-marriage-attestation in October 2026), which gives them a one-flag toggle for "share specific folder, read-only, no propagation" without going through the generic share-link mechanism.

She taps **Share with spouse · read-only · this folder** on the `/saanvi/mba-2027/essays/wharton/` directory. The modal asks for one explicit confirmation:

> **Grant read-only access to:** Arjun Mehta (`arjun.mehta.personal`)
> **Scope:** `/saanvi/mba-2027/essays/wharton/`
> **Capability:** read-only, no download, no resharing, no propagation
> **Expires:** 2026-12-22 23:59 IST (after HBS deadline)
> **Audit:** dual-seal in both spousal tenants

She taps Grant. The audit-chain seals `EVT-J159-SPOUSAL-REVIEW-012` dual-sealed in `saanvi.mehta.personal` AND `arjun.mehta.personal` at 22:18:42 IST. Arjun gets a soft push notification on his iPhone (he uses his personal tenant for personal things; his work tenant is `manipal-hospitals-pvt-ltd-bangalore` and he keeps it strictly separate).

She closes the iPad. She has set her alarm for 06:30 — Anaya wakes at 06:45 on weekdays and gets to her preschool at Stonehill International by 08:30. Saanvi's Stripe standup is at 09:30 IST (it's 09:00 PT in San Francisco). Tomorrow Priya will get the Wharton recommender invitation in her work-tenant mail.

## §2 — Monday Dec 7 09:34 IST — Priya gets the recommender invitation

Priya Krishnamurthy is 41, lives in Koramangala 5th Block, has been at Stripe for 4 years (joined from Razorpay in 2022 after a brief stint as a PM at Flipkart), holds an MBA from ISB Hyderabad (2014), and manages a team of 11 product managers across India, Singapore, and Sydney covering Stripe Connect APAC. She is currently in her home office on her Stripe-issued MacBook Pro 16" M4 Pro, signed into her work tenant `stripe-india-pvt-ltd`. The tenant chip at the top of her screen reads:

> **💼 priya.krishnamurthy@stripe-india-pvt-ltd · work · 1 tenant active**

At 09:34 IST her work-tenant `mail` µservice receives an email from `noreply@wharton-mba.upenn.edu` with subject **"Saanvi Mehta has invited you as a recommender — Wharton MBA Round 2 2027"** and a personalized link.

The link's payload is the recommender invitation. oyatie's `mail` µservice classifies the link as a `cross_tenant_capability_request` (recognized via Wharton's pre-published OIDC well-known recommender-invite signature) and renders an inline UI card instead of just letting Priya click the raw URL:

```
┌─ Recommender invitation ──────────────────────────┐
│  From:       Saanvi Mehta (saanvi.mehta.personal) │
│  Subject:    Wharton MBA Round 2 2027             │
│  Capability: write-once recommendation letter     │
│              to Saanvi's personal-tenant drive    │
│              at /saanvi/mba-2027/recommenders/    │
│                                                    │
│  Tenant context:                                  │
│  ⚠  You will be SIGNING this as your             │
│     work-tenant identity (priya.krishna…@stripe) │
│     because you are Saanvi's manager AT WORK.    │
│                                                    │
│  Cedar capability scope:                          │
│  ✓  Write the recommendation letter once         │
│  ✗  No browse of other Saanvi personal files     │
│  ✗  No share/forward/copy capability             │
│  ✗  Capability auto-revokes after Saanvi closes  │
│     her Round 2 cycle (2027-01-06)              │
│                                                    │
│  ┌──────────────┐  ┌────────────────────────┐    │
│  │  ✕ DECLINE   │  │  ✓ ACCEPT (work ident.)│    │
│  └──────────────┘  └────────────────────────┘    │
└────────────────────────────────────────────────────┘
```

Priya reads the card carefully. She has done this dance twice before for other reports — but oyatie's UX makes the boundary explicit in a way her old Gmail+DocuSign+random-PDF flow never did. She thinks for a moment about whether to do this as a personal tenant — she could; she has one (`priya.krishnamurthy.personal`). But the recommendation IS about Saanvi as her direct report; that is a work-context statement; it would be dishonest to sign as a personal contact when she has only ever been Saanvi's manager.

She taps **ACCEPT (work identity)**.

Cedar evaluates the cross-tenant capability grant in 64 ms:

- Principal: `priya.krishnamurthy@stripe-india-pvt-ltd`
- Action: `drive.write_recommendation_letter_to_slot`
- Resource: `RecommendationLetterSlot::"slot-saanvi-wharton-r2-2027-primary"`
- Context: `capability_grant.granted_by = saanvi.mehta.personal`, `capability_grant.scope = write_once_no_browse`, `target_tenant = saanvi.mehta.personal`

Permit. Audit `EVT-J159-RECOMMENDER-ACCEPT-002` dual-seals in BOTH `saanvi.mehta.personal` AND `stripe-india-pvt-ltd` at 09:34:42 IST.

Priya then opens her oyatie `notes` µservice on the same MacBook — but the document is auto-routed to the recommendation-letter slot in Saanvi's personal tenant, not Priya's work-tenant drive. The notes window shows a clear banner:

> **🔀 Cross-tenant slot — write-once recommendation letter for Saanvi Mehta**
> **Home tenant:** `saanvi.mehta.personal`
> **Your identity:** `priya.krishnamurthy@stripe-india-pvt-ltd` (work)
> **Capability:** WRITE-ONCE — no save-draft outside this slot; one final submit

Priya begins drafting. She types in English, professional register, ~1100 words covering Saanvi's Indonesia work, her thesis development on regional elasticity, a specific anecdote about Saanvi catching a payments routing bug in Manila that would have cost ~$2.4M in inadvertent FX losses, and an honest paragraph on Saanvi's growth areas (Priya notes that Saanvi sometimes over-indexes on consensus-building in early-stage debates). The letter is due to Wharton's portal by Dec 22 23:59 ET.

Priya saves the in-progress draft at 10:12 IST. The notes µservice auto-flushes to the recommendation slot — but the slot is **append-only** until final submit; Priya can revise but each revision is audit-chained. She closes the document and goes to her 10:30 IST standup.

## §3 — Monday Dec 7 14:18 IST — Rajesh accepts from Marico tenant

Rajesh Subramanian is 47, VP of Digital Transformation at Marico (the Mumbai-based FMCG firm — Parachute coconut oil, Saffola, etc.), was Saanvi's skip-manager at HUL from 2017–2019 when she worked on the Lakmé brand digital, and has remained in casual professional contact ever since. He works from Marico's Andheri East office. His tenant is `marico-india-pvt-ltd`.

At 14:18 IST his work-tenant mail receives the equivalent Wharton recommender-invite card. His Cedar evaluation runs the same path; he taps Accept (work identity); audit `EVT-J159-RECOMMENDER-MARICO-ACCEPT-003` dual-seals in `saanvi.mehta.personal` AND `marico-india-pvt-ltd` at 14:18:38 IST.

Rajesh's recommendation slot is the "supplementary" slot — Wharton accepts a primary + a supplementary, with the supplementary expected to cover a different observation window. Rajesh will draft his (covering Saanvi's HUL years 2017–2022) over the next 10 days.

## §4 — Wednesday Dec 9 14:18 IST — the HR audit sweep arrives

`hr-systems@stripe-corporate-us` is a synthetic principal that runs from Stripe's US tenant and walks all global Stripe work-tenant employee data access logs once per quarter. The Q4 2026 sweep is in flight; Saanvi's work-tenant access logs are walked at 14:18 IST Wednesday Dec 9. The sweep's purpose is anti-leak: detect work-tenant employees who are uploading work-confidential documents to non-work destinations or who are accessing work-confidential documents they should not.

The sweep walks `stripe-india-pvt-ltd`'s work-tenant `drive` for Saanvi's documents. It finds:

- 217 work-tenant Drive documents (roadmap docs, design reviews, Connect APAC strategy artifacts, OKR documents) — all expected for a senior product manager
- ZERO references to MBA applications, Wharton, Stanford, HBS, Booth, or INSEAD
- ZERO references to Saanvi's personal Drive paths
- ZERO references to her personal-tenant email
- ZERO references to her HDFC card

The sweep then attempts a probe: it issues a discovery query `discovery.walk_all_principal_artifacts(principal=saanvi.mehta)` — a broad "find anything this person touches" query.

Cedar evaluates:

- Principal: `hr-systems@stripe-corporate-us`
- Action: `discovery.walk_all_principal_artifacts`
- Resource scope: `*`
- Context: `principal.tenant_class = work_tenant`, `target_principal_dual_tenant = true`, `cross_tenant_capability_grant_present = false`

Forbid. ADR-0311 invariant: a work-tenant principal does NOT have ambient access to a personal-tenant resource, even when the personal-tenant principal is the same human as a work-tenant principal in another tenant. The capability grant for the recommendation letter is **scoped** to the specific slot Priya wrote into — it does NOT extend to a broader audit-sweep crawl.

Audit `EVT-J159-CEDAR-DENY-WORK-TENANT-INTO-PERSONAL-014a` seals in `stripe-corporate-us` (the source tenant gets the deny for its audit log) AND in `saanvi.mehta.personal` (Saanvi's personal-tenant log records that an external query was refused — important for her transparency).

The sweep's HR dashboard returns its standard "no anomalies" verdict for Saanvi. `EVT-J159-HR-SWEEP-NO-PERSONAL-LEAK-004` is generated as a positive-attestation event in the Stripe-corporate-US tenant: "Saanvi Mehta — Q4 2026 sweep: all 217 work-tenant artifacts compliant; no personal-tenant leakage detected; no anomalies."

Saanvi never learns this happened. The point of ADR-0311 is that she SHOULDN'T have to learn this happened — the boundary holds without her vigilance.

## §5 — Friday Dec 11 22:14 IST — Wharton submit + HDFC card

Friday evening. Anaya is asleep. Arjun is on call with his hospital admin counterpart. Saanvi has finalized her Wharton optional disclosure (a 3-paragraph note about a 4-month personal leave in 2023 to be primary caregiver after Anaya's premature birth; she does not feel obligated to disclose this but she wants Wharton to know because it shaped her thesis on flexible-work product design). The cover page is uploaded. Both recommenders have submitted (Priya at 18:42 IST yesterday; Rajesh at 11:30 IST today). The GMAT 745 score has been sent. Her IIT Bombay and IIM Calcutta transcripts (uploaded as official PDFs via the WES-equivalent IND-eval service) have arrived in Wharton's system.

She is on the **Pay & Submit** screen.

```
┌─ Wharton MBA — Round 2 — Application Fee Payment ──┐
│  Amount: USD 275.00                                │
│                                                     │
│  Choose payment method:                            │
│                                                     │
│  ◉ HDFC Bank Personal Credit Card · MILLENNIA       │
│    last 4 digits 7314 · INR 23,650 equivalent      │
│    provider-credential BYOK · personal credential   │
│                                                     │
│  ◯ Add new method                                   │
│                                                     │
│  ⚠ Stripe corporate Amex (last 4: 4119)           │
│    is NOT eligible for personal-tenant payments.   │
│    Personal-vs-work boundary forbids this routing. │
│                                                     │
│  Tenant: saanvi.mehta.personal                     │
│  Cell:   ap-mumbai-primary                         │
│                                                     │
│  ┌───────────────────────────────────────────────┐ │
│  │   💳 PAY USD 275.00 · MILLENNIA 7314          │ │
│  └───────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

She taps. Payments µservice routes the charge via her personal-tenant provider-credential BYOK credential (her HDFC Bank Millennia card, registered to her personal-tenant under ADR-0255 §D-4). Authorization: Yes. The corporate Amex is grayed out and a tooltip explains why: `corporate_card_not_eligible_for_personal_tenant_payment` — this is a deliberate Cedar `forbid` to prevent the most common dual-tenant mistake (paying a personal expense on the corporate card by accident).

HDFC's 3D-Secure SMS OTP arrives at her registered mobile in 4 seconds. She types `184729`. Authorization clears at 22:14:18 IST. Settlement is T+1.

`EVT-J159-WHARTON-FEE-PAID-005` seals at 22:14:22 IST. The Wharton application portal accepts the fee, displays the **APPLICATION SUBMITTED** confirmation, and emails Saanvi's personal-tenant mail at 22:14:34 IST. Within 90 seconds Wharton's tenant `wharton-mba-admissions-us` issues a cross-tenant ack to `saanvi.mehta.personal`: `EVT-J159-WHARTON-ACK-006` dual-sealed.

Saanvi exhales. She texts her IIM Calcutta cohort group chat (on her personal phone, oyatie messenger, personal-tenant): "Wharton R2 submitted. 4 to go. Stanford by Monday. HBS by Tuesday. Booth by Wednesday. INSEAD by next Friday."

The cohort replies in waves over the next 30 minutes. Three are also applying R2 this year (HBS + Stanford + Booth). They will share notes.

## §6 — Saturday Dec 12 19:30 IST — Manhattan Prep refresher

Saturday evening. Anaya is at her cousin's birthday party in Whitefield with Arjun. Saanvi has the apartment to herself for 90 minutes. She opens her `learning-management` µservice on the iPad and starts the Manhattan Prep GMAT-Focus Quant DI Advanced module.

The module is a 90-minute timed problem-set with 28 questions covering Data Insights at the Q90+ level — the exact band Saanvi scored at. The score-up insurance: if any school offers an interview where they want to discuss her quant capability or if she decides to retake the GMAT for a 755+ before a later round, she needs to be sharp.

She works through 28 questions in 84 minutes. She scores 26/28 (93% accuracy) — consistent with her 745 sitting. The `learning-management` µservice records the session, updates her competency profile in her personal-tenant, and adds the result to the GMAT-prep dashboard.

`EVT-J159-GMAT-PREP-LMS-SESSION-007a` seals at 21:02 IST.

## §7 — Sunday Dec 13 11:08 IST — community question

Sunday late morning. She opens the `wharton-r2-2027-prospective-applicants-community` tenant on her iPad. This is a third tenant — a private peer community founded September 2026 by 47 prospective Wharton R2 applicants who met through a Reverse-Coffee mixer + a GMAT-prep cohort. Membership is invite-only; oyatie's `community` µservice hosts it; the tenant has zero connection to her personal tenant beyond the membership link, and zero connection to her work tenant.

She posts a question:

> **Saanvi Mehta · 11:08 IST**
> Quick gut-check, folks. I have a 4-month gap in 2023 (caregiver leave after my daughter's premature birth). I chose to address it in the Wharton optional essay. Did anyone NOT disclose a similar gap and regret it? Or disclose and regret it? Trying to calibrate for Stanford + HBS where the optional essay framing is different.

Within 4 hours, 11 community members reply. Aggregated view (paraphrased from real anonymous threads of this kind):

- 6 replies say "Disclose. The gap is a strength when framed as a thesis-shaping experience, which is what your essay does."
- 3 replies say "Don't disclose if it's not asked — keep the optional essay for diversity context."
- 2 replies share their own similar gaps (one for elder-care, one for a startup that failed)

The thread is end-to-end MLS encrypted within the community tenant. Saanvi's work-tenant has zero visibility into this conversation. Saanvi's personal-tenant has visibility (because she's a member of the community as her personal identity) but the message bodies are stored in the community tenant, not copied into her personal-tenant drive.

`EVT-J159-COMMUNITY-PARTICIPATION-008` seals at 11:08:42 IST.

She decides to disclose for all 5 schools, keeping the framing consistent.

## §8 — Monday Dec 14 09:00–18:00 IST — calibration day at Stripe

Monday is the Stripe Connect APAC calibration meeting. The Bangalore office is full (Stripe runs hybrid 3 days a week; today is one). Saanvi arrives at 08:42 IST, badges in via her work-tenant identity (`saanvi.mehta@stripe-india-pvt-ltd` + passkey on her MacBook), walks to her 5th-floor desk, opens her work laptop, signs in. The tenant chip reads:

> **💼 saanvi.mehta@stripe-india-pvt-ltd · work · 1 tenant active**

She does NOT sign into her personal tenant on the work laptop. Ever. Not even via the browser. That is the discipline.

Her work-day is normal: 09:30 standup, 10:00 Connect Pricing Workshop, 12:00 lunch with two product designers, 13:00 a 1:1 with her own report Mohammed Akram, 14:00 a Connect APAC deep-dive design review where she chairs, 15:30 the calibration prep, 16:00 the calibration meeting itself.

In the calibration meeting Priya advocates for an "Exceeds" rating for Saanvi. Priya's case is anchored on the Indonesia GMV trajectory + Saanvi's Manila bug catch + her cross-team work with the Sydney engineering team. Two peer managers cross-question on whether Saanvi's "consensus-building" style is sometimes a velocity drag; Priya agrees there is a development edge but argues the net is strongly positive. The calibration concludes at 17:42 IST with Saanvi's rating set to **Exceeds**.

Saanvi never learns the specifics of Priya's advocacy until the formal review readback the following Tuesday — but Priya is wholly within her work-tenant role here. Saanvi's personal tenant has ZERO activity log entries between 09:00 and 18:00 IST on Dec 14. The boundary holds.

`EVT-J159-CALIBRATION-DAY-CLEAN-BOUNDARY-009` is generated automatically at 18:00 IST as an attestation: "Personal-tenant `saanvi.mehta.personal` had 0 activity events during 09:00–18:00 IST 2026-12-14 (Stripe calibration day)."

## §9 — Tuesday Dec 15 19:18 IST — Booth + INSEAD submit

Tuesday evening. Anaya is doing homework on the dining table with Arjun. Saanvi submits Chicago Booth at 19:18 IST (after the same fee-payment dance via HDFC card; same cross-tenant ack pattern). She submits INSEAD Singapore-Fontainebleau at 20:42 IST (INSEAD's portal accepts a separate $250 fee; same routing).

`EVT-J159-BOOTH-SUBMIT-006a` and `EVT-J159-INSEAD-SUBMIT-006b` dual-seal at their respective timestamps.

Three of five submitted. Stanford on Friday Dec 18, HBS on Tuesday Dec 22.

She closes the iPad at 21:18 IST and joins Anaya at the table. Anaya is solving a 2nd-grade math problem about elephants in a zoo. Saanvi helps.

## §10 — Tuesday Dec 22 22:48 IST — all 5 submitted

The journey closes at 22:48 IST Tuesday December 22, 2026 — six days before her 35th birthday (Dec 28). HBS R2 submitted. All five schools are now in `submitted` state in her personal-tenant MBA-application tracker.

`EVT-J159-ALL-SCHOOLS-SUBMITTED-010` seals in `saanvi.mehta.personal` at 22:48:18 IST.

She texts Arjun (he's in the kitchen): *"Sagle 5 submit zhale. आता decision-wait."* ("All 5 submitted. Now decision-wait.")

He comes out, hugs her, says nothing for a long minute.

## §11 — Beats not on the wire (the human texture)

- At 21:46 Sunday Dec 6, just before Saanvi finalized the essay, Anaya's preschool teacher Ms. Lavanya emailed (work tenant for the preschool, but Saanvi's personal mail receives it) about a Christmas potluck on Dec 19. Saanvi's personal-mail flagged it inline. She marked it "respond Tuesday" — never opened it on her work laptop. The boundary held even for this minor parental obligation.
- Priya's recommendation letter for Saanvi is the third she has written for direct reports. The first (2024 for a different report applying to HBS) used Gmail + a PDF attachment. The second (2025 for another report applying to Stanford) used a school-provided portal. This one (2026 for Saanvi, on oyatie) is the first time Priya felt the system meaningfully prevented her from accidentally leaking the existence of the recommendation to other Stripe colleagues. The Cedar-gated cross-tenant capability changes the cultural posture: Priya never had to think about whether to use Stripe Slack to ask a colleague to proofread the letter (she WAS NOT ABLE TO), so she didn't ask.
- Anaya's premature birth in 2023 (born at 32 weeks, 1.8 kg) is the most personal data point in this journey. Saanvi's disclosure of the 4-month leave is honest but does not name Anaya's medical specifics. The optional essay describes it as "a personal caregiver responsibility" without identifying the family member. This is a deliberate choice — protect Anaya's privacy even when telling Saanvi's own story.
- The HDFC Millennia card Saanvi uses for the application fees is in her personal name only (Arjun has his own card on a separate HDFC account; they consolidate later for tax purposes). The $1,150 + $250 = $1,400 USD total (approx INR 1,17,600) is a meaningful expense, but it is unambiguously a personal expense, paid from personal funds, on a personal credit card, in a personal tenant.
- Stripe's calibration meeting on Dec 14 had 31 PMs reviewed. Saanvi was the only one in the meeting who had a graduate school application out — but the calibration committee did not know this, and Priya did not raise it. Priya's advocacy for the Exceeds rating was entirely on the merits of Saanvi's 2026 work. This is exactly the ADR-0311 doctrine working as designed: career-defining personal moves do not influence (and are not influenced by) work-tenant performance assessment.

## §12 — Stop condition for this story

This story documents the lived texture of the 16-day journey from Wharton essay finalization through 5th-school submit. The acceptance criteria in `README.md`, the API shapes in `handshake.md`, the test cases in `integration-test-plan.md`, and the schema files together encode the machine semantics. The story exists so that the next human or agent reading the codepath can understand WHY the personal-vs-work boundary in ADR-0311 matters in lived practice, WHY the cross-tenant recommender capability is scoped write-once-no-browse and not a generic share, WHY the HDFC card vs Stripe Amex routing matters as a Cedar invariant, and WHY a Stripe HR audit walking a work tenant's drive must be cleanly refused at the personal-tenant boundary even when the same human is the principal of both tenants.
