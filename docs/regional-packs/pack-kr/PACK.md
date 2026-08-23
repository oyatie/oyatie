---
doc_status: published
---

# Regional Pack: pack-kr (South Korea)

> Initial KR pack content. Authored from `_TEMPLATE.md`. Status moves through `draft → preview → stable → GA` per ADR-0010-regional-pack-architecture.

## 0. Pack metadata

- **Pack id:** `pack-kr`
- **Region:** South Korea (대한민국)
- **Locale codes:** ko-KR (primary), en-KR (secondary for English-speaking residents + global tenants)
- **Status:** draft (target preview at W-Cloud-Preview)
- **Owning team:** [`teams/regional-packs/`](../../teams/regional-packs/) (KR-pack maintainer council-owned)
- **Catalog record:** `registry/catalog/regional-packs/pack-kr.yaml`

## 1. Regulatory binding (KR statutes + agencies)

| Regulator | Statute / framework | Cadence | Implementing |
|---|---|---|---|
| KISA (한국인터넷진흥원) | KR Internet & Security Agency | continuous | per-axis security adapters; CSAP path |
| PIPC (개인정보보호위원회) | Personal Information Protection Commission | per-incident + annual | DSR cascade per [PRIVACY-PROGRAM §2.2.9](../../PRIVACY-PROGRAM.md) |
| MFDS (식품의약품안전처) | Ministry of Food and Drug Safety | per healthcare release | clinical-safety per [products/vertical-healthcare](../../products/vertical-healthcare/PRD.md) |
| FSC (금융위원회) + 금감원 (FSS) | Financial Services Commission + Supervisory Service | continuous + per-incident 24h | per [`standards/fintech-compliance.md`](../../standards/fintech-compliance.md) |
| KCC (방송통신위원회) | Korea Communications Commission | per ad/comms | per content-safety + ad-policy gate |
| NIS (국가정보원) | National Intelligence Service | per public-sector tenant | public-sector vertical |
| NTS (국세청) | National Tax Service | per invoice | 전자세금계산서 integration |
| KFTC (금융결제원) | Korea Financial Telecommunications & Clearings Institute | per payment / MyData event | payment-rails + MyData adapters |
| BOK (한국은행) | Bank of Korea | per RTGS / MyData event | payment-rails (digital bank) + foreign-currency |
| KSD (한국예탁결제원) | Korea Securities Depository | per securities transaction | brokerage |
| 조달청 (Public Procurement Service) | per public-sector procurement | per public-sector tenant | public-sector vertical |
| 게임물관리위원회 (GRAC) | Game Rating and Administration Committee | per game/gamified content | content-safety |
| 한국디지털광고협회 (KODA) | KR Digital Ad Industry | per ad campaign | ad-policy gate |
| 의료광고심의위원회 | KR Medical Ad Review Committee | per medical ad | ad-policy gate |

### KR statutes (key)
- 개인정보보호법 (PIPA)
- 정보통신망법 (Information & Communications Network Act)
- 신용정보법 (Credit Information Act)
- 전자금융거래법 (EFTA)
- 전자금융감독규정 (FSC supervisory regulation)
- 자본시장법 (Capital Markets Act)
- 은행법 (Banking Act)
- 보험업법 (Insurance Business Act)
- 외국환거래법 (FETA)
- 특금법 (Specific Financial Transactions Act / AML)
- 의료법 + 약사법
- 청소년보호법
- 공공정보법 + 정보공개법
- 노동기준법 (Labor Standards Act)
- 근로자퇴직급여보장법
- 산업안전보건법
- 클라우드 컴퓨팅 발전법

### Certifications (target)
- CSAP (Cloud Security Assurance Program) — Level 하/중/상 per surface
- K-ISMS-P (Information Security + Privacy Management System certification)
- KCMVP (KR cryptographic module validation) — KCMVP-validated HSM (6-9 month procurement lead)
- 정보보호제품 인증 (per-product InfoSec certification)
- 개인정보보호인증 (PIPL-cert)
- 클라우드보안인증 (KISA cloud security)
- KS X ISO/IEC 27001 (KR-localized)

## 2. Compliance packs (per vertical)

