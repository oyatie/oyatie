---
id: ADR-0189
status: Superseded
deciders: council-architecture, axis-identity, ops-security, council-compliance
date: 2026-05-18
owner: axis-identity
supersedes: []
superseded_by: [ADR-0709]
related: [ADR-0145, ADR-0183, ADR-0187, ADR-0188, ADR-0162-per-tenant-audit-log-slicing]
related_specs:
  - /specs/microservices/manifest-schema.json
microservice: identity
versions_current_as_of: 2026-05-18
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0189 — Step-up authentication ACR classes (`routine`, `elevated`, `sensitive`, `critical`); ACR-bound Cedar gates

## Status

Accepted (2026-05-18). Defines four ACR (Authentication Context Class Reference) levels with explicit factor requirements + maximum session age per level + Cedar policy obligation to enforce step-up at the waypoint ext_authz tier.

## Context

OIDC `acr` (Authentication Context Class Reference, RFC 9068 §3.1) is the canonical claim conveying "how strong was this auth event." Without a controlled enum + per-operation policy, `acr` becomes a free-form string that fragments across products. The hyperscaler reference: Stripe's "verified mode" (re-auth for secret-key rotation, dashboard sensitive ops), Google Workspace's "Recent sensitive action" prompt (15-minute window for IAM changes), AWS Console MFA step-up (15-minute re-auth for billing changes).

The unification: operations have intrinsic risk tiers, and risk tiers map to ACR floor requirements. The policy decision point (Cedar PDP at the waypoint per ADR-0183) must refuse any action whose required ACR exceeds the principal's current ACR — even if the principal is otherwise authorized.

## Decision

**Four ACR classes, named `routine`, `elevated`, `sensitive`, `critical`. Each declares min-factor count, accepted factor mix, max session age. Cedar policies attach an `acr_required` to every action; ext_authz returns `step_up_required` when the principal's ACR is below the floor. The OIDC ID-token carries `acr` as a string-enum claim per RFC 9068.**

### ACR enum

| ACR | Factors required | Accepted mix | Max session age | Max idle | Examples |
|---|---|---|---|---|---|
| `routine` | 1 (Passkey OR password+TOTP) | any | 24h | 4h | read dashboard, read profile, list resources |
| `elevated` | 1 (Passkey ONLY; password+TOTP not accepted) | Passkey or hardware key | 4h | 1h | mutate own data, create/update resources, invoke workflows |
| `sensitive` | 2 (Passkey + step-up Passkey OR Passkey + hardware key) | factor must be re-presented; same Passkey not accepted twice in same 5 min | 1h | 15min | rotate secret, delete resource, admin user list, secret access, payment auth |
| `critical` | 2 + IT-approval | hardware key (FIDO-MDS3 L2+) + Passkey + JIT IT-approval grant within ±5min window | 15min | 5min | tenant deletion, key rotation, residency policy change, billing currency change, audit-chain export-all |

### Mapping operations → ACR

`acr_required` is declared per Cedar action in the µservice's `policy/` directory. Examples:

```cedar
// Read profile = routine
permit (
  principal,
  action == ProfileAction::"Read",
  resource is Profile
) when {
  principal.acr_level >= AcrLevel::"routine"
};

// Rotate secret = sensitive
permit (
  principal,
  action == SecretAction::"Rotate",
  resource is Secret
) when {
  principal.acr_level >= AcrLevel::"sensitive"
} unless {
  principal.last_acr_event_within_seconds > 3600
};

// Tenant deletion = critical
permit (
  principal,
  action == TenantAction::"Delete",
  resource is Tenant
) when {
  principal.acr_level >= AcrLevel::"critical" &&
  principal.it_approval_token.matches_resource(resource) &&
  principal.it_approval_token.issued_within_seconds < 300
};
```

### Step-up flow at the waypoint

```
1. Principal presents OIDC bearer with acr=elevated to /api/v1/secrets/rotate
2. Envoy waypoint forwards request to Cedar PDP via ext_authz
3. Cedar policy requires acr_required=sensitive
4. PDP returns DENY with response header X-Step-Up-Required: sensitive,reason=secret-rotate
5. Browser redirects to Zitadel /step-up?required_acr=sensitive&return_to=...
6. Zitadel prompts re-Passkey + (optional) hardware key
7. New ID-token issued with acr=sensitive; client retries original request
8. PDP grants; action proceeds; audit event sealed with acr=sensitive
```

### Session age policy

