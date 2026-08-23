---
doc_status: published
---

# Team: Platform — Tenancy & Identity

## Mission
This team owns the single tenancy kernel and identity kernel that every Oyatie axis inherits. It exists to ensure that PHI/PII/PCI never cross tenant boundaries, that Cedar policy is the single source of truth for RBAC/ABAC decisions, and that the `Tenant` and `Identity` shape changes are gated by all-axis review. It does **not** own per-axis billing, per-axis audit, or the Data Use Boundary ADR (those belong to `platform-audit-evidence` and `platform-privacy-dub` respectively).

## Owned axes / surfaces / contracts
- **Axis(es):** Cross-cutting (SaaS kernel, consumed by all 7 axes)
- **Surfaces:**
  - `platform-tenant-kernel` — `TenantId`, `Tenant`, `ResidencyClass`, `RegulatoryPackId`, `TenantPlaneGrants`, `AutonomyTier`, `DataUseConsent`, `BillingAccountId`
  - `platform-identity-kernel` — `UserId`, `Principal`, `RoleBinding`, `PolicySet`, Cedar integration
  - `platform-tenant-domain` — tenant lifecycle use-cases (onboard, suspend, migrate, off-board)
  - `platform-identity-domain` — RBAC/ABAC use-cases, SSO federation, STS token issuance
  - `platform-identity-adapter-cedar` — Cedar policy engine adapter
  - `platform-identity-api` — REST/gRPC identity surface
  - `platform-address-kernel` — `AddressValidator` trait + KR/JP/US/EU default impls
- **Cross-axis contracts (DESIGN §10):**
  - `Tenant` kernel (owner) — consumed by all axes
  - `Identity / RBAC / Cedar policy` (owner) — consumed by all axes
  - `IAM / SSO / SAML / OIDC IdP` (co-owner with `axis-cloud` for cloud-customer-facing IAM)
- **Catalog records:** `crates/platform-tenant-*`, `crates/platform-identity-*`, `crates/platform-address-*`
- **Runbooks:** `runbooks/tenant-onboarding.md`, `runbooks/identity-provider-federation.md`, `runbooks/cedar-policy-rollback.md`
- **ADRs:** ADR-0044 (corp data tier + residency), ADR-0006 (cross-product auth), ADR-0017 (plane separation — tenancy sections)

## In-scope work
- Authoring and evolving `Tenant` shape (all changes require all-axis review gate)
- Cedar policy schema, policy evaluation, policy publish/rollback lifecycle
- RBAC and ABAC use-case implementation across all axes
- SSO federation adapters: SAML, OIDC, local identity providers (KR 본인확인서비스, JP マイナンバー, EU eIDAS, etc.) — seam definitions; regional-pack impls shipped by the regional-pack team
- STS token issuance and validation
- Tenancy lifecycle: onboard, suspend, migrate-region, off-board
- Cell-routing read of `Tenant.region` — tenant-kernel is the source; cloud axis reads it
- `AddressValidator` trait contract; default impls for initial regional packs
- All-axis review participation when any consuming axis touches a `Tenant` field
- Fitness-function gate: `governance-tenant-shape` (blocks unauthorized `Tenant` mutations)

## Out-of-scope (anti-scope)
- Per-axis billing and metering (→ `platform-audit-evidence` owns audit; `axis-cloud` / `axis-saas` own metering kernels)
- Data Use Boundary ADR authorship (→ `platform-privacy-dub`)
- Cloud-customer-facing IAM surface (→ `axis-cloud` owns `cloud-iam-kernel`; this team co-owns the seam contract only)
- Eventing backbone (→ `platform-eventing-og`)
- Public REST/SDK stability tier for non-identity surfaces (→ `platform-api-sdk`)
- Per-vertical regulatory-pack impls (→ per-vertical team)
- Hiring, vendor procurement (→ founder + ops teams)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `platform-eventing-og` | Outbox for tenant-lifecycle events, Cedar policy publish events | Per-release |
| `platform-audit-evidence` | Audit-chain emission for tenant onboarding, Cedar policy changes | Per-release |
| `platform-privacy-dub` | Data Use Boundary ADR — defines `DataUseConsent` shape we embed in `Tenant` | ADR lifecycle |
| `axis-cloud` | `RegionCode` and cell taxonomy so `Tenant.region` stays in sync | Monthly sync |
| `crew-adr-promotion` | ADR-0006, ADR-0044 promotion to Accepted | ADR batch |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| All 7 axes | `TenantId`, `Tenant`, Cedar policy evaluation | Every PR touching tenancy |
| `axis-foundry` | `AutonomyTier` field, `TenantPlaneGrants` | Every capability invocation path |
| `axis-search` | `DataUseConsent.search_indexable_classes` | Search index lifecycle |
| `axis-ads-analytics` | `DataUseConsent.ad_targeting_classes` | Ads-axis data-use gate |
| `axis-cloud` | `Tenant.region`, `Tenant.residency` for cell routing | Cloud provisioning |
| `platform-api-sdk` | Identity token validation for public API gateway | Per-release |

## Success metrics
- **Cross-axis contract violations on `main`:** 0 per quarter (rolls up to PRD §4.2 structural metric)
- **Unauthorized `Tenant` shape changes reaching `main`:** 0 (fitness gate enforces)
- **Cedar policy publish latency:** p99 < 200 ms (control-plane SLO)
- **SSO federation adapter coverage:** ≥ 1 adapter per regional pack onboarded
- **Tenant onboarding end-to-end (API → audit-chain record):** p99 < 5 s
- **Foundation-bypass ledger entries for tenancy gates:** 100% retire within declared expiry (PRD §4.2)

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council (`teams/council-architecture/CHARTER.md`) — any `Tenant` shape change proposal goes here
- Privacy: privacy council (`teams/council-privacy/CHARTER.md`) — `DataUseConsent` field changes
- Founder: as last resort

## Communication cadence
- Stand-up: daily async (Slack thread)
- Weekly: 45-min sync — ADR proposals, shape-change queue, cross-axis review backlog
- Cross-team review: participates in monthly cross-axis contract audit (DESIGN §11)

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules; every `Tenant` shape PR requires all-axis review label
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch; shape changes are P0 proposals

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| `Tenant` shape change lands without all-axis review | High | `governance-tenant-shape` CI gate hard-fails on unauthorized mutations |
| Cedar policy misconfiguration allows cross-tenant data access | Catastrophic | Policy publish requires security-reviewer agent sign-off; rollback runbook tested quarterly |
| Identity provider federation bug leaks tokens cross-tenant | Catastrophic | STS token validation is per-tenant-scoped; audit-chain emits every issuance |
| Residency field drift between tenant-kernel and cloud-kernel | High | Monthly sync with `axis-cloud`; fitness function checks cross-reference |

## Sources scanned
PRD.md §2 (tenancy taxonomy), DESIGN.md §5 (unifying tenancy model), §10 (contracts: Tenant kernel, Identity/RBAC/Cedar, IAM row), ADR-0044, ADR-0006, ADR-0017, DOC-CATALOG.md §2.1 (doc.design owner).
