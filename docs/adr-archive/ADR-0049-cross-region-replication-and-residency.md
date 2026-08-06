---
id: ADR-0049
status: Superseded
superseded_by: [ADR-708]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0049: Cross-region replication + residency — per-pack default residency class, opt-in cross-region per consent, immutable post-create

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `council-architecture`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0008, ADR-0028, ADR-0029, ADR-0033, ADR-0034, ADR-0038, ADR-0043, ADR-0045

---

## Context

Data residency is the single most-litigated cloud-services compliance dimension. KR Personal Information Protection Commission (개인정보보호위원회) has issued multiple guidelines tightening cross-border transfer rules; EU regulators continue to test the legal viability of standard contractual clauses post-Schrems II/III; KR Article 28-8 ("개인정보의 국외 이전") added explicit per-class transfer requirements in 2024. The pack-of-19 foundation ADRs named residency as a need but did not pin (a) the per-pack default residency class, (b) the opt-in cross-region transfer mechanics, (c) the post-create immutability rule, (d) the residency-change-as-recreate semantics with DSR cascade.

The cohesion thesis applies: Cloud cells (per ADR-0028), Workspace cells (per ADR-0029), Vertical cells (per ADR-0033) all consume the same residency contract; per-tenant residency is a property of the tenant, not of any individual axis.

---

## Decision

We adopt **per-pack default residency class** (`strict_kr` / `kr_with_us_failover` / `global` at GA; per-pack additions later); **cross-region replication opt-in per residency class per Data Use Boundary** (per ADR-0008); **tenant residency immutable post-create** (residency change requires re-create of tenant + DSR cascade on the old cell); per-pack regulator-binding per region; cross-region transfer governance covers Schrems III + KR Art 28-8 + Russia data localization (if onboarded) + per-region rules.

### Residency classes

```rust
// crates/oya-tenancy-residency-kernel
pub enum ResidencyClass {
    /// All data + all replicas + all backups + all derived data stay in KR cells
    StrictKr,
    /// Primary in KR; cold/warm replica in US permitted with per-class consent
    KrWithUsFailover,
    /// Multi-region for performance; per-tenant per-class fine-tuned
    Global,
    /// Per-pack additions (eu_only, jp_only, etc.) onboarded via ADR amendment
    PerPack(PerPackResidency),
}

pub struct PerPackResidency {
    pub allowed_primary_regions: Vec<RegionId>,
    pub allowed_replica_regions: Vec<RegionId>,
    pub forbidden_regions: Vec<RegionId>,
    pub regulator_overlay: RegulatorOverlay,
}
```

### Per-pack defaults

| Pack / vertical | Default residency class |
|---|---|
| KR healthcare | `StrictKr` |
| KR fintech | `StrictKr` (per 「전자금융감독규정」 §15 + KR FSC 2024 cloud guidance) |
| KR public-sector | `StrictKr` (per 「전자정부법」 + 「공공기관의 정보공개에 관한 법률」) |
| KR education K12 | `StrictKr` |
| KR general SaaS | `KrWithUsFailover` |
| EU healthcare | per-pack `eu_only` (GDPR + EHDS) |
| EU fintech | per-pack `eu_only` (GDPR + DORA) |
| EU general | per-pack `eu_only` |
| US general | `Global` (US-primary) |
| JP general | per-pack `jp_only` |
| Global multi-region tenant | `Global` (per-class fine-tuned) |

### Cross-region replication opt-in per residency class

For `StrictKr`:

- Cross-region replication forbidden.
- Backups encrypted-at-rest with per-cell HSM partition (per ADR-0043) and stored exclusively in KR cells.
- DR within KR (KR-Seoul1 ↔ KR-Chuncheon).

For `KrWithUsFailover`:

