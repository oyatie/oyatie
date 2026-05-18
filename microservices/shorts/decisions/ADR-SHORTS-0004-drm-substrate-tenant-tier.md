---
id: ADR-SHORTS-0004
status: Accepted
date: 2026-05-17
microservice: shorts
deciders: council-architecture, ops-security, cloud-secrets, axis-shorts, gtm-strategy, ops-finops
owner: axis-shorts + cloud-secrets
supersedes: []
superseded_by: []
related:
  - ADR-0117
  - ADR-0126
  - ADR-0131
  - ADR-SHORTS-0001
related_artifacts:
  - microservices/shorts/PRD.md (FR-29; Open Question 3)
  - microservices/shorts/threat-model.md (T-I-10, T-E-06)
  - microservices/shorts/policy/dual-context-isolation.md (DCI-09)
  - microservices/shorts/runbooks/drm-key-rotation.md
  - microservices/shorts/slos/drm-license-issuance-latency.openslo.yaml
purpose: Establish DRM substrate (Widevine + FairPlay + PlayReady) and tenant-tier gating policy for shorts.
---

# ADR-SHORTS-0004: DRM substrate + tenant-tier gating — Widevine + FairPlay + PlayReady; Premium-tier-only by default; per-content key rotation 7d; root key rotation 90d; HSM-bound

## Status

Accepted — 2026-05-17.

## Context

PRD FR-29 + Open Question 3 specifies the need for DRM substrate at tenant-tier granularity. DRM-protected delivery requires:
- Widevine (Google; Chrome/Edge/Android decoder).
- FairPlay (Apple; Safari/iOS decoder).
- PlayReady (Microsoft; Edge/Xbox decoder).

W3C EME (Encrypted Media Extensions) 2017 standardises the browser-side license-acquisition API; the server side requires per-key-system integration.

Threat-model:
- T-I-10: DRM key material leak (Sev-1 risk).
- T-E-06: DRM key-system substrate compromise (Sev-1; very-high impact).

Per `policy/dual-context-isolation.md` DCI-09: Personal-tier shorts never DRM-protected (semantics inconsistent + consumer-ownership-class data); only Professional-tier with per-tenant tier gating.

OCI is primary cloud per ADR-0117; OpenBao manages secret material; HSM-binding for root key.

PRD §Performance NFR: DRM license issuance p95 ≤ 150ms.

## Decision

oyatie shorts adopts:

1. **Three key systems supported**: Widevine (`com.widevine.alpha`), FairPlay (`com.apple.fps`), PlayReady (`com.microsoft.playready`).

2. **Tenant-tier gating**:
   - **Free / Basic tier**: DRM unavailable.
   - **Premium tier**: DRM available; tenant-opt-in per video (default OFF; tenant configures per-video or tenant-wide).
   - **Enterprise tier**: DRM default-ON; tenant-policy override available.

3. **Per-content key rotation**: 7 days. New per-content key derived from current root; old key marked expiring; grace window 24h.

4. **Root key rotation**: 90 days. Coordinated maintenance window with cloud-secrets; OpenBao HSM-bound; vendor (Google / Apple / Microsoft) coordination per key system.

5. **Compile-time tier enforcement**:
   ```rust
   pub fn issue_drm_license(post: ProfessionalShort, tier: TenantTier) -> Result<DrmLicense, DrmError>;
   // Personal posts have no implementation — compile-time refusal.
   // Non-Premium/Enterprise tier: runtime refusal with Sev-1 metric oya_shorts_drm_tier_violation_total.
   ```

6. **HSM-bound root key**: never extracted in clear; per-content key derivation via non-extractable wrapper.

7. **Active-active HA per pack**: 2 key-server replicas per pack; cross-AZ; immediate revocation list propagation.

8. **EME license-acquisition flow**:
   - Client: WPA-bound device fingerprint + EME `requestMediaKeySystemAccess`.
   - Server: validates Cedar (`tenant.tier in [Premium, Enterprise]`); validates device fingerprint salted per tenant; issues key-system-specific license blob.
   - License-expiry: 24h (configurable per tenant).

9. **Forced rotation on compromise indicator**: Sev-1 forced rotation per `runbooks/drm-key-rotation.md`; revocation list propagation ≤ 15min; vendor coordination.

10. **Pack-residency invariant**: per-content keys never cross pack boundary; per-pack key-server clusters.

11. **Per-pack tier overrides** (overrides above):
    - **pack-us-healthcare**: DRM OFF by default for PHI accounts (HIPAA Safe Harbor; opt-in with BAA).
    - **pack-cn-future**: out-of-scope for M03; future evaluation.

## Alternatives Considered

### A. Single DRM provider (only Widevine; covers Chrome/Edge/Android)

- Pros: simpler operations; one vendor relationship.
- Cons: Safari/iOS clients unable to play DRM-protected content (FairPlay required); Edge/Xbox can fall back to Widevine but PlayReady is native; competitor parity gap.
- Rejected: client coverage gap.

### B. Two DRM providers (Widevine + FairPlay; skip PlayReady)

- Pros: simpler than three; covers ~95% of clients.
- Cons: Edge/Xbox clients suboptimal; Microsoft enterprise tenants (Premium / Enterprise tier sensitive) lose native PlayReady fallback.
- Rejected: marginal complexity savings; native Microsoft client coverage matters for enterprise.

