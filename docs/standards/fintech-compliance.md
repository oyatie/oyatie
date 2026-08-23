---
purpose: Oyatie — Fintech Compliance Deep-Dive
doc_status: published
---

# Oyatie — Fintech Compliance Deep-Dive

> **Status:** Draft v0.1 — 2026-05-09. Authored per user directive: depth on PCI-DSS scope and per-jurisdiction regulatory regimes for Oyatie fintech operations modeled on Toss / KakaoBank / KakaoPay / NaverPay / PayPal / Stripe / Adyen / Wise / Robinhood / Coinbase-class.
> **Owner:** `vertical-fintech` + `ops-compliance` + `regional-packs` (per-jurisdiction).
> **Companion:** [COMPLIANCE-MATRIX.md](../COMPLIANCE-MATRIX.md), [PRIVACY-PROGRAM.md](../PRIVACY-PROGRAM.md), [security-program.json](../security-program/security-program.json), [`products/vertical-fintech/PRD.md`](../products/vertical-fintech/PRD.md).
> **Critical reference for:** any tenant onboarding into the `vertical-fintech` axis; any cloud-region launch where fintech is in scope; any Workspace/Mail surface that touches cardholder data; any Foundry capability that touches financial accounts.

---

## 1. The fintech offering taxonomy (what Oyatie can operate per license tier)

Each row is a distinct license posture; Oyatie can pursue any combination per regional pack.

| Posture | Reference (KR / global) | License path | PCI-DSS class | KR-specific overlay |
|---|---|---|---|---|
| **Payment Gateway (PG)** | Toss Payments / Inicis / NICEpay / KICC / Stripe / Adyen / Braintree | KR FSC PG 등록 (전자금융거래법 §28) + Level 1 PCI Service Provider | Level 1 SP | 전자금융거래법, 전자금융감독규정, 망분리 (separated VLAN for CDE) |
| **e-Money issuer / Wallet** | Toss / KakaoPay / NaverPay / PayPal / Wise / Square Cash | KR 선불전자지급수단 발행업 등록 + AML | Level 1 SP if storing PAN; Level 2-4 if tokenized only | 특금법 KoFIU + 외국환거래법 if cross-border |
| **Digital bank (Internet-only bank)** | KakaoBank / K뱅크 / Toss뱅크 / Chime / N26 / Revolut Bank | KR 인터넷전문은행 본인가 (은행법 §8 + Internet-Only Bank Act) | Level 1 SP + bank-grade controls | 한국은행 BOK 결제망 + KFTC + 망분리 (full separation) + 예금자보호법 |
| **Cross-border remittance / FX** | Wise / Remitly / WorldRemit / Western Union | KR 외화송금업 등록 (외국환거래법) + per-jurisdiction MTL | Level 2-3 SP typically | KoFIU travel-rule reporting (R/T data) |
| **Brokerage / Securities trading** | KakaoPay 증권 / Toss증권 / Robinhood / E*TRADE / Schwab | KR 금융투자업 본인가 (자본시장법) + KSDA + KRX membership | Level 2-3 SP | 자본시장법, 한국예탁결제원 KSD, KOFIA |
| **Insurance / Insurtech** | Lemonade / Carrot손보 / 캐롯 | KR 보험업 본인가 (보험업법) | Level 2 SP | 보험업법, 보험요율산출기관 |
| **Open Banking aggregator (PISP/AISP)** | Toss aggregation / Plaid / Tink | KR 마이데이터 본인가 (신용정보법) + EU PSD2 PISP/AISP | Level 2-3 SP | 신용정보법 + KoFIU |
| **Lending / BNPL** | KakaoPay 대출 / Affirm / Klarna / Afterpay / Upstart | KR 대출업 등록 (대부업법 / 여신전문금융업법) + per-jurisdiction lending license | Level 2-3 SP | 신용정보법, 채권추심법 |
| **Crypto exchange / Custody** | Upbit / Bithumb / Coinbase / Binance / Kraken | KR 가상자산사업자 신고 (특금법) + KoFIU + per-jurisdiction VASP | Level 2-3 SP for fiat on/off-ramp | EU MiCA, US FinCEN MSB, NY BitLicense |

The **PCI-DSS scope class** is determined by *cardholder data flow*, not by the license posture — but the two correlate strongly.

---

## 2. PCI-DSS v4.0 scope (what triggers what level)

