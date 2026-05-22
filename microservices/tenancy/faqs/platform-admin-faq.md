---
doc_class: FAQ
microservice: tenancy
persona: platform-admin
related_adrs: [ADR-0329, ADR-0330, ADR-0331, ADR-0244, ADR-0251]
date: 2026-05-20
doc_status: published
---

# tenancy — Platform Administrator FAQ

## Q1: Why is tenant the universal scoping primitive (ADR-0244)?

Because every row of data in oyatie carries a tenant_id, every audit event carries a tenant_id, every cost record carries a tenant_id. This makes tenant the canonical partition key for ALL operational concerns: data isolation (RLS), cost attribution (FinOps), compliance evidence (audit-chain), residency enforcement (per-pack overlay). Alternatives (organization-id, account-id) were considered + rejected — they conflate "billing entity" with "data-isolation boundary" with "compliance scope", and the resulting confusion shows up across competitors as cross-tenant data leakage incidents. Tenant-everywhere is uncomfortable initially but proves out: per ADR-0244 it's the single primitive that scales correctly across all 14 keystone doctrines.

## Q2: What's the difference between a sub-tenant and a sub-scope?

**Sub-tenant** = separate legal entity / billing entity under a parent. Matrix-org pattern: AcquirerCo has SubsidiaryA + SubsidiaryB as sub-tenants. Each has its own billing, KYB, audit-chain branch. Children inherit some defaults from parent (pack, residency) but can override.

**Sub-scope** = logical partition WITHIN a tenant. Example: a single tenant "BigCorp" has sub-scopes `marketing`, `engineering`, `finance`. Sub-scopes share the same billing + audit-chain + pack. Sub-scopes are for ORG-CHART decomposition; sub-tenants are for LEGAL-ENTITY decomposition.

Rule of thumb: if it has its own legal counsel or its own billing, it's a sub-tenant; if it's just a department of one company, it's a sub-scope.

## Q3: How does RLS (Postgres Row-Level Security) enforce tenant isolation?

For each tenant-scoped table, the substrate auto-generates a RLS policy:

```sql
CREATE POLICY tenant_isolation ON documents
    USING (tenant_id = current_setting('oya.current_tenant_id')::text);
```

Before every database query, the application sets the session variable `oya.current_tenant_id` from the principal's JWT. The RLS policy ensures the query only returns rows for that tenant, even if the application accidentally builds a query without a WHERE clause. Postgres enforces this at the row level — no application bug can bypass it.

This is layered with Cedar permits (application-level) and is the second line of defense. Cedar can be misconfigured; RLS is the substrate-enforced floor.

## Q4: Can a tenant operate across multiple data residency regions?

Yes via the cross-region-consent flow. By default, a tenant's data stays in its `data_residency_region`. To allow cross-region operations (e.g. EU tenant with a US subsidiary), the tenant must explicitly file a Consent resource that:
- Identifies the source region + destination region.
- Identifies the legal basis (GDPR Art. 6(1) + adequacy / SCCs / BCRs).
- Identifies the data categories that may cross.
- Has a defined sunset date or termination condition.

The substrate stamps every cross-region operation with the consent's ID + cross-emits to audit-chain. If the consent expires or is revoked, cross-region operations halt immediately.

For KR-PIPA tenants, cross-region requires PIPC notification + 본인 동의 (data subject consent). The substrate templates these via the `contract-lifecycle-management` µservice.

## Q5: What's a "lifecycle lock" and why do they matter?

A lifecycle lock is a per-tenant or per-sub-scope semaphore that prevents conflicting operations. Examples:
- Lock on "billing-cycle-active" prevents tenant offboarding mid-cycle (would corrupt billing reconciliation).
- Lock on "active-merger" prevents sub-tenant restructuring during M&A ceremony.
- Lock on "active-DR-failover" prevents schema changes while DR is in failover mode.

Locks are advisory (the substrate refuses operations that conflict) and audited. A lock can be force-released only by 2-of-3 platform-admin quorum (on paid tenant_class regulated-pack overlay) with explicit justification logged to audit-chain.

## Q6: A tenant requests deletion (GDPR Art. 17 right-to-erasure). What happens?

The DSR cascade runner (IP-009) executes:
1. Verify the requester has authority (typically the DPO or named legal contact).
2. Issue a DSR initiation event to every µservice via the `consent-graph`.
3. Each µservice purges or anonymises the tenant's PII per its data model.
4. The tenant enters "deletion-pending" state with a 30-day grace period (configurable per pack; KR-PIPA allows up to 90 d for certain financial-record retention requirements).
5. After grace period expires, hard-delete: drop database schemas, purge SeaweedFS objects, remove from sub-scope registries, archive audit-chain anchors (cannot be deleted; provides regulator evidence of compliance).
6. Cross-emit `tenancy::tenant::deleted` to audit-chain (with hash-only metadata; no PII).

Total elapsed time: ~ 30-90 days depending on pack.

## Q7: How does the substrate handle tenant impersonation by support staff?