### C. DRM available to all tenants (no tier gating)

- Pros: simpler product surface.
- Cons: DRM overhead (HSM ops + vendor coordination + key rotation) is significant; not all tenants need or want DRM; pricing implications of Premium-only model.
- Rejected: tier gating preserves pricing differentiator + reduces ops cost.

### D. Self-build DRM (proprietary key system; non-W3C-EME)

- Pros: full sovereignty.
- Cons: client SDK must implement custom decoder; not compatible with native browser decoders; massive engineering cost; no client adoption path.
- Rejected: standards-conformant W3C EME is the only viable path.

### E. Use AWS Elemental MediaPackage as managed DRM service

- Pros: managed key rotation + EME license issuance; less vendor coordination overhead.
- Cons: ADR-0117 primary cloud is OCI; cross-cloud complexity; vendor lock-in; per-license pricing high at scale.
- Rejected: cross-cloud + lock-in.

### F. Soft DRM (HLS encryption only; no EME hardware-binding)

- Pros: simpler than full DRM; no per-vendor HSM coordination.
- Cons: no hardware-binding; trivial circumvention; not industry-grade; would fail enterprise tenant requirements.
- Rejected: insufficient.

### G. Per-content key rotation cadence: 1 day (more frequent rotation)

- Pros: shorter key lifetime; tighter security.
- Cons: client license-reacquisition every day; user-visible playback friction (browser may prompt for re-auth more often); rotation operational overhead 7x.
- Rejected: 7d is the industry sweet-spot between security + UX.

### H. Per-content key rotation cadence: 90 days (less frequent rotation)

- Pros: minimal ops overhead.
- Cons: long key exposure window; insufficient security posture.
- Rejected: too long; industry best-practice ≤ 7d for per-content keys.

## Consequences

### Positive

- Full client coverage: Chrome / Edge / Android (Widevine); Safari / iOS (FairPlay); Edge / Xbox (PlayReady).
- Premium-tier-only gating preserves pricing differentiator + reduces ops cost for tenants who don't need DRM.
- HSM-bound root key + 90d rotation: industry best-practice secret-management.
- Compile-time DCI-09 invariant: Personal-tier never DRM-protected; type-system refusal.
- Per-pack residency: DRM keys never cross pack boundary; satisfies GDPR Arts. 44-50 + KR PIPA Art. 28.
- Per-content rotation 7d: limits blast-radius of any per-content key compromise.
- Industry-grade hardware-binding via EME satisfies enterprise tenant DRM requirements.

### Negative

- 3 vendor relationships (Google + Apple + Microsoft); each has its own provisioning + key-server + revocation API.
- Per-vendor HSM coordination overhead.
- Vendor pricing scales with license throughput (per Microsoft / Apple / Google per-license fees).
- Tenant-tier gating UX: tenants on Free/Basic see DRM as Premium-feature upsell; potential friction.
- Forced rotation path (Sev-1) requires multi-day vendor coordination if root-key compromise reaches vendor scope.

### Operational

- `runbooks/drm-key-rotation.md` covers routine 90d + forced rotation paths.
- `slos/drm-license-issuance-latency.openslo.yaml` tracks p95 ≤ 150ms.
- Per-pack active-active key-server cluster; 2 replicas minimum.
- Per-pack drill: quarterly DRM key rotation per `runbooks/drm-key-rotation.md`.
- Tier gating Cedar fragment in `policy/tenant-scope.cedar` PERMIT 1.
- Metric `oya_shorts_drm_tier_violation_total` alerts at > 0 (Sev-1).

### Regulatory

- **DCI-09 (dual-context isolation)**: Personal-tier never DRM-protected; compile-time + Cedar + runtime.
- **HIPAA Safe Harbor §164.514**: pack-us-healthcare DRM OFF by default for PHI accounts.
- **GDPR Arts. 44-50 + KR PIPA Art. 28**: per-pack key residency.
- **EU AVMSD Art. 28b(2)**: minor-protection — DRM tier-gating consistent with broader minor-protection floor (minor accounts on Premium-tier still subject to age-classification gating).
- **EU AI Act / EU DSA**: DRM not in scope (delivery mechanism); but underlying content moderation + ranking remain in scope per ADR-SHORTS-0003 + ADR-SHORTS-0005.

## References

- ADR-0117 OCI primary cloud.
- Parallel ADR-0126 dual-context.
- ADR-0131 per-µservice flat layout.
- ADR-SHORTS-0001 (transcode pipeline; DRM operates over HLS/DASH manifest layer).
- W3C EME 2017 `www.w3.org/TR/encrypted-media`.
- Widevine SecureStop API (Google).
- FairPlay key-server API (Apple Developer).
- PlayReady DRM-server API (Microsoft).
- OpenBao HSM docs.
- `microservices/shorts/threat-model.md` T-I-10, T-E-06.
- `microservices/shorts/policy/dual-context-isolation.md` DCI-09.
- `microservices/shorts/runbooks/drm-key-rotation.md`.
- `microservices/shorts/slos/drm-license-issuance-latency.openslo.yaml`.
- `microservices/shorts/policy/data-residency.md` §DRM Tenant-Tier Gating.
