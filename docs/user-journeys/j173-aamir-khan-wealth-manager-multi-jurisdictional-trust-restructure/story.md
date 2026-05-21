---
doc_class: User-Journey-Story
journey_id: j173-aamir-khan-wealth-manager-multi-jurisdictional-trust-restructure
date: 2026-05-20
authority_tier: 2
status: draft
---

# j173 — Story: Aamir Khan opens the Al-Maktoum-Hartington-Tan trust restructure at 06:42 GST Monday May 10

## §0 — Monday May 10, 2027, 06:42 GST — DIFC Gate Building, 38th floor

Dubai is dry and warm at 06:42. 31°C and rising. The southwest wind from the Hatta side. Aamir Khan arrives at the DIFC Gate Building at 06:42:14 having driven in from his villa in Emirates Hills in his 2024 Lexus LX 600. He drinks Tahini-Cardamom coffee (he likes it strong from the Emirati Trader Joe's; a small pot brewed at home + transported in a thermal flask). He prays Fajr in the building's small prayer room at 06:48, then to his office.

His office faces south. View across SZR to the Burj. He sets the AC to 22°C. He turns on three monitors. The right monitor is the **wealth-management cockpit** (the Multi-Family Office workspace). The left monitor is for email + privileged comms. The middle monitor is for documents — currently displaying the 6-document binder for the Al-Maktoum-Hartington-Tan restructure.

He authenticates: passkey + YubiKey + DFSA SMCR attestation token + STEP TEP attestation token. The active-tenant pill reads `halberd-mercer-private-wealth-difc · senior_director_multi_family_office · STEP_TEP_qualified`.

```
[WEALTH MANAGEMENT COCKPIT] Multi-Family Office · Aamir Khan
─
active engagement:        client-family-al-maktoum-hartington-tan
state:                    counsel_cross_review_round_3_pending (day 6 of 11)
restructure_target:       consolidation + bilateral trust + Cayman STAR-trust
aum_in_scope:             $340.4M (liquid $292.8M + real estate/PE $47.6M)
documents_in_clm:         6 (drafting → review → execution lifecycle)
counsel_firms:            3 (Mishcon de Reya · Maples Group · Allen & Gledhill)
family_principals:        7 (4 signing parties: Saira + Khalid + Aisha + Yusuf)
consolidation_amount:     $42.0M (UK + SG + KY → DIFC)
settlement_target:        2027-05-21 Friday
sanctions_screening:      armed (4 lists + UAE)
aml_screening:            armed (AMLD5 + UK MLR 2017 + UAE FDL 20/2018)
crs_reissuance:           pending 2 new entities
fatca_form_reissuance:    pending 2 new entities
cgt_clearance:            pending HMRC (Mishcon driving)
dtaa_optimization:        UK-UAE Art 13 applied · UK-SG Art 7 applied · UK-KY no-treaty documented
```

`EVT-J173-COCKPIT-OPENED-Δ000` sealed at 06:42:48 GST.

## §1 — May 10 07:00–08:30 GST: counsel cross-review round 3 incoming

The CLM workspace shows 6 documents in progress. Each document has its own state machine. As of 06:42 GST, status:

```
[CLM DOCUMENT STATE] 2027-05-10 06:42 GST
─
doc-1  uk-pilot-trust-2019-deed-of-dissolution.docx
       state: counsel_cross_review_round_3_PENDING
       owner: mishcon-de-reya (Sir William Pemberton-Brodsky)
       last_redline: 2027-05-09 18:42 BST (Allen & Gledhill perspective; Sg-tax-comment)

doc-2  uk-side-new-bilateral-trust-2027-settlement-deed.docx
       state: counsel_cross_review_round_3_PENDING
       owner: mishcon-de-reya (Sir William)
       last_redline: 2027-05-09 19:18 BST

doc-3  uae-side-new-bilateral-trust-2027-difc-settlement-deed.docx
       state: counsel_cross_review_round_3_PENDING
       owner: halberd-mercer-private-wealth-difc (Aamir + outside DIFC counsel — uses local DIFC counsel Bin Suwaidan & Co)
       last_redline: 2027-05-09 23:48 GST

doc-4  singapore-trust-2021-variation-deed-widen-beneficiary-class.docx
       state: counsel_cross_review_round_3_PENDING
       owner: allen-gledhill (Mrs. Mei-Ling Tan-Whitford)
       last_redline: 2027-05-10 03:14 SGT (overnight)

doc-5  cayman-spv-2023-novation-to-star-trust-2027.docx
       state: counsel_cross_review_round_3_PENDING
       owner: maples-group (Conrad Hartman-Whyte)
       last_redline: 2027-05-09 22:48 EST (overnight)

doc-6  family-consolidation-transfer-mandate-deed.docx
       state: counsel_cross_review_round_3_PENDING
       owner: halberd-mercer-private-wealth-difc (Aamir)
       last_redline: 2027-05-09 16:18 GST
```

The CLM workspace shows 24 unresolved redlines + 8 cross-counsel comments. Aamir starts with **doc-1** (the UK pilot trust dissolution deed).

Mei-Ling's Sg-tax-comment from yesterday reads:

> "Para 14.3 — the dissolution mechanism should explicitly disapply UK ITA 2007 s.685(2)(b) re: beneficiary residence test, OR explicitly recite that no UK-resident beneficiary will receive a distribution. The current drafting is ambiguous as to Sg-side residency for the beneficiary class. Suggest clarifying."

Aamir reviews. He agrees. He pings Sir William via the STEP-privileged advisory channel (a messenger µservice channel restricted to Aamir + Sir William + Conrad + Mei-Ling):

> "Will — Mei-Ling's para 14.3 comment is right. The current drafting could be read either way. I lean toward explicit recital that no UK-resident beneficiary receives a distribution — cleaner under ITA 2007 s.685(2)(b) and avoids the Sg-residency ambiguity. Can you redraft this morning? Settlement target Friday is tight."

Sir William replies 18 minutes later (he's at his Hampstead home; it's 03:18 BST):

> "Aamir, agree. I'll have Eleanor redraft and circulate by 09:30 BST. Sending to all three firms via the CLM workflow. — W"

`EVT-J173-CLM-DOC-1-COMMENT-Δ001a` sealed at 07:14 GST.

## §2 — May 10 08:30–09:18 GST: pack manifest activation + intelligence tax-scenario

Aamir activates the pack manifest for the restructure:

```
[PACK MANIFEST ACTIVATION]
─
engagement_id:            client-family-al-maktoum-hartington-tan-restructure-2027
active_packs:             8
  · pack-oecd-crs-2025
  · pack-fatca-form-w8-ben-e-v3
  · pack-mifid-ii-suitability-2027
  · pack-difc-trust-law-5-2018
  · pack-uk-trustee-act-2000
  · pack-sg-trust-companies-act-2005
  · pack-cayman-trust-law-star
  · pack-aml-5-cross-jurisdiction
cross_validation_state:   passed
pack_manifest_signature:  sha256:f7e2…3c19
asserted_at:              2027-05-10T08:30:48+04:00
```

`EVT-J173-PACK-MANIFEST-008` (preliminary; finalised at end-of-engagement) sealed at 08:30 GST.

Then he runs the **intelligence µservice tax-scenario** for CGT clearance + DTAA optimization:

```
[TAX SCENARIO ML — CGT + DTAA] 2027-05-10 08:48 GST
─
inputs:
  uk_pilot_trust_2019_market_value_baseline:       £128.4M (settled value)
  uk_pilot_trust_2027_disposal_value:                £148.2M
  unrealized_gain_chargeable:                         £19.8M
  uk_trustee_cgt_rate_2027:                            20%
  computed_uk_cgt_provisional:                         £3.96M (gross)
  ddt_holdover_relief_applicable_TCGA_1992_s260:      true (gift to discretionary trust)
  effective_uk_cgt_after_holdover:                     £2.4M (provisional)
  hmrc_clearance_required_TCGA_1992_s225:              true (variation)

  uk_uae_dtaa_2016_article_13_capital_gains_path:
    treaty_relief_applicable:                          true (UK→UAE for non-UK-real-estate)
    UAE_side_taxation:                                 0% (no income tax)
    net_effect:                                        UK cgt only after holdover (£2.4M)

  uk_sg_dtaa_1997_article_7_business_profits_path:
    sg_trust_2021_amendment_widen_beneficiary_class
    sg_estate_duty_path:                               0% (no estate duty since 2008)
    sg_income_tax_on_trustee:                          17% on chargeable income (de minimis FY2027)
    net_effect:                                        Sg-side minimal

  uk_ky_no_treaty_path:
    cayman_star_trust_novation
    UK_side_chargeable_transfer_test:                  NOT triggered (no UK-domiciled settlor capacity-add)
    cayman_side_no_tax_no_duty
    net_effect:                                        clean

confidence_score:                                       0.92 (ml_provenance: tax-scenario-v4-uk-uae-sg-ky-2027-05)
recommendations:
  - apply TCGA 1992 s260 holdover relief (saves £1.56M vs. no-holdover)
  - structure cayman novation to avoid UK domicile capacity-add
  - schedule HMRC clearance application via Mishcon (Sir William)
  - reissue CRS for 2 new entities + FATCA Form W-8BEN-E for 2 new entities
  - schedule provisional UK CGT payment £2.4M for Q4 2027 (HMRC due 31 Jan 2028 with self-assessment)
```

`EVT-J173-TAX-SCENARIO-COMPUTED-Δ001b` sealed at 08:48 GST.

Aamir shares this with Sir William via STEP-privileged channel. Sir William reviews + concurs at 06:18 BST: "Agree on holdover; agree on s260 election. I'll draft the HMRC clearance application this week."

## §3 — May 10 09:30–12:42 GST: counsel cross-review round 3 incoming + reconciliation

Through the morning, redlines circulate:

- 09:48 BST — Mishcon (Eleanor) circulates revised doc-1 with the para 14.3 clarification
- 10:14 BST — Mei-Ling reviews + approves the clarification (Sg-side OK)
- 10:42 BST — Conrad reviews + flags a Cayman-side cross-check (the Cayman STAR-trust novation must preserve the SPV's tax-residency in Cayman without UK domicile capacity-add; he requests doc-3 (UAE-side) be cross-referenced)
- 11:18 BST — Sir William cross-references doc-3 ↔ doc-5 and reconciles the cross-border domicile-capacity test
- 12:42 BST — doc-1 + doc-3 + doc-5 all marked counsel-cross-review-complete

`EVT-J173-COUNSEL-CROSS-REVIEW-Δ002a` sealed per document throughout.

Doc-2 + doc-4 + doc-6 still pending; will resolve over Tuesday-Wednesday.

## §4 — May 11–13 (Tuesday-Thursday): counsel cross-review continues + CRS/FATCA reissuance

**Tuesday May 11 09:14 GST — Sir William sends CGT clearance application to HMRC.** HMRC's Customer Compliance Manager (Mr. Daniel Carmichael-Holt, CCM-North-Region) acknowledges receipt; advance clearance under TCGA 1992 s225 typically takes 30 business days but Mishcon has requested expedited under the "complex multi-jurisdictional restructure" provision. Mr. Carmichael-Holt commits to a 6-business-day turnaround.

**Tuesday May 11 14:00 GST — CRS reissuance for 2 new entities.** Aamir submits CRS reporting for `new-bilateral-trust-uk-2027` + `new-bilateral-trust-uae-2027-difc` via the compliance µservice:

```
[CRS REISSUANCE] 2027-05-11 14:00 GST
─
new_entity_1:
  legal_name:               Al-Maktoum-Hartington-Tan UK Bilateral Trust 2027
  jurisdiction_of_trust:    United Kingdom (settled in UK; trustee Halberd-Mercer Trustees UK Ltd)
  crs_status:               Investment Entity managed by FI (CRS Reporting Financial Institution)
  reportable_jurisdictions: UAE (residence of Saira); UK (residence of Khalid + Aisha + Yusuf + grandchildren)
  giin:                     UK-HMT-FI-2027-Δ48210 (pending issuance; expected 2027-05-25)

new_entity_2:
  legal_name:               Al-Maktoum-Hartington-Tan UAE Bilateral Trust 2027 (DIFC)
  jurisdiction_of_trust:    UAE / DIFC (settled in DIFC; trustee Halberd-Mercer Trustees DIFC Ltd)
  crs_status:               Investment Entity managed by FI (CRS Reporting Financial Institution)
  reportable_jurisdictions: UK (UK-domiciled beneficiaries) + UAE
  giin:                     UAE-DFSA-FI-2027-Δ48211 (pending issuance)
```

`EVT-J173-CRS-REISSUED-Δ004a` sealed at 14:00 GST.

**Tuesday May 11 14:30 GST — FATCA Form W-8BEN-E reissuance for same 2 entities.** No US-person beneficial owner identified; both entities classify as "Non-Reporting IGA FFI - Trustee-Documented Trust" status under Annex II of the UK + UAE IGAs respectively.

`EVT-J173-FATCA-FORM-REISSUED-Δ004b` sealed at 14:30 GST.

**Wednesday May 12 — counsel cross-review on doc-2 (UK-side bilateral trust) completes.** 4 redlines accepted + 1 deferred to family.

**Thursday May 13 — counsel cross-review on doc-4 (Singapore variation) + doc-6 (consolidation mandate) completes.** All 6 documents now in `final_counsel_review` state.

**Thursday May 13 16:42 BST — HMRC CGT clearance arrives via Mr. Carmichael-Holt.** Clearance granted under TCGA 1992 s225 with conditions:
- holdover relief under s260 confirmed applicable
- provisional CGT £2.4M payable with FY2027-2028 self-assessment by 31 January 2028
- HMRC reserves right to reopen if material facts change

`EVT-J173-CGT-CLEARANCE-006` sealed at 16:42 BST (20:42 GST).

## §5 — May 14 (Friday): family principal review begins

The CLM workflow opens family-principal review on all 6 documents simultaneously. Each family principal receives a notification:

```
[FAMILY PRINCIPAL REVIEW REQUEST]
─
to:                           saira.al-maktoum-hartington@personal-tenant (Dubai)
                              khalid.al-maktoum-hartington@personal-tenant (London)
                              aisha.al-maktoum-hartington@personal-tenant (Edinburgh)
                              yusuf.al-maktoum-hartington@personal-tenant (Geneva)
from:                         aamir.khan@halberd-mercer-private-wealth-difc
documents:                    6 (listed above)
explanatory_memo:             explanatory-memo-trust-restructure-2027.pdf (28 pages; en-UK + ar)
counsel_opinions:             3 (one per firm; en-UK + ar)
cgt_clearance_letter:         hmrc-cgt-clearance-2027-05-13.pdf
ml_tax_scenario_summary:      tax-scenario-summary-aamir-2027-05-10.pdf
review_window:                72 hours (deadline 2027-05-17 16:00 GST)
signing_method:               passkey + KYC attestation per jurisdiction
suitability_acknowledgment:   MiFID II suitability acknowledgment for UK principals (Khalid + Aisha + Yusuf)
```

`EVT-J173-FAMILY-REVIEW-REQUESTED-Δ002b` sealed at 14:00 GST May 14.

Family principals review through Saturday + Sunday. No major objections. Saira asks for one clarification (the holdover relief mechanic — Aamir + Sir William jointly respond via her privileged channel).

## §6 — Sunday May 16 18:18 GST: family principal signing wave begins

Saira signs first (she's in Dubai; convenient time). She authenticates: passkey + UAE KYC attestation (Emirates ID + passport scan match + face attestation). She signs doc-1, doc-2, doc-3, doc-4, doc-5, doc-6 in sequence. The signing UI shows her the document text in Arabic + English side-by-side; her signature is recorded with timestamp + UTC + her HLC-tagged signature event.

```
[FAMILY SIGNATURE] 2027-05-16 18:18 GST
─
principal:                    saira.al-maktoum-hartington@personal-tenant (Dubai)
documents_signed:             6 (all)
kyc_attestation:              UAE-emirates-id-+-passport-+-face (passed)
signature_method:             passkey + face attestation
hlc_signature_timestamp:      hlc:2027-05-16T14:18:08Z:Δ001
mifid_ii_suitability:         N/A (Saira is UAE resident; SMCR exempted)
language_of_acknowledgment:   ar + en-UK (dual)
```

`EVT-J173-FAMILY-SIGN-003-saira` sealed at 18:18 GST.

Khalid signs from London Monday morning. Aisha from Edinburgh Monday afternoon. Yusuf from Geneva Monday evening.

```
[FAMILY SIGNATURE WAVE COMPLETE] 2027-05-17 21:48 CET
─
saira:    signed 2027-05-16T14:18:08Z
khalid:   signed 2027-05-17T09:42:18Z (London; MiFID II suitability acknowledgment Y)
aisha:    signed 2027-05-17T14:18:18Z (Edinburgh; MiFID II suitability acknowledgment Y)
yusuf:    signed 2027-05-17T19:42:18Z (Geneva; MiFID II suitability acknowledgment Y - dual UK domicile)
all_signed:  true
```

`EVT-J173-FAMILY-SIGN-003` (composite) sealed at 21:48 CET on May 17.

## §7 — Monday-Tuesday May 17-18: $42M consolidation transfer

With all signatures in, Aamir initiates the $42M consolidation transfer.

```
[CONSOLIDATION TRANSFER] 2027-05-17 22:00 GST
─
total_transfer_amount_usd:           $42,000,000
transfer_legs:                        3
  leg_1_uk_to_difc:
    source_bank:                      Coutts (UK; A/C 18234820 GBP)
    source_amount_gbp:                £14,400,000 (≈ $18,200,000 at FX 1.264 USDGBP)
    destination_bank:                 Mashreq Bank (UAE; DIFC; A/C 4820012)
    destination_amount_usd:           $18,200,000
    swift_method:                     MT103 + cover MT202
    correspondent_bank:               HSBC London (UK) → HSBC HQ Dubai (UAE)
    estimated_arrival:                T+0 EOB (UK 17:00 → DIFC 21:00 GST)

  leg_2_sg_to_difc:
    source_bank:                      DBS (Singapore; A/C 0144822 SGD)
    source_amount_sgd:                S$20,000,000 (≈ $14,800,000 at FX 1.351 SGDUSD)
    destination_bank:                 Mashreq Bank (UAE; DIFC)
    destination_amount_usd:           $14,800,000
    swift_method:                     MT103 + cover MT202
    correspondent_bank:               JPMorgan New York → JPMorgan Dubai
    estimated_arrival:                T+0 EOB

  leg_3_ky_to_difc:
    source_bank:                      Butterfield (Cayman; A/C 71248 USD)
    source_amount_usd:                $9,000,000
    destination_bank:                 Mashreq Bank (UAE; DIFC)
    destination_amount_usd:           $9,000,000
    swift_method:                     MT103 + cover MT202
    correspondent_bank:               BNY Mellon → Mashreq
    estimated_arrival:                T+1 EOB
```

Sanctions screening kicks off first:

```
[SANCTIONS SCREENING] 2027-05-17 22:00 GST
─
lists_checked:
  - OFAC SDN List (US)
  - UK HMT Consolidated List
  - EU Consolidated List
  - UN Consolidated List
  - UAE Local Lists (FATF + UAE-specific)
  - Cayman CIMA Sanctions List

principals_screened:
  - Mrs. Saira Al-Maktoum-Hartington
  - Mr. Khalid Al-Maktoum-Hartington (London)
  - Mrs. Aisha Al-Maktoum-Hartington (Edinburgh)
  - Mr. Yusuf Al-Maktoum-Hartington (Geneva)
  - Sir Jonathan Hartington-Pemberton (London)
  - Mr. Nathaniel Tan-Lim (Singapore)
  - Halberd-Mercer Trustees UK Ltd
  - Halberd-Mercer Trustees DIFC Ltd
  - Al-Maktoum-Hartington-Tan UK Bilateral Trust 2027
  - Al-Maktoum-Hartington-Tan UAE Bilateral Trust 2027 (DIFC)
  - Cayman SPV 2023 (current name)
  - Cayman STAR-Trust 2027 (proposed new name)

results:
  total_hits:                          0 (zero — all clean)
  fuzzy_matches_within_threshold:      0
  manual_review_required:              false
```

`EVT-J173-SANCTIONS-SCREEN-CLEAN-Δ005a` sealed at 22:18 GST May 17.

AML screening:

```
[AML SCREENING] 2027-05-17 22:18 GST
─
amld5_compliance:                   passed (per AMLD5)
uk_mlr_2017_compliance:              passed (per UK MLR 2017)
uae_fdl_20_2018_compliance:           passed (per UAE Federal Decree-Law 20/2018)
sg_mas_amla_compliance:              passed (per MAS notice 626)
ky_mlr_compliance:                   passed (per KY Money Laundering Regulations)

source_of_funds_verified:             true (audited trust accounting + client engagement letter)
beneficial_ownership_clear:           true (4 signatories + 3 non-signing beneficiaries documented)
pep_status:                           Saira flagged as PEP (Persian Gulf prominent family; documented + risk-assessed)
high_risk_jurisdiction_exposure:    none (UK + UAE + SG + KY all FATF white-list)
```

`EVT-J173-AML-SCREEN-CLEAN-Δ005b` sealed at 22:42 GST May 17.

SWIFT MT103 messages are dispatched:

```
[SWIFT MT103 DISPATCH] 2027-05-17 23:00 GST
─
mt103_message_uk_to_difc:
  message_id:                       MT103-UK-DIFC-2027-05-17-Δ0001
  sender_bic:                       COUTGB22 (Coutts London)
  receiver_bic:                     BOMLAEAD (Mashreq Bank Dubai)
  field_20_transaction_reference:    AAMHT-2027-CONSOLIDATION-LEG1
  field_32a_value_currency_amount:  20270518GBP14400000.00 (T+1)
  cover_mt202_via:                  HSBC London → HSBC Dubai
  audit_event_id:                   EVT-J173-MT103-LEG1-Δ005c

mt103_message_sg_to_difc:
  [similar structure]
  audit_event_id:                   EVT-J173-MT103-LEG2-Δ005d

mt103_message_ky_to_difc:
  [similar structure]
  audit_event_id:                   EVT-J173-MT103-LEG3-Δ005e
```

`EVT-J173-CONSOLIDATION-TRANSFER-005` (composite) initiated.

## §8 — Wednesday May 19: arrivals + reconciliation

UK leg arrives 17:18 GST May 18 (T+0 due to cut-off). SG leg arrives 16:42 GST May 18. KY leg arrives 22:18 GST May 18 (T+1 due to Cayman cut-off).

```
[CONSOLIDATION ARRIVAL RECONCILIATION] 2027-05-19 09:14 GST
─
expected_total:                       $42,000,000
actual_arrived:                       $42,001,184 (FX favourable +$1,184)
discrepancy_explanation:              FX spread favourable on 2 of 3 legs
reconciliation_state:                 ok
post_arrival_balance_difc_mashreq:    $42,001,184 (new bilateral trust account)
post_uk_balance:                       £0 (UK pilot trust 2019 account closed; dissolution deed effective)
post_sg_balance:                       S$0 on the consolidation portion (trust amendment effective; remaining S$148M stays in SG trust)
post_ky_balance:                       $0 on the consolidation portion (SPV novation effective; remaining $36M stays in Cayman STAR-trust)
```

`EVT-J173-CONSOLIDATION-RECONCILED-Δ005f` sealed at 09:14 GST May 19.

## §9 — May 19–21: per-document Merkle attestation + WORM cell placement

Each of the 6 executed documents is anchored with tax-authority-compellable inclusion proof:

```
[MERKLE ATTESTATION PER DOCUMENT] 2027-05-19 12:00 GST
─
doc-1 (uk-pilot-trust-dissolution):
  jurisdiction:                      UK (12y retention)
  worm_cell:                          eu-london-tier-1-worm-trust-retention
  merkle_root:                        sha256:c4d2…9a14
  tax_authority_compulsion_path:     HMRC
  audit_event_id:                    EVT-J173-MERKLE-DOC-1-Δ009a

doc-2 (uk-side-new-bilateral-trust):
  jurisdiction:                      UK (12y)
  worm_cell:                          eu-london-tier-1-worm-trust-retention
  merkle_root:                        sha256:d3e5…fa21
  tax_authority_compulsion_path:     HMRC + IRS (FATCA path armed)
  audit_event_id:                    EVT-J173-MERKLE-DOC-2-Δ009b

doc-3 (uae-side-new-bilateral-trust-difc):
  jurisdiction:                      UAE (8y retention; DIFC Trust Law 5/2018 §22)
  worm_cell:                          me-dubai-tier-1-worm-trust-retention
  merkle_root:                        sha256:e4f6…0b32
  tax_authority_compulsion_path:     DFSA
  audit_event_id:                    EVT-J173-MERKLE-DOC-3-Δ009c

doc-4 (singapore-variation):
  jurisdiction:                      Singapore (7y retention)
  worm_cell:                          apac-singapore-tier-1-worm-trust-retention
  merkle_root:                        sha256:f5g7…1c43
  tax_authority_compulsion_path:     MAS + IRAS
  audit_event_id:                    EVT-J173-MERKLE-DOC-4-Δ009d

doc-5 (cayman-spv-novation-to-star-trust):
  jurisdiction:                      Cayman (6y retention)
  worm_cell:                          kyc-grand-cayman-tier-2-worm-trust-retention
  merkle_root:                        sha256:g6h8…2d54
  tax_authority_compulsion_path:     CIMA + IRS (FATCA path armed)
  audit_event_id:                    EVT-J173-MERKLE-DOC-5-Δ009e

doc-6 (consolidation-transfer-mandate):
  jurisdiction:                      UAE (8y; primary execution in DIFC)
  worm_cell:                          me-dubai-tier-1-worm-trust-retention
  merkle_root:                        sha256:h7i9…3e65
  tax_authority_compulsion_path:     DFSA + HMRC (for UK leg)
  audit_event_id:                    EVT-J173-MERKLE-DOC-6-Δ009f
```

`EVT-J173-MERKLE-PER-DOCUMENT-009` (composite) sealed at 12:00 GST May 19.

`EVT-J173-WORM-JURISDICTION-AWARE-010` (composite) sealed at 12:42 GST May 19.

External transparency log batched at 18:00 GST May 19:

```
[EXTERNAL TRANSPARENCY LOG BATCH] 2027-05-19 18:00 GST
─
batch_id:                              external-transparency-log-batch-2027-05-19-difc
contained_anchors:                    6
proof_class:                           inclusion_proof_only_without_payload
emitted_at:                            2027-05-19T18:00:00+04:00
```

## §10 — Thursday-Friday May 20-21: settlement complete + final attestation

Aamir works through the final settlement checklist Thursday May 20 + Friday May 21:

**Thursday May 20** — DTAA optimization positions documented:

```
[DTAA OPTIMIZATION ATTESTATION] 2027-05-20 14:00 GST
─
uk_uae_dtaa_2016_article_13_capital_gains:
  applied_to:                          uk-pilot-trust-2019 → uae-side-new-bilateral-trust-2027 component
  treaty_benefit:                     UK CGT only after s260 holdover; UAE-side 0% income tax
  effective_tax_position:             £2.4M UK CGT only (provisional; payable 31 Jan 2028)
  evidence_anchor:                    EVT-J173-MERKLE-DOC-1-Δ009a + EVT-J173-MERKLE-DOC-3-Δ009c

uk_sg_dtaa_1997_article_7_business_profits:
  applied_to:                          singapore-trust-2021 variation
  treaty_benefit:                     SG-side 17% on trustee chargeable income (de minimis FY2027)
  effective_tax_position:             SG income tax de minimis; no UK reduction needed
  evidence_anchor:                    EVT-J173-MERKLE-DOC-4-Δ009d

uk_ky_no_treaty_path:
  applied_to:                          cayman-spv-2023 novation to STAR-trust
  treaty_position:                    no UK-KY DTAA exists; UK chargeable transfer test NOT triggered
  effective_tax_position:             clean
  evidence_anchor:                    EVT-J173-MERKLE-DOC-5-Δ009e
```

`EVT-J173-DTAA-OPTIMIZATION-007` sealed at 14:00 GST May 20.

**Friday May 21 16:48 GST** — settlement complete. Aamir transitions the engagement state to `settlement_complete`. All 6 documents executed + sealed; $42M consolidation cleared; HMRC clearance in hand; CRS + FATCA reissuances filed; pack manifest + DTAA positions documented; Merkle attestations + WORM cell placement complete.

```
[SETTLEMENT COMPLETE] 2027-05-21 16:48 GST
─
engagement:                            client-family-al-maktoum-hartington-tan-restructure-2027
final_state:                            settlement_complete
total_engagement_days:                  58 (started 2027-03-24)
intensive_phase_days:                   11 (2027-05-10 to 2027-05-21)
documents_executed:                     6
documents_anchored:                     6
worm_cells_populated:                   4 (UK + UAE + SG + KY)
consolidation_transfer_amount_usd:      $42,001,184
counsel_firms_engaged:                  3 (Mishcon + Maples + Allen & Gledhill)
family_principals_signed:               4 of 7 (4 signing party; 3 non-signing beneficiaries documented)
sanctions_screening:                    100% clean across 6 lists
aml_screening:                          100% pass across 5 jurisdictions
crs_reissuances:                        2
fatca_form_reissuances:                 2
cgt_clearance:                          granted (HMRC)
dtaa_optimizations_documented:          3 (UK-UAE + UK-SG + UK-KY)
pack_manifest_signature:                sha256:f7e2…3c19
```

`EVT-J173-PACK-MANIFEST-008` (final) sealed at 16:48 GST May 21.

Cedar deny coverage report:

```
[CEDAR DENY COVERAGE] engagement Δ
─
denied_step_privileged_enumeration:   12 (non-counsel principals)
denied_sign_without_kyc:              4 (sign attempts pre-KYC-attestation)
denied_consolidation_pre_sanctions:    2 (transfer initiation pre-sanctions-screening)
total_denied:                         18
observability_redaction_pct:          100
```

`EVT-J173-CEDAR-DENY-COVERAGE-011` sealed at 17:18 GST.

Aamir closes the engagement view at 18:18 GST. He calls his wife Sana. He's home for Maghrib + dinner.

## §11 — Stop condition

All 12 AC pass on the seeded fixture; 6 documents executed + Merkle-anchored; $42M consolidation cleared with FX-favourable arrival; new bilateral trusts settled in UK + UAE + SG + KY (and Cayman STAR-trust established); HMRC CGT clearance granted; CRS + FATCA reissuances filed; 8 packs cross-validated; jurisdiction-aware WORM cells with per-jurisdiction retention; STEP-privileged channel preserved end-of-line; Arabic + Urdu + Cantonese + Cambridge-English + Cayman-English + Singapore-English + diacritics UTF-8 NFC byte-exact.