| Vertical | Local regulator | Per-vertical override |
|---|---|---|
| Healthcare | MFDS + 의료법 + 약사법 | PHI hard-deny per [PRIVACY-PROGRAM §2.2.3](../../PRIVACY-PROGRAM.md); 환자번호 (RRN) handling per KISA-approved encryption |
| Fintech | FSC + 금감원 + 신용정보법 + 특금법 | PCI hard-deny + 신용정보 hard-deny; per [`standards/fintech-compliance.md`](../../standards/fintech-compliance.md) |
| Education | 청소년보호법 + 학교교육법 | CHILDREN_UNDER_14 hard-deny; per-tenant minor-protect mode |
| Public-sector | 정부24 + 조달청 + 공공정보법 | per-pack residency strict_kr; 망분리 (network separation) enforced |
| Industrial | 산업안전보건법 + KOSHA | OT safety per ADR-0033 |
| Logistics | 화물운송법 + 도로교통법 | per logistics vertical |
| Legal | 변호사법 + 개인정보보호법 | per legal vertical (attorney-client privilege handling) |
| Corporate | 노동기준법 + 근로자퇴직급여보장법 | KR statutory payroll depth (통상임금 / 휴일/야간 / 5/6일제 / 주52시간 / 연차 사용촉진) |
| Retail / Hospitality / Construction / Real-estate / Agriculture / Food | per-vertical | per-vertical |

## 3. i18n

- **Languages:** ko-KR (primary), en-KR (fallback)
- **Tokenizer / morphology:** mecab-ko (FFI from C++ via crate; LGPL — legal-isolation analysis required) OR khaiii (Apache-2 — preferred); long-horizon: in-house Rust port
- **Date format:** YYYY-MM-DD or 2026년 5월 9일
- **Time format:** 24h ("14:30")
- **Address normalization:** 도로명 (road-name) + 지번 (lot-number); both supported; 도로명 preferred
- **Name conventions:** 성 + 이름 (surname-first); given names may be 1-3 syllables; some hanja overlay
- **RTL support:** no
- **Sort collation:** Hangul collation (NFC + locale collation)

## 4. Currency

- **ISO 4217:** KRW
- **Decimal precision:** 0 (no minor unit)
- **Currency display:** "1,000원" or "₩1,000" (suffix preferred)
- **FX rate source:** Bank of Korea (BoK) daily reference rate

## 5. Calendar

- **Holiday list source:** KR National Holidays (per KR Public Holidays Act)
- **Working days:** Mon-Fri (some industries Sat half-day)
- **Fiscal year:** Jan 1 - Dec 31 (calendar year for most companies)
- **School year:** Mar 1 - Feb 28 (next year)
- **Business-quarter convention:** calendar quarters (Q1: Jan-Mar)

## 6. Tax

- **Tax-invoice format:** 전자세금계산서 (e-Tax Invoice) via NTS Hometax + 국세청 협력업체 (e.g. 빌소프트, 더존비즈온, 영림원)
- **Tax-id format:** 사업자등록번호 (10 digits, with check digit; format `xxx-xx-xxxxx`)
- **Tax-engine adapter:** `platform-billing-tax-kr-app` (in-pack impl of `TaxInvoiceFormatter`)

## 7. Identity providers

| Provider | Use case | Implementing trait impl |
|---|---|---|
| 본인확인서비스 (KISA-designated) | Real-name verification (CI/DI 88-byte/64-byte hashes) | `platform-identity-kr-bonin-app` |
| NICE / KCB / SCI Plus | Tertiary identity verification per service | per-provider adapters |
| 휴대폰 인증 (mobile carrier) | Phone-based KYC | per-carrier adapters |
| 아이핀 (i-PIN) | Alternative identity verification | (legacy adapter; migrating to 본인확인서비스) |
| 공동인증서 (joint cert) | Public Key Infrastructure (former 공인인증서) | adapter |
| 금융인증서 (financial cert) | KR finance-sector PKI | adapter |
| KakaoTalk auth | Simple-auth provider | adapter |
| Naver auth | Simple-auth provider | adapter |
| Toss auth | Simple-auth provider | adapter |
| PASS (휴대폰 통합인증) | Carrier-shared simple-auth | adapter |

## 8. Payment rails

| Rail | Use case | Implementing |
|---|---|---|
| 카카오페이 (KakaoPay) | Tenant + consumer payment | adapter |
| 네이버페이 (NaverPay) | same | adapter |
| 토스페이먼츠 (Toss Payments) | PG | adapter |
| 페이코 (Payco) | wallet | adapter |
| 신용카드 (credit cards via PG: KICC / NICEpay / Inicis / KG이니시스) | card payment | per-PG adapter |
| 계좌이체 (bank transfer via 금융결제원 KFTC API) | direct bank | adapter |
| 가상계좌 (virtual account) | per-tenant per-payment | adapter |
| 무통장입금 | offline bank | adapter |
| BOK 한은금융망 (RTGS) | high-value | (digital-bank only) |
| MyData / Open Banking API | aggregation | adapter |

## 9. Address book

- **Address-validation impl:** `platform-address-kr-app`
- **Postal-code format:** KR 5-digit (post-2015 revision; legacy 6-digit supported as fallback)
- **Geocoding source:** 도로명주소 안내시스템 (KR Road-Name Address) + KAKAO LBS / Naver Maps API per partner

