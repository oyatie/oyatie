---
id: ADR-0010
status: Rejected
doc_status: published
---

# ADR-0010: Regional pack architecture — canonical seams + per-locale plug-ins for regulatory, compliance, i18n, currency, calendar, tax, identity, payment, address, ecosystem partners, content safety, ad policy, industry data models, and vendor partners

> **Status:** Proposed
>
> **Amendment note — 2026-06-02 platform-readiness:** the `{oya,cloud}` pure split does not automatically delete
> canonical pack authoring roots. `regional-packs/<pack-id>/` remains valid for shared/versioned pack authoring until
> ADR-0010/ADR-0064 are explicitly superseded; service-shaped code accidentally placed under pack roots is sprawl and
> migrates to `{oya,cloud}/<service>` or `libs/` after inventory proves ownership.
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `regional-packs` + `council-architecture`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0002, ADR-0003, ADR-0008, ADR-0009, ADR-0011, ADR-0012

---

## Context

Korea-as-launch-locale was the prior framing; the 2026-05-09 reframing retired it in favor of **canonical-architecture + regional-pack plug-ins** so multiple markets onboard in parallel rather than retrofit Korea-specific assumptions per locale. Every market — KR, JP, US, EU, IN, BR, KSA, UAE, ANZ, SG, plus successor-IPs — has a similar regulatory moat that prefers in-locale integrated providers; Oyatie's posture is to ride every window in parallel by treating each market as a *swappable pack* that plugs into well-defined seams.

Without explicit pack architecture, three failure modes recur: (a) Korea-only assumptions leak into the kernel ("`region` is always KR-Seoul1", "tax invoice is `전자세금계산서`"); (b) per-locale work duplicates across teams (each vertical re-implements address validation per region); (c) regulator changes in one locale require kernel patches that risk other locales. The seam contract makes a regional pack a versioned, signed, swappable bundle that supplies *all* per-locale concerns to the canonical architecture.

---

## Decision

We adopt **canonical-architecture + regional-pack plug-ins** as the locale model. The architecture is locale-agnostic; every per-locale concern lives in a regional pack that plugs into published seams. One pack per market, versioned and signed.

### Pack contents (per pack)

| Pack section | What's inside |
|---|---|
| `regulatory` | Regulator names, control-mapping tables, evidence-collection cadence, ADR cross-references (PIPA, GDPR, HIPAA, DPDP, LGPD, PDPL, APPI, Privacy Act AU) |
| `compliance_packs` | Per-vertical-per-locale: healthcare regulator (MFDS, FDA, EMA, PMDA, MHRA, CDSCO, ANVISA, SFDA, TGA, HSA), fintech regulator (FSC, OCC, FCA, FSA-JP, RBI, BACEN, SAMA, UAE-CB, ASIC, MAS), payment scheme (NACHA, FedNow, RTP, Pix, UPI, FPS, KFTC) |
| `i18n` | Language(s), morphology / tokenizer (mecab-ko, MeCab-ja, NLTK-en, IndicNLP, Stanza), date/time, address normalization, name conventions, RTL support |
| `currency` | ISO-4217 currency, decimal precision, formatting, FX-source identity |
| `calendar` | Holidays, working days, fiscal year, school year, business-quarter convention |
| `tax` | Tax-invoice format (전자세금계산서, 適格請求書, EU per-country e-invoicing, IN GST, BR NF-e, KSA FATOORA, UAE e-invoicing), tax-id format, tax-engine selection |
| `identity_providers` | Local IdP surfaces (KR `본인확인서비스`, JP `マイナンバーカード`, EU eIDAS, US Login.gov, IN Aadhaar, BR ICP-Brasil, KSA Absher, UAE UAEPass, ANZ Digital ID, SG SingPass) |
| `payment_rails` | Local rails (KR `카카오페이`/`네이버페이`/`토스`/`계좌이체`, JP `振込`/Pay, US ACH/Wire/RTP, EU SEPA/SEPA-Inst, IN UPI/RTGS, BR Pix, KSA SADAD/Mada, UAE UAEFTS/AaniPay, AU NPP, SG FAST) |
| `address_book` | Address validation, post code, geocoding source |
| `ecosystem_partners` | Integrations the locale expects (Naver / Kakao in KR; Yahoo!JP / LINE in JP; Google / Facebook in US-EU; WeChat / Alipay where ToS allows; Apple / Google ID universally; KR `정부24` / EU `Once-Only`) |
| `content_safety` | Local content moderation (KR `청소년보호법` + `정보통신망법` + `게임물관리위원회`; JP `児童ポルノ法` + `不正アクセス禁止法`; US COPPA + child-safety; EU DSA + online-platform; IN IT Rules 2021; UAE federal media council) |
| `ad_policy_gate` | Local ad review (KR `의료광고`/`금융광고`/`정치광고`; US FTC/AdSafe; EU consumer-protection; ANZ TGA medical advertising; IN ASCI; KSA GAEH/SFDA) |
| `industry_data_models` | Per-locale clinical coding extensions (KR-EDI 보건의료, US-LOINC, JP-ReceiptCode), labor (KR `통상임금`, JP `賞与`, US W-2/1099), accounting (K-IFRS, J-GAAP, US-GAAP, IFRS, Ind-AS) |
| `vendor_partners` | Local cloud / colo (KR Naver Cloud / NHN / KT / Kakao; JP Sakura / IDC Frontier; EU OVH / Hetzner / Scaleway / IONOS; IN Yotta / Reliance Jio Cloud) |

