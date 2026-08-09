---
doc_class: Tutorial
microservice: tenancy
related_adrs: [ADR-0329, ADR-0330, ADR-0331, ADR-0244, ADR-0251]
date: 2026-05-20
doc_status: published
---

# Tutorial — Provision a B2B tenant with KYB verification, HIPAA-Provider pack, and DR pairing

Goal: walk through provisioning a new B2B-organization tenant end-to-end: KYB verification via Stripe Identity + D&B, HIPAA-Provider pack auto-applied (BAA signed), data-residency enforced to us-east-1 with us-west-2 DR pairing, default sub-scope provisioned, and the tenant ready for first user invite.

Prereqs: `tenancy::platform-admin` Cedar role, paid tenant_class with required billing components, ~ 90 min.

## Step 1 — gather tenant info

Tenant details (provided by sales / customer):
- Legal name: "MedCenter LLC"
- Audience type: b2b-organization
- Country: US (Delaware-registered)
- Industry: Healthcare
- Use case: clinical EHR cloud platform for ~ 5 000 employees
- Data classes: PHI (US HIPAA)
- Estimated annual revenue: $850 M
- DPO email: dpo@medcenter.com
- Privacy officer: privacy@medcenter.com
- Tax ID: 12-3456789 (US EIN)

## Step 2 — initiate provisioning

```sh
oya tenancy tenant-provision \
    --legal-name "MedCenter LLC" \
    --audience-type b2b-organization \
    --country-code US \
    --industry-code "healthcare-provider" \
    --estimated-revenue-usd 850000000 \
    --tax-id "12-3456789" \
    --data-classes-attested "phi-us" \
    --dpo-email "dpo@medcenter.com" \
    --privacy-officer-email "privacy@medcenter.com" \
    --primary-contact "ceo@medcenter.com" \
    --data-residency-region us-east-1 \
    --requester-principal-id "user:cs-tier2@your-platform.com"
```

Substrate response:
```
Provisioning workflow started.
Workflow ID: prov_wf_01HXYZ...

Steps:
  1. KYB initiation: Stripe Identity + D&B (estimated 5-15 min)
  2. Pack-applicability evaluation: HIPAA-Provider candidate
  3. BAA generation: contract-lifecycle-management µservice
  4. PostgreSQL schema provisioning: us-east-1 primary + us-west-2 DR
  5. IAM tenant binding + default Cedar policies
  6. Sub-scope registry initialisation
  7. Audit-chain anchor

Estimated total time: 30-60 min.
```

## Step 3 — KYB verification flow

The substrate calls Stripe Identity + D&B. Substrate updates as the calls complete:

```
[KYB] Stripe Identity check: completed
  - Verified business: MedCenter LLC
  - Risk score: 0.18 (low)
  - Sanctions check: clear
  - Adverse media: none
[KYB] D&B World-Check: completed
  - DUNS number resolved: 12-345-6789
  - Tax-ID match: confirmed
  - Officers verified: yes
  - Court records: 2 minor commercial litigations (acknowledged)
[KYB] PHI-handler attestation: required
  - Substrate paused; awaiting signed BAA via contract-lifecycle-management
```

The substrate auto-creates a BAA contract in the CLM µservice + emails it to the tenant's privacy officer. While awaiting signature, provisioning is paused.

## Step 4 — sign BAA + resume provisioning

Tenant signs BAA via DocuSign / QES flow. CLM cross-emits `contract::signed` event. Tenancy substrate resumes:

```
[BAA] Signed by privacy@medcenter.com at 2026-05-20T15:42:00Z
[BAA] Counterparty: oyatie platform-operator
[BAA] Signature class: AES (DocuSign)
[BAA] Pack: HIPAA-Provider activated
[Pack] HIPAA-Provider applied:
  - BAA reference: contract_01HXYZ...
  - Retention overlay: 6 y minimum
  - Audit-chain tier: elevated (cryptographic anchoring at every PHI write)
  - Subcontractor BAA flowdown: required
```

## Step 5 — PostgreSQL schema provisioning

```
[Postgres] Provisioning tenant database
[Postgres] us-east-1 primary: schema "tenant_medcenter_001" created
  - Citus distributed tables: 47 (default oyatie µservice tables)
  - RLS policies: 47 generated
  - Default Cedar policies: 234
[Postgres] us-west-2 DR replica: schema "tenant_medcenter_001" mirrored
  - Sync lag: 0.3 s (initial)
  - WAL streaming: active
[Postgres] Per-tenant database connection pooling: PgBouncer pool sized 32
```

