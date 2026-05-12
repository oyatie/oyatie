# ADR-0034: Per-vertical data class overrides — vertical-side hard-deny pack that tenant admin cannot raise

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `council-architecture`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0007, ADR-0008, ADR-0011, ADR-0029, ADR-0030, ADR-0031, ADR-0033, ADR-0038, ADR-0049

---

## Context

The Data Use Boundary (DUBO, per ADR-0008) defines per-data-class flow policy at the tenant level. That is necessary but not sufficient: certain data classes must be hard-denied for *every* tenant in a given vertical, regardless of what that tenant's admin would prefer. A healthcare tenant must not be able to opt PHI into ad sourcing even if the tenant admin signs a consent form, because patient consent under PIPA Art 23 + 「의료법」 §21 cannot be delegated to the tenant administrator. A K12 education tenant must not be able to opt children-under-14 data into any cross-tenant flow regardless of administrator preference, per 「개인정보보호법」 Art 22(6) and KR 「아동·청소년의 성보호에 관한 법률」.

The pack-of-19 foundation ADRs decided the DUBO pattern but did not pin the **vertical-side override pack** that ships with the vertical kernel and binds at runtime. Without this pin, the failure mode is: vertical kernel ships with default DUBO policy → tenant admin (mistakenly or deliberately) raises the policy ceiling → regulator-relevant data classes leak. This ADR pins the override pack so that per-vertical hard-denies are mechanically enforced and unraisable by tenant admin.

---

## Decision

Every vertical in `crates/oya-vertical-<name>-*` (per ADR-0033) ships a **vertical override pack** in its kernel crate. The pack is a structured map from data class to hard-deny policy. The pack binds at runtime in the Cedar policy gate (ADR-0007); tenant admin Cedar policies cannot raise the ceiling — the vertical override is the policy floor.

### Override pack schema

```rust
// crates/oya-vertical-<name>-kernel/src/override_pack.rs
pub struct VerticalOverridePack {
    pub vertical_id: VerticalId,
    pub hard_denies: BTreeMap<DataClass, HardDenyScope>,
    pub conditional_denies: BTreeMap<DataClass, ConditionalDeny>,
    pub minor_subject_class_overlay: Option<MinorSubjectOverlay>,
}

pub enum HardDenyScope {
    AdSourcing,                    // never source for ads
    CrossTenantSharing,            // never share with another tenant
    CrossRegionTransfer,           // never transfer outside residency class
    AnyAxisExceptHomeAxis,         // never leave the originating axis
    All,                           // hard-deny everywhere
}
```

The pack is loaded at vertical-kernel init; the Cedar policy gate evaluates the pack *before* tenant policy. A vertical hard-deny short-circuits any tenant grant.

### Per-vertical override pack contents

#### Healthcare tenant pack

| Data class | Hard-deny scope |
|---|---|
| `PHI` (Protected Health Information; FHIR resources containing patient identity) | `All` for ad sourcing; `CrossRegionTransfer` blocked outside residency class |
| `PII-identifying` (주민등록번호, 의료보험증 번호, etc.) | `AdSourcing`, `CrossTenantSharing` |
| `PII-quasi` (date of birth + ZIP + gender combinations that re-identify) | `AdSourcing` |
| `Sensitive-PIPA-Art23` (race, religion, political views, union membership, health, sex life) | `All` for ad sourcing; `CrossTenantSharing` blocked |
| `Genomic` | `All` |
| `MentalHealth` | `All` for ad sourcing; tenant Cedar gate enforces strict access on internal use |

#### Fintech tenant pack

| Data class | Hard-deny scope |
|---|---|
| `PCI` (Payment Card Industry — PAN, CVV, mag stripe) | `All` for ad sourcing; PCI DSS-compliant storage required |
| `Financial-KR-신용정보` (신용정보법 §2 — credit information) | `AdSourcing`, `CrossTenantSharing` |
| `BankingAccountFull` (account number + routing + name) | `AdSourcing`, `CrossTenantSharing` |
| `KycPii` (passport / 신분증 image / liveness check) | `All` for ad sourcing |
| `AmlAlert` | `All` |

#### Education-K12 tenant pack

