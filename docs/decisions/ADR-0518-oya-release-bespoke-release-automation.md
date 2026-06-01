---
id: ADR-0518
status: Proposed
date: 2026-05-31
owner: ops-sre-reliability
deciders:
  - founder
  - council-architecture
  - ops-sre-reliability
related:
  - ADR-0514
  - ADR-0515
  - ADR-0513
  - ADR-0254
  - ADR-0039
  - ADR-0392
  - ADR-0408
  - ADR-0510
  - ADR-0363
  - ADR-0181
door: two-way
planning_impact: true
---
# ADR-0518: oya-release — bespoke-Rust release-automation (goreleaser-equivalent)

## Status

Proposed — 2026-05-31.

## Context

### The release-stage gap

The CI/CD pipeline as of ADR-0513 covers:

```
PR presubmit gate (oya-ci-controller / buck2 affected-targets)
  → tide merge-queue (ADR-0111 projected-state + auto-merge)
    → buck2 build + buck2-native OCI image assembly (ADR-0408 / ADR-0515)
      → cosign sign + SLSA L3 provenance + SBOM (ADR-0039)
        → ArgoCD GitOps deploy (ADR-0349)
```

There is no **release-automation stage**: no tool converts a semver tag on `dev` into a versioned, multi-arch, signed artifact release with a changelog, checksums, and a published release entry on the forge. That gap is the release equivalent of leaving the build stage un-orchestrated.

### goreleaser as the reference shape