PCI-DSS v4.0 (effective 2024-03-31; 2025-03-31 future-dated requirements) defines the scope as **the people, processes, and technology that store, process, or transmit cardholder data (CHD) or sensitive authentication data (SAD)**.

### 2.1 What counts as cardholder data and SAD

| Data | Storage allowed? | Notes |
|---|---|---|
| **PAN (Primary Account Number)** | Yes if encrypted | If stored, encrypt at rest (Req 3.5); display masked except last 4 |
| Cardholder name | Yes | Bound to PAN |
| Service code | Yes | Bound to PAN |
| Expiration date | Yes | Bound to PAN |
| **CAV2 / CVC2 / CVV2 / CID** (card verification value) | **NEVER** post-authorization | Sensitive Authentication Data — Req 3.2 |
| **Full magnetic stripe / chip data** | **NEVER** post-authorization | SAD — Req 3.2 |
| **PIN / PIN block** | **NEVER** post-authorization | SAD — Req 3.2 |

If Oyatie *ever* touches SAD post-authorization, that is an immediate Req 3.2 breach.

### 2.2 Service Provider levels

| Level | Trigger | Annual assessment |
|---|---|---|
| **Level 1 SP** | > 300,000 transactions per year for **any** brand OR designated by a brand OR experienced a data compromise | On-site QSA-led RoC + quarterly ASV scan + penetration test (internal + external) annually |
| **Level 2 SP** | 1 to 300,000 transactions/year | Annual SAQ-D-SP + quarterly ASV scan |
| **Level 3 SP** | (Visa: < 80,000 transactions/year) | Annual SAQ + quarterly ASV |
| **Level 4 SP** | smallest | Annual SAQ |

For Toss-class PG operating millions of transactions per day: **Level 1 SP, RoC required, on-site QSA**.

### 2.3 Scope-reduction strategies (always preferred over expansion)

| Strategy | Reduces scope by | Cost / trade-off |
|---|---|---|
| **Tokenization at the edge** | Downstream services see tokens, not PAN. Tokenization vault is in scope; everything downstream is out of scope or "connected to" scope | Token vault must be PCI-validated; format-preserving encryption (FPE) optional |
| **P2PE (Point-to-Point Encryption)** | Payment terminal encrypts PAN at swipe; only the certified PSP can decrypt; Oyatie back-office is out of scope | Requires PCI-listed P2PE solution (e.g. Verifone, Ingenico, Spire) |
| **Outsource card capture to certified gateway** | Cardholder enters PAN on PSP-hosted page (iframe / redirect); Oyatie never sees PAN | Loses some UX control; SAQ-A applicable for the merchant slice |
| **Network segmentation** | Cardholder Data Environment (CDE) separated by VLAN / firewall / namespace; non-CDE systems out of scope | Strict firewall rules (Req 1.4); penetration-test the segmentation annually (Req 11.4.5) |
| **HSM-backed key management (KCMVP for KR)** | Encryption keys never in software; key compromise blast radius limited | KCMVP HSM 6-9 month procurement lead; FIPS 140-3 Level 3+ for global |
| **Per-tenant CDE isolation** | One tenant's CHD breach doesn't reach another tenant; multi-tenant scope reduction | Per-tenant cell + per-tenant CDE namespace + per-tenant HSM partition or per-tenant DEK |

Oyatie's posture: **maximum scope reduction**. Default architecture is:
1. Oyatie does NOT capture PAN directly. Payment capture goes through a partner PG (KICC / NICE / Toss / Stripe / Adyen) with PCI L1 certification.
2. Oyatie's tokenization-vault (`crates/vertical-fintech-tokenvault-*`) holds tokens only; never raw PAN post-authorization.
3. CDE = `crates/vertical-fintech-cde-*` flat-crate set, deployed to a dedicated cell with per-cell HSM-backed KMS, separated VLAN, no shared infra with non-CDE.
4. Foundry, Search, Ads, Workspace, and SaaS axes are **OUT OF SCOPE** for PCI-DSS by construction — they cannot read CDE.
5. Per-tenant CDE isolation: each fintech tenant is its own CDE cell.

### 2.4 PCI-DSS v4.0 control families (12 + 1 customized approach)

> Per [COMPLIANCE-MATRIX.md §3.4](../COMPLIANCE-MATRIX.md) row, expanded here.

