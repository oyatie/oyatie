---
doc_class: Onboarding
microservice: tenancy
persona: platform-admin
related_adrs: [ADR-0329, ADR-0330, ADR-0331, ADR-0244, ADR-0251]
date: 2026-05-20
doc_status: published
---

# tenancy — Platform Administrator First Week

Audience: a platform administrator at an oyatie-substrate-operator (you operate the substrate; your customers are tenants). You're responsible for tenant lifecycle, sub-scope architecture, DR pairing, KYB/KYC policies.

## Day 1 — orientation + access + canonical doctrine

Morning (3 h):
1. Receive `iam` invite. Cedar role `tenancy::platform-admin` binds: `tenancy::*::*` (full admin on tenant lifecycle, sub-scope, KYB/KYC, DR pairing, quotas).
2. Log in to tenancy admin portal: `https://tenancy-admin.<your-platform>.oyatie.io`.
3. Read ADR-0244 (tenant-as-universal-scoping-primitive) end-to-end (~ 20 min).
4. Read [[oyatie-is-a-tenant]] doctrine note (~ 10 min).
5. Read ADR-0064 (canonical-base + localization).

Afternoon (4 h):
6. Read tenancy substrate primer: portal → Help → "Tenancy Substrate 101" (~ 45 min).
7. Read the IP series IP-001 through IP-014 (the in-flight implementation plans for the µservice).
8. Survey existing tenants: portal → Tenants → list. Note: tenant_id format (ULID), audience_type distribution, country_code distribution.
9. Survey reserved-namespace enforcer rules.

Deliverable: doctrine internalised; tenant inventory documented.

## Day 2 — tenant lifecycle hands-on

Morning (4 h):
1. Provision a test tenant:
   ```sh
   oya tenancy tenant-provision \
       --legal-name "Test Tenant Inc." \
       --audience-type b2b-organization \
       --country-code US \
       --data-residency-region us-east-1 \
       --primary-pack "us-default" \
       --requester-principal-id "user:platform-admin@your-platform.com"
   ```
2. Observe the provisioning workflow: substrate creates Postgres schemas, IAM bindings, default Cedar policies, audit-chain anchoring, default sub-scope (`<tenant-slug>::root`).
3. Verify the tenant exists: portal → Tenants → search → see the new tenant.

Afternoon (3 h):
4. Provision a sub-scope: `oya tenancy sub-scope-create --tenant <id> --parent-scope root --name "us-east"`.
5. Provision a sub-tenant under the test tenant (matrix-org pattern):
   ```sh
   oya tenancy sub-tenant-create \
       --parent-tenant <id> \
       --legal-name "Test Sub Tenant LLC" \
       --audience-type b2b-organization \
       --country-code US
   ```
6. Try to delete the test tenant: `oya tenancy tenant-delete --tenant <id>`. Substrate enforces:
   - DSR cascade (Data Subject Request): all PII anonymised or deleted across every µservice.
   - 30-day grace period before hard delete.
   - Per-pack policy (e.g. KR-PIPA pack requires KCC notification 30 d before deletion).

Deliverable: 1 tenant + 1 sub-tenant + 1 sub-scope provisioned; deletion workflow observed.

## Day 3 — KYB/KYC + data residency

Morning (4 h):
1. Configure KYB integration: portal → KYB/KYC → "Providers" → choose Stripe Identity (default for US) + D&B for org verification.
2. Configure auto-verify thresholds: tenants attesting < $1M revenue + standard industries (Stripe risk score < 0.3) auto-verify. Above thresholds → manual review.
3. Test: provision a new tenant with KYB enabled; observe the verification flow.

Afternoon (3 h):
4. Configure data residency enforcer: portal → Data Residency → "Region rules". Default: `country_code = KR → data_residency_region = ap-northeast-2`; `country_code IN (EU member states) → data_residency_region = eu-central-1`; `country_code = US → data_residency_region = us-east-1`.
5. Try to provision a KR-country tenant with `data_residency_region = us-east-1` — substrate blocks the operation unless tenant signs the cross-border transfer consent.
6. Audit the data-residency enforcement: `oya audit-chain query --event-class tenancy::residency::enforced --since "1 hour ago"`.