[goreleaser](https://goreleaser.com) is the de-facto Go release-automation tool. Its feature surface is the canonical reference for what a release-automation stage must cover:

| goreleaser capability | Description |
|---|---|
| Version / semver | Reads the tag; resolves the version string |
| Cross-platform build | `go build` per GOOS/GOARCH matrix |
| Archive packaging | `.tar.gz` / `.zip` with binaries + docs |
| Checksums | `sha256sums.txt` covering all artifacts |
| Changelog | Git log between tags → structured CHANGELOG |
| Container images | `docker buildx` multi-arch push |
| Signing | Cosign integration |
| Publish | GitHub/Gitea release creation + artifact upload |
| SBOM | Syft SBOM generation per artifact |

### Why goreleaser itself is rejected

goreleaser is a Go binary. Adopting it would:

1. **Import Go into a pure-Rust toolchain** — an unacceptable platform-coherence regression against the bespoke-over-OSS + pure-Rust doctrine.
2. **Duplicate and bypass buck2** — goreleaser runs its own `go build` cross-compilation matrix; Oyatie's builds run through buck2 (ADR-0392) with musl-static cross-compilation already proven and the content-addressed action cache as the performance primitive. A goreleaser-style "do your own build" pattern would bypass the hermetic buck2 action graph, invalidate cache, and produce provenance gaps.
3. **Embed GitHub/Gitea-release-centric publishing** — goreleaser's publish primitives are tightly coupled to GitHub/Gitea release UX conventions and their respective APIs. Oyatie's SCM is Forgejo-transitory with a bespoke-VCS destination (ADR-0510); a publisher trait abstracted over the forge API is the correct seam.
4. **Not self-hostable at the build level** — goreleaser's container-image step is `docker buildx`, which pulls in Docker Inc. tooling forbidden by ADR-0381 + the container-tooling-canonical policy (`containerd` + `BuildKit` + `nerdctl`).

The correct pattern — consistent with the bespoke-over-OSS doctrine and sharpened by "adopt mature trusted Rust if exists, else bespoke" — is a **thin bespoke-Rust orchestrator** that drives the existing buck2 + OCI + supply-chain primitives through a release-shaped workflow rather than replacing them.

### Existing primitives that oya-release reuses

- **buck2 build matrix** — cross-target builds including musl-static for Linux, aarch64-darwin for Apple Silicon; content-addressed caching via NativeLink RBE (ADR-0392 + ADR-0408).
- **buck2-native OCI assembler** — multi-arch OCI image construction from buck2 targets (ADR-0515); avoids Docker Inc. tooling.
- **cosign keyless signing** — OIDC-based, Sigstore Fulcio + Rekor; signs binaries, images, and attestations (ADR-0039).
- **SLSA L3 provenance** — per-artifact, per-build attestation (ADR-0039).
- **SBOM dual-format** — SPDX 2.3 + CycloneDX 1.5 per artifact (ADR-0039).
- **Scm trait** (ADR-0510 / oya-ci-webhook-gateway Forgejo client) — the forge-abstracted publish surface; Forgejo-concrete today, bespoke-VCS tomorrow.
- **oya-ci pipeline position** — oya-release is invoked as a **postsubmit** job by oya-ci (ADR-0513 job-types), triggered on a semver tag push after tide merge; it is not a presubmit gate.

## Decision

### Architecture: thin Rust orchestrator

`oya release` is a **thin bespoke-Rust orchestrator** for the release stage. It does NOT implement a builder, an image assembler, a signer, or a SBOM generator — those are driven by existing primitives. Its role is to:

1. **Resolve the version** from the tag and produce a canonical `ReleaseVersion` struct (semver-conformant; calendar-versioned for platform releases; supports pre-release and build-metadata suffixes).
2. **Drive the buck2 build matrix** — invoke `buck2 build` across the configured target × platform matrix (incl. `x86_64-unknown-linux-musl` static, `aarch64-unknown-linux-musl` static, `aarch64-apple-darwin`). Parallelism is bounded by NativeLink RBE availability. Build targets are declared in a per-microservice `release.toml` manifest.
3. **Drive the buck2-native OCI assembler** — invoke the OCI assembler target to produce multi-arch OCI manifests and push to the internal registry (`registry.oya.run`). Defers to the OCI layer (ADR-0515); `oya release` does not touch image layers directly.
4. **Drive cosign signing + SLSA provenance** — invoke `cosign sign` for each binary and OCI manifest; invoke the SLSA provenance generator to produce a signed attestation covering the build inputs and outputs. Reuses the ADR-0039 signing chain; no new signing primitives.
5. **Generate SBOM** — invoke `cargo cyclonedx` + `syft` against the built artifacts; attach SBOMs to artifacts via `cosign attest`. Both SPDX 2.3 and CycloneDX 1.5 outputs are produced per artifact.
6. **Produce checksums + archives** — `sha256sum` over all release binaries; `.tar.gz` archives per platform with binaries + licenses; archive contents are deterministic (mtime-stripped for reproducibility).
7. **Generate the changelog** — diff the git log between the tag and its predecessor; filter by conventional-commit prefix; produce a structured changelog entry (Markdown + JSON). Stores the entry into the forge release body.
8. **Publish to forge** — invoke the `Scm` trait implementation (Forgejo-concrete today, bespoke-VCS later per ADR-0510) to create a release entry on the forge, upload binaries, archives, checksum file, and SBOMs.

### Pipeline position

`oya release` occupies the **postsubmit lane on tag push** in the oya-ci pipeline:

```
[semver tag push on dev]
      |
      v
[oya-ci-controller: postsubmit job trigger]
      |
      v
[oya release orchestrator]
      |-- buck2 build matrix (NativeLink RBE)
      |-- buck2-native OCI multi-arch assemble + push
      |-- cosign sign binaries + OCI manifests
      |-- SLSA L3 provenance attestation
      |-- SBOM CycloneDX + SPDX dual-format + cosign attest
      |-- sha256 checksums + .tar.gz archives
      |-- changelog generation
      |-- forge publish (Scm trait: Forgejo-interim)
      v
[Release entry on forge + signed artifacts in registry]
```

It does not run on PR presubmit. It does not gate merge. It does not execute in the tide merge-queue (ADR-0111). It runs AFTER the merge-queue commits to `dev` and the tag is pushed.

### Microservice layout

Per ADR-0131 per-microservice flat layout and ADR-0132 no-grouping policy:

```
microservices/oya-release/
├── manifest.json
├── slos/
│   └── release-publish.openslo.yaml
├── tests/
│   └── integration/
└── src/
    └── crates/
        ├── oya-oya-release-version-domain/       (semver + calendar-version resolution)
        ├── oya-oya-release-version-app/
        ├── oya-oya-release-build-domain/          (buck2 build matrix orchestration)
        ├── oya-oya-release-build-app/
        ├── oya-oya-release-oci-domain/            (OCI assembler invocation + registry push)
        ├── oya-oya-release-oci-app/
        ├── oya-oya-release-supply-chain-domain/   (cosign sign + SLSA + SBOM)
        ├── oya-oya-release-supply-chain-app/
        ├── oya-oya-release-package-domain/        (checksums + archives + changelog)
        ├── oya-oya-release-package-app/
        ├── oya-oya-release-publish-domain/        (Scm trait abstraction + Forgejo impl)
        ├── oya-oya-release-publish-app/
        ├── oya-oya-release-adapter/               (oya-ci job interface + CLI surface)
        └── oya-oya-release-cli/                   (the `oya release` CLI binary)
```

The `µservice slot = oya`, BC tokens = `oya-release-<bc>` per ADR-0105 / BNF v4.1. The CLI surface is `oya release [--tag <tag>] [--targets <matrix>] [--dry-run]`.

### Feature-parity table vs goreleaser

| goreleaser capability | oya-release approach | New vs reuse |
|---|---|---|
| Version / semver from tag | `oya-oya-release-version-domain` semver parser + calendar-version support | New (thin) |
| Cross-platform build | **Reuses** buck2 build matrix (ADR-0392); `oya release` invokes `buck2 build` per target | Reuse (drives existing primitive) |
| Container image build | **Reuses** buck2-native OCI assembler (ADR-0515); `oya release` invokes the OCI assembler target | Reuse (drives existing primitive) |
| Cosign signing | **Reuses** ADR-0039 cosign keyless signing chain; `oya release` invokes `cosign sign` per artifact | Reuse (drives existing primitive) |
| SLSA provenance | **Reuses** ADR-0039 SLSA L3 provenance generator | Reuse (drives existing primitive) |
| SBOM | **Reuses** `cargo cyclonedx` + `syft`; `cosign attest` per artifact | Reuse (drives existing primitive) |
| Checksums | `sha256sum` over all release artifacts; deterministic `.tar.gz` archives | New (thin) |
| Changelog | Git log between tags → conventional-commit filtered Markdown + JSON | New (thin) |
| Forge publish | `Scm` trait; Forgejo-concrete impl reuses oya-ci-webhook-gateway Forgejo client | Reuse (reuses existing Scm/Forgejo client) |

goreleaser's entire build + sign + SBOM stack is subsumed by existing buck2 / ADR-0039 primitives. The genuinely new logic in `oya release` is limited to: version resolution, checksum packaging, changelog generation, and the `Scm::create_release` orchestration call.

### Scm trait

The publish domain exposes a `Scm` trait:

```rust
pub trait Scm: Send + Sync {
    async fn create_release(&self, params: CreateReleaseParams) -> Result<ReleaseId>;
    async fn upload_asset(&self, release: ReleaseId, asset: ReleaseAsset) -> Result<AssetId>;
}
```

The Forgejo concrete implementation (`ForgejoConcrete`) reuses the HTTP client from `oya-ci-webhook-gateway`. The trait seam ensures a clean cutover to the bespoke-VCS host when ADR-0510's numeric trigger fires; only the concrete implementation changes.

### Honesty / non-claims

`oya release` is 0% adopted — no microservice directory exists, no `BUCK` target exists, no oya-ci postsubmit job is wired, no release has been published through this path. This is doctrine + target, not implementation. No numeric release-latency, build-throughput, or artifact-size figure is asserted; every such claim is `blocked_until_required_evidence_is_green` per `hyperscaler-gates.json`.

## Rejected alternatives

### A. Adopt goreleaser as-is

**Description.** Use the existing goreleaser Go binary; configure it to call `buck2` as an external build hook.

**Rejected because:**
- Imports Go into a pure-Rust toolchain — violates the bespoke-over-OSS + pure-Rust doctrine.
- goreleaser's "build" step is a first-class concept; configuring it to call `buck2` instead of `go build` is a fighting-the-tool pattern that forfeits goreleaser's native output types (Go binary artifacts, CGO handling) while inheriting its publishing idioms.
- goreleaser's image-build step uses `docker buildx` — forbidden by the container-tooling-canonical policy (ADR-0381).
- goreleaser's publish model is GitHub/Gitea-release-centric; its SCM abstraction is shallow and does not extend to a bespoke hyperscaler VCS (ADR-0510 destination).
- License: goreleaser is MIT — clean — but the above concerns are architectural, not license-gating.

### B. Shell script around buck2 + cosign + gh CLI

**Description.** A bash/zsh script invokes buck2, cosign, syft, and the Forgejo API directly via curl or `gh`.

**Rejected because:**
- Scripts have no error-model, no structured retry, no observability hooks (OTel spans), and no test surface — all of which are required by the SLO-gated promotion model (ADR-0130 + ADR-0513).
- Shell is not pure-Rust; it adds a sh/bash toolchain dependency to CI agents.
- A shell script cannot express the `Scm` trait seam needed for the Forgejo→bespoke-VCS cutover.
- Scripts are not composable with the oya-ci job-type model (ADR-0513 ProwJob equivalent) without wrapping them in a binary anyway.

### C. Adopt cargo-dist

**Description.** Use [cargo-dist](https://github.com/axodotdev/cargo-dist) (axo, MIT), a Rust-native release automation tool for Cargo workspaces.

**Rejected because:**
- cargo-dist builds via `cargo build`, not `buck2` — same bypass-the-build-graph problem as goreleaser.
- cargo-dist's OCI and multi-arch cross-compile story is shallow compared to the buck2-native OCI assembler (ADR-0515) and the musl-static cross-build already proven in the CI gate.
- cargo-dist targets GitHub Actions as the primary CI surface; its GitLab / Forgejo publish path is community-supported and not the primary surface.
- As a Cargo-centric tool it does not cover non-Rust artifacts (Helm charts, Cedar policy bundles, workflow definitions) that also land in a platform release bundle (ADR-0254 `.oab` format). The Oyatie release surface is wider than a Cargo binary release.

### D. Extend the deployment-control-plane artifact-bundle assembler (ADR-0254 §D-5)

**Description.** Rather than a new µservice, extend the `deployment-control-plane` bundle-authoring component to also drive the forge release publish.

**Rejected because:**
- The `.oab` bundle (ADR-0254 §D-5) is a **deployment-model artifact** — it contains Helm charts, Cedar bundles, OCI layouts, workflow definitions, and compliance packs for a full platform-version upgrade. A forge release entry is a **developer-facing distribution artifact** — binaries, checksums, changelogs. These are separate concerns at different scopes (platform-upgrade vs. forge-publish).
- Coupling release-publish to deployment-control-plane violates the no-grouping doctrine (ADR-0132) and would turn a single-concern µservice into a multi-concern bundle.
- `oya-release` may invoke the deployment-control-plane's bundle-authoring stage (by producing a `.oab` bundle as one of its outputs) — the dependency is one-directional, not collapsed.

## Consequences

### Positive

- Closes the release-stage gap in the CI/CD pipeline; every semver tag on `dev` produces a signed, SLSA-attested, SBOM-attached, checksummed, changelogs-annotated release entry.
- All build provenance flows from the same buck2 content-addressed action graph — no provenance gaps from a parallel "do your own build" step.
- The `Scm` trait seam makes the Forgejo→bespoke-VCS forge cutover (ADR-0510) a two-line implementation swap, not an `oya release` rewrite.
- Pure-Rust throughout — no Go, no shell, no Docker Inc. tooling — consistent with the bespoke-over-OSS + pure-Rust + container-tooling-canonical (ADR-0381) doctrines.
- A new OTel-instrumented, SLO-gated, oya-ci-postsubmit-integrated release service with a clean test surface.

### Negative / cost

- A new µservice to build and operate; the release path is a new production-SLO surface.
- Multi-arch musl-static cross-builds through NativeLink RBE must be operational before `oya release` can produce binaries — it inherits the buck2 migration dependency (ADR-0392).
- The buck2-native OCI assembler (ADR-0515) must be operational before `oya release` can produce signed multi-arch images.
- The changelog generation step requires conventional-commit discipline across all PRs; inconsistent commit messages degrade changelog quality.

### Neutral

- ADR-0039 supply-chain primitives (cosign, SLSA, SBOM) are unchanged; `oya release` drives them, not reimplements them.
- The `.oab` deployment bundle (ADR-0254 §D-5) is a **complementary output** that `oya release` may trigger as a downstream step; their formats and scopes do not overlap.
- tide merge-queue (ADR-0111), ArgoCD (ADR-0349), image-promotion ladder (ADR-0181), and the oya governance overlay (ADR-0513) are all unchanged.

## Verification

The following must be green before any release-throughput or provenance-completeness claim is made:

1. `oya release --dry-run --tag v0.1.0` executes the full orchestration against the buck2 build matrix and produces a local artifact set without publishing; exits 0.
2. A published release on the internal Forgejo instance contains: versioned binaries for each target platform, `.tar.gz` archives, `sha256sums.txt`, CycloneDX SBOM, SPDX SBOM, cosign signature bundle, SLSA L3 provenance attestation, and a changelog entry.
3. `cosign verify` against every binary and OCI manifest in the release exits 0.
4. `cosign verify-attestation --type slsaprovenance` exits 0 for every artifact.
5. The oya-ci postsubmit job for a tag-push run completes with a terminal success status posted to Forgejo.
6. Release latency (tag-push → forge-release-published) SLO is declared in `slos/release-publish.openslo.yaml` and reported via the observability substrate (ADR-0130).

## References

- ADR-0392: Buck2 canonical build graph
- ADR-0408: Buck2-driven CI/CD — affected-targets + image builds
- ADR-0513: oya-ci — bespoke-Rust Prow CI/CD platform
- ADR-0515: buck2-native OCI image assembler (referenced; not yet filed in this corpus)
- ADR-0254: Deployment model spectrum + `.oab` artifact bundle format (§D-5)
- ADR-0039: Supply chain security — Trivy, cosign, SBOM, SLSA
- ADR-0510: SCM destination = bespoke hyperscaler monorepo-VCS; Forgejo transitory
- ADR-0363: Forgejo + git as canonical change-coordination substrate
- ADR-0181: cosign image-promotion dev→staging→prod ladder
- ADR-0131: Per-microservice flat layout
- ADR-0132: No-grouping forward policy
- ADR-0105: Thirteen-layer canonical enum (crate naming BNF)
- goreleaser upstream: https://goreleaser.com (reference shape only; not adopted)
- cargo-dist upstream: https://github.com/axodotdev/cargo-dist (rejected alternative C)
