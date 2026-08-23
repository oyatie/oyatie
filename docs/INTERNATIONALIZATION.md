---
purpose: Oyatie — Internationalization (i18n)
doc_status: published
---

# Oyatie — Internationalization (i18n)

> **Status:** Draft v0.1 — 2026-05-09.
> **Owner:** `council-architecture` + `gtm-marketing`.
> **Companion:** [DESIGN.md §12 Regional Pack Architecture](DESIGN.md), [COMPLIANCE-MATRIX.md](COMPLIANCE-MATRIX.md).

## 1. Per-locale matrix (initial)

| Locale | Default region | Currency | Calendar | Tax invoice format | Identity provider | Payment rails | Address normalization |
|---|---|---|---|---|---|---|---|
| ko-KR | KR-Seoul | KRW | gregorian + KR holidays | 전자세금계산서 (NTS) | 본인확인서비스 / CI / DI / KakaoTalk auth / Naver auth / 공동인증서 / 금융인증서 | 카카오페이 / 네이버페이 / 토스 / 계좌이체 / 신용카드 | 도로명 + 지번 |
| ja-JP | JP-Tokyo | JPY | gregorian + JP era + holidays | 適格請求書 | マイナンバーカード / 法人番号 | MUFG / Mizuho / SMBC / Pay-easy / クレジットカード | 〒 + 都道府県市区町村 |
| en-US | US-East | USD | gregorian + US fed holidays | per-state sales tax | Login.gov / SSO / SAML | ACH / Wire / RTP / FedNow / card | USPS |
| en-GB / en-EU | EU-DE | EUR / GBP | gregorian + per-country holidays | per-country e-invoicing (DE Zugferd / IT FatturaPA / FR Chorus / ES Facturae / PL KSeF / RO eFactura …) | eIDAS / per-country IdP | SEPA / SEPA-Inst / per-country | per-country format |
| hi-IN / en-IN | IN-Mumbai | INR | gregorian + IN holidays | GST e-invoicing | Aadhaar / DigiLocker | UPI / RTGS / IMPS / NEFT / cards | India PIN |
| pt-BR | BR-Sao-Paulo | BRL | gregorian + BR holidays | NF-e | gov.br / ICP-Brasil | Pix / TED / DOC / boleto / cards | CEP |
| ar-SA / en-SA | KSA-Riyadh | SAR | gregorian + Hijri + KSA holidays | FATOORA | Absher / Nafath | SADAD / Mada | Saudi addressing |
| ar-AE / en-AE | UAE-Dubai | AED | gregorian + UAE holidays | UAE e-invoicing | UAE-PASS | UAEFTS / AaniPay | Makani |
| en-AU / en-NZ | AU-Sydney | AUD / NZD | gregorian + AU/NZ holidays | per-country | myGovID | NPP / OSKO / cards | per-country |
| en-SG / zh-SG | SG | SGD | gregorian + SG holidays | per-IMDA | Singpass | FAST / PayNow / cards | SG postal |
| es-MX | MX | MXN | gregorian + MX holidays | CFDI | e.firma | SPEI / cards | per-MX |
| zh-Hans-CN / zh-Hant-TW / zh-Hant-HK | per-locale | per-locale | per-locale + lunar | per-locale | per-locale | per-locale | per-locale |

## 2. UI / content translation policy

Per [DOCUMENTATION.md §8](DOCUMENTATION.md):
- English is canonical
- Per-pack translations for: tutorials, how-to guides, concepts, admin docs, Studio UI, plugin docs
- Auto-generated reference docs are locale-neutral (code identifiers stay in English)
- ADRs + consolidated PM docs stay English (engineering canon)

## 3. Currency + tax engines

- Multi-currency at the platform tenancy kernel (`platform-tenant-kernel.billing_account.currency`)
- Per-region tax-engine adapter via [DESIGN §12.2 seam](DESIGN.md): KR NTS, JP NTA, US per-state, EU per-country e-invoicing, IN GST, BR NF-e, KSA FATOORA, UAE FTA
- FX rate: per-day from Bank of Korea / FRBK / ECB / Reserve Bank of India; per-pack source

## 4. Time zones

- Server time: UTC always
- Display time: per-tenant default + per-user override
- KR: KST (UTC+9), no DST
- US: per-state DST
- EU: per-country DST (CEST/CET, BST/GMT)
- Per-pack `LocaleFormatter` impl

## 5. Sort order + character handling

- Korean: Hangul collation (NFC); per-locale sort
- Japanese: per-locale (JIS / Unicode collation)
- Chinese: pinyin / stroke-count / Unicode default
- Arabic / Hebrew: RTL; per-locale Unicode collation
- Devanagari (Hindi): NFC + per-locale collation
- Per-pack `Tokenizer` for search per [DESIGN §12.2](DESIGN.md)

## 6. Number / date formatting

- Per-pack `LocaleFormatter` covering: thousands separator, decimal separator, date format, time format, ordinal, currency display, percentage

## 7. Sources
ADR-0017 (domain naming canon), ADR-0008 (multi-jurisdiction policy), ADR-0034 (form schema), per-pack research, [GLOSSARY.md §9](GLOSSARY.md) KR↔EN parity, [DESIGN.md §12](DESIGN.md).