| Req | Title | Implementing surface | Evidence | Cadence |
|---|---|---|---|---|
| Req 1 | Install + maintain network security controls | VPC + cell-segmented CDE + Cedar-policied firewall | Firewall rule audit | quarterly + on-change |
| Req 2 | Apply secure configurations | CIS-benchmark-conformant images; immutable infra; cargo-deny + Trivy | Image-config attestation | per-release |
| Req 3 | Protect stored account data | KMS-shred per record; PAN never stored; tokens only; SAD never stored post-auth | Crypto inventory; tokenization audit | continuous |
| Req 4 | Protect cardholder data with strong cryptography during transmission | mTLS via Istio Ambient; TLS 1.3 minimum; cipher allowlist | TLS scan; cert audit | continuous |
| Req 5 | Protect all systems and networks from malicious software | EDR on all CDE hosts; AV scan on all uploads | EDR alerts | continuous |
| Req 6 | Develop and maintain secure systems and software | Trivy 4-layer per ADR-0039; dependency-license gate; secure SDLC | CI lane | per-build |
| Req 7 | Restrict access to system components and cardholder data by business need to know | Cedar policy + RBAC + per-role allowlist; quarterly access review | Per-access audit | continuous + quarterly review |
| Req 8 | Identify users and authenticate access | MFA enforcement (incl. step-up auth for CDE); short-lived STS credentials | Per-login audit | continuous |
| Req 9 | Restrict physical access to cardholder data | Colo / DC physical security; badge + bio access; visitor log | DC audit | annual |
| Req 10 | Log and monitor all access to system components and cardholder data | Audit chain per ADR-0003; per-CDE-access emission; log integrity (WORM) | Continuous + monthly review | continuous |
| Req 11 | Test security of systems and networks regularly | Quarterly external ASV scan; annual internal + external pen test; quarterly internal vulnerability scan; annual segmentation test (Req 11.4.5) | Scan / test reports | quarterly + annual |
| Req 12 | Support information security with organizational policies and programs | SECURITY-PROGRAM + INCIDENT-MANAGEMENT + annual training | Training records; policy attestation | annual |
| Customized Approach (v4.0 NEW) | Risk-based alternative implementations | Per-control TRA + monitoring; QSA approves the alternative | TRA + ongoing measurement | per-control |

### 2.5 PCI-DSS v4.0 future-dated requirements (effective 2025-03-31)

These are NOT optional after the date. Oyatie's W-Foundation gate must include them.

- Req 8.4.2 / 8.5.1: MFA for all access into CDE (was: just admin access)
- Req 11.6.1: Continuous change-and-tamper detection on payment pages (e.g. defacement / supply-chain skimming)
- Req 12.10.7: Targeted risk analysis for critical incident scenarios
- Req 6.4.3: Scripts on payment pages must be inventoried + integrity-verified
- Req 3.7: Cryptographic-key inventory + lifecycle documentation

---

## 3. KR-specific fintech regulations (deep)

Oyatie's KR fintech operations fall under the most extensive regulatory regime in our launch set.

### 3.1 전자금융거래법 (Electronic Financial Transactions Act / EFTA)

Primary law for PG / e-money / digital banks / wallets. Key provisions:

| Article | Title | Implementing |
|---|---|---|
| §8 | Authentication of identification, etc. | 본인확인서비스 / KISA-designated identity-verification provider mandatory |
| §15 | Secure financial transactions | mTLS + HSM + audit-chain |
| §17 | Service provider's obligation | Pre-registration; capital requirements; real-name verification |
| §21 | Liability for accidents | Tenant-side losses up to limit unless subscriber gross negligence |
| §28 | PG registration | KR FSC PG license required; per-license type (online vs offline) |
| §39 | Sanctions | Per-violation administrative penalty |

### 3.2 전자금융감독규정 (FSC supervisory regulation)

Detailed operational rules issued by KR FSC. Key:

- **망분리** (network separation): financial sector requires logical and/or physical separation of internet-facing and CDE networks. For Oyatie cloud: separated VLANs + air-gapped CDE per tenant; egress allowlist enforced.
- **데이터센터 위치** (data-center location): KR FSS may require KR-resident DC for some surfaces; per-tenant residency declaration in `platform-tenant-kernel.residency`.
- **암호 모듈 KCMVP**: cryptographic modules must be KCMVP-validated (not just FIPS-validated).
- **분기별 보안취약점 점검** (quarterly security vulnerability assessment): submit to FSS.
- **연간 정보보호공시** (annual InfoSec disclosure): public report.
- **사고 발생시 24시간 내 통보** (24-hour incident notification to FSS).

