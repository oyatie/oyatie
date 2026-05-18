---
doc_class: ImplementationPlan
milestone: M03-foundation
phase: P01-shorts-foundation
impl_plan_id: IP-011-content-moderation-and-copyright-claim-bc
status: pending
owner: axis-shorts + axis-foundry-runtime + ops-legal + council-privacy
depends_on: [IP-004]
acceptance_lanes: [cargo-build, cargo-nextest, oya-governance-eu-ai-act-conformance, oya-governance-eu-dsa-conformance, oya-governance-dmca-safe-harbor-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-011: content-moderation + copyright-claim BC end-to-end

## Intent

- `content-moderation` BC: NSFW + violence + minor-protection classifier (foundry-runtime T2); manual reviewer queue; appeal workflow per EU DSA Art. 20 (7d SLA); EU AI Act Art. 9-15 + Art. 50 obligations operative per ADR-SHORTS-0003.
- `copyright-claim` BC: Chromaprint audio-fingerprint + DCT video perceptual-hash match against per-pack corpus; DMCA takedown + counter-notice + repeat-infringer cycle per ADR-SHORTS-0002; designated agent registered with US Copyright Office.

## ChangeSet boundary

8 + 8 = 16 crates.

Plus integrations:
- `foundry-runtime` µservice (classifier endpoint).
- `audit-chain` for every verdict + claim seal (Ed25519).
- `ontology` for `CopyrightClaim` entity write.

## Concrete File Targets

Key entities: `ModerationVerdict`, `AbuseReport`, `Appeal`, `ClassifierVersion`, `MinorProtectionVerdict`, `FingerprintMatch`, `CopyrightClaim`, `CounterNotice`, `RepeatInfringerRecord`, `Strike`.

Ports: `ModerationClassifier`, `AbuseReportStore`, `AppealStore`, `FingerprintMatcher`, `ClaimStore`, `RepeatInfringerTracker`.

Required EU AI Act Art. 50 / EU DSA Art. 17 emission:
- Every verdict carries `eu_ai_act_label` + Statement of Reasons (`grounds` + `facts_and_circumstances` + `automated_means` + `redress`).
- Every appeal resolution emits within 7d SLA.

DMCA cycle:
- Claim filing requires perjury-attestation per §512(c)(3)(A)(vi).
- Counter-notice requires perjury-attestation per §512(g)(3)(C) + jurisdiction-consent per §512(g)(3)(D).
- Repeat-infringer policy: 3+ confirmed claims within 6mo triggers auto-suspend per §512(i)(1)(A); ops-legal weekly audit.

## Acceptance Gates

```bash
cargo build -p oya-shorts-content-moderation-worker
cargo build -p oya-shorts-copyright-claim-worker
cargo nextest run -p oya-shorts-content-moderation-{kernel,domain,usecase,adapter-clamav,adapter-opswat,worker}
cargo nextest run -p oya-shorts-copyright-claim-{kernel,domain,usecase,adapter-postgres,worker}
cargo run -p oya-dev-cli -- gate validate eu-ai-act-conformance --microservice shorts
cargo run -p oya-dev-cli -- gate validate eu-dsa-conformance --microservice shorts
cargo run -p oya-dev-cli -- gate validate dmca-safe-harbor-conformance --microservice shorts
```

E2E: NSFW classifier verdict within 2s p99; copyright fingerprint match ≤ 2s p95; DMCA cycle (claim→takedown→counter-notice→restoration) completes; repeat-infringer 3-strike triggers auto-suspend; appeal resolved ≤ 7d.

## Halt Conditions

- EU AI Act Art. 50 label missing on any verdict — block.
- Statement of Reasons missing — block.
- DMCA perjury-attestation absent — block.
- Repeat-infringer policy not enforced — block.

## Next IP

[`IP-012-age-gate-and-parental-controls-bc.md`](IP-012-age-gate-and-parental-controls-bc.md)

## References

- PRD FR-13, FR-14, FR-15, FR-16, FR-17.
- ADR-SHORTS-0002 (copyright system).
- ADR-SHORTS-0003 (moderation classifier bounds).
- `capabilities/T2-auto.yaml`.
- `runbooks/moderation-classifier-rollback.md`.
- `runbooks/copyright-claim-storm-throttle.md`.
- EU AI Act Arts. 9-15, 50, 73.
- EU DSA Arts. 16, 17, 20, 24.
- DMCA Title II 17 USC §512(c)-(i).