### Seam contracts (canonical → pack plug-in)

The architecture publishes seams; packs supply impls.

```rust
// crates/oya-tenancy-regional-pack-kernel
pub trait RegulatoryPack {
    fn pack_id(&self) -> RegulatoryPackId;
    fn controls(&self) -> &[ControlMapping];
    fn evidence_emission_cadence(&self) -> EvidenceCadence;
    fn adr_references(&self) -> &[AdrId];
}

pub trait Tokenizer {
    fn tokenize(&self, text: &str) -> Vec<Token>;
    fn morphological_features(&self, token: &Token) -> Vec<Morpheme>;
}

pub trait TaxInvoiceFormatter {
    fn format(&self, invoice: &Invoice) -> Result<TaxInvoiceArtifact, TaxFormatError>;
    fn submit_to_authority(&self, artifact: &TaxInvoiceArtifact) -> Result<AuthorityReceipt, SubmissionError>;
}

pub trait IdentityProvider {
    fn issue_session(&self, user_assertion: UserAssertion) -> Result<Session, IdpError>;
    fn verify(&self, claim: &IdpClaim) -> Result<VerifiedClaim, IdpError>;
}

pub trait PaymentRail {
    fn rail_id(&self) -> PaymentRailId;
    fn quote(&self, payment: &PaymentIntent) -> Result<PaymentQuote, RailError>;
    fn execute(&self, payment: &PaymentIntent) -> Result<PaymentReceipt, RailError>;
}

pub trait AddressValidator {
    fn validate(&self, address: &RawAddress) -> Result<NormalizedAddress, AddressError>;
    fn geocode(&self, address: &NormalizedAddress) -> Result<GeoPoint, GeocodeError>;
}

pub trait LocalAdPolicy {
    fn pre_publish_review(&self, ad: &Ad) -> Result<ReviewVerdict, AdPolicyError>;
    fn applicable_categories(&self) -> &[AdCategory];
}

pub trait ContentSafetyRules {
    fn classify(&self, content: &Content) -> Result<SafetyClassification, ClassificationError>;
    fn applicable_subject_classes(&self) -> &[SubjectClass];   // ADR-0008
}

pub trait LocaleFormatter {
    fn currency(&self, amount: &MonetaryAmount) -> String;
    fn date(&self, dt: &chrono::DateTime<chrono::FixedOffset>) -> String;
    fn personal_name(&self, name: &PersonalName) -> String;
}
```

### Per-pack residency

Each pack declares the residency classes it supports. A tenant's `Tenant.residency` (ADR-0002) must intersect the pack's supported set. The KR pack supports `strict_kr` and `kr_with_us_failover`; the EU pack supports `strict_eu`, `eu_with_us_failover`, `eu_with_global_failover`; etc. Cross-pack residency conflicts are detected at tenant onboarding.

### Pack lifecycle

- **Authoring**: pack lives at `regional-packs/<pack-id>/` with its own `pack.yaml`, seam impls under `crates/oya-pack-<pack-id>-{regulatory,tax,idp,payment,address,...}-*`, and per-pack regulator-watch lane.
- **Validation**: `oya-governance-pack` validates that the pack supplies an impl for every required seam; missing seam = pack rejected.
- **Signing**: each pack release is Cosign-signed (per ADR-0013 supply-chain bar) and published to the pack registry.
- **Versioning**: per-pack semver; tenant binding pins pack version; pack upgrade is a controlled migration with audit emission.
- **Regulator-change watch**: per-pack lane subscribes to regulator-publication feeds; updates auto-open a pack-revision PR.

### Boundary

