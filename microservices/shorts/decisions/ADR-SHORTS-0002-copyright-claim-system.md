---
id: ADR-SHORTS-0002
status: Accepted
date: 2026-05-17
microservice: shorts
deciders: council-architecture, ops-legal, axis-shorts, council-privacy, ops-security
owner: axis-shorts + ops-legal
supersedes: []
superseded_by: []
related:
  - ADR-0126
  - ADR-0131
  - ADR-SHORTS-0001
  - ADR-SHORTS-0003
related_artifacts:
  - microservices/shorts/PRD.md (Open Question 7)
  - microservices/shorts/threat-model.md (T-S-03, T-S-06, T-T-03)
  - microservices/shorts/compliance.md (§DMCA)
  - microservices/shorts/runbooks/copyright-claim-storm-throttle.md
  - microservices/shorts/capabilities/T2-auto.yaml (copyright auto-actions)
purpose: Establish the copyright-claim system architecture (Content-ID-class fingerprint matching + DMCA cycle) for the shorts µservice.
---

# ADR-SHORTS-0002: Copyright-claim system — Content-ID-class fingerprint matching via Chromaprint audio + DCT video perceptual-hash; DMCA Title II Safe Harbor cycle

## Status

Accepted — 2026-05-17.

## Context

Short-form video platforms attract copyright-claim activity at scale. Industry leaders ship Content-ID-class fingerprint matching to detect potential infringement at upload time, before publication. The choice of fingerprint algorithm (audio + video), corpus governance, and DMCA workflow design are technically + legally load-bearing.

PRD FR-15, FR-16, FR-17 specify: fingerprint-match audio + video at ingest; copyright-claim takedown; counter-notice workflow. PRD §Performance NFR: fingerprint match p95 ≤ 2s per ingest. PRD Open Question 7 explicitly asks about fingerprint-corpus governance (per-tenant private corpus + global licensed corpus split).

DMCA Title II Safe Harbor (17 USC §512) is the regulatory floor for the US pack:
- §512(c) requires designated agent registration with US Copyright Office.
- §512(c)(3) defines elements of valid notification.
- §512(c)(3)(A)(vi) requires perjury attestation by claimant.
- §512(f) covers misrepresentation; claimants liable for damages.
- §512(g) defines counter-notice cycle (10-14 business day window).
- §512(i)(1)(A) requires repeat-infringer termination policy.

EU DSA Art. 16 (notice-and-action) + Art. 17 (Statement of Reasons) cover the EU pack equivalent obligations.

Threat-model identifies T-S-03 (forged DMCA claim), T-S-06 (counter-notice forgery), T-T-03 (fingerprint corpus poisoning) as elevated risks.

Capabilities T1 (copyright pre-check) and T2 (copyright auto-actions) carry EU AI Act high-risk-adjacent classification — auto-actions affect creator commercial interests.

## Decision

oyatie shorts adopts:

1. **Audio fingerprint: Chromaprint 1.5.1 LTS** — open-source perceptual audio-fingerprint algorithm; battle-tested at MusicBrainz scale; sub-second matching against ~1B-entry corpus is feasible.
2. **Video fingerprint: DCT-based perceptual-hash (pHash variant)** — open-source perceptual-hash; sub-second matching against ~1B-entry corpus; complemented by SIFT-feature-set for cropped/rotated content.
3. **Fingerprint corpus governance** (closing PRD Open Question 7):
   - **Per-licensor namespace**: each rights-holder uploads via signed-manifest workflow; ops-legal sign-off per licensor; per-licensor authentication + audit-chain seal on every write.
   - **Per-pack scope**: corpus partitions per pack; cross-pack sharing forbidden except under explicit licensor cross-pack license.
   - **Per-tenant private corpus**: tenants may upload private corpus entries against their own content for monitoring (e.g., enterprise media co); per-tenant isolated; never matched against other tenants' uploads.
4. **DMCA cycle implementation**:
   - **Claim filing**: REST POST /copyright-claims; requires claimant business-verification + perjury-attestation per §512(c)(3)(A)(vi); rate-limited per claimant (default 100/hr; verified-business 1000/hr per `runbooks/copyright-claim-storm-throttle.md`).
   - **Auto-action on high-confidence match**: auto-hide pending counter-notice cycle per capabilities/T2-auto.yaml; auto-attach licensing attribution at lower confidence.
   - **Counter-notice**: REST POST /copyright-claims/{id}/counter-notice; requires perjury-attestation per §512(g)(3)(C) + jurisdiction-consent per §512(g)(3)(D); 10-14 business day window before restoration.
   - **Repeat-infringer policy**: 3+ confirmed claims within 6mo triggers auto-suspend per §512(i)(1)(A); ops-legal weekly audit; permanent suspension on second-tier threshold.
   - **DMCA designated agent**: ops-legal registered with US Copyright Office; backup agent designated per ops-legal rotation policy.
5. **Audit-chain seal** on every claim + counter-notice + takedown event per Bominal ADR-0028.
6. **EU DSA Art. 17 Statement of Reasons** emitted with every auto-hide; per `capabilities/T2-auto.yaml`.
7. **EU DSA Art. 24 transparency report**: quarterly export per-tenant.

## Alternatives Considered

### A. Use a single audio + video fingerprint algorithm (e.g., AcoustID only, no video)

- Pros: simpler; lower compute cost; broader open-source community for AcoustID.
- Cons: video-only infringement (cropped/recompressed without audio) wouldn't be detected; competitive parity gap vs TikTok / Reels which use both.
- Rejected: incomplete coverage.