| Data class | Hard-deny scope |
|---|---|
| `CHILDREN_UNDER_14` (any data tied to a subject under age 14) | `All` everywhere — including across the K12 tenant's own surfaces; no ad sourcing, no cross-tenant share, no cross-region without explicit guardian + DPA dual consent (PIPA Art 22-2) |
| `MinorSubjectClass` (any data tied to a subject under age 18) | `All` for ad sourcing; `CrossTenantSharing` blocked |
| `EducationalRecord-FERPA-equivalent` | `AdSourcing`, `CrossTenantSharing` |
| `Disability` | `All` for ad sourcing |
| `BehavioralRecord` (학교생활기록부 sensitive entries) | `All` for ad sourcing |

#### Education-HE tenant pack

| Data class | Hard-deny scope |
|---|---|
| `MinorSubjectClass` (per K12 overlay where present) | per K12 overlay |
| `EducationalRecord` | `AdSourcing`, `CrossTenantSharing` |
| `ResearchSubject-PHI` | per healthcare pack overlay |

#### Public-sector tenant pack

| Data class | Hard-deny scope |
|---|---|
| `CitizenIdentifier` (주민등록번호 + 외국인등록번호 + 운전면허번호) | `AdSourcing`, `CrossTenantSharing`, `CrossRegionTransfer` outside residency class |
| `ServiceApplicationData` | `AdSourcing`, `CrossTenantSharing` |
| `LegalDecision` | `All` for ad sourcing |
| `TaxRecord` | `All` for ad sourcing |
| Per-jurisdiction tighter rules | per-region overlay (KR 「개인정보보호법」, EU GDPR, US state privacy) |

#### Defense tenant pack

| Data class | Hard-deny scope |
|---|---|
| `MilitaryClassified` | `All` — and the entire vertical is gated; defense tenant onboarding requires founder ratification (per ADR-0001 axis-admission protocol) |
| `OperationsData` | `All` |
| `PersonnelClearance` | `All` |

(Anti-scope reminder: at GA we ship defense vertical as proposed-only; no defense tenants are onboarded without explicit founder ratification.)

### Minor subject class overlay

For any tenant in any vertical, when a record's subject is a minor (under 18 in KR; per-jurisdiction definition elsewhere), the **minor subject overlay** binds:

- Ad sourcing: hard-deny.
- Cross-tenant sharing: hard-deny.
- Cross-region transfer: requires explicit dual consent (guardian + DPA).
- Behavioral targeting: hard-deny.
- Profiling under GDPR Art 22 / PIPA Art 22-2: hard-deny.