The "tenant-shadow" feature allows authorised support staff to perform actions as if they were the tenant. Implementation:
- Cedar permit `tenancy::tenant::shadow` granted only to a small support team (e.g. CS-tier-3-engineers).
- Activation requires a justification + ticket reference + explicit "I am acting on behalf of tenant <id> for purpose <ticket>" prompt.
- Substrate logs every shadow action with both the actual principal AND the impersonated tenant.
- All shadow actions emit `tenancy::shadow::action_performed` to audit-chain with the justification.
- Shadow session times out after 30 min idle; renew with re-justification.
- Tenant admin can opt out of shadow via the privacy panel; some operations may then require tenant-side coordination.

## Q8: We're operating in multiple regions. How does DR pairing work technically?

Per DR-pairing-controller (IP-019):
- Each tenant has primary + DR region. Both are oyatie cells.
- Primary writes to PostgreSQL + emits change events to a Kafka topic in the primary region.
- A bidirectional replication daemon in the DR region consumes the primary's topic + applies to DR's PostgreSQL.
- Sync lag p99 ≤ 60 s in steady state.
- On primary failure (auto-detected by health check loss for ≥ 5 min): the DR region is promoted to primary; the original primary is marked "needs failback".
- Failback: when the original primary recovers, the substrate runs a recovery procedure (reverse-sync; once caught up, promote back to primary). Takes ~ 30-60 min depending on lag.

Important: DR is for region-level failure, not data-corruption recovery. Data corruption requires point-in-time-recovery (PITR) from PostgreSQL WAL archives.

## Q9: Can tenants merge their data with another tenant during M&A?

Yes via the merger ceremony (paid tenant_class regulated-pack overlay). Structured workflow:
1. Both tenants sign a Merger Agreement via `contract-lifecycle-management`.
2. Both tenants designate a "merger ceremony lead" (typically the GC).
3. The ceremony begins: lifecycle locks placed on both tenants.
4. Schema-level merge: TargetCo's data partition is renamed/moved under AcquirerCo's parent partition.
5. Sub-scope assignment: TargetCo becomes a sub-tenant of AcquirerCo, OR (more common) TargetCo's data is merged into AcquirerCo's tree.
6. Audit-chain provenance: TargetCo's audit-chain is preserved as a forked-archive linked to AcquirerCo's main chain. Future audits can still trace pre-merger TargetCo events.
7. Sub-tenant deletion: after merger, TargetCo's standalone tenant record is offboarded via standard DSR cascade (with the merger as the legal basis for retention exemption).
8. Cross-emit `tenancy::merger::completed` to audit-chain.

For KR-PIPA: KCC notification within 30 days; affected data subjects notified within 14 days.

## Q10: What's the canonical base + localization (ADR-0064) overlay model in tenancy?

Every tenant runs the canonical-base configuration + zero or more localization overlays:
- Canonical base: the global default substrate behavior (tenant model, IAM model, audit-chain shape).
- Localization overlays: per-pack adjustments. KR-PIPA pack adds: Korean address validation, 신용평가회사 KYB, KISA-rooted certificate trust, FSC pre-notification triggers. EU pack adds: GDPR DSR cascade with 30-day completion, eIDAS QES support, DPA template auto-attach.

Tenants can opt into multiple overlays (e.g. a global SaaS with US + KR + EU customers loads US-default + KR-PIPA + EU-GDPR overlays). The substrate composes overlays without code branches in the canonical base.

## Q11: How do reserved namespaces work in paid tenant_class regulated-pack overlay?

Per [[oyatie-is-a-tenant]] doctrine + ADR-0242, oyatie itself is a reserved-namespace tenant (`oyatie.*` namespace; `tenant_id = "oyatie"` for substrate-level operations). The reserved-namespace enforcer prevents customers from creating tenants with slug-collisions with reserved prefixes:
- `oyatie.*` (substrate)
- `gov-*` (government-only namespace per pack policy)
- `kr-*` (Korean sovereign namespace)
- `eu-*` (EU sovereign namespace)
- `pack-*` (substrate-managed pack identifiers)
- Per-pack additional reservations.

Any tenant slug attempting collision is rejected at provision. The enforcer is updated by 2-of-3 platform-admin quorum on paid tenant_class regulated-pack overlay; updates are audited.

## Q12: Our pack is CSAP-defense. Are there special tenancy requirements?

Yes. CSAP-defense pack overlay enforces:
- All tenants must be Korean entities (verified via 법인등기부등본 + 국가보훈처 defense-vendor registration).
- All tenant data stays in CSAP-certified Korean cells (no cross-region).
- DR pairing within Korean sovereign cells only (no cross-Asia, no out-of-Korea).
- Dual-control on every tenant lifecycle operation (provision, sub-scope create, deletion).
- KISA security audit annually.
- Tenant offboarding requires 90 days minimum (Defense Acquisition Program Administration retention requirements).
- All sub-tenants must also be Korean entities (no foreign sub-tenants).

This is a stringent set; tenants meeting these requirements are typically in the defense supply chain.