`acr` is bound to a session not just a token. Refreshing a token does NOT bump `acr`: rotating refresh tokens preserves the original acr-grant event timestamp. To upgrade `acr` the principal MUST re-present the higher factor.

Zitadel custom claim `acr_event_at` carries the unix-second of the most recent ACR-grant event; Cedar policies reference `principal.last_acr_event_within_seconds` as a function of this claim vs. current time.

### Audit emission

Every ACR-grant event emits `IdentityStepUpGranted` (AsyncAPI; sealed in audit-chain per ADR-0162) with:

- `tenant_id`
- `user_id`
- `acr_old`, `acr_new`
- `factors_presented` (array)
- `aaguid_used` (if WebAuthn)
- `client_ip` (geo-coarse, residency-aware)
- `requested_resource` (the operation that triggered step-up)
- `granted` (bool)

## Alternatives considered

### Free-form acr claim

Rejected. Without a controlled enum, products invent ad-hoc strings ("mfa", "strong", "high") and policies fragment.

### One-shot re-authentication (no session binding)

Rejected. The Stripe model requires re-auth on every sensitive call which is hostile to the agentic-Foundry where the policy itself is the long-lived consent envelope.

### Continuous risk-scoring (CAEP / Cisco Duo Trust Monitor)

Considered as adjunct, not a replacement. Risk signals (impossible-travel, new-device, unusual-time) can DOWNGRADE acr (force step-up earlier) but do NOT replace the factor-based ACR floor. Implementation deferred to IP-014.

## Consequences

### Positive

- Operations have an explicit risk tier; policy authors don't invent ACR semantics.
- The Cedar gate is uniform: `principal.acr_level >= AcrLevel::"<x>"`.
- Step-up flow is observable: every grant emits an audit event.
- Session-age + idle policy bounded per ACR class.

### Negative

- Adds latency: step-up adds 5-15s for the user when crossing tier boundaries.
- Cedar policies become slightly more verbose (acr predicate per action).
- Browser-step-up requires a redirect; CLIs need device-code flow.

### Neutral

- The four-class enum is intentionally coarse; adding a 5th class requires this ADR's amendment.

## Implementation

- Standards doc `docs/standards/step-up-auth-classes.md` lists per-operation ACR requirements per µservice.
- Zitadel custom claim `acr` + `acr_event_at` configured in the IdP.
- Cedar policies use `AcrLevel` entity-set with hierarchy: routine < elevated < sensitive < critical.
- `crates/oya-check-step-up-auth-coverage` advisory gate scans OpenAPI specs for missing `x-acr-required` extension on sensitive paths.

## Verification

- Lane `lean-a15-step-up-acr-coverage` (advisory) — every µservice contract declares `x-acr-required` on every mutating path.
- Lane `lean-a16-cedar-acr-policy-uniformity` (advisory) — every Cedar mutate-policy references `principal.acr_level`.
- Integration test: token with `acr=routine` is denied at sensitive operation; step-up grants `acr=sensitive`; retry succeeds.

## In-house roadmap

Per user directive 2026-05-18, evaluated under in-house policy:

- **ACR claim**: OIDC Core §3.1.2 + RFC 9068 §3.1 **standard**. KEEP.
- **Cedar PDP**: KEEP per ADR-0183 (already in-house policy engine via `oya-policy-cedar-domain` / `oya-policy-cedar-api`); the step-up gate is a Cedar policy predicate against `principal.acr_level` — in-house logic atop a Cedar runtime we own.
- **Step-up redirect flow**: Phase 0 served by Zitadel `/step-up?required_acr=X` endpoint; Phase 2 served by `oya-identity-server` step-up endpoint with the identical wire shape. No client-side change required.
- **JIT IT-approval grant** (for `acr=critical`): in-house from inception — lives in `oya-identity-step-up-orchestrator-*` crates; integrates with `governance` µservice approval-workflow. No vendor.
- **Risk-scoring adjunct (CAEP)**: deferred to IP-014; built in-house against in-house signal streams (Hubble flow, audit-chain).

Conclusion: ACR + step-up are in-house from inception. Only the redirect endpoint is Zitadel-served during Phase 0; that transparently moves with the Phase-2 swap covered by ADR-0187.

## Cross-references

- RFC 9068 §3.1 (acr claim)
- OpenID Core §3.1.2 (acr_values request param)
- ADR-0187 canonical-oidc-idp-zitadel-primary
- ADR-0188 passkey-webauthn-substrate
- ADR-0183 policy-engine-separation-cedar-app-authz-kyverno-admission
- ADR-0162 per-tenant-audit-log-slicing
