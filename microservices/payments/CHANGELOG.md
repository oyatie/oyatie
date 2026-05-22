---
doc_class: Changelog
microservice: payments
status: Accepted
date: 2026-05-20
doc_status: published
related_adrs: [ADR-0258]
---

# Payments µservice — Changelog

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and SemVer per ADR-0258.

## [Unreleased]

### Added
- Initial full doc-suite buildout (Wave-3-A) — PRD (Slice 8) + 110+ artifact suite landed.
- 7 bounded contexts scaffolded: `charge`, `refund`, `payout`, `settlement`, `kyc-kyb`, `dispute`, `subscription-lifecycle`.
- PSP-adapter trait + per-PSP adapters (Stripe, Adyen, Toss, KakaoPay, LINE Pay, WeChat Pay, Alipay) targeted for M02.
- Cedar fragments (charge / payout / refund / sub-merchant-onboarding / abuse-defence / auditor-scope / ci-scope) authored.
- OpenAPI 3.2.0 + AsyncAPI 3.1.0 + proto3 contracts authored.
- SLO manifests (charge-api availability / latency, refund availability, payout completion, dispute response latency, webhook delivery success).
- Compliance packs: PCI-DSS L1 v4, KR-FSS, EU PSD2 / SCA, US state MTL, CCPA / CPRA, AU-AML, BR-LGPD, IN-RBI, CN-PIPL, COPPA refusal.

### Pending (M02-foundation)
- Implementation plans IP-001 through IP-025 — domain / kernel / usecase / adapter / REST / gRPC / worker / app crates.
- First non-trivial consumer (messenger sticker store) wired.

## [0.0.0] — 2026-05-20 (doc-suite-only seed)

### Added
- `PRD.md` (1612 lines, Slice 8).
- Full doc-suite at PR-143 baseline + operating-bar ≥100 artifacts (this changelog).

## Versioning policy

Per ADR-0258 (API versioning + deprecation cadence):

- **MAJOR**: breaking changes to OpenAPI 3.2.0 / AsyncAPI 3.1.0 / proto3 contracts. Minimum 18-month sunset.
- **MINOR**: backward-compatible addition (new endpoints, new event channels, new fields with defaults).
- **PATCH**: bug fix / clarification / non-behavioural change.

Internal Rust crate versions follow Cargo SemVer.

## References

- [ADR-0258 — API versioning + deprecation cadence](../../docs/decisions/ADR-0258-api-versioning-deprecation-cadence.md).
- [`PRD.md`](PRD.md).
- [`README.md`](README.md).