### B. Use Echoprint instead of Chromaprint for audio fingerprinting

- Pros: more recent algorithm; alternative open-source community.
- Cons: smaller community than Chromaprint; less battle-tested at billion-entry scale; documentation thinner.
- Rejected: Chromaprint's track record + MusicBrainz proof-at-scale wins.

### C. Use proprietary ContentID-as-a-service (e.g., YouTube ContentID API, Audible Magic)

- Pros: managed; turnkey rights-holder onboarding via partners; established ecosystem.
- Cons: vendor lock-in; per-match pricing high; loss of per-tenant private corpus capability; loss of pack-residency control.
- Rejected: vendor lock-in + sovereignty.

### D. Defer copyright fingerprinting to post-publication (claim-driven only)

- Pros: simpler architecture; no upload-time latency penalty.
- Cons: DMCA Safe Harbor §512(c) protection requires "expeditious" takedown but pre-publication match offers a better protection posture for creators (reducing innocent-infringer exposure); industry parity expects pre-publication match.
- Rejected: insufficient creator protection; lags industry.

### E. Build proprietary ML-based copyright matcher (audio + video deep-feature embedding)

- Pros: highest accuracy; resilient to adversarial transformations.
- Cons: years of model-development; per-match cost orders-of-magnitude higher than perceptual-hash; EU AI Act high-risk classification multiplied; defer to M05+.
- Rejected (for P01); revisit when foundry-runtime has mature media-embedding capacity.

### F. Per-pack global single corpus (no per-licensor namespace)

- Pros: simpler corpus governance; single audit trail.
- Cons: licensor cannot independently manage their corpus; fingerprint corpus poisoning (T-T-03) risk elevates; ops-legal review burden centralised.
- Rejected: namespace isolation is critical for licensor governance.

## Consequences

### Positive

- Pre-publication copyright pre-check protects creators from innocent infringement before publication.
- Chromaprint + DCT perceptual-hash: open-source, well-understood; matches against 1B-entry corpus within 2s p95 (PRD target).
- Per-licensor namespace: licensors manage their own corpus; ops-legal oversight per-licensor.
- DMCA cycle implementation conforms to §512(c)(3) + §512(f) + §512(g) + §512(i)(1)(A) statutory elements.
- Repeat-infringer policy + audit-chain seal protects Safe Harbor eligibility.
- Counter-notice workflow accessible from creator UI per PRD competitor-parity-matrix differentiation.
- EU DSA Art. 17 Statement of Reasons satisfied automatically on every auto-action.
- Transparency report per Art. 24 supports tenant compliance disclosure.

### Negative

- Per-claimant rate-limiting required to prevent forged-claim storms (T-S-03); operational cost.
- Per-licensor namespace requires onboarding workflow + signed-manifest validation; ops-legal labor.
- Fingerprint corpus storage at S+ tier: 1B+ entries per pack; Postgres + per-pack partitioning required.
- Counter-notice cycle (10-14 day window) creates an operational pendulum for creators.
- Per-tenant private corpus adds complexity (separate namespace + tenant-scoped Cedar policy).

### Operational

- Workers run Chromaprint + DCT perceptual-hash in gVisor sandbox (Chromaprint historical CVE rate similar to ffmpeg).
- `runbooks/copyright-claim-storm-throttle.md` covers forged-claim mitigation.
- ops-legal weekly audit of repeat-infringer policy enforcement.
- DMCA designated-agent contact: kept current with US Copyright Office; quarterly verification.
- Per-licensor SLA for corpus update propagation: ≤ 24h to global match-availability.

### Regulatory

- **DMCA Title II 17 USC §512(c) Safe Harbor**: implementation conforms to all statutory elements.
- **DMCA §512(c)(3)(A)(vi) perjury attestation**: enforced at filing UI + REST schema.
- **DMCA §512(f) misrepresentation**: forged-claim detection + per-claimant rate-limit + ops-legal escalation per `runbooks/copyright-claim-storm-throttle.md`.
- **DMCA §512(g) counter-notice**: 10-14 business day window + jurisdiction-consent.
- **DMCA §512(i)(1)(A) repeat-infringer**: 3-strike auto-suspend within 6mo; weekly audit.
- **EU DSA Art. 16 (notice-and-action)**: claim filing UI + Statement of Reasons.
- **EU DSA Art. 17 (Statement of Reasons)**: emitted with every auto-action.
- **EU DSA Art. 24 (transparency report)**: per-tenant quarterly export.

## References

- ADR-0126 dual-context (paired); ADR-0131 per-µservice flat layout.
- ADR-SHORTS-0001 (transcode pipeline; fingerprint corpus is upstream-of-publication).
- ADR-SHORTS-0003 (content-moderation classifier; copyright is sibling auto-action surface).
- DMCA Title II 17 USC §512.
- EU DSA Regulation 2022/2065 Arts. 16, 17, 20, 24.
- EU DMA where applicable.
- Chromaprint algorithm `acoustid.org/chromaprint`.
- Perceptual hashing (pHash) DCT-based algorithm.
- AcoustID / MusicBrainz scale references.
- `microservices/shorts/compliance.md` §DMCA Title II Safe Harbor.
- `microservices/shorts/threat-model.md` T-S-03, T-S-06, T-T-03.
- `microservices/shorts/capabilities/T2-auto.yaml`.
- `microservices/shorts/runbooks/copyright-claim-storm-throttle.md`.
- `microservices/shorts/slos/copyright-claim-match-latency.openslo.yaml`.