The overlay is set at vertical-pack level (always-on for K12) or per-record (any vertical can flag a record's subject as a minor and the overlay binds for that record).

### Override pack ships with vertical kernel

The override pack is a kernel artifact; it ships in the vertical kernel crate (`crates/oya-vertical-<name>-kernel`). Tenant onboarding into the vertical loads the pack; the pack cannot be modified by tenant admin via any API.

### Tenant admin cannot raise the ceiling

The Cedar policy evaluation order is:

1. Vertical override pack (this ADR).
2. Per-region regulatory overlay (per regional-pack architecture).
3. Tenant admin Cedar policy.

A higher-precedence deny short-circuits a lower-precedence allow. The tenant admin Cedar policy can *narrow* (deny more), never *raise* (deny less).

### Override pack changes require ADR + regulator review

Modifying a vertical override pack — adding a data class, removing a hard-deny, narrowing a scope — requires:

1. An ADR amendment in this pack (or downstream pack).
2. Per-jurisdiction regulator review evidence (e.g. DPO review for PIPA-relevant changes).
3. Cohesion fitness lane confirmation (`oya-foundry-fitness-vertical-override-pack`).
4. A per-tenant transparency notice on the change before it takes effect (per ADR-0038 trust framework).

### Anti-scope

This ADR does not define **per-tenant** Cedar policy (that is the tenant admin's responsibility, scoped under the override floor). It does not redefine data classes (those live in the data-class registry per ADR-0008). It does not define per-region overlays (those live in regional-pack architecture).

---

## Consequences

### Positive

- Per-vertical hard-denies are mechanical, not advisory; the failure mode where a tenant admin opts in to ad sourcing of PHI is structurally impossible.
- Adding a vertical includes a structured override pack authoring step, which forces explicit thinking about regulator binding.
- The minor-subject overlay travels with the data, not with the tenant — so a healthcare tenant treating a 12-year-old patient gets the K12-style overlay automatically for that record.
- Tenant admins cannot make compliance mistakes that originate in optimistic policy edits.

### Negative

- Vertical onboarding pays an authoring tax for the override pack; per-vertical regulator review is required before pack changes.
- Per-jurisdiction regulator binding evidence is a recurring cost (annual review minimum).
- Some tenant admins will request features the override pack denies (e.g. "I want to retarget my K12 students with relevant educational content"); we will say no.

### Operational

- Override pack diff reviewed at each vertical-pack release.
- Per-vertical override pack runtime audit (samples per week) confirms no override has been bypassed.
- Tenant admin console surfaces the active override pack (transparency); attempt-to-raise is logged + audit-chained even when blocked.
- Per-DSR (per ADR-0038) the override pack determines erasure cascade scope.
- Per-tenant trust portal shows which classes are hard-denied for that tenant.

---

## Alternatives considered

### Alternative A — Per-tenant DUBO only (no vertical override pack)

- **Pros:** simpler policy stack; tenant admin has full discretion.
- **Cons:** PHI / PCI / minor-subject failures one consent-form away; regulator-binding evidence becomes per-tenant rather than per-vertical, which does not satisfy regulators.
- **Rejected because:** the failure mode is exactly what this ADR prevents.

### Alternative B — Per-region override only (no per-vertical layer)

- **Pros:** simpler; one override layer (region).
- **Cons:** per-region rules are jurisdictional, but per-vertical rules are categorical; a healthcare PHI hard-deny exists in EU, KR, US, JP, regardless of region. Folding both into one layer multiplies surface and loses categorical clarity.
- **Rejected because:** per-vertical and per-region are orthogonal axes of constraint.

### Alternative C — Soft warning instead of hard-deny (tenant admin sees warning, can override with documented justification)

- **Pros:** flexibility for edge cases.
- **Cons:** the soft-warning failure mode is well-documented across enterprise SaaS — admins click through; auditors find the override; regulators fine.
- **Rejected because:** "hard-deny by default unless founder + DPO ratify" is the *only* posture that survives regulator scrutiny.

---

## Open questions

1. **Q1.** Where does `Genomic` data class hard-deny live — healthcare pack only, or a separate "biomedical" overlay that healthcare + research-HE both consume? Default: healthcare pack only at GA; revisit when research-HE vertical onboarded. → ADR-0033.
2. **Q2.** Minor subject definition cutoff — KR uses 14 (under-14 children) and 18 (minors); other jurisdictions vary (US: 13 COPPA; EU: 13-16 per member state). Default: per-region overlay defines the cutoff; vertical override pack uses `under-14` and `minor` as abstract classes. → regional-pack architecture.
3. **Q3.** Defense vertical override pack — finalize at founder ratification or pre-author at proposed status? Default: pre-author at proposed status with founder ratification gating activation. → ADR-0033.
4. **Q4.** Per-record minor-subject flagging UX — automatic from age field in canonical entity model, or manual flag? Default: automatic from age field in canonical entity model (where present); manual flag fallback. → ADR-0033.
5. **Q5.** How does the override pack interact with per-tenant data residency change request (per ADR-0049)? Default: residency change requires re-creation; override pack travels with new tenant. → ADR-0049.

---

## References

- `docs/PRD.md` §11 (data use boundary), §11 (per-vertical residency)
- `docs/DESIGN.md` §10 (cross-axis contracts), §11 (cross-axis contradictions)
- KR 「개인정보보호법」 Art 22(6), Art 22-2, Art 23 (sensitive); 「의료법」 §21; 「신용정보법」 §2; 「청소년보호법」 §16; 「아동·청소년의 성보호에 관한 법률」
- US: COPPA (Children's Online Privacy Protection Act); FERPA (Family Educational Rights and Privacy Act); HIPAA Privacy Rule
- EU: GDPR Art 8 (children's consent), Art 9 (special categories), Art 22 (automated decision)
- PCI DSS v4.0
- ADR-0001 (cohesion), ADR-0007 (Cedar + persona tier), ADR-0008 (DUBO), ADR-0011 (capability registry), ADR-0029 (workspace), ADR-0030 (search), ADR-0031 (ads), ADR-0033 (vertical pack architecture), ADR-0038 (trust framework + DSR cascade), ADR-0049 (residency)