### 3.3 신용정보법 (Credit Information Act)

Governs credit data + MyData (personal-data portability) + open banking aggregation.

| Provision | Implementing |
|---|---|
| MyData 본인가 (license) | Required for open-banking aggregator posture |
| Credit-information consent | Per-data-class consent; PRIVACY-PROGRAM §2.2.1 class 6 (FINANCIAL_KR_신용정보) HARD_DENY for ads |
| Credit-data retention | 5-year retention by default; per-class override |
| Right to view / correct / delete | DSR cascade per PRIVACY-PROGRAM §2.2.9 |
| Audit log | Per-access emission to audit chain |

### 3.4 자본시장법 (Capital Markets Act)

For brokerage / Robinhood-class.

| Provision | Implementing |
|---|---|
| 금융투자업 본인가 | Brokerage license; capital + governance requirements |
| Suitability + KYC | Per-customer suitability assessment; recorded |
| Conflict-of-interest disclosure | Per-trade disclosure |
| Best-execution obligation | Per-trade evidence |
| 한국예탁결제원 KSD settlement | Real-time settlement integration |
| KOFIA + KRX membership | Per-exchange |
| 자본시장조사단 (Capital Market Investigation Bureau) reporting | Per suspicious trade |

### 3.5 은행법 + Internet-Only Bank Act (digital bank, KakaoBank-class)

| Provision | Implementing |
|---|---|
| 본인가 (banking license) | Capital ≥ 250B KRW typically; FSC approval; founder fit-and-proper |
| 예금자보호법 (Deposit Protection Act) | KDIC integration; per-depositor 50M KRW protection |
| BOK 결제망 (BOK payment network) | Direct settlement participation OR via clearing bank |
| KFTC clearing | Real-time clearing integration |
| 한국은행 RTGS 한은금융망 | Real-time gross settlement for large-value |
| Per-product capital adequacy (Basel III + KR overlay) | Risk-weighted asset calculation; Pillar 3 disclosure |

### 3.6 특금법 (Specific Financial Transactions Act / AML)

Governs AML + KYC + KoFIU reporting for **all** financial activity (including crypto VASPs).

| Requirement | Implementing |
|---|---|
| 고객확인 (KYC) | per-customer onboarding; ID verification; risk-rating |
| 강화된 고객확인 (EDD) | High-risk customer; PEP screening; sanctions screening (UN, EU, US OFAC, KR Foreign Exchange Transactions Act blacklist) |
| 의심거래보고 (STR) | Per suspicious transaction; KoFIU within prescribed window |
| 고액현금거래보고 (CTR) | Cash transactions ≥ 10M KRW; KoFIU |
| Travel rule (가상자산사업자 / VASP) | R/T data ≥ 1M KRW transfer; KoFIU per FATF Travel Rule |
| 거래기록 보관 5년 | 5-year retention |
| 임직원 교육 | Annual AML training |

### 3.7 외국환거래법 (Foreign Exchange Transactions Act / FETA)

For cross-border money movement.

| Requirement | Implementing |
|---|---|
| 외화송금업 등록 | License for cross-border remittance |
| 외환신고 (BOK reporting) | Per-transaction reporting if ≥ $10K equivalent |
| 외환위반방지 | Sanctions screening + UN list + KR FETA blacklist |
| 환전업 등록 | License for FX |

### 3.8 본인확인서비스 (Identity Verification — KISA designated providers)

Critical for any KR fintech onboarding:

- **CI (Connecting Information)**: 88-byte hashed identifier from NICE / KCB / SCI Plus / etc. Per-user, per-service consistent.
- **DI (Duplication Information)**: 64-byte service-specific hash; allows cross-service de-duplication without sharing PII.
- Real-name verification via 휴대폰 (mobile carrier) / 아이핀 (i-PIN) / 공동인증서 (joint cert) / 금융인증서 (financial cert) / 간편본인확인 (simple-auth providers like KakaoTalk auth, Naver auth).
- Resident registration number (RRN) handling: collection requires explicit legal basis; storage requires KISA-approved encryption.

### 3.9 마이데이터 / Open Banking

KR's open-banking framework run by 금융결제원 (KFTC) under FSC supervision.