- Primary writes in KR.
- Cold/warm replica permitted in US with per-class consent receipt (per ADR-0008 DUBO).
- Replication is opt-in per class — `PHI` / `PCI` / `Sensitive-PIPA-Art23` always denied.
- Per-replica encryption-at-rest with per-region HSM (per ADR-0043).

For `Global`:

- Per-class per-region map.
- Sensitive classes default to home region only.
- Tenant admin can opt in per class.

### Tenant residency immutable post-create

Once a tenant is created with a residency class, the class is immutable. Changing residency requires:

1. Create a new tenant in the desired residency class.
2. Run per-tenant migration (Workspace mail / Drive / Vertical record / etc.) from old to new.
3. Run DSR cascade on the old tenant (per ADR-0038).
4. Old tenant audit-chain-sealed; deletion certificate issued to tenant DPO.

Reason: in-place residency change is not auditable for "data did not transit the forbidden region during change" — the only credible assurance is migrate + erase.

### Per-pack regulator binding

Per region:

| Region | Regulator binding |
|---|---|
| KR | 개인정보보호위원회 (PIPA enforcement); 금융감독원 (FSS, financial); 식품의약품안전처 (clinical); 방송통신위원회 (KCC, telecom/messaging); KISA (security) |
| EU | per-member-state DPA + EDPB; ENISA (security); EBA / EIOPA / ESMA (financial) |
| US | FTC; per-state AG (CCPA/CPRA, etc.); HHS OCR (HIPAA); SEC / FINRA / CFPB (financial) |
| JP | 個人情報保護委員会 (PPC); 金融庁 (FSA) |
| ZH (PRC) | 国家网信办 (CAC) — _not onboarded at GA; ADR-amendment if onboarded_ |
| RU | Роскомнадзор — _not onboarded at GA; founder ratification gate_ |

Per-pack regulator binding ships in the regional pack and binds at runtime via Cedar policy + Data Use Boundary.

### Cross-region transfer governance

For any cross-region data flow:

1. **Legal basis check.** PIPA Art 28-8 + GDPR Art 44-49 + per-region equivalent.
2. **Per-class consent verification** (per ADR-0008 DUBO).
3. **Cedar policy** (per ADR-0007) gates the transfer.
4. **Encryption in transit** (mTLS per ADR-0044) + **encryption at rest** (per ADR-0043) at destination.
5. **Audit-chain emission** (per ADR-0003) of the transfer event with source region + destination region + class + consent receipt.
6. **Per-tenant trust portal entry** (per ADR-0038) recording the transfer.

### Schrems III + KR Art 28-8 + Russia data localization

- **Schrems III risk.** EU↔US transfers under EU-US DPF face periodic legal challenge. Mitigation: per-tenant standard contractual clauses + per-class minimization + per-tenant SCC-revocation runbook (ability to halt cross-region in 24h).
- **KR Art 28-8.** Cross-border transfer of personal information requires per-class consent + per-recipient information disclosure + per-region equivalency assessment. Mitigation: per-tenant Art 28-8 evidence record in trust portal.
- **Russia data localization.** Federal Law 242-FZ requires personal data of Russian citizens to be stored in Russia. **We do not onboard Russia-citizen-data tenants at GA.** If onboarded later (founder ratification per ADR-0001 axis-admission protocol), per-pack `ru_only` residency class adopted.

### Per-region failover semantics

For `StrictKr`:

- KR-Seoul1 primary → KR-Chuncheon secondary.
- Failover is intra-KR only.
- DR drill quarterly.

For `KrWithUsFailover`:

- KR-Seoul1 primary → KR-Chuncheon secondary → US warm replica.
- US replica activated only if both KR cells unavailable simultaneously (catastrophic).
- Per-tenant DPO notification on activation (within 1h).

For `Global`:

- Per-pack failover topology.
- Per-class minimization on failover.

### Anti-scope

This ADR does not own the data class registry (per ADR-0008). Does not own the per-vertical override (per ADR-0034). Does not own the trust portal (per ADR-0038). Does not own the cell architecture (per ADR-0028).