Deliverable: KYB providers configured + data residency rules verified.

## Day 4 — DR pairing + pack lifecycle

Morning (4 h):
1. Configure DR pairing: portal → DR Pairing → "Default policy". Per region:
   - us-east-1 primary ↔ us-west-2 DR
   - eu-central-1 primary ↔ eu-west-1 DR
   - ap-northeast-2 primary ↔ ap-southeast-1 DR (or pack-specific KR-resident DR if paid tenant_class regulated-pack overlay)
2. Provision a paid tenant_class expanded deployment tenant with DR pairing. Verify the DR replica appears + is in sync.
3. Run the DR failover drill: portal → DR Pairing → tenant → "Simulate primary failure". Substrate failovers within 15 min; verify the tenant operates correctly on the DR region.
4. Restore the primary: substrate auto-reverses sync direction; primary catches up; failback after 30 min of clean operation.

Afternoon (3 h):
5. Configure pack lifecycle: portal → Packs → "Pack policy". For each pack (KR-PIPA, HIPAA-Provider, etc), define: auto-apply criteria (country, attestation, contract clause), required additional tenant-data (DPO email, BAA signed), required additional verifications.
6. Test: provision a tenant attesting "we handle US PHI" — substrate auto-applies the HIPAA-Provider pack + requires the BAA via `contract-lifecycle-management` µservice.

Deliverable: DR pairing tested + pack lifecycle automation verified.

## Day 5 — tenant merger ceremony + quota engine + handoff

Morning (4 h):
1. Author the merger ceremony for a B2B acquisition scenario: portal → Mergers → "New ceremony".
   - Source tenant: TargetCo (being acquired).
   - Destination tenant: AcquirerCo (the parent).
   - Sub-scope assignment: TargetCo becomes a sub-tenant of AcquirerCo.
   - Audit-chain provenance: TargetCo's chain merged into AcquirerCo's; original chain preserved as a forked-archive.
2. Approve + execute the merger ceremony. Audit-chain anchors every step.

Afternoon (4 h):
3. Configure per-tenant quotas: portal → Quotas → "Default quotas per tenant_class". Examples:
   - demo_trial tenant: max 100 users, 10 GB storage, 10 K API calls/day.
   - paid tenant_class baseline tenant: max 1 000 users, 1 TB storage, 1 M API calls/day.
   - paid tenant_class expanded deployment tenant: max 100 000 users, 100 TB storage, 100 M API calls/day.
4. Set per-tenant quota overrides via the quota REST API.
5. Document your platform's runbooks: tenant onboarding, offboarding, merger, DR drill, KYB escalation.
6. Receive substrate-team escalation contacts.

End of Week 1 deliverable: tenancy substrate operationally understood + first DR drill green + first merger ceremony exercised.

## What you should know by end of week 1

- Tenant lifecycle (provision → operate → offboard).
- Sub-scope + sub-tenant patterns.
- KYB/KYC automation.
- Data-residency enforcement.
- DR pairing + failover/failback.
- Pack lifecycle automation.
- Merger ceremony workflow.
- Quota engine.

## What you should NOT do in week 1

- Don't hard-delete a tenant without the DSR cascade + grace period. The substrate refuses but for emergency-deletions there's an override that you should NOT use without legal sign-off.
- Don't bypass data-residency enforcement. EU GDPR + KR-PIPA + similar regimes enforce residency by law.
- Don't reduce sub-scope cardinality limits without analysis. Some tenants legitimately need 100k+ sub-scopes (matrix-org patterns at scale).
- Don't disable lifecycle locks. They prevent concurrent operations (e.g. tenant deletion during active billing) that would corrupt audit-chain provenance.
- Don't manually create RLS policies. The substrate's isolation enforcer is canonical; manual policies create attack surface.