## Step 6 — IAM + default users

```
[IAM] Tenant binding created: tenant_id=tenant_medcenter_001
[IAM] Default Cedar policies provisioned: 234
[IAM] Initial admin user: ceo@medcenter.com (role: tenant_admin)
  - Invite email sent
  - WebAuthn registration pending
```

## Step 7 — sub-scope registry initialisation

```
[Sub-scope] Default sub-scope: "root" (tenant_id=tenant_medcenter_001)
[Sub-scope] Reserved sub-scopes: ["ehr-clinical", "billing", "admin", "audit"]
  These can be activated as needed; reserved means they can't be claimed for other purposes.
```

## Step 8 — DR pairing

```
[DR] Pairing configured:
  - Primary: us-east-1
  - DR: us-west-2
  - RPO target: 60 s
  - RTO target: 15 min
[DR] Initial sync complete; lag < 1 s
[DR] Health probes: every 30 s
[DR] First synthetic failover drill: scheduled for 2026-06-15 14:00 UTC
```

## Step 9 — audit-chain anchor

```
[audit-chain] Tenant provisioning sealed
[audit-chain] Anchor: bun_abc...def (Merkle root of provisioning lifecycle)
[audit-chain] Cross-emits to:
  - "tenancy::tenant::provisioned"
  - "tenancy::pack::applied (HIPAA-Provider)"
  - "tenancy::dr_pair::established"
  - "kyb::verification::completed"
  - "contract::baa::signed"
```

## Step 10 — verification + first user

```sh
oya tenancy tenant-status --tenant tenant_medcenter_001
```

Output:
```
Tenant: tenant_medcenter_001 (MedCenter LLC)
  Status: active
  Audience type: b2b-organization
  Country: US
  Data residency: us-east-1
  DR region: us-west-2
  Packs: [HIPAA-Provider]
  Active users: 1 (pending WebAuthn registration)
  Lifecycle locks: none
  Quotas:
    - Users: 0 / 50 000 (paid tenant_class expanded deployment)
    - Storage: 0 / 100 TB
    - API calls/day: 0 / 100 M
  Audit-chain anchors: 6 (provisioning events)
```

CEO completes WebAuthn registration → first user active.

Tenant is now ready for additional users + workload deployment.

## Step 11 — test the DR failover

Schedule a non-disruptive synthetic DR drill:

```sh
oya tenancy dr-drill \
    --tenant tenant_medcenter_001 \
    --mode synthetic \
    --duration 5min
```

Substrate simulates primary failure; routes a small synthetic-load fraction to DR; verifies DR responds correctly; auto-restores after 5 min. No customer impact.

After drill:
```
[DR drill] Synthetic failover: succeeded
[DR drill] Routing time: 4 min 12 s
[DR drill] Lost events during failover: 0
[DR drill] Audit-chain anchored: drill_drill_01HXYZ...
```

## Step 12 — operational handoff

```sh
oya tenancy handoff \
    --tenant tenant_medcenter_001 \
    --to "team:cs-tier3@your-platform.com" \
    --notes "MedCenter LLC; HIPAA-Provider pack; 5000 employees target; first deployment scheduled 2026-06-01."
```

The customer success team takes over for first-deployment workshops + ongoing support.

## What you've accomplished

A production B2B tenant provisioned with:
- KYB verified via Stripe Identity + D&B (low risk).
- HIPAA-Provider pack auto-applied with signed BAA.
- PostgreSQL schemas in us-east-1 primary + us-west-2 DR with < 1 s sync lag.
- IAM + default Cedar policies + first admin user.
- Sub-scope registry initialized with reserved future sub-scopes.
- DR failover capability verified via synthetic drill.
- Cryptographic audit-chain anchoring for every step.

## Common pitfalls

| Pitfall | Mitigation |
|---|---|
| KYB risk score borderline (0.3-0.6) | Substrate routes to manual review; coordinate with risk team for case-by-case decision |
| BAA signing delayed (tenant takes weeks) | Substrate timeout: 30 days; after that, provisioning auto-cancels + cleanup runs |
| DR region capacity-exhausted | Coordinate with capacity team; may need to commission new DR region or move existing tenants |
| Tenant initial admin email typo | Cancel + restart provisioning; substrate refuses partial-modify of pending lifecycle workflows |
| Sub-scope reserved name conflicts with tenant business unit | Choose a non-reserved alternative or work with substrate team for namespace allowlist exception |
