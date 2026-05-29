---
doc_class: PolicySpec
title: Data Residency Contract
microservice: application
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-application
deciders: council-privacy, ops-security, axis-application, gtm-customer-success
related_adrs: [ADR-0117, ADR-0123, ADR-0131]
related_artifacts:
  - microservices/application/threat-model.md (R-08, T-01)
  - microservices/application/dpia.md (R-08, R-11)
  - microservices/application/multi-region.md
  - microservices/application/policy/route-isolation.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract (application µservice)

## Purpose

Define where Application Shell data lives (sessions, OIDC/SAML assertions,
audit logs, route registrations, module manifests, bundle versions) per
jurisdiction, the cross-pack replication policy, and the legal-transfer
mechanisms that gate any exception. Canonical residency artifact reviewed
by EU DPAs (per GDPR Arts. 44-50), KR PIPC (per PIPA Art. 28 + Art. 23-2),
HIPAA tenant counsel (per BAA), and equivalent supervisory authorities.

## Residency Model

### Default: pack-pinning

Every tenant assigned a primary pack at onboarding. Shell state stored
in that pack's region-pinned Postgres + Valkey cluster. Cross-pack
movement is **forbidden by default**.

| Pack | Primary region | DR pair | Postgres + Valkey | CDN POPs | Activated? |
|---|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | — | kr-pg-1, kr-valkey-1 | OCI CDN seoul; (no Cloudflare overlay for KR) | YES (M03 launch) |
| pack-eu | OCI eu-frankfurt-1 | OCI eu-amsterdam-1 | eu-pg-{1,2}, eu-valkey-{1,2} | OCI CDN eu; Cloudflare eu-only | Conditional (SCC) |
| pack-us | OCI us-ashburn-1 | OCI us-phoenix-1 | us-pg-{1,2}, us-valkey-{1,2} | OCI CDN us; Cloudflare | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | OCI us-phoenix-1 (HIPAA-eligible) | us-hc-pg-1, us-hc-valkey-1 | OCI CDN us-hc only; no Cloudflare | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | — | jp-pg-1, jp-valkey-1 | OCI CDN tokyo | Conditional |
| pack-sg | OCI ap-singapore-1 | — | sg-pg-1, sg-valkey-1 | OCI CDN sg | Conditional |
| pack-au | OCI ap-sydney-1 | OCI ap-melbourne-1 | au-pg-{1,2}, au-valkey-{1,2} | OCI CDN au | Conditional |
| pack-in | OCI ap-hyderabad-1 | OCI ap-mumbai-1 | in-pg-{1,2}, in-valkey-{1,2} | OCI CDN in (DPDPA: data fiduciary-controlled) | Conditional (DPDPA 2023) |
| pack-br | OCI sa-saopaulo-1 | OCI sa-vinhedo-1 | br-pg-{1,2}, br-valkey-{1,2} | OCI CDN br | Conditional (LGPD) |
| pack-ae | OCI me-abudhabi-1 | OCI me-dubai-1 | ae-pg-{1,2}, ae-valkey-{1,2} | OCI CDN ae | Conditional |
| pack-ksa | OCI me-jeddah-1 | OCI me-riyadh-1 | ksa-pg-{1,2}, ksa-valkey-{1,2} | OCI CDN ksa | Conditional (KSA NCA cloud-residency) |

### Pack-assignment routing

```text
Tenant onboarding
    ↓
gtm-customer-success: collects HQ jurisdiction + regulated-data declarations
    ↓
Pack-router (Cedar policy in cloud-iac):
    - HQ jurisdiction → primary pack
    - PHI/sensitive flag → may force secondary pack (us-healthcare; ae/ksa)
    - Conflict: ops-legal escalation
    ↓
OpenBao assigns tenant → pack
    ↓
DNS: <hash>.app.oyatie.dev → per-pack ingress (per-pack TLS cert via ACME)
```

## CDN Residency Posture

Because CDN POPs are inherently geo-distributed, the contract is:

- **Public-class assets ONLY at CDN edge**: Leptos WASM bundle, fonts,
  CSS, generic shell HTML template, module manifest signatures (NOT the
  manifest body when it contains tenant-pinned routes).
- **Per-tenant content NEVER cached at CDN**: shell HTML with tenant
  data, admin portal renders, audit log views are origin-served with
  `Cache-Control: private, no-store`.
- **Pack-scoped POP routing**: per-pack DNS resolves to pack-region POPs;
  for KR / EU strict-residency tenants, only pack-region POPs are
  reachable.
- **Cloudflare overlay**: optional for pack-eu / pack-us / pack-jp / pack-au
  / pack-sg only; ALWAYS disabled for pack-kr (KR-only POPs) and pack-us-healthcare
  (HIPAA POP set only).

## Cross-pack policy

| Movement | Default | Override | Mechanism |
|---|---|---|---|
| Session token cross-pack | FORBID | none | n/a |
| Audit log cross-pack | FORBID | none | n/a |
| Route registration metadata cross-pack | FORBID | per-product team approval + ops-legal sign-off; SCC required | manual cloud-iac PR |
| Module manifest cross-pack | FORBID | publisher-team approval (Ed25519 publish key isolated per pack) | manual SDK PR |
| Bundle binary cross-pack | ALLOW (binaries are public-class) | n/a | signed manifest |

## Audit + Verification

| Verification | How |
|---|---|
| Cross-pack-leak scan | `oya-application-residency-pin` lane queries Postgres for any tenant_id whose data appears in a non-matching pack |
| CDN POP egress audit | per-pack POP access log (1 % sample) verified against pack region list weekly |
| DSR (erasure) cascade | DELETE on Postgres + DEL on Valkey + CDN purge in same pack only |
| Cross-region replication is | none for shell state; in-pack DR-pair replication only |

## DSR Cascade (data-subject-rights)

When a data subject exercises Art. 17 (erasure) or PIPA right to be
forgotten:

1. Tenant admin posts the DSR via `/admin/dsr/erasure`.
2. tenancy µservice resolves the (data_subject_id, scope) and emits
   `DsrErasureRequested` event.
3. application µservice handles the event for its data classes:
   - Postgres: DELETE FROM sessions WHERE user_id = $1 AND tenant_id = $2.
   - Postgres: DELETE FROM route_audit WHERE user_id = $1 AND tenant_id = $2 AND ts < now() - retention.
   - Valkey: DEL session:<user_id>:*.
   - audit-chain: emit `DsrErasureExecuted` (the event itself retained
     per Art. 17(3)(b) audit obligation, but with redacted body).
4. observability µservice purges per-user metric series.
5. Acknowledge to tenant in ≤ 30 days (GDPR), ≤ 30 days (PIPA), ≤ 60 days (CPRA).

## References

- ADR-0117 data residency packs.
- Bominal ADR-0028 audit chain.
- GDPR Arts. 25, 44-50; PIPA Arts. 17, 28, 28-2; HIPAA §164.502(b).
- `microservices/application/multi-region.md` BCDR posture.
