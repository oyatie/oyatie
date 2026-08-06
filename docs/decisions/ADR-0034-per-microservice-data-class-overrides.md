---
id: ADR-0034
status: Accepted
doc_status: published
---

# ADR-0034: Per-microservice data class overrides — microservice-side hard-deny pack that tenant admin cannot raise

> **Status:** Accepted
> **Owner:** `council-architecture`
> **Date:** 2026-05-09 (rewritten 2026-05-13 — "vertical" terminology replaced with "microservice")
> **Related:** ADR-0001, ADR-0007, ADR-0008, ADR-0011, ADR-0029, ADR-0030, ADR-0031, ADR-0038, ADR-0049, ADR-0058

---

## Context

The Data Use Boundary (DUBO, per ADR-0008) defines per-data-class flow policy at the tenant level. That is necessary but not sufficient: certain data classes must be hard-denied for every tenant using a given microservice, regardless of what that tenant's admin would prefer. A medical tenant must not be able to opt PHI into ad sourcing even if the tenant admin signs a consent form, because patient consent under PIPA Art 23 + 「의료법」 §21 cannot be delegated to the tenant administrator. A microservice handling children-under-14 data must hard-deny all cross-tenant flows regardless of administrator preference, per 「개인정보보호법」 Art 22(6).

Every microservice in the flat catalog that handles regulated data classes ships a **microservice override pack** in its kernel crate. The pack binds at runtime in the Cedar policy gate (ADR-0007); tenant admin Cedar policies cannot raise the ceiling.

---

## Decision

Every microservice crate in the flat catalog that handles regulated data ships a **microservice override pack** in its kernel crate under `oya-<microservice>-override-pack-kernel`. The pack is a structured map from data class to hard-deny policy. It binds at runtime; tenant admin Cedar policies cannot raise the ceiling.

**Naming justification (BNF v4.1, ADR-0056):**
- `oya-medical-override-pack-kernel`: slot2 = `medical` (registered µservice); slot3 = `override-pack` (multi-token BC); slot4 = `kernel`

### Override pack schema

```rust
// oya-<microservice>-override-pack-kernel
pub struct MicroserviceOverridePack {
    pub microservice_id: MicroserviceId,
    pub hard_denies: BTreeMap<DataClass, HardDenyScope>,
    pub conditional_denies: BTreeMap<DataClass, ConditionalDeny>,
    pub minor_subject_class_overlay: Option<MinorSubjectOverlay>,
}

pub enum HardDenyScope {
    AdSourcing,
    CrossTenantSharing,
    CrossRegionTransfer,
    AnyMicroserviceExceptHome,
    All,
}
```

The pack is loaded at microservice-kernel init; the Cedar policy gate evaluates the pack before tenant policy. A microservice hard-deny short-circuits any tenant grant.

### Per-microservice override pack contents

#### Medical, pharmacy, healthcare-portal, emergency, clinical microservices

| Data class | Hard-deny scope |
|---|---|
| `PHI` (FHIR resources containing patient identity) | `All` for ad sourcing; `CrossRegionTransfer` blocked outside residency class |
| `PII-identifying` (주민등록번호, 의료보험증 번호) | `AdSourcing`, `CrossTenantSharing` |
| `PII-quasi` (DOB + ZIP + gender combinations that re-identify) | `AdSourcing` |
| `Sensitive-PIPA-Art23` | `All` for ad sourcing; `CrossTenantSharing` blocked |
| `Genomic` | `All` |
| `MentalHealth` | `All` for ad sourcing |

#### Payments, insurance, finance, banking microservices

| Data class | Hard-deny scope |
|---|---|
| `PCI` (PAN, CVV, mag stripe) | `All` for ad sourcing; PCI DSS-compliant storage required |
| `Financial-KR-신용정보` (신용정보법 §2) | `AdSourcing`, `CrossTenantSharing` |
| `BankingAccountFull` | `AdSourcing`, `CrossTenantSharing` |
| `KycPii` | `All` for ad sourcing |
| `AmlAlert` | `All` |

#### HR, payroll, accounting, ATS, GRC, performance microservices

| Data class | Hard-deny scope |
|---|---|
| `EmploymentRecord` | `AdSourcing`, `CrossTenantSharing` |
| `SalaryData` | `AdSourcing`, `CrossTenantSharing` |
| `Sensitive-PIPA-Art23` (union membership, political views) | `All` for ad sourcing |
| `DisciplinaryRecord` | `All` for ad sourcing |

#### microservice (mail / calendar / chat / docs)

| Data class | Hard-deny scope |
|---|---|
| `WorkplaceCommunication` | `AdSourcing` by default (tenant can opt in per explicit DUBO consent receipt) |
| `LegalHoldContent` | `All` outside legal-hold workflow |

### Minor subject class overlay

For any tenant using any microservice, when a record's subject is a minor (under 18 in KR; per-jurisdiction definition elsewhere), the **minor subject overlay** binds:
- Ad sourcing: hard-deny
- Cross-tenant sharing: hard-deny
- Cross-region transfer: requires explicit dual consent (guardian + DPA)
- Behavioral targeting: hard-deny

### Cedar evaluation order

1. Microservice override pack (this ADR) — highest precedence
2. Per-region regulatory overlay (regional-pack architecture)
3. Tenant admin Cedar policy — lowest precedence

A higher-precedence deny short-circuits a lower-precedence allow. Tenant admin can narrow (deny more), never raise (deny less).

### Override pack changes require ADR + regulator review

Modifying a microservice override pack requires:
1. An ADR amendment in this pack
2. Per-jurisdiction regulator review evidence
3. `oya-check-override-pack` fitness lane confirmation
4. Per-tenant transparency notice before the change takes effect (ADR-0038)

---

## Consequences

### Concrete crate layout (BNF v4.1)

Each regulated microservice ships:
```
oya-medical-override-pack-kernel
oya-pharmacy-override-pack-kernel
oya-payments-override-pack-kernel
oya-banking-override-pack-kernel
oya-hr-override-pack-kernel
oya-payroll-override-pack-kernel
oya-connect-override-pack-kernel
  ... (one per regulated microservice)
```

The override-pack kernel is a dependency of the microservice's main kernel crate. It is loaded at startup and injected into the Cedar policy evaluation chain.

### Quality / Performance / Scalability (per ADR-0062)

- Override pack evaluation is in the Cedar policy hot path; it must add ≤1ms p99 overhead per call.
- Packs are loaded once at startup and held in memory (immutable; no per-request reload).
- `oya-check-override-pack` CI lane verifies every regulated µservice ships a pack; runs in <2s.

### Positive

- Per-microservice hard-denies are mechanical; the failure mode where a tenant admin opts in to ad sourcing of PHI is structurally impossible.
- The minor-subject overlay travels with the data, not with the tenant.

### Negative

- Microservice onboarding pays an authoring tax for the override pack.
- Per-jurisdiction regulator binding evidence is a recurring annual cost.

---

## Related

- ADR-0001 (cohesion — every microservice in the flat catalog)
- ADR-0007 (Cedar — override pack binds in Cedar evaluation chain)
- ADR-0008 (DUBO — override pack is the floor below tenant DUBO)
- ADR-0038 (trust framework + DSR)
- ADR-0058 (Flat microservice catalog — "vertical" terminology retired)
- ADR-0062 (Quality/Performance/Scalability bar)
- `[[feedback-flat-product-catalog]]` — "vertical" override → "microservice" override
