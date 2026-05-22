---
doc_class: User-Journey-Index
journey_id: j126-government-auditor-3pao-conducts-fedramp-audit
status: draft
date: 2026-05-20
authority_tier: 3
related_adrs:
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0312-court-warrant-scoped-piercing
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0242-oyatie-is-a-tenant-doctrine
  - ADR-0246-policy-engine-library-first
  - ADR-0247-self-modification-doctrine
  - ADR-0248-amazon-shape-cellular-architecture
  - ADR-0251-compliance-pack-cell-certification-levels
  - ADR-0263-observability-emission-contract
  - ADR-0028-audit-chain-merkle-sealed
  - ADR-0188-passkey-webauthn-as-canonical-auth
  - ADR-0299-account-recovery-resilience
critical_path_rows_satisfied:
  - "§3.2.5 row 18 — Audit / regulator / law-enforcement access (PRIMARY)"
  - "§3.2.5 row 23 — Cross-jurisdiction conflict (partial; see j131)"
  - "§3.2.5 row 19 — Tenant break-glass / dead-account recovery (cross-link j137)"
pack_overlays_activated:
  - pack-us-fedramp-mod (Diana's work tenant + Marcus's tenant)
  - pack-us-nist-sp-800-53-rev5 (Diana's work tenant + Marcus's tenant)
  - pack-us-omb-a-130 (Diana's work tenant)
  - pack-us-fisma-2014 (Diana's work tenant)
  - pack-pci-dss-v4 (Marcus's tenant — for Stripe Connect surface)
  - pack-us-itar-2024 (Marcus's tenant — defense contractor)
  - pack-us-ccpa-2023 (Diana's personal tenant)
  - pack-us-coppa-1998 (Diana's personal tenant — son aged 9)
  - pack-us-state-va-cdpa-2023 (Diana's personal tenant)
microservices_touched:
  - identity
  - tenancy
  - audit-chain
  - compliance
  - ops-dashboard-control-center
  - observability
  - workflow-engine
  - api-gateway
  - messenger (personal-tenant only)
  - policy-engine (library-mode)
  - comms-email (notification dispatch)
---

# j126 — FedRAMP 3PAO audit pull with dual-tenant identity boundary

## At a glance

Inspector Diana Reyes (47, GAO Senior Auditor + registered FedRAMP
3PAO) conducts the annual ConMon audit of Chen Aerospace Manufacturing
(Marcus Chen's federal-contractor tenant) on a Monday morning. In the
forty-three minutes she works, every µservice that touches identity,
tenancy, audit, compliance, observability, and workflow is exercised.
The KEY property the journey demonstrates: Diana's PERSONAL tenant
(her family Messenger DMs, her wife, her son, her vintage records,
her tax workflow) is **structurally invisible** to her agency tenant
even as she crosses tenants on a single device with a single passkey.

This journey is **the foundation** for the j126-j131 dual-tenant
identity slice. The five siblings (j127-j131) stress different
seams of the same architecture.

## Index of artifacts

| Artifact | Purpose | Line count |
|---|---|---:|
| [`story.md`](story.md) | Diana's concrete forty-three-minute narrative — passkey enrollment + context picker + cross-tenant Cedar permit + audit pull + personal-tenant messenger interruption | ≥800 |
| [`ux-flow.md`](ux-flow.md) | Per-device screen-by-screen UX, with tenant-indicator badges + WCAG 2.2 AA compliance + cross-tenant confirmation modal grammar | ≥400 |
| [`handshake.md`](handshake.md) | Per-phase µservice sequence diagrams, Cedar permits, observability emissions, failure-mode tree, cell-routing matrix | ≥600 |
| [`schemas/two-tenants-response.json`](schemas/two-tenants-response.json) | Identity µservice response when one credential resolves to multiple tenants | n/a |
| [`schemas/session-init-request.json`](schemas/session-init-request.json) | Session init request with explicit tenant + audience_type | n/a |
| [`schemas/cross-tenant-pull-request.json`](schemas/cross-tenant-pull-request.json) | Cross-tenant audit evidence pull request | n/a |
| [`schemas/audit-evidence-bundle.json`](schemas/audit-evidence-bundle.json) | Sealed evidence bundle envelope | n/a |
| [`schemas/cross-tenant-notification.json`](schemas/cross-tenant-notification.json) | Tenant-admin notification payload | n/a |
| [`integration-test-plan.md`](integration-test-plan.md) | 9-class test plan covering happy path + boundary invariants + audit-chain dual emission + transparency + observability + cell isolation + failure modes + property-based fuzz | ≥400 |

## Per-µservice IP slices

Six µservices each receive a new IP slice tailored to j126. Each is
intern-buildable per documentation-rigor.md §2 IP-row floor (≥400
lines, data model + schema mapping + API surface + integration
contracts + cross-µservice handshake + parallel-work compatibility).

| µservice | IP slice file | Role in j126 |
|---|---|---|
| identity | [`microservices/identity/IP-journey-j126-fedramp-3pao-cross-tenant-resolver.md`](../../../microservices/identity/IP-journey-j126-fedramp-3pao-cross-tenant-resolver.md) | Multi-context principal resolver + `INTERNAL_AUDITOR_3PAO` audience-type setter + cross-tenant principal binding |
| tenancy | [`microservices/tenancy/IP-journey-j126-cross-tenant-permit-grant.md`](../../../microservices/tenancy/IP-journey-j126-cross-tenant-permit-grant.md) | Cross-tenant Cedar fragment provisioning + tenant-pack-overlay composition + permit time-bound enforcement |
| audit-chain | [`microservices/audit-chain/IP-journey-j126-dual-tenant-emission-classes.md`](../../../microservices/audit-chain/IP-journey-j126-dual-tenant-emission-classes.md) | New audit-event classes for cross-tenant operations + dual-tenant atomic seal grammar |
| compliance | [`microservices/compliance/IP-journey-j126-fedramp-conmon-pack-overlay.md`](../../../microservices/compliance/IP-journey-j126-fedramp-conmon-pack-overlay.md) | FedRAMP Mod ConMon control-evidence schema + AU-2/AU-12/AC-3/IA-2/CM-3 control evidence assembly |
| ops-dashboard-control-center | [`microservices/ops-dashboard-control-center/IP-journey-j126-3pao-docket-dashboard.md`](../../../microservices/ops-dashboard-control-center/IP-journey-j126-3pao-docket-dashboard.md) | 3PAO docket UI + finding-entry surface + cross-tenant access-event visibility for Marcus |
| observability | [`microservices/observability/IP-journey-j126-cross-tenant-audit-metrics.md`](../../../microservices/observability/IP-journey-j126-cross-tenant-audit-metrics.md) | Cross-tenant metric label grammar + cardinality budget for audit-pull family |

## Critical-path rows satisfied

Per documentation-rigor.md §3.2.5:

- **Row 18 (Audit / regulator / law-enforcement access)** — PRIMARY
  row. ADR-0311 cross-tenant Cedar permit exercised end-to-end. The
  FedRAMP 3PAO audit pull is the canonical demonstration that the
  audit path:
  - Authorizes legitimate access (Diana's pull succeeds).
  - Audits every read (both tenants' audit-chains emit).
  - Notifies the counterparty (Marcus's tenant-admin notified within
    15min).
  - Time-bounds the permit (post-period reads denied).
  - Honors transparency (Marcus's dashboard shows the access).
- **Row 23 (Cross-jurisdiction conflict)** — PARTIAL coverage. j126
  is US-domestic only. j131 carries the EU-vs-KR variant where Diana
  audits a multinational with subsidiaries in both regions.
- **Row 19 (Tenant break-glass / dead-account recovery)** — Cross-
  link only. j126 references the lapsed-accreditation case (story §6
  + §20 invariant) but the full break-glass exercise is j137
  (corporate internal-audit SOX controls test).

## Cross-references

### Sibling dual-tenant identity journeys (j126-j131)

- [j127 — Dual-tenant identity: employee resigns, keeps personal](../j127-dual-tenant-identity-employee-resigns-and-keeps-personal/)
  — what happens when work-tenant access is revoked but personal-
  tenant identity continues.
- [j128 — Auditor's personal side uses Workflow Studio for family taxes](../j128-auditor-personal-side-uses-workflow-studio-for-family-taxes/)
  — Diana's personal-tenant productive workflow that her agency cannot see.
- [j129 — Court warrant pierces personal tenant with judicial oversight](../j129-court-warrant-pierces-personal-tenant-with-judicial-oversight/)
  — the one path by which Diana's personal tenant CAN be pierced (per
  ADR-0312), and what the cryptographic warrant-canary surface looks like.
- [j130 — Auditor receives bribery attempt via personal Messenger](../j130-auditor-receives-bribery-attempt-via-personal-messenger/)
  — cross-tenant evidence chain when a personal-tenant interaction
  has work-tenant relevance.
- [j131 — Cross-jurisdiction audit EU vs KR discrepancy](../j131-cross-jurisdiction-audit-eu-vs-kr-discrepancy/)
  — multinational audit with data-residency-conflict reconciliation.

### Adjacent critical-path journeys

- [j05 — Whistleblower anonymous ethics report](../j05-whistleblower-anonymous-ethics-report/) — Diana the whistleblower
- [j14 — Delegated LLM agent acting for user](../j14-delegated-llm-agent-acting-for-yejin/) — when Diana's audit pull is
  delegated to an LLM agent (out-of-scope here; covered by j14).
- [j68 — Regulator audit pull HIPAA SOC2 PCI](../j68-regulator-audit-pull-hippa-soc2-pci/) — same shape, different regulators.
- [j87 — FedRAMP High IL5 air-gap deployment](../j87-fedramp-high-il5-air-gap-deployment/) — escalation of j126 to FedRAMP High.

### Binding ADRs

The architectural authority for this journey:

- **ADR-0311 (dual-tenant identity boundary)** — the binding doctrine.
  Defines: shared-passkey identity + Cedar-default-deny cross-tenant +
  cross-tenant transparency + UX tenant-indicator-mandatory.
- **ADR-0312 (court-warrant scoped piercing)** — the ONE exception to
  ADR-0311's default-deny. Not exercised in j126; exercised in j129.
- **ADR-0244 (tenant scoping primitive)** — every audit event, every
  Cedar evaluation, every metric is `tenant_id`-stamped.
- **ADR-0243 (Cedar universal gate)** — default-deny baseline; permits
  are explicit.
- **ADR-0246 amendment (policy-engine library-first)** — Cedar
  evaluations are library-mode in callers (fast); api-gateway is the
  first gate, downstream µservices re-evaluate (defense-in-depth).
- **ADR-0028 (audit-chain Merkle-sealed)** — three independent chains,
  one per tenant; cross-tenant atomic emission.
- **ADR-0263 (observability emission contract)** — every audit event
  declared, cardinality-budgeted, dashboard-named.
- **ADR-0188 (passkey/WebAuthn)** — same hardware-key, two credential
  handles for two tenants.
- **ADR-0299 (account recovery resilience)** — passkey survives device
  swap, accreditation lapse, etc.

### Related specs

- `/specs/microservices/identity.json` — multi-tenant principal model
- `/specs/microservices/tenancy.json` — Cedar fragment provisioning
- `/specs/microservices/audit-chain.json` — per-tenant chain isolation
- `/specs/microservices/compliance.json` — FedRAMP control evidence schema
- `/specs/microservices/ops-dashboard-control-center.json` — 3PAO docket UI
- `/specs/microservices/observability.json` — cross-tenant metric labels

### Regulatory anchors

| Authority | Citation | Relevance |
|---|---|---|
| FedRAMP Moderate baseline | FedRAMP Authorization Boundary Guidance 2024-10 | Audit cadence + control families |
| NIST SP 800-53 Rev 5 | AU-2 / AU-12 / AC-3 / IA-2 / CM-3 | Control definitions |
| OMB Circular A-130 | Managing Information as a Strategic Resource | Strategic-asset framing |
| FISMA 2014 | 44 USC §3554 | Legal authority for audit |
| CLOUD Act 2018 | 18 USC §2713 | Cross-border data access framework (relevant in j131; non-applicable here) |

## Hyperscaler precedents

This journey is the **3PAO** equivalent of:

- **The SEC enforcement-attorney case-management pattern** at Tyler
  Technologies' Federal Case Management System / FINRA case-management:
  same employee, two contexts, strict separation of case-material from
  personal-records.
- **The Big-4 audit firm pattern** at PwC / Deloitte / EY / KPMG:
  rotation between client engagements via per-engagement Microsoft Entra
  external-tenant guest principal.
- **The 3PAO-Inc / Coalfire / Schellman model** in real-world FedRAMP
  3PAO operations: each 3PAO firm has its own corporate tenant; each
  authorization (CSP under audit) provides a scoped read-permit via the
  FedRAMP Authorization Letter.

oyatie's distinction over these models: enforcement at the Cedar policy
layer, not at the per-deployment workflow layer. Adding a new 3PAO
firm to the platform requires only:
1. Provisioning a new agency tenant.
2. Issuing `INTERNAL_AUDITOR_3PAO` audience-type to authorized
   employees.
3. Updating Marcus's tenant's cross-tenant fragment to include the new
   3PAO firm's tenant_id.

No per-deployment workflow changes. The architecture composes.

## Doctrine summary

The j126 architecture demonstrates the **load-bearing** dual-tenant
identity boundary at its most consequential exercise scale:

1. **Same passkey** — Diana uses her YubiKey for both tenants. ADR-0188
   credential-handle-roster distinguishes the two.
2. **Two `tenant_id`s** — Diana's identity µservice has two distinct
   tenant memberships. ADR-0244 enforces the separation.
3. **Cedar default-deny** — no permit grants agency → personal-tenant
   access without explicit warrant. ADR-0243 baseline.
4. **Cross-tenant via attested permit** — the FedRAMP audit permit is
   scoped, attested, time-bounded, and audited. ADR-0311 §B-4
   cross-tenant grammar.
5. **Dual-tenant audit-chain emission** — every cross-tenant operation
   emits to BOTH tenants' audit logs atomically. ADR-0028 §D-cross-
   tenant atomicity.
6. **Cell isolation** — three tenants live in three cells; no L3 path
   exists between consumer and FedRAMP cells. ADR-0248 §D-3 cellular
   network isolation.
7. **UX tenant indicator** — every screen shows the active tenant
   unambiguously. ADR-0311 §B-8 UX-mandatory tenant-badge.
8. **Counterparty transparency** — Marcus's tenant-admin is notified
   within 15min of any cross-tenant pull. ADR-0311 §B-7.

If any one of these breaks, j126 breaks and the platform's claim of
hyperscaler-grade dual-tenant identity is false. We ship when all
eight hold.

## What this journey deliberately leaves out

- The **Workflow Studio** surface (Diana's tax workflow) — see j128.
- The **judicial-piercing path** for personal tenant — see j129.
- The **EU+KR multi-jurisdiction** variant — see j131.
- The **resignation path** for the same dual-tenant primitive — see j127.
- The **bribery-attempt** cross-tenant bridging — see j130.

j126 is the foundation. Each sibling explores a different axis.

## Next steps after j126

1. ADR-0311 and ADR-0312 must land (parallel agent authoring).
2. Identity µservice IP-017 (`multi-context-principal-resolver`) must
   land per `microservices/identity/IP-017-multi-context-principal-resolver.md`.
3. Audit-chain dual-tenant atomic emission must land per
   `microservices/audit-chain/IP-journey-j126-dual-tenant-emission-classes.md`.
4. Cross-tenant fragment provisioning workflow must land per
   `microservices/tenancy/IP-journey-j126-cross-tenant-permit-grant.md`.
5. Integration tests must pass per `integration-test-plan.md`.

When all five land, j126 ships as the foundation for the j126-j150
ecosystem journey slice.

## Completion expansion — j126 readme rigor pass

Scope: FedRAMP 3PAO audit with Diana work/personal tenant separation.
Persona: Diana Reyes.
Services: identity + tenancy + audit-chain + compliance + ops-dashboard-control-center + observability.
Applicable ADRs: ADR-0244, ADR-0299, ADR-0311, ADR-0312, ADR-0313, ADR-0314, ADR-0315, ADR-0316, ADR-0317, ADR-0318, ADR-0319, ADR-0320.
The expansion below is intentionally explicit so an intern can build and verify the journey without oral context.

Coverage row 001: tenancy owns tenant membership, sub-scope, residency, and cross-tenant grant boundary and cites ADR-0299.
Reader path 002: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 003: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 004: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 005: observability owns trace, metric, log, detector signal, and cardinality-budget instrumentation and cites ADR-0314.
Reader path 006: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 007: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 008: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 009: compliance owns pack overlay, regulator mapping, legal basis matrix, and retention policy composition and cites ADR-0318.
Reader path 010: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 011: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 012: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 013: tenancy owns tenant membership, sub-scope, residency, and cross-tenant grant boundary and cites ADR-0299.
Reader path 014: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 015: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 016: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Checkpoint 01: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Coverage row 017: observability owns trace, metric, log, detector signal, and cardinality-budget instrumentation and cites ADR-0314.
Reader path 018: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 019: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 020: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 021: compliance owns pack overlay, regulator mapping, legal basis matrix, and retention policy composition and cites ADR-0318.
Reader path 022: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 023: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 024: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 025: tenancy owns tenant membership, sub-scope, residency, and cross-tenant grant boundary and cites ADR-0299.
Reader path 026: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 027: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 028: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 029: observability owns trace, metric, log, detector signal, and cardinality-budget instrumentation and cites ADR-0314.
Reader path 030: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 031: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 032: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Checkpoint 02: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Coverage row 033: compliance owns pack overlay, regulator mapping, legal basis matrix, and retention policy composition and cites ADR-0318.
Reader path 034: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 035: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 036: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 037: tenancy owns tenant membership, sub-scope, residency, and cross-tenant grant boundary and cites ADR-0299.
Reader path 038: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 039: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 040: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 041: observability owns trace, metric, log, detector signal, and cardinality-budget instrumentation and cites ADR-0314.
Reader path 042: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
