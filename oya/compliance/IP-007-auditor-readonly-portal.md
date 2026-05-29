---
microservice: compliance
ip: IP-007
title: Auditor read-only portal (Backstage plugin + Cedar per-engagement role binding)
status: Drafting
authority_tier: 3
owner: axis-compliance
co_owners: [axis-frontend]
date: 2026-05-18
related_adrs: [ADR-0170, ADR-0183, ADR-0207, ADR-0209]
---

# IP-007 — Auditor read-only portal

## Purpose

Stand up a Backstage plugin (per ADR-0170 developer portal) that gives external auditors read-only access to per-framework artifact inventories + audit-chain seal verification. Per-engagement Cedar role binding ensures access expires on engagement close. WCAG 2.2 AA per ADR-0207.

## Acceptance criteria

1. Backstage plugin at `microservices/compliance/clients/auditor-portal/`.
2. Routes:
   - `/auditor/<framework>/` — artifact inventory + filters.
   - `/auditor/seal-verify/<artifact_id>` — verify audit-chain seal.
   - `/auditor/engagement/<id>/` — engagement scope + active artifact set.
3. Authentication: Zitadel OIDC; per-engagement identity.
4. Authorization: Cedar capability `auditor:engagement-<id>:read`; scoped to engagement's tenant set.
5. Audit log: `EVT-AUDITOR-ARTIFACT-VIEWED` per view.
6. Engagement-end webhook revokes Cedar role binding; integration test asserts revoke.
7. WCAG 2.2 AA: axe-core + pa11y CI gate green.
8. RTL support for ar-SA + he-IL locales (some auditors in MENA).
9. ≥ 8 integration tests: auditor-can-read + auditor-cannot-cross-tenant + seal-verify-success + seal-verify-failure-banner + revoke-on-engagement-end + axe-core-green + EVT-AUDITOR-ARTIFACT-VIEWED-emitted + i18n-ar-SA-renders-rtl.

## Cedar capability

```cedar
// capabilities/auditor-engagement-read.cedar
permit (
  principal in Auditor::"engagement-<id>",
  action == Action::"read-artifact",
  resource is EvidenceArtifact
) when {
  resource.tenant_id in principal.engagement_tenants &&
  resource.emitted_unix_ms >= principal.engagement_window_open_unix_ms &&
  resource.emitted_unix_ms <= principal.engagement_window_close_unix_ms
};
```

## Engagement lifecycle

```
[admin opens engagement]
  → Cedar role bound: auditor:engagement-X:read with tenants=[A, B]
  → engagement_window_open_unix_ms = now()
  → engagement_window_close_unix_ms = now() + 30 days (default)
[auditor accesses portal]
  → Zitadel OIDC → engagement identity
  → Cedar policy evaluator scopes artifact queries
  → EVT-AUDITOR-ARTIFACT-VIEWED per fetch
[engagement closes]
  → webhook revokes Cedar role binding
  → auditor sessions terminated
  → audit-chain seal records engagement close
```

## A11y commitments

- Per WCAG 2.2 AA (per ADR-0207).
- Keyboard navigable end-to-end.
- Screen-reader-friendly (live region announces audit-chain seal verify results).
- High-contrast theme available for low-vision auditors.

## Risk + mitigation

- **Risk:** auditor session persistence post-engagement (cookie leak). **Mitigation:** session TTL bound to engagement_window_close; Zitadel revokes on close.
- **Risk:** auditor downloads gigabytes of artifacts and walks them off. **Mitigation:** audit-log of every view; Cedar policy can scope to artifact metadata only (no payload download) for sensitive frameworks.

## Acceptance evidence

`evidence/ip-007-auditor-portal-acceptance.json`.

## Cross-references

- ADR-0170 — developer portal (Backstage).
- ADR-0183 — Cedar policy.
- ADR-0207 — a11y.
- ADR-0209 — substrate authority.
- IP-002 — SOC 2 control mapping (drives per-framework inventory rendering).

## Pod runtime tier (per ADR-0338)
- `pod_runtime_tier: 0`
- Runtime: Kata Containers plus Cloud Hypervisor are REQUIRED for this tenant-customer execution path.
- Justification: this IP matched `plugin`, so tenant-customer or third-party code can enter the execution path.
- Surface evidence: `microservices/compliance/IP-007-auditor-readonly-portal.md` plus `crates/oya-shared-compliance-evidence-kernel/src/lib.rs`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.
