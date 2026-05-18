---
doc_kind: implementation-plan
id: IP-016
title: Tenant Admin Console control surface
status: Accepted
owner_team: axis-application
related_adrs: [ADR-0215, ADR-0218, ADR-0219]
---

# IP-016: Tenant Admin Console Control Surface

## Intent

Add the Application-shell surface required by ADR-0218: tenant admins manage tenant-local controls through no-code builders with Cedar-backed policy simulation and audit-chain evidence.

## Scope

- Contract: `contracts/openapi/tenant-admin-console.yaml`.
- Capability: `capabilities/tenant-admin-console-control.yaml`.
- Policy: `policy/tenant-admin-console.cedar`.
- Audit events for policy draft, policy apply, and JIT access review.

## Acceptance

- Tenant admins can only manage work-context tenant controls.
- Personal contexts are hidden by construction.
- Drafts are simulated before apply.
- Applied changes emit audit-chain events with previous and next policy fragment ids.
