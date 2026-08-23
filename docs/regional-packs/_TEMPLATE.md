---
doc_status: published
---

# Regional Pack: pack-<region>

> Template for every regional pack under [`regional-packs/`](.). Copy verbatim, then fill in. Status moves through `draft → preview → stable → GA` per ADR-0010-regional-pack-architecture.

## 0. Pack metadata

- **Pack id:** `pack-<region-code>` (e.g. `pack-kr`, `pack-jp`, `pack-us`, `pack-eu`, `pack-in`, `pack-br`, `pack-ksa`, `pack-ae`, `pack-au`, `pack-sg`, ...)
- **Region:** <full region name + scope>
- **Locale codes:** <e.g. ko-KR, en-KR>
- **Status:** draft / preview / stable / GA
- **Owning team:** [`teams/regional-packs/`](../teams/regional-packs/)
- **Per-pack maintainer:** TBD (per-pack hire)
- **Catalog record:** `registry/catalog/regional-packs/pack-<region>.yaml`

## 1. Regulatory binding

| Regulator | Statute / framework | Implementing trait impl | Cadence | Evidence |
|---|---|---|---|---|
| (per-region regulator 1) | (statute) | `platform-regulatory-kernel::RegulatoryPack` impl | (cadence) | (where evidence emits) |
| ... | | | | |

## 2. Compliance packs (per vertical)

| Vertical | Local regulator | Per-vertical override |
|---|---|---|
| Healthcare | (e.g. KR MFDS / US FDA / EU EMA / JP PMDA / IN CDSCO / BR ANVISA / KSA SFDA / AU TGA / SG HSA) | (PHI hard-deny per [PRIVACY-PROGRAM §2.2.3](../PRIVACY-PROGRAM.md)) |
| Fintech | (e.g. KR FSC / US OCC + FFIEC / EU EBA + DORA / JP FSA / IN RBI / BR BACEN / KSA SAMA / UAE UAE-CB / AU APRA + ASIC / SG MAS) | (per [`standards/fintech-compliance.md`](../standards/fintech-compliance.md)) |
| Education | (per-region) | (CHILDREN_UNDER_14 hard-deny per minors-protection) |
| Public-sector | (per-region procurement + cyber) | (per-region) |
| ... per other verticals as applicable | | |

## 3. i18n

- **Languages:** <list>
- **Tokenizer / morphology impl:** <e.g. mecab-ko, MeCab-ja, NLTK-en, IndicNLP, Stanza, ...>
- **Date format:** <ISO 8601 + locale form>
- **Time format:** <12h / 24h>
- **Address normalization:** <impl>
- **Name conventions:** <surname-first / given-first / per-locale>
- **RTL support:** yes / no
- **Sort collation:** <NFC + locale collation impl>

## 4. Currency

- **ISO 4217:** <e.g. KRW, USD, EUR, JPY, INR, BRL, SAR, AED, AUD, SGD>
- **Decimal precision:** <0 for KRW/JPY; 2 for most>
- **Currency display:** <prefix / suffix / symbol>
- **FX rate source:** <e.g. BoK / FRBK / ECB / RBI>

## 5. Calendar

- **Holiday list source:** <per-region authoritative>
- **Working days:** <Mon-Fri / region-specific>
- **Fiscal year:** <per-region>
- **School year:** <per-region>
- **Business-quarter convention:** <per-region>

## 6. Tax

- **Tax-invoice format:** <e.g. KR 전자세금계산서 / JP 適格請求書 / IN GST e-invoicing / BR NF-e / KSA FATOORA / EU per-country>
- **Tax-id format:** <e.g. KR 사업자등록번호 / US EIN / EU VAT / IN GSTIN / BR CNPJ>
- **Tax-engine adapter:** `platform-billing-tax-kernel::TaxInvoiceFormatter` impl

## 7. Identity providers

| Provider | Use case | Implementing trait impl |
|---|---|---|
| (e.g. KR 본인확인서비스 / NICE / KCB / SCI Plus) | Real-name verification | `platform-identity-kernel::IdentityProvider` impl |
| (e.g. JP マイナンバーカード) | National ID | (impl) |
| (e.g. US Login.gov / SAML / OIDC) | SSO | (impl) |
| (e.g. EU eIDAS) | National ID + cross-border | (impl) |
| (e.g. IN Aadhaar) | National ID | (impl) |
| ... | | |

## 8. Payment rails

| Rail | Use case | Implementing trait impl |
|---|---|---|
| (e.g. KR 카카오페이 / 네이버페이 / 토스 / 계좌이체 / 신용카드) | Tenant payment | `saas-billing-rail-kernel::PaymentRail` impl |
| (e.g. JP 振込 / Pay-easy) | Tenant payment | (impl) |
| (e.g. US ACH / Wire / RTP / FedNow / card) | Tenant payment | (impl) |
| ... | | |

## 9. Address book

- **Address-validation impl:** `platform-address-kernel::AddressValidator`
- **Postal-code format:** <per-region>
- **Geocoding source:** <e.g. KR road-name + 지번; JP 〒 + 都道府県市区町村; US USPS; ...>

## 10. Ecosystem partners

| Partner | Integration | Status |
|---|---|---|
| (e.g. Naver / Kakao for KR; Yahoo!JP / LINE for JP; Google / Facebook globally) | (description) | (status) |

## 11. Content safety

- **Local content moderation rules:** <per-region>
- **Implementing trait impl:** `platform-content-safety-kernel::ContentSafetyRules`

## 12. Ad-policy gate

- **Local ad review workflows:** <per-region>
- **Implementing trait impl:** `ads-policy-kernel::LocalAdPolicy`

## 13. Industry data models

| Vertical | Per-locale extension |
|---|---|
| Healthcare | (e.g. KR 보건의료 EDI / US LOINC / JP ReceiptCode) |
| Labor / payroll | (e.g. KR 통상임금 / JP 賞与 / US W-2/1099 / EU GDPR Art 9) |
| Accounting | (e.g. K-IFRS / J-GAAP / US-GAAP / IFRS / Ind-AS) |
| ... | |

## 14. Vendor partners

| Partner | Type |
|---|---|
| (e.g. KR Naver Cloud / NHN / KT / Kakao Cloud) | Local cloud peer |
| (e.g. KR PG: KICC / NICEpay / Toss Payments) | Local payment partner |
| ... | |

## 15. Per-pack residency

- **Default residency class:** <e.g. strict_kr / kr_with_us_failover / global>
- **Cross-border transfer constraints:** <per-jurisdiction>

## 16. Per-pack roadmap

- W-Cloud-Preview onboarding: <date or status>
- Per-vertical pack-extension rollout: <per vertical>
- Stable: <criteria>
- GA: <criteria>

## 17. Sources scanned

- <per-region statute / regulator / industry sources>
- ADR-0010-regional-pack-architecture
- [INTERNATIONALIZATION.md](../INTERNATIONALIZATION.md)
- [COMPLIANCE-MATRIX.md](../COMPLIANCE-MATRIX.md)