| Component | Implementing |
|---|---|
| MyData API standard | Conform to FSC-published OAS spec; per-quarter version updates |
| Consent management | Per-data-source per-purpose per-time-window; revocable |
| Aggregator posture | MyData 본인가 license; data-controller responsibilities |
| Audit log per access | Per-access emission to audit chain |
| 90-day consent window | Default; renewable per user explicit consent |

### 3.10 망분리 (Network separation)

Fintech-specific stronger version vs general 망분리:

- **물리적 분리** (physical separation): for some workloads (e.g. brokerage trading floor) FSS may require fully air-gapped networks
- **논리적 분리** (logical separation): VLANs + Cedar policy + traffic-mirroring for monitoring
- **Bridge servers** (브릿지서버): controlled gateways between networks with content scanning
- **Browser isolation**: web access from CDE only via remote-browser-isolated session
- Per-tenant: each fintech tenant gets its own logical 망분리 enforcement

### 3.11 KR fintech compliance summary table

| Posture | 라이선스 | PCI | 망분리 | KCMVP HSM | KSD/BOK |
|---|---|---|---|---|---|
| PG | FSC PG | L1 SP | logical | required | optional |
| e-money / wallet | 선불전자지급수단 | L1 if PAN | logical | required | KFTC clearing |
| Digital bank | 인터넷전문은행 | L1 SP | physical (some) | required | BOK + KFTC + KSD |
| Brokerage | 금융투자업 | L2-3 | logical+ | required | KSD + KRX + KOFIA |
| Cross-border | 외화송금업 | L2-3 | logical | required | BOK reporting |
| MyData | 마이데이터 | L2-3 | logical | required | per-source |
| VASP | 특금법 신고 | L2-3 if fiat | logical | required | none |
| Lending / BNPL | 대부업 | L2-3 | logical | required | KFTC clearing |
| Insurance | 보험업 | L2 | logical | required | 보험요율산출기관 |

---

## 4. Per-jurisdiction overlays (other regions)

### 4.1 US (Toss US / Robinhood-class)

| Regime | Application |
|---|---|
| PCI-DSS v4.0 | Same; quarterly ASV (PCI-listed) |
| GLBA | Financial-data privacy + safeguards rule |
| SOX | If SEC-reporting issuer |
| BSA / USA PATRIOT Act | AML; CIP / CDD / EDD |
| OFAC | Sanctions screening |
| FinCEN MSB registration | Money services business |
| Per-state MTL | 51 jurisdictions (incl. NY DFS, CA DFPI) |
| OCC / FDIC | If chartered bank |
| FFIEC | InfoSec guidelines |
| Reg E (12 CFR 1005) | Consumer EFT |
| Reg Z (12 CFR 1026) | Truth in Lending |
| SEC + FINRA + SIPC | If brokerage |
| NY DFS Part 500 | NY-specific cybersecurity for FIs |

### 4.2 EU (PayPal EU / Wise / N26)

| Regime | Application |
|---|---|
| PCI-DSS v4.0 | Same |
| PSD2 | SCA (Strong Customer Authentication); TPP licensing for PISP/AISP |
| EBA RTS on SCA | Regulatory technical standards on auth |
| GDPR | Financial data is special-category in some interpretations |
| AMLD6 | AML directive 6 |
| MiCA | Markets in Crypto-Assets |
| DORA | Digital Operational Resilience Act (effective 2025-01-17) — incident notification, ICT risk management, third-party risk |
| EBA guidelines | Per-product (e.g. crowdfunding, BNPL) |
| Per-member-state implementation | E.g. Germany BaFin, Ireland CBI, France ACPR |

### 4.3 JP (Toss Japan / KakaoPay Japan if any)

| Regime | Application |
|---|---|
| PCI-DSS v4.0 | Same |
| 銀行法 (Banking Act) | If banking |
| 資金決済法 (Payment Services Act) | E-money issuer / fund-transfer business |
| 金融商品取引法 (Financial Instruments and Exchange Act) | Brokerage |
| 貸金業法 (Money Lending Act) | Lending |
| APPI | Personal-info protection (special category for financial) |
| FSA + Local Finance Bureau | Supervisor |
| AML / CFT (犯収法) | Customer verification + STR |
| 適格請求書 | E-invoicing for billing |
| マイナンバー | National ID for KYC; protection rules |

### 4.4 IN (UPI / NPCI)