- Applies to: every cross-microservice surface that touches per-locale concerns (any axis that takes locale, currency, tax, IdP, payment, address, content safety, ad policy, industry data model, or vendor partner as input).
- Does not apply to: kernel surfaces that are explicitly locale-agnostic (Tenant kernel itself, audit chain, eventing backbone, OG kernel).

---

## Consequences

### Positive

- Locale onboarding becomes "ship a pack" rather than "patch the kernel"; multiple markets onboard in parallel.
- Regulator-change drift is contained per pack; KR `의료법` revision does not require a US kernel patch.
- Per-locale specialist teams own per-pack scope; `regional-packs/oya-pack-kr` ships KR expertise without blocking JP / US / EU work.
- Each pack carries its own continuous-evidence emission (ADR-0003) per its regulatory_packs set — auditor evidence is per-pack scoped.

### Negative

- Per-pack maintenance burden grows linearly with markets; mitigated by automation (regulator-watch lane) and per-pack ownership clarity.
- Cross-pack contracts (e.g. cross-region replication for tenants spanning KR + JP) require explicit declaration; not free.
- Pack version lag — a tenant on pack version N may behave differently from version N+1; mitigated by per-tenant version pinning + controlled migration.

### Operational

- On-call: per-pack regulator-watch lane monitored daily; `EVT-REGULATORY-CHANGE-DETECTED` opens a pack-revision PR.
- Runbooks: `runbooks/pack-onboarding.md`, `runbooks/pack-version-upgrade.md`, `runbooks/cross-pack-tenant-residency.md`, `runbooks/regulator-publication-feed-health.md`.
- CI: `oya-governance-pack` (seam coverage), `oya-governance-pack-residency` (no cross-pack residency conflict at onboarding).
- Per-pack scorecard: regulator-coverage %, seam-coverage %, evidence-emission %, regulator-watch lane health.

---

## Alternatives considered

### Alternative A — KR-first kernel + per-locale forks

- **Pros:** Korea ships fastest.
- **Cons:** every other market is a retrofit; kernel patches risk KR; LEDG-024 captured the prior gap.
- **Rejected because:** parallel-market posture requires locale-agnostic kernel.

### Alternative B — Per-locale microservice (one service tree per market)

- **Pros:** strong per-locale autonomy.
- **Cons:** breaks cohesion at the substrate; tenant boundary fragments per locale.
- **Rejected because:** ADR-0001.

### Alternative C — Pack as configuration only (no per-pack code)

- **Pros:** simpler.
- **Cons:** seam impls (tokenizer, tax formatter, IdP) require code; YAML-only is insufficient.
- **Rejected because:** seam impls are real code.

---

## Open questions

1. **Q1.** Defense / public-safety pack — separate vehicle (`oya-pack-defense`) or carve-out inside per-region pack? Default: separate vehicle per LEDG-017 + COMPLIANCE-MATRIX §6 Q1. → ADR-0012.
2. **Q2.** Per-state US sub-packs (NY SHIELD, IL BIPA, TX TDPSA, CA CPRA) — overlay model or sub-packs? Default: overlay packs that depend on `oya-pack-us`. → owner: `regional-packs/oya-pack-us`.
3. **Q3.** Cross-pack tenant (e.g. JP tenant with KR subsidiary) — per-subsidiary tenant or per-tenant cross-pack residency? Default: per-subsidiary tenant; cross-tenant identity link per ADR-0008. → owner: `council-privacy`.
4. **Q4.** Pack signing key rotation cadence — quarterly? Default: per-release with key rollover quarterly. → ADR-0013.
5. **Q5.** Pack vendor_partners disclosure — public or per-tenant only? Default: public-by-default; sovereignty-tier tenants opt out. → owner: `regional-packs`.

---

## References

- `docs/DESIGN.md` §12 (Regional Pack Architecture; full pack section list and seam definitions)
- `docs/COMPLIANCE-MATRIX.md` §2 (cross-region regulator inventory — drives the initial pack set)
- `docs/CONTRADICTION-LEDGER.md` LEDG-014 (KR binders missing), LEDG-024 (KR identity coverage gap), LEDG-017 (defense scope)
- `docs/PRD.md` §1 ("cloud sovereignty is a global window, not a Korea-specific one"), §3.1 commitment 3 (canonical-architecture + regional-pack plug-ins)
- ADR-0001 (cohesion), ADR-0002 (Tenant.regulatory_packs + residency), ADR-0003 (per-pack evidence emission), ADR-0008 (tenant-class overrides per regulator), ADR-0009 (per-cell residency), ADR-0011 (catalog cross-microservice contracts include pack-bound seams), ADR-0012 (axis admission — defense scope decision)
