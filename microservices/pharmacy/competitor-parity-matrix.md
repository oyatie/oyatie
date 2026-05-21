# Pharmacy Competitor Parity Matrix

UNION of capability footprints from the leading pharmacy systems. Oyatie pharmacy MUST cover every capability tagged "OYA-T".

- **Top-3 counterparts**: Oracle Health (Cerner) Pharmacy Manager / Epic Willow / BD Pyxis
- **Secondary counterparts**: McKesson EnterpriseRx / Omnicell / Talyst / Parata Mini / Surescripts / NCPDP / GS1 EPCIS

Legend:
- ✓ = covered by counterpart
- — = not covered by counterpart
- OYA-T = required target for Oyatie pharmacy (every row is OYA-T)

| # | Capability | Cerner Pharmacy Mgr | Epic Willow | BD Pyxis | McKesson EnterpriseRx | Omnicell | Surescripts | Notes |
|---|---|---|---|---|---|---|---|---|
| 1 | NDC catalog (NDC10 + NDC11) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | OYA-T |
| 2 | RxNorm RxCUI canonical id | ✓ | ✓ | — | ✓ | — | ✓ | OYA-T |
| 3 | GPI therapeutic class | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 4 | ATC (WHO) classification | ✓ | ✓ | — | — | — | — | OYA-T |
| 5 | UNII ingredient ID | ✓ | ✓ | — | — | — | — | OYA-T |
| 6 | Brand/generic linking | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | OYA-T |
| 7 | Form/route/strength axes | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | OYA-T |
| 8 | Package configuration (NDC11, GTIN-14) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | OYA-T |
| 9 | FDB MedKnowledge ingestion | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 10 | Multum Lexicon ingestion | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 11 | Medi-Span ingestion | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 12 | A/B knowledge package switch | — | ✓ | — | — | — | — | OYA-T (Oyatie-distinct: safety rollback) |
| 13 | Monthly RxNorm release reconciliation | ✓ | ✓ | — | ✓ | — | ✓ | OYA-T |
| 14 | Formulary classifier | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 15 | Per-tenant formulary | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 16 | Per-cell formulary overlay | — | — | — | — | — | — | OYA-T (Oyatie-distinct) |
| 17 | P&T committee workflow | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 18 | Therapeutic interchange | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 19 | Prior-authorization criteria | ✓ | ✓ | — | ✓ | — | ✓ | OYA-T |
| 20 | NCPDP SCRIPT 2017-071 | ✓ | ✓ | — | ✓ | — | ✓ | OYA-T |
| 21 | Surescripts mTLS production | ✓ | ✓ | — | ✓ | — | ✓ | OYA-T |
| 22 | EPCS Schedule II–V | ✓ | ✓ | — | ✓ | — | ✓ | OYA-T |
| 23 | DEA-bound KMS individual key | ✓ | ✓ | — | ✓ | — | ✓ | OYA-T |
| 24 | NewRx | ✓ | ✓ | — | ✓ | — | ✓ | OYA-T |
| 25 | RxRenewal request/response | ✓ | ✓ | — | ✓ | — | ✓ | OYA-T |
| 26 | RxChange request/response | ✓ | ✓ | — | ✓ | — | ✓ | OYA-T |
| 27 | CancelRx | ✓ | ✓ | — | ✓ | — | ✓ | OYA-T |
| 28 | RxFill | ✓ | ✓ | — | ✓ | — | ✓ | OYA-T |
| 29 | RxHistory request/response | ✓ | ✓ | — | ✓ | — | ✓ | OYA-T |
| 30 | REMS init request | ✓ | ✓ | — | — | — | ✓ | OYA-T |
| 31 | DDI (drug-drug) | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 32 | DAI (drug-allergy) | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 33 | DCI (drug-condition) | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 34 | DPI (drug-pregnancy/lactation) | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 35 | DDxI (drug-diagnosis) | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 36 | DLI (drug-lab) | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 37 | DFI (drug-food) | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 38 | DDoseI (drug-dose-range) | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 39 | Severity stratification (6 bands) | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 40 | Monograph evidence linking | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 41 | Per-tenant severity suppression | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 42 | Cedar-gated severe/contraindicated suppression | — | — | — | — | — | — | OYA-T (Oyatie-distinct) |
| 43 | Allergy normalize to RxNorm + UNII + SNOMED | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 44 | Allergy cross-class derivation | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 45 | Allergy override with reason + attestation | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 46 | Allergy two-step override on severe | ✓ | ✓ | — | — | — | — | OYA-T |
| 47 | Weight-based DRC | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 48 | BSA-based DRC | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 49 | Renal (eGFR CKD-EPI + CrCl) DRC | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 50 | Hepatic (Child-Pugh) DRC | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 51 | Age-band (neonatal/pediatric/geriatric) DRC | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 52 | Single-dose-max + daily-max + lifetime-cumulative DRC | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 53 | Duplicate-therapy detection (RxCUI + ATC + brand-generic) | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 54 | Pharmacist verification (single) | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 55 | Pharmacist dual-verification (CII) | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 56 | Tall-man-lettering rendering | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 57 | Alert dismissal capture | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 58 | USP 795 non-sterile compounding | ✓ | ✓ | — | — | — | — | OYA-T |
| 59 | USP 797 sterile compounding | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 60 | USP 800 hazardous compounding | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 61 | Master formulation record | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 62 | Compounding record (per batch) | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 63 | BUD calculator (USP table) | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 64 | Environmental monitoring binding | ✓ | ✓ | — | — | — | — | OYA-T |
| 65 | ISO-7 negative-pressure cell tag | — | — | — | — | — | — | OYA-T (Oyatie-distinct) |
| 66 | Par/min/max inventory | ✓ | ✓ | ✓ | ✓ | ✓ | — | OYA-T |
| 67 | Lot tracking | ✓ | ✓ | ✓ | ✓ | ✓ | — | OYA-T |
| 68 | Expiry tracking + stratification | ✓ | ✓ | ✓ | ✓ | ✓ | — | OYA-T |
| 69 | Recall sequestration with hard-block | ✓ | ✓ | ✓ | ✓ | ✓ | — | OYA-T |
| 70 | Pyxis adapter | — | — | ✓ | — | — | — | OYA-T |
| 71 | Omnicell adapter | — | — | — | — | ✓ | — | OYA-T |
| 72 | Carousel (Talyst) adapter | — | — | — | — | — | — | OYA-T |
| 73 | AcuDose adapter | — | — | — | — | — | — | OYA-T |
| 74 | MedDispense adapter | — | — | — | — | — | — | OYA-T |
| 75 | Cabinet vendor-neutral contract | — | — | — | — | — | — | OYA-T (Oyatie-distinct) |
| 76 | Cabinet offline mode + reconcile | ✓ | ✓ | ✓ | — | ✓ | — | OYA-T |
| 77 | Cabinet override events captured | ✓ | ✓ | ✓ | — | ✓ | — | OYA-T |
| 78 | Cabinet discrepancy reconciliation | ✓ | ✓ | ✓ | — | ✓ | — | OYA-T |
| 79 | BCMA five-rights verification | ✓ | ✓ | — | — | — | — | OYA-T |
| 80 | BCMA scan latency p99 ≤ 100 ms | — | ✓ | — | — | — | — | OYA-T |
| 81 | BCMA override with reason + pharmacist callback | ✓ | ✓ | — | — | — | — | OYA-T |
| 82 | MAR write-back | ✓ | ✓ | — | — | — | — | OYA-T |
| 83 | IV admixture compound | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 84 | Alaris pump library push | ✓ | ✓ | — | — | — | — | OYA-T |
| 85 | Plum 360 pump library push | ✓ | ✓ | — | — | — | — | OYA-T |
| 86 | Hospira (ICU Medical) pump library push | ✓ | ✓ | — | — | — | — | OYA-T |
| 87 | DERS hard/soft limit programming | ✓ | ✓ | — | — | — | — | OYA-T |
| 88 | Pump auto-program via QR/barcode | ✓ | ✓ | — | — | — | — | OYA-T |
| 89 | DEA Form 222 ordering | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 90 | Perpetual CII–CV inventory | ✓ | ✓ | ✓ | ✓ | ✓ | — | OYA-T |
| 91 | Witnessed waste two-person | ✓ | ✓ | ✓ | ✓ | ✓ | — | OYA-T |
| 92 | CII transaction witness signature | ✓ | ✓ | ✓ | ✓ | ✓ | — | OYA-T |
| 93 | DEA inspection-ready report | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 94 | 340B HRSA eligibility evaluator | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 95 | 340B replenishment lot tagging | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 96 | 340B OPAIS reporting | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 97 | 340B mixed-use classification | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 98 | PBM NCPDP D.0 claims | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 99 | PBM reject-code classification | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 100 | Payer copay calculation | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 101 | Plan accumulator tracking | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 102 | Pharmacy ops order queue | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 103 | Pharmacy ops prep queue | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 104 | Pharmacy ops verify queue | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 105 | Pharmacy ops deliver queue | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 106 | Pharmacist workload balancing | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 107 | Prospective DUR | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 108 | Retrospective DUR | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 109 | Clinical intervention capture | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 110 | MTM CMR (CPT 99605/99606/99607) | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 111 | MTM TMR | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 112 | Medication action plan (MAP) | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 113 | Personal medication list (PML) | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 114 | Admission medication reconciliation | ✓ | ✓ | — | — | — | — | OYA-T |
| 115 | Transfer medication reconciliation | ✓ | ✓ | — | — | — | — | OYA-T |
| 116 | Discharge medication reconciliation | ✓ | ✓ | — | — | — | — | OYA-T |
| 117 | Retail counter dispensing | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 118 | Drive-through pharmacy | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 119 | Mail-order pharmacy | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 120 | Specialty pharmacy (LDD) | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 121 | REMS enrollment + tracking | ✓ | ✓ | — | ✓ | — | ✓ | OYA-T |
| 122 | Refill request handling | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 123 | Will-call expiration | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 124 | DSCSA SGTIN-198 serialization | ✓ | ✓ | ✓ | ✓ | ✓ | — | OYA-T |
| 125 | DSCSA T1/T2/T3 transactions | ✓ | ✓ | ✓ | ✓ | ✓ | — | OYA-T |
| 126 | DSCSA EPCIS 2.0 alignment | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 127 | DSCSA saleable returns verification | ✓ | ✓ | ✓ | ✓ | ✓ | — | OYA-T |
| 128 | DSCSA suspect product investigation | ✓ | ✓ | ✓ | ✓ | ✓ | — | OYA-T |
| 129 | HIPAA covered + minimum necessary | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | OYA-T |
| 130 | HIPAA access review cadence | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | OYA-T |
| 131 | 42 CFR Part 2 SUD re-disclosure | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 132 | State board of pharmacy overlay | ✓ | ✓ | — | ✓ | — | — | OYA-T |
| 133 | Audit chain seal (sub-tick) | — | — | — | — | — | — | OYA-T (Oyatie-distinct) |
| 134 | Bilateral chain cross-pointer (cross-tenant) | — | — | — | — | — | — | OYA-T (Oyatie-distinct) |
| 135 | Cedar-gated every action | — | — | — | — | — | — | OYA-T (Oyatie-distinct) |
| 136 | Break-glass with audit (24h review) | — | ✓ | — | — | — | — | OYA-T |
| 137 | 12 self-SLOs registered with observability | — | — | — | — | — | — | OYA-T (Oyatie-distinct) |
| 138 | Cell-tier shuffle sharding | — | — | — | — | — | — | OYA-T (Oyatie-distinct) |
| 139 | HTTP/3 + QUIC default | — | — | — | — | — | — | OYA-T (Oyatie-distinct) |
| 140 | OpenTofu-only deployment | — | — | — | — | — | — | OYA-T (Oyatie-distinct) |
| 141 | 6 deployment contexts (incl. always-free) | — | — | — | — | — | — | OYA-T (Oyatie-distinct) |
| 142 | Multi-pack compliance overlay | — | — | — | — | — | — | OYA-T (Oyatie-distinct) |
| 143 | Sovereign air-gapped deployment | — | — | — | — | — | — | OYA-T (Oyatie-distinct) |
| 144 | A/B knowledge-package per-tenant | — | — | — | — | — | — | OYA-T (Oyatie-distinct) |
| 145 | Confidential compute (SEV-SNP/TDX/CCA) EPCS | — | — | — | — | — | — | OYA-T (Oyatie-distinct) |
| 146 | Intelligence-substrate MTM PML drafting (T3 + redaction) | — | — | — | — | — | — | OYA-T (Oyatie-distinct) |
| 147 | HLC default + TrueTime tier opt-in | — | — | — | — | — | — | OYA-T (Oyatie-distinct) |
| 148 | Multi-vendor cabinet contract (swap-vendor smoke test) | — | — | — | — | — | — | OYA-T (Oyatie-distinct) |
| 149 | Multi-vendor pump contract (Alaris/Plum/Hospira) | — | — | — | — | — | — | OYA-T (Oyatie-distinct) |
| 150 | Multi-vendor knowledge-base ingest (FDB/Multum/Medi-Span) | — | — | — | — | — | — | OYA-T (Oyatie-distinct) |

Total OYA-T capabilities: **150** (target ≥ 100; covered).

Counterpart coverage profile:
- Cerner Pharmacy Manager: ~115 / 150 covered.
- Epic Willow: ~118 / 150 covered.
- BD Pyxis: ~30 / 150 covered (cabinet-focused).
- McKesson EnterpriseRx: ~95 / 150 covered.
- Omnicell: ~25 / 150 covered (cabinet-focused).
- Surescripts: ~14 / 150 covered (network-focused).

Oyatie-distinct capabilities (rows 12, 16, 42, 65, 75, 133–150): 21 capabilities that no single counterpart covers. These derive from oyatie substrate primitives — Cedar universal gate, audit chain, cellular topology, multi-context deployment, intelligence-substrate, compliance pack overlay, vendor-neutral adapter contracts.
