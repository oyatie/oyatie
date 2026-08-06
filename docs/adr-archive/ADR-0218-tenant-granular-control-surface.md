---
id: ADR-0218
status: Superseded
superseded_by: [ADR-0701]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0218: Tenant Granular Control Surface

- **Status:** Accepted
- **Date:** 2026-05-18
- **Owner:** council-architecture
- **Deciders:** council-architecture, axis-application, axis-identity, axis-tenancy, council-security
- **Lane:** governance / substrate-doctrine
- **Supersedes:** none
- **Superseded by:** none
- **Related:** ADR-0061, ADR-0212, ADR-0215, ADR-0216, ADR-0219, ADR-0220
- **Source:** `evidence/pr-143-session-decisions-checkpoint-2026-05-18.json#queued_adrs_to_author.ADR-0218`
- **Task:** #E substrate doctrines follow-up

## Context

Enterprise customers need more than a platform-level permission model. They need tenant-authored control over roles, policies, data classifications, approvals, product enablement, API keys, identity federation, environment tiers, and audit visibility.

Without a granular tenant control surface, every enterprise variation becomes a support ticket or a forked deployment. With an unsafe control surface, tenant admins can accidentally weaken platform-base-deny, leak personal context, or create roles that bypass audit expectations.

The checkpoint decision is to make tenant customization first-class while preserving platform guardrails. Tenant-authored controls compose with platform policies; they do not replace them.

## Decision

Ship a Tenant Admin Console inside the Application B2B shell. The console is the canonical tenant-facing control plane for:

- employees and roles, including SCIM-provisioned users and tenant-extension roles;
- products enabled a-la-carte per tenant;
- access policies through visual Cedar-fragment builders;
- tenant-scoped data classifications and labels;
- approval workflows through Workflow Studio templates;
- per-tenant audit visibility and audit-log slicing;
- environment tiers: test, staging, production;
- API keys per tier and per purpose;
- IdP federation through external OIDC or SAML where supported;
- JIT access grants for contractors and time-bound operators;
- per-product, per-role, per-data-class permission matrix.

Tenant-authored Cedar fragments are scoped to the tenant and compose with platform-base-deny. Tenant roles extend platform defaults but cannot remove mandatory platform roles or bypass break-glass, audit, or sovereignty checks. Tenant data-class extensions are tenant-scoped labels that map onto platform data-class semantics; they cannot erase platform classifications.

### Existing substrate used

- Cedar policy engine for authorization composition.
- SCIM 2.0 inbound provisioning for employees and groups.
- OIDC plus WebAuthn passkeys for authentication.
- Per-tenant audit-log slicing.
- Per-tenant environment tiers.
- Application B2B shell per ADR-0061.
- Tenancy RLS, JWT tenant claims, and cell isolation.

### UX requirement

The console must use no-code-first visual builders per ADR-0219. Raw Cedar, JSON, or CLI-only configuration is allowed for developers and advanced admins, but cannot be the primary path for normal tenant admins.

## In-house roadmap

Tenant control is Class C differentiation per ADR-0211 because it is the enterprise trust surface. The visual policy builder, role matrix, approval template authoring, and audit visibility are built in-house in the Application B2B shell.

Phase 1: product enablement, employee/role management, and read-only audit visibility. Phase 2: visual Cedar fragments, custom roles, and custom data classes. Phase 3: JIT access, approval workflows, and API-key tiering. Phase 4: advanced policy simulation, dry-run impact analysis, and per-tenant evidence export.

## Alternatives considered

### Alternative 1 - Platform-only roles

**Rejected because** enterprise tenants need local vocabulary and local delegation patterns. A fixed platform enum cannot represent every department, contractor model, approval path, or regulated data label.

### Alternative 2 - Raw Cedar editor as primary UX

**Rejected because** most tenant admins are not policy engineers. Raw policy text should exist for advanced users, but the primary UX must prevent accidental broad grants and must explain effective access visually.

### Alternative 3 - Support-ticket-driven tenant configuration

**Rejected because** it does not scale and creates hidden configuration state outside audit-chain visibility. Self-service with evidence emission is the only credible hyperscaler-grade control plane.

### Alternative 4 - Full tenant override of platform guardrails

**Rejected because** tenants can own their local roles and policies, but they cannot disable platform-base-deny, sovereignty enforcement, audit-chain emission, or personal-context isolation.

## Consequences

### Positive

- Enterprise tenants can adapt Oyatie to their organization without code forks.
- Security teams get inspectable effective access and audit evidence.
- Product enablement becomes self-service and tied to billing, compliance, and data-class policy.
- Tenant controls reinforce ADR-0215 context isolation instead of bypassing it.

### Negative

- Policy composition and simulation become product requirements, not just backend internals.
- UI mistakes can create overbroad access if builders do not explain scope clearly.
- Custom roles and data classes require migration, export, and support semantics.

### Operational

- Tenant-authored policy changes emit audit-chain events and require rollback metadata.
- Visual builders must include preview, simulation, and diff before activation.
- JIT grants must have explicit expiration and owner.
- API keys must bind to tier, purpose, scopes, and rotation schedule.
- Tenant Admin Console changes must be covered by runbooks for bad-policy rollback and emergency deny.

## Named industry sources

- Okta and Microsoft Entra admin centers: enterprise identity buyers expect delegated user, group, role, and IdP controls.
- Stripe Dashboard: API keys, environments, and purpose-scoped access are table-stakes for developer-facing enterprise control.
- AWS IAM and Organizations: customers expect local policy composition but with provider guardrails.
- ServiceNow workflow administration: business admins expect configurable approvals without writing code.
- Google Workspace Admin Console: product enablement and user controls sit in one tenant-facing admin shell.

## References

- ADR-0061: Application B2B shell hosts the Tenant Admin Console.
- ADR-0212: Buildability doctrine applies to console IPs and runbooks.
- ADR-0215: Context isolation constrains tenant-admin visibility.
- ADR-0216: Import/export and API-key management must be tenant-visible.
- ADR-0219: No-code-first UX shapes visual builders.
- ADR-0220: Intelligence features must respect tenant controls, consent, and cost attribution.
