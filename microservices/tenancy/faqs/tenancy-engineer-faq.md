---
doc_class: FAQ
microservice: tenancy
persona: tenancy-engineer + iam-engineer + platform-engineer
related_adrs: [ADR-TEN-001, ADR-0313, ADR-0009]
date: 2026-05-20
doc_status: published
---

# Tenancy Engineer FAQ — tenancy

## Why 10 lifecycle states (instead of just active/suspended/deleted)?

Per ADR-TEN-001 § Alternatives Considered. Three or four states cannot model:

- KYB/KYC verification gates (`kyb_pending`).
- Provisioning fan-out (downstream µservices need a `provisioning` state to coordinate).
- Restricted operation (`restricted` — between active and suspended; partial denial).
- Offboarding with retention obligations (`offboarding`, `retained`).
- Cryptoshred ceremony (`cryptoshredded` — destructive but recoverable for audit; then `retired`).

Compliance demands these distinct states. Audit-chain queries must answer "was this tenant `active` on date X" — fuzzy states fail audit.

## What does "owns" mean if it doesn't imply data_read?

Per ADR-TEN-001 Constraint TEN-C10 + § Decision. `owns` is **administrative authority**:

- Can manage billing, policy, governance.
- Can read administrative metadata (file counts, msg counts, user counts) at aggregate level.
- Can request data-plane access via explicit Cedar permit (always logged + audit-chained).

Why this matters: a holding company owning a healthcare subsidiary CAN administratively control the subsidiary but CANNOT read patient records without explicit subsidiary admin grant + HIPAA-compliant audit trail.

Separating administrative authority from data-plane access is a key conglomerate primitive that competitors miss.

## What's "sovereign child" (ADR-0313)?

Per ADR-TEN-001 Constraint TEN-C5 + ADR-0313. A child tenant whose pack policy denies parent overrides. Example:

- Parent = Berkshire-Hathaway-style conglomerate (operations holding company).
- Child = GEICO (US-state-regulated insurer).
- Pack: state-insurance-regulator pack restricts cross-corporate-boundary data access.

Even though the parent administratively `owns` the child, the parent CANNOT decrypt child policyholder data because the child's pack policy denies parent access. Cedar `tenancy::relationship::parent_override_sovereign_child` is **forbidden** when child pack denies.

This is critical for M&A scenarios where regulatory requirements bind the acquired entity independently.

## How does DSR cascade work?

Per ADR-TEN-001 § Decision + IP-016-sub-scope-registry-kernel. When a tenant enters `offboarding`:

1. tenancy emits `tenancy.offboarding.cascade.requested.v1` to Kafka.
2. Each product µservice (drive, messenger, mail, calendar, identity, etc.) consumes + processes:
   - drive: schedule per-file deletion + CMK cryptoshred plan.
   - messenger: schedule MLS group removal + cryptoshred.
   - mail: schedule mailbox closure + retention.
   - calendar: cancel future events + archive past.
   - identity: revoke active sessions + queue passkey revocation.
3. Each emits `<service>.tenant.offboarding.acknowledged.v1` when complete.
4. tenancy waits for all expected services to ack (or timeout per pack policy).
5. Move to `retained` (retention window) → `cryptoshredded` (destructive) → `retired` (audit metadata only).

Per ADR-TEN-001 § Constraint TEN-C8: offboarding must cascade to data-bearing services without leaving hidden active grants.

## What's the conglomerate depth limit?

Per ADR-TEN-001 Implementation Notes + capacity envelope:

- demo_trial: 1 level (effectively flat).
- paid tenant_class baseline: up to 5 levels.
- paid tenant_class regulated-pack overlay: up to 10 levels.

Why 10 (not unlimited): graph traversal complexity grows; permit prefetch becomes expensive; Cedar evaluation time increases. 10 levels covers virtually all real-world holding-company structures (typical M&A tree depth: 3-5 levels).

Cycles are prevented by the relationship-graph rejects-cycles fuzz test (per ADR-TEN-001 verification).

## Why is parent-child relationship time-bounded?

Per ADR-TEN-001 Constraint TEN-C9. Permits with `starts_at` + `ends_at` because:

1. M&A windows: relationship may be valid only during a transitional period (e.g., 90 d post-acquisition).
2. Engagement scopes: vendor accessing customer data only during the contract.
3. Audit-friendly: clear scope-of-authority for each time period (regulator can replay "who had access on date X").

After `ends_at`, Cedar denies the action. Active workflows under the permit receive cancellation events (per ADR-TEN-001 § Decision).

## How does lifecycle lock work?

Per ADR-TEN-001 § Decision + IP-021-lifecycle-locks-kernel. A `LifecycleLock` freezes destructive transitions during incidents:

Lock types:

- `incident_freeze`: applied by incident commander.
- `legal_hold`: applied by legal counsel (per pack policy).
- `regulator_hold`: applied during regulator engagement.
- `audit_freeze`: applied during external audit.

While a lock is active, Cedar denies:
- Lifecycle transitions to `suspended`, `offboarding`, `cryptoshredded`.
- Destructive permit revocations.
- Tenant migration.

Locks expire automatically (default 7 d) or via explicit release ceremony.

## How is cross-region tenant migration handled?

Per ADR-TEN-001 § Decision + IP-024-tenant-migration-ceremony. Cross-region migration is a council-approved ceremony:

1. Tenant admin requests migration with destination cell.
2. Compliance pack residency check (some packs deny cross-region migration).
3. Council (multi-party) approves; audit-chain seals.
4. Provisioning in destination cell starts.
5. Per-µservice migration (drive files re-encrypted under destination KEK, messenger MLS groups recreated, etc.).
6. Identity continuity (sessions remain valid; signing keys rotated to destination).
7. DNS + load-balancer flip.
8. Source cell becomes read-only for soak period (90 d).
9. Source cell decommissioned.

Migrations are rare (typical: 0-2 per tenant per year). Reasons: regulatory residency change, M&A, performance.

## What's the workforce-personal-tenant boundary?

Per `feedback_oyatie_is_a_tenant_doctrine` + ADR-TEN-001 Constraint TEN-C6. A human can have:

- A workforce tenant principal (e.g., u-alice@acme-corp.com, tenant=acme-corp, audience_type=workforce).
- A personal tenant principal (e.g., u-alice@personal-tenant, tenant=alice-personal, audience_type=personal).

The personal tenant is NEVER admin-recoverable through the workforce tenant. The acme-corp admin CANNOT recover Alice's personal-tenant credentials even if Alice forgets them. Per identity recovery: only Alice + her recovery passphrase + her recovery code can.

This dual-context boundary is enforced by:
- Identity µservice (recovery isolation per ADR-ID-001 Constraint ID-C8).
- Tenancy µservice (relationship type cannot span workforce→personal).
- Cedar policy (cross-context Cedar permits require explicit dual-context proof).

## What happens to data when a tenant is `retired`?

Per ADR-TEN-001 § Decision. The `retired` state keeps:

- Tenant ID + display name (for audit reference).
- Lifecycle transition timestamps.
- Pack subscription history.
- Audit-chain event IDs.

The `retired` state does NOT keep:

- Content (cryptoshredded in the `cryptoshredded` state).
- User credentials.
- Sessions.
- Recovery envelopes.

Audit-chain retains the transition history forever (or until pack retention class expires). This satisfies regulator requirements like SEC 17a-4 (7-y record retention) without keeping decryptable content.

## How are quotas enforced?

Per ADR-TEN-001 + IP-009-quota-engine. Tenant has per-resource quotas:

- Storage (drive, mail, messenger ciphertext).
- Msgs/sec (messenger send rate).
- Users (identity principals).
- Channels (messenger).
- Files (drive).
- Events (calendar).
- Recurrence-expansion (calendar; per ADR-CAL-001).

Quotas are tenant-set or pack-default. Enforcement is at the producing µservice (e.g., drive rejects uploads at storage quota). Tenancy provides the quota lookup API + Cedar policy fragments.

## How is KYB/KYC integrated?

Per ADR-TEN-001 § Decision + IP-019-kyb-kyc-integration. Provider integrations:

- Persona (default).
- Onfido (alternative).
- Jumio (alternative).
- Sumsub (regulated finance).

Per-tenant pack policy determines required verification level:
- Standard B2B: company-document verification + UBO disclosure.
- Financial regulated: full UBO + sanctions screening + AML.
- Healthcare: BAA + compliance officer KYC.

KYB result feeds the `kyb_pending → provisioning` transition. Audit-chain records the verification evidence ID.

## What's the relationship between tenancy and identity µservices?

- **tenancy**: owns tenant lifecycle + relationships + permits. Tenant-scoped primitive.
- **identity**: owns principal authentication + credentials + recovery. Principal-scoped primitive.

A principal is bound to a tenant. Tenancy emits `tenancy.tenant.created.v1` events that identity consumes (creates the default admin principal). Identity emits `identity.user.created.v1` events that tenancy consumes (updates principal-count metric for quota).

## How does tenancy enforce pack residency?

Per ADR-TEN-001 Constraint TEN-C4 + ADR-0009. When a tenant is provisioned:

1. Pack residency requirements computed from `pack_set`.
2. Eligible cells filtered (e.g., HIPAA → US-East cells; GDPR → EU cells).
3. Cell assigned during `provisioning` transition.
4. Once `active`, home cell IMMUTABLE except via migration ceremony.
5. Cross-region permits validated against pack residency at evaluation time.

The home cell becomes part of the tenant's identity. Tokens include `home_cell` claim per ADR-ID-001.

## How are multiple packs reconciled (higher-restriction-wins)?

Per ADR-TEN-001 + ADR-COMP-001. When a tenant has multiple packs:

1. Each pack rule has `restriction_level` (0-10; higher = stricter).
2. For each action, candidate rules from each pack are collected.
3. Highest `restriction_level` wins.
4. Tenant-explicit stricter policy can raise floor but never weaken regulator floor.

Example: tenant subscribes to GDPR + HIPAA. Both packs touch "data retention". GDPR says 90 d unless legal hold; HIPAA says 6 y minimum. Higher restriction wins: 6 y retention.

## How does migration from Azure AD B2C work?

See `migration-playbooks/from-azure-ad-b2c.md` for details. Short version:

1. Export Azure AD B2C tenants + users + relationships.
2. Run `oya tenancy migrate import-azure-b2c`.
3. Map Azure B2C custom policies → Cedar policies.
4. Re-issue tenant SCIM tokens.
5. Bridge mode: oyatie tenancy accepts Azure B2C tokens for 60-90 d.
6. Cutover.
