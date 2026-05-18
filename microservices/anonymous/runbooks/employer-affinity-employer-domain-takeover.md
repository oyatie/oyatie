---
doc_class: Runbook
template_id: TPL-RUNBOOK
title: Employer-Affinity Domain Takeover (Employer Change-of-Ownership)
microservice: anonymous
severity: "Sev-2 (planned acquisition) / Sev-1 (hostile or compromise-driven)"
status: Accepted
owner_team: axis-anonymous + general-counsel + ops-security
date: 2026-05-17
related_adrs: [ADR-ANON-0002, ADR-ANON-0007]
related_artifacts:
  - microservices/anonymous/policy/affinity-attestation-verification.md
doc_status: published
---

# Runbook: Employer-Affinity Domain Takeover

## Purpose

Handle a change-of-ownership / merger / acquisition / domain-takeover event for an employer whose affinity is bound to the platform. The classic scenario: Bominal acquires FooCorp; FooCorp's employees were posting in the @foocorp.com employer-affinity cluster; we must decide whether to merge clusters, keep separate, or revoke.

## Trigger

| Signal | Severity |
|---|---|
| Employer notifies us of acquisition / merger | Sev-2 |
| Employer-domain DNS / WHOIS ownership change detected | Sev-2 |
| Employer IdP migrated to new SAML / OIDC provider | Sev-2 |
| Employer domain is hijacked / compromised (hostile takeover) | Sev-1 |
| Employer files bankruptcy / domain expires | Sev-2 |

## Pre-checks

1. Confirm the takeover kind via WHOIS / SAML metadata / corporate-records check
2. Identify the affected affinity cluster and its membership count
3. Verify whether existing bindings will remain valid under new ownership
4. Coordinate with general-counsel on labor-law / NLRA / works-council notifications

## Steps — Planned acquisition (Sev-2)

| Step | Action | Time budget |
|---|---|---|
| 1 | Open Sev-2 incident; coordinate with general-counsel | ≤ 1 day |
| 2 | Confirm acquirer's IdP / domain configuration; register new affinity issuer if needed (per `affinity-attestation-key-rotation.md`) | ≤ 7 days |
| 3 | Decide cluster posture: (a) merge (FooCorp employees → Bominal employer cluster), (b) keep separate (legacy FooCorp cluster + new Bominal cluster), (c) revoke (employees re-attest under new IdP) | per general-counsel + product decision |
| 4a (merge) | Execute cluster merge: `cargo run -p oya-dev-cli -- anonymous community-definition merge --child <foocorp> --parent <bominal>` | ≤ 5 min |
| 4b (keep separate) | No-op; FooCorp cluster remains for legacy members | – |
| 4c (revoke) | Revoke FooCorp affinity issuer's key; force re-attestation: `cargo run -p oya-dev-cli -- anonymous affinity-attestation revoke-issuer --issuer-id <foocorp>` | ≤ 5 min |
| 5 | Notify members of the affected cluster: 14-day notice of cluster change | 14 days |
| 6 | Audit-chain seal the takeover event | – |

## Steps — Hostile takeover / compromise (Sev-1)

| Step | Action | Time budget |
|---|---|---|
| 1 | Declare Sev-1 in `#inc-<id>` | ≤ 5 min |
| 2 | Immediately revoke the affinity issuer's signing key: `cargo run -p oya-dev-cli -- anonymous affinity-attestation revoke-key --issuer-id <hijacked> --key-id <compromised>` | ≤ 1 min |
| 3 | Invalidate all bindings under that issuer | ≤ 5 min |
| 4 | Notify affected members: bindings invalidated; re-attestation required after verification of legitimate domain ownership | ≤ 1h |
| 5 | Coordinate with general-counsel on whether to engage law enforcement (domain hijack is potentially Computer Fraud and Abuse Act §1030 violation in US) | per case |
| 6 | Post-mortem within 5 business days | – |

## Member-side communication template

Within the affinity-revocation notification, surface to members:

```
Subject: Your employer-affinity binding on oyatie:anonymous has been invalidated.

This is a privacy-protection action. We detected a change of ownership / domain
configuration / IdP change for <issuer-id>. To protect your anonymity:

- Your existing bindings to the <cluster-id> affinity cluster have been invalidated.
- Posts you authored under the bindings remain published (they are bound to a
  blinded commitment, not your identity, per our I1 invariant).
- To continue posting in this affinity cluster, please re-attest your affinity
  via your new IdP.

If you have questions, contact your tenant administrator.
```

## Failure modes

| Failure | Mitigation | Severity escalation |
|---|---|---|
| Acquirer refuses to register new IdP / signing key | revoke cluster; members must rebind to a different cluster | Sev-2 |
| Legacy IdP cannot be cleanly revoked (legacy bindings remain valid under compromised key) | force-invalidate all bindings; full re-attestation cycle | Sev-1 |
| Members' posts in legacy cluster need to be merged into new cluster | NOT POSSIBLE — posts are bound to blinded commitments; merging would correlate; we leave legacy posts in legacy cluster as-is | – |

## Cross-µservice coordination

- `tenancy`: tenant operator (the employer) coordinates with us
- `audit-chain`: every revocation, merge, re-attestation sealed
- `legal-process-disclosure`: if a court-order is involved (e.g., subpoena tied to acquisition due-diligence), Path A flow

## References

- ADR-ANON-0002 (affinity-attestation verification)
- ADR-ANON-0007 (affinity-cluster design)
- US Computer Fraud and Abuse Act 18 USC §1030 (if domain hijack)
- NLRA + works-council notifications (when employer-side change affects employee speech rights)