| Regime | Application |
|---|---|
| PCI-DSS v4.0 | Same |
| RBI Master Directions | Multiple per posture (PA / PG / wallets / cross-border) |
| RBI PA license | Payment Aggregator |
| RBI AePS / NACH | National payment systems |
| NPCI UPI compliance | Real-time UPI integration |
| DPDP Act 2023 | Privacy |
| AePS travel rule | Cross-border |
| FEMA | Foreign exchange |
| SEBI | If brokerage |

### 4.5 BR (Pix-class)

| Regime | Application |
|---|---|
| PCI-DSS v4.0 | Same |
| BACEN | Central bank regulator |
| Pix protocol | BACEN-mandated instant payment system |
| LGPD | Privacy |
| BACEN Circular 3909 | Cyber policy + incident reporting |
| CVM | Brokerage |

### 4.6 KSA (Mada / SADAD / SAMA-licensed)

| Regime | Application |
|---|---|
| PCI-DSS v4.0 | Same |
| SAMA Cyber Security Framework | Mandatory |
| SAMA Open Banking | Phased rollout |
| Mada / SADAD integration | Local payment networks |
| PDPL | Privacy |
| Sharia compliance audit | If Islamic-finance product |
| Capital Markets Authority | Brokerage |

### 4.7 UAE (UAE-CB / FATOORA / DIFC FSRA / ADGM FSRA)

| Regime | Application |
|---|---|
| PCI-DSS v4.0 | Same |
| UAE-CB | Federal regulator |
| DIFC FSRA | If DIFC-licensed |
| ADGM FSRA | If ADGM-licensed |
| FATOORA | E-invoicing |
| AaniPay / UAEFTS | National payment systems |
| AML / CFT federal law | Multiple updates 2018-2024 |

### 4.8 AU / NZ (NPP / OSKO)

| Regime | Application |
|---|---|
| PCI-DSS v4.0 | Same |
| APRA | Banking + insurance prudential |
| ASIC | Markets + conduct |
| AUSTRAC | AML / CTF |
| NPP / OSKO | Real-time payments |
| Privacy Act 1988 | Privacy |
| CPS 234 (APRA) | Cybersecurity |

### 4.9 SG (FAST / PayNow / MAS)

| Regime | Application |
|---|---|
| PCI-DSS v4.0 | Same |
| MAS PSA (Payment Services Act) | Per-license tier |
| MAS Tech Risk Mgmt Guidelines | Cybersecurity |
| MAS Notice 626 / 1015 / etc. | Per-product |
| Securities and Futures Act | Brokerage |
| FAST / PayNow | Real-time payments |
| PDPA-SG | Privacy |

---

## 5. Cross-jurisdictional architectural implications for Oyatie fintech

The fintech vertical lives at `crates/vertical-fintech-*`. Per [DESIGN.md §4](../DESIGN.md), the per-pack regulatory binding plugs into seams. Specific implications:

1. **CDE per fintech tenant per region.** Each fintech tenant onboarded in each region gets its own dedicated CDE cell with per-cell HSM partition. No multi-tenant CDE.
2. **Per-region payment-rails adapter** at `crates/vertical-fintech-rail-{kr,jp,us,eu,in,br,ksa,ae,au,sg}-*`. Each implements `PaymentRail` trait (per regional pack §12.2 seam).
3. **Per-region identity-verification adapter** at `crates/vertical-fintech-identity-{kr,jp,us,eu,in,...}-*` plugging into `IdentityProvider` seam.
4. **Per-region AML adapter** at `crates/vertical-fintech-aml-{kr,...}-*` for KoFIU / FinCEN / KoFIU / etc.
5. **Tokenization vault** at `crates/vertical-fintech-tokenvault-*` shared across regions but per-tenant per-cell.
6. **Audit chain** per ADR-0003 emission for every CDE access, every settlement, every KYC, every AML alert, every regulator notification.
7. **Per-region license registry** at `regional-packs/<pack>/fintech-licenses.yaml` declaring which postures the pack supports.
8. **Workspace integration**: any Workspace surface that touches CDE (e.g. Mail attachment containing PAN) is treated as in-scope; DLP must redact PAN automatically.
9. **Foundry agent integration**: Foundry capabilities operating on fintech tenant data run with autonomy ceiling T2 max (flag + freeze, human approves SAR / regulator notification); no T3 or T4 for fintech regulated capabilities.
10. **Search axis**: fintech tenant data is HARD_DENY for cross-tenant search index; per-tenant private search OK with per-class allowlist; PCI / 신용정보 NEVER indexed.