## 10. Ecosystem partners

| Partner | Integration |
|---|---|
| Naver | Search, Naver Cloud (peer cloud), Naver 검색광고 |
| Kakao | KakaoTalk auth, KakaoPay, Kakao Cloud, Kakao Moment ads |
| Toss | PG, brokerage |
| KT / SKT / LGU+ | carrier APIs (SMS, voice, identity) |
| Samsung SDS / LG CNS / SK C&C / POSCO ICT | KR Big-4 SI partners |

## 11. Content safety

- 청소년 보호법 → minors-protect mode (CHILDREN_UNDER_14 + KR-specific 만19세 boundary)
- 정보통신망법 → 위치정보, 개인영상정보 (CCTV) handling
- 게임물관리위원회 → game/gamified-content rating
- 의료법 → 의료광고심의 review queue
- 신용정보법 → financial ad disclosure
- 정치자금법 → political ad transparency
- 명예훼손 (defamation) takedown SLA per KCC
- 자살예방 (suicide-prevention) → 1393 redirection on relevant searches/queries

## 12. Ad policy gate

- 의료광고심의위원회 review for medical
- 금감원 financial ad review
- 정치광고 transparency archive
- 청소년 광고 가이드라인
- KODA standards
- per-region adtech compliance

## 13. Industry data models (KR-specific extensions)

| Vertical | Per-locale extension |
|---|---|
| Healthcare | KR 보건의료 EDI / 환자번호 (RRN) handling / 보험사 (NHIS, KMD) integration |
| Labor / payroll | 통상임금 / 평균임금 / 휴일근로 / 야간근로 / 5/6일제 / 주52시간 / 연차 사용촉진 / 퇴직금 / 4대보험 |
| Accounting | K-IFRS + 국세청 reporting |
| Fintech | KR-신용정보 (NICE / KCB / KCS / SCI) integration |
| Education | 학사관리 (school-admin) integration |

## 14. Vendor partners

| Partner | Type |
|---|---|
| Naver Cloud / NHN Cloud / KT Cloud / Kakao Cloud | local cloud peer |
| KICC / NICEpay / Toss Payments / Inicis / KG이니시스 | KR PG partners |
| KFTC | clearing |
| BOK | settlement |
| KSD | securities settlement |
| KCB / NICE / SCI Plus | credit info / identity |
| 통신사 (KT / SKT / LGU+) | carrier APIs |
| 빌소프트 / 더존비즈온 / 영림원 | 국세청 협력업체 (e-tax invoice) |

## 15. Per-pack residency

- **Default residency class:** `strict_kr` (all personal data stays in KR; cross-region only via explicit consent)
- **Cross-border transfer constraints:** PIPA Art 28-8 — explicit user/tenant consent + purpose declaration; per-export audit-chain emission
- **Data center locations:** KR-Seoul (KR-Seoul1 cloud region; multiple AZs); KR-Chuncheon (Naver Cloud region); KR-Busan (secondary KR region future); KR-Sejong (public-sector region future)

## 16. Per-pack roadmap

- **W-Cloud-Preview onboarding:** (target wave)
  - Foundation pack content (regulatory binding, i18n, currency, calendar, tax-invoice, payment rails, identity providers)
  - First commercial tenant per `vertical-corporate` (KR Group HR/payroll/GL/mail) anchor
- **Per-vertical pack-extension rollout:**
  - Corporate (W-Vertical-Pilot): KR statutory payroll depth
  - Healthcare (W-Vertical-Fan-Out): MFDS + 의료법 + 의료광고심의
  - Fintech (W-Vertical-Fan-Out): per [`standards/fintech-compliance.md`](../../standards/fintech-compliance.md)
- **Stable:** CSAP-Level 중 + K-ISMS-P + KCMVP HSM operational + ≥ 10 tenants
- **GA:** CSAP-Level 상 + 조달청 procurement-eligible + ≥ 100 tenants

## 17. Sources scanned

- KR statutes (PIPA, EFTA, FETA, 신용정보법, 자본시장법, 은행법, 보험업법, 의료법, 약사법, 청소년보호법, 공공정보법, 노동기준법)
- KISA + PIPC + MFDS + FSC + KFTC + BOK + KSD + NTS + 조달청 + KCC + NIS public guidance
- ADR-0010-regional-pack-architecture
- [INTERNATIONALIZATION.md](../../INTERNATIONALIZATION.md)
- [COMPLIANCE-MATRIX.md §3.1 KR PIPA deep-dive](../../COMPLIANCE-MATRIX.md)
- [`standards/fintech-compliance.md`](../../standards/fintech-compliance.md)
