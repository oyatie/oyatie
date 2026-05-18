---
doc_class: ImplementationPlan
milestone: M03-foundation
phase: P01-shorts-foundation
impl_plan_id: IP-015-drm-and-hg-shorts-registration
status: pending
owner: axis-shorts + cloud-secrets + ops-security + ops-governance
depends_on: [IP-005, IP-014]
acceptance_lanes: [cargo-build, cargo-nextest, oya-governance-authority-cohesion, oya-governance-hyperscaler-maturity-claims, oya-governance-version-pinning-conformance]
---

# IP-015: drm-stub BC end-to-end + HG-SHORTS hyperscaler-grade conformance gate registration + branch-protection wiring

## Intent

- `drm-stub` BC end-to-end: Widevine + FairPlay + PlayReady EME license issuance; tenant-tier gated (Premium tier only); per-content key rotation 7d; root key rotation 90d; OpenBao HSM-bound root key. Per ADR-SHORTS-0004.
- Register HG-SHORTS hyperscaler-grade conformance gate per ADR-0133 + ADR-0123 authority-cohesion.
- Wire `release/shorts/*` pattern protection in `.github/branch-protection.yaml` with 8 required checks.

## ChangeSet boundary

8 crates: `oya-shorts-drm-{kernel,domain,usecase,api,adapter,adapter-widevine,adapter-fairplay,adapter-playready,sdk}`.

Plus repo-level:
- `.github/branch-protection.yaml` — add `release/shorts/*` pattern.
- `/specs/hyperscaler-gates.json` — register `HG-SHORTS` per ADR-0133.

## Concrete File Targets

Key entities: `DrmLicense`, `KeySystem`, `LicenseRequest`, `DrmTier`.

Ports: `DrmLicenseIssuer`, `KeyMaterialProvider`, `RevocationListPublisher`.

Per-key-system adapters:
- `oya-shorts-drm-adapter-widevine`: Widevine SecureStop API; CDM provisioning.
- `oya-shorts-drm-adapter-fairplay`: Apple FairPlay key-server; SPC/CKC roundtrip.
- `oya-shorts-drm-adapter-playready`: Microsoft PlayReady DRM-server.

Tier gating compile-time refusal:
```rust
pub fn issue_drm_license(post: ProfessionalShort, tier: TenantTier) -> Result<DrmLicense, DrmError>;
// Personal posts have no implementation; compile-time refusal.
// Non-Premium/Enterprise tier: runtime refusal with Sev-1 metric.
```

HG-SHORTS gate covers:
- Per-microservice flat layout conformance.
- Dual-context isolation invariant verified.
- EU AI Act + EU DSA + EU AVMSD + DMCA conformance lanes green.
- Pack-aware age-gate conformance.
- DRM tier gating conformance.
- All 9 OpenSLO manifests passing.
- All 7 runbooks have `last_drill_date` within 90d.
- Continuous compliance evidence dashboard green.

## Acceptance Gates

```bash
cargo build -p oya-shorts-drm-app
cargo nextest run -p oya-shorts-drm-{kernel,domain,usecase,adapter-widevine,adapter-fairplay,adapter-playready}
cargo run -p oya-dev-cli -- gate validate authority-cohesion --microservice shorts
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice shorts
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
helm lint microservices/shorts/iac/helm/shorts
```

E2E: Widevine + FairPlay + PlayReady license issuance for Premium-tier tenant succeeds within 150ms p95; non-Premium-tier denied with 403; per-content key rotation 7d cycle works; root key rotation 90d cycle works.

## Halt Conditions

- HG-SHORTS gate fails on any pack — block.
- DRM tier-violation metric > 0 in pre-promotion canary — block.
- Branch-protection misconfig — block.

## References

- PRD FR-29.
- ADR-0123 authority cohesion; ADR-0133 industry best-practice; ADR-SHORTS-0004 DRM substrate.
- `runbooks/drm-key-rotation.md`.
- `slos/drm-license-issuance-latency.openslo.yaml`.
- W3C EME 2017; Widevine SecureStop; Apple FairPlay key-server; Microsoft PlayReady DRM-server.
- `microservices/shorts/PHASE-01-SHORTS-FOUNDATION.md`.