---

## 6. Operational checklist (per fintech tenant onboarding)

> Mirrored at [`../../templates/checklists/fintech-tenant-onboarding.md`](../../templates/checklists/tenant-onboarding.md).

1. ☐ Confirm tenant posture (PG / wallet / bank / brokerage / lending / etc.)
2. ☐ Confirm region(s) → load relevant pack(s)
3. ☐ Issue per-tenant CDE cell + HSM partition
4. ☐ Bind per-region payment-rails / identity / AML adapters
5. ☐ Verify license evidence (per-jurisdiction license number recorded in `platform-tenant-kernel.regulatory_packs`)
6. ☐ Run per-jurisdiction onboarding-evidence pack
7. ☐ Activate per-class HARD_DENY (PCI, FINANCIAL_KR_신용정보)
8. ☐ Bind autonomy-ceiling cap (T2 max for regulated capabilities)
9. ☐ Schedule first quarterly ASV scan + first internal vulnerability scan
10. ☐ Schedule annual QSA assessment (Level 1 SP)
11. ☐ Schedule annual penetration test (internal + external)
12. ☐ Schedule annual segmentation test (Req 11.4.5)
13. ☐ Activate audit-chain emission per CDE access
14. ☐ Activate KoFIU / FinCEN / equivalent reporting integration
15. ☐ Issue tenant DPIA per [`templates/dpia-template.md`](../templates/dpia-template.md)
16. ☐ Issue tenant-side AML training schedule
17. ☐ Bind incident-notification SLA (24h FSS in KR; 72h GDPR in EU; etc.)
18. ☐ Add row to [`COMPLIANCE-MATRIX.md §3.4`](../COMPLIANCE-MATRIX.md) per regulator

---

## 7. Open questions

1. **Tokenization vault: in-house build or partner with Adyen / Stripe / KICC / NICE?** In-house preferred per [TOOLCHAIN.md](../TOOLCHAIN.md) but PCI L1 SP attestation cost is high; partner reduces our CDE scope substantially.
2. **Direct BOK 결제망 participation OR via clearing bank?** Direct requires bank license; via clearing-bank reduces our license requirement but adds counterparty risk + fees.
3. **Crypto exchange / VASP — in-scope or anti-scope?** Currently anti-scope per PRD §3.3 unless founder ratifies; KSA / UAE / EU MiCA stories possible.
4. **Cross-border remittance corridors**: which to support first (KR↔US, KR↔JP, KR↔SE-Asia)?
5. **Robinhood-class brokerage**: full brokerage license or piggyback on existing brokerage partner?
6. **Custody for securities + crypto**: 자체 vs partner (한국예탁결제원 / Anchorage / Coinbase Custody)?
7. **Open Banking PISP/AISP in EU**: licensed in which member-state first (likely IE for English-speaking + DAS)?

---

## 8. Sources scanned

- PCI Security Standards Council: PCI-DSS v4.0 + v4.0.1 (2024); PCI Token Service Provider Standard; PCI P2PE
- KR FSC + FSS guidance
- KR 전자금융거래법, 전자금융감독규정, 신용정보법, 자본시장법, 은행법, 보험업법, 외국환거래법, 특금법
- KISA identity portal (https://identity.kisa.or.kr/)
- KR 금융결제원 (KFTC) Open Banking + MyData specs
- US: PCI-DSS, GLBA, BSA, FinCEN, OCC FFIEC, NY DFS Part 500, SEC, FINRA, SIPC, OFAC
- EU: PSD2, EBA RTS-SCA, GDPR, AMLD6, MiCA, DORA
- JP: 銀行法, 資金決済法, 金融商品取引法, 貸金業法, FSA guidance
- IN: RBI Master Directions, NPCI UPI, DPDP Act 2023, FEMA
- BR: BACEN Circular 3909, LGPD, Pix protocol
- KSA: SAMA CSF, SAMA Open Banking, PDPL
- UAE: UAE-CB, DIFC FSRA, ADGM FSRA, FATOORA
- AU: APRA CPS 234, ASIC, AUSTRAC, NPP
- SG: MAS PSA, MAS Notices, PDPA-SG
- Industry references: Toss / KakaoBank / KakaoPay / NaverPay public materials; PayPal / Stripe / Adyen / Wise / Robinhood / Coinbase compliance docs
- Existing Oyatie consolidated docs

*Footer regenerated whenever this doc is edited.*