---

## Consequences

### Positive

- Per-pack default residency class makes "where does my data live" answerable without per-tenant negotiation.
- Cross-region opt-in per class makes the failure mode "we transferred PHI to US by accident" structurally impossible.
- Tenant residency immutable + recreate-with-DSR-cascade gives a regulator-defensible posture for residency change requests.
- Per-pack regulator binding makes per-region compliance evidence per-region rather than per-tenant.

### Negative

- Recreate-as-residency-change is heavy operationally (per-tenant migration + DSR cascade).
- US warm replica is a real cost for `KrWithUsFailover` tenants.
- Per-pack regulator binding maintenance is a recurring legal + product cost.
- We don't onboard certain regions (Russia, China-data-localization-strict) at GA, which constrains addressable market.

### Operational

- Per-cell residency class manifest reviewed monthly.
- Per-tenant residency change runbook exercised quarterly.
- Per-region regulator change tracking quarterly.
- Per-tenant cross-region transfer audit weekly.
- Per-quarter Schrems-class legal posture review.

---

## Alternatives considered

### Alternative A — In-place residency change (no recreate)

- **Pros:** simpler tenant UX.
- **Cons:** cannot prove data didn't transit forbidden region during change; regulator posture weak.
- **Rejected because:** the regulator-defensible posture requires recreate.

### Alternative B — No residency classes; per-class per-tenant per-region matrix

- **Pros:** maximum flexibility.
- **Cons:** per-tenant complexity; default-deny becomes ambiguous; the "default" question becomes a per-tenant negotiation.
- **Rejected because:** per-pack defaults give operational sanity.

### Alternative C — Single global residency (`global` only)

- **Pros:** simplest.
- **Cons:** KR + EU + per-region regulator requirements not met; major addressable markets unreachable.
- **Rejected because:** sovereignty markets are core.

### Alternative D — Cross-region opt-in by default, opt-out for sensitive classes

- **Pros:** convenience.
- **Cons:** "opt-in by default" is not consent under PIPA Art 28-8 / GDPR Art 7; failure mode automatic.
- **Rejected because:** consent requirements forbid opt-in-by-default.

---

## Open questions

1. **Q1.** EU pack onboarding at GA or W+12? Default: W+12; KR + US at GA. → ADR-0033.
2. **Q2.** JP pack at GA or W+12? Default: W+12. → ADR-0033.
3. **Q3.** Per-tenant SCC-revocation 24h SLA — automated or human-loop? Default: human-loop at GA; automated at W+18. → ADR-0038.
4. **Q4.** Cross-region encryption key custody — per-region HSM or per-source-region HSM? Default: per-source-region HSM (data encrypted with source-region key; destination decrypts via federated KMS). → ADR-0043.
5. **Q5.** Russia + China onboarding gate — founder ratification only or also council? Default: founder ratification + council unanimous. → ADR-0001.

---

## References

- `docs/PRD.md` §11 (residency)
- `docs/DESIGN.md` §11 (residency), §10 (cross-microservice contracts)
- KR 「개인정보보호법」 Art 28-8 (cross-border transfer); 「전자금융감독규정」 §15; 「전자정부법」
- EU GDPR Art 44-49 (cross-border); EU-US Data Privacy Framework; Schrems II/III judgments; EHDS (European Health Data Space)
- US: HIPAA Privacy Rule; CCPA/CPRA; per-state privacy
- JP 個人情報保護法 (APPI) Art 24-30 (cross-border)
- Russia Federal Law 242-FZ (data localization)
- ADR-0001 (cohesion), ADR-0008 (DUBO), ADR-0028 (cloud cells), ADR-0029 (workspace), ADR-0033 (vertical pack), ADR-0034 (per-vertical override), ADR-0038 (trust portal), ADR-0043 (HSM + KMS), ADR-0045 (database tier)
