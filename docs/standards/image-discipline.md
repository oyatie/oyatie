---
purpose: "Container-image discipline standard. Mandates `gcr.io/distroless/static-debian13` (or `cc-debian13` for FFI), `musl` static linking where feasible, per-binary CI image-size budgets, Cosign keyless OIDC signing, Syft-generated SBOM."
doc_status: published
---

---
doc_class: Standard
shape: ~
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Container-image discipline standard. Mandates `gcr.io/distroless/static-debian13`
  (or `cc-debian13` for FFI), `musl` static linking where feasible, per-binary CI
  image-size budgets, Cosign keyless OIDC signing, Syft-generated SBOM, and SLSA
  Level 2 provenance attestation. Implements MASTERPLAN Directive 5 (distroless
  + smallest-image containers) and Directive 6 (hyperscaler-bar).
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json
planned_enforcement_ref: oya-governance-image-discipline
enforcement_status:
  oya-governance-image-discipline: F-PENDING-IMAGE-DISCIPLINE (crate missing; tracked in registry/stub-audit/2026-05-17/missing-fitness-crates.json)
  oya-governance-container-base: F-PENDING-CONTAINER-BASE (crate missing)
meta_policy: ADR-0133 (chained-enforcement planning contract, pending)
companion_docs:
  - docs/standards/security-review.md
  - docs/standards/release-management.md
  - docs/standards/dependency-policy.md
related_adrs:
  - ADR-0053
  - ADR-0052
  - ADR-0054
---

# Image Discipline

## Doctrinal authority — [decision-principles.json](../../specs/decision-principles.json) + [forbidden-operations.json](../../specs/forbidden-operations.json)

Per MASTERPLAN Directive 5, production binaries are statically-linked Rust
artifacts shipped in **distroless** containers — no shells, no package
managers, no debug tooling. Per Directive 6 (hyperscaler-bar), every image
carries Cosign signing + Syft SBOM + SLSA L2 provenance.

The program-level container strategy and registry plumbing live in
[`docs/TOOLCHAIN.md`](../TOOLCHAIN.md) and the release pipeline; this
standard supplies the per-image authoring rules.

## 1. Canonical base images

| Need | Image | Tag policy | Notes |
|---|---|---|---|
| Static-linked binary (musl + scratch-equivalent) | `gcr.io/distroless/static-debian13` | digest-pinned | smallest; preferred default |
| Glibc-linked binary (FFI, dynamic linkage) | `gcr.io/distroless/cc-debian13` | digest-pinned | for crates requiring libc |
| Java service (no Rust path) | `gcr.io/distroless/java21-debian13` | digest-pinned | only for legacy adapters |
| Build stage (multi-stage) | `clux/muslrust:1.95.0-stable` | digest-pinned | musl toolchain for static linking |
| Build stage (FFI) | `rust:1.95.0-slim-trixie` | digest-pinned | when musl is incompatible |

Per [`lts-versions-verified.md`](lts-versions-verified.md):
**debian13 is current**; debian12 is deprecated (EOL ~Sep 2026). All new
image work targets debian13; legacy debian12 images carry a tracked
migration ADR.

Sources: [distroless support policy](https://github.com/GoogleContainerTools/distroless/blob/main/SUPPORT_POLICY.md),
[muslrust](https://github.com/clux/muslrust),
[Chainguard distroless overview](https://edu.chainguard.dev/chainguard/chainguard-images/overview/).

## 2. Forbidden bases

| Base | Why forbidden |
|---|---|
| `alpine:*` in production | musl libc subtleties + GPLv3 BusyBox surface |
| `ubuntu:*` in production | shells + package managers attack surface |
| `debian:*` in production | shells + package managers (use distroless instead) |
| `latest` tag, any image | non-reproducible |
| Self-hosted FROM scratch with manual cert bundles | distroless ships the cert bundle |

Lane: `oya-governance-container-base` refuses production manifests
that pull from the forbidden list. Build-stage usage is allowed (multi-
stage with FROM gcr.io/distroless/... as final stage).

## 3. Static linking with musl

Default Rust binary target:

```sh
cargo zigbuild --release --target x86_64-unknown-linux-musl
```

or with muslrust:

```sh
docker run --rm -v $PWD:/volume -w /volume clux/muslrust:1.95.0-stable \
  cargo build --release --target x86_64-unknown-linux-musl
```

Rules:

1. Default target: `x86_64-unknown-linux-musl` (and
   `aarch64-unknown-linux-musl` for arm64 builds).
2. FFI-dependent crates (postgres `libpq`, OpenSSL, etc.) MAY use the
   glibc target + `distroless/cc-debian13`. Each such case carries an
   ADR cite naming the FFI dependency.
3. `rustls` is the default TLS stack (no OpenSSL FFI); use `OpenSSL` only
   when an upstream library forces it.

Source: [muslrust](https://github.com/clux/muslrust),
[cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild).

## 4. Dockerfile shape (canonical)

```dockerfile
# syntax=docker/dockerfile:1.7
ARG RUST_VERSION=1.95.0
ARG DISTROLESS=gcr.io/distroless/static-debian13

FROM clux/muslrust:${RUST_VERSION}-stable AS builder
WORKDIR /src
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl \
    --package oya-intelligence-runtime-rag

FROM ${DISTROLESS} AS runtime
ARG GIT_SHA
LABEL org.opencontainers.image.source="https://github.com/oyatie/oyatie"
LABEL org.opencontainers.image.revision="${GIT_SHA}"
COPY --from=builder /src/target/x86_64-unknown-linux-musl/release/oya-intelligence-runtime-rag \
     /usr/local/bin/runtime
USER 65532:65532
ENTRYPOINT ["/usr/local/bin/runtime"]
```

Required rules:

1. Multi-stage with `distroless/*` as the FINAL stage.
2. `USER 65532:65532` (non-root distroless `nonroot` user) — never run
   as root in production.
3. `LABEL org.opencontainers.image.*` per the OCI spec (source, revision,
   created, version).
4. No `apt`, `apk`, `yum`, `microdnf` in the final layer.
5. No `curl`, `wget`, `bash`, `sh` in the final layer.

## 5. Image-size budget

Per-binary budget planned for advisory enforcement by `oya-governance-image-discipline`:

| Binary class | Compressed budget | Decompressed budget |
|---|---|---|
| Kernel / runtime services | ≤ 25 MB | ≤ 60 MB |
| Adapter sidecars | ≤ 15 MB | ≤ 40 MB |
| CLI / migration tooling | ≤ 50 MB | ≤ 120 MB |
| Foundry capability host | ≤ 80 MB | ≤ 200 MB |

A PR that pushes a binary > budget MUST justify in `## Verification`
(reason: new dependency, new feature, etc.) and trigger an ADR if the
budget itself needs to grow.

## 6. Cosign keyless signing

Per [`security-review.md`](security-review.md) §4:

```sh
COSIGN_EXPERIMENTAL=1 cosign sign \
  --bundle target/cosign/oya-intelligence-runtime-rag.bundle \
  ghcr.io/oyatie/oya-intelligence-runtime-rag@sha256:...
```

- Cosign pin: **≥ v3.0.6** (the `--bundle` form is now mandatory; v2
  scripts will fail).
- Identity: GitHub OIDC via Fulcio.
- Transparency: Rekor entry UUID recorded in
  `EVT-RELEASE-DEPLOYED` audit event.

## 7. SBOM (Syft, CycloneDX)

```sh
syft ghcr.io/oyatie/oya-intelligence-runtime-rag@sha256:... \
  -o cyclonedx-json=target/sbom/runtime-rag.cdx.json
cosign attest --predicate target/sbom/runtime-rag.cdx.json \
  --type cyclonedx --bundle target/cosign/runtime-rag.sbom.bundle \
  ghcr.io/oyatie/oya-intelligence-runtime-rag@sha256:...
```

- Syft produces CycloneDX JSON.
- Cosign attests the SBOM against the image digest.
- The SBOM is published alongside the image release and referenced by
  the catalog supply-chain row (per DOC-CATALOG.md §4 `supply-chain`
  lane).

## 8. SLSA L2 provenance

```sh
cosign attest --predicate provenance.json --type slsaprovenance \
  --bundle target/cosign/runtime-rag.provenance.bundle \
  ghcr.io/oyatie/oya-intelligence-runtime-rag@sha256:...
```

SLSA L2 requires:

1. **Provenance exists** (L1).
2. **Provenance signed and tamper-resistant** — Cosign + Fulcio + Rekor
   satisfies this.

SLSA L3 (verified source + isolated build) is the next milestone after
the GitHub Actions reusable workflow lands per ADR-SUP-002.

Sources: [SLSA Provenance v0.1](https://slsa.dev/spec/v0.1/provenance),
[in-toto & SLSA](https://slsa.dev/blog/2023/05/in-toto-and-slsa),
[Wiz — SLSA Framework](https://www.wiz.io/academy/application-security/slsa-framework).

## 9. Admission control

Cluster-side admission verifies signature + provenance before scheduling.
Two options (regional-pack-dependent):

- **policy-controller** (Sigstore project) + Kyverno.
- **Connaisseur** (independent admission controller).

Policy expectation:

- Image MUST be signed by the project Fulcio identity.
- Image MUST carry an SBOM attestation.
- Image MUST carry SLSA-L2 provenance.
- Image MUST be pinned by digest (no tags).

Lane: `oya-governance-release-supply-chain` (per DOC-CATALOG.md §4).

## 10. Trivy scan gate

Per [`dependency-policy.md`](dependency-policy.md) §1.1: **Trivy
≥ v0.70.0** (v0.69.4 forbidden). Image-level CVE scan runs at release;
HIGH + CRITICAL CVEs block release.

```sh
trivy image --severity HIGH,CRITICAL --exit-code 1 --no-progress \
  ghcr.io/oyatie/oya-intelligence-runtime-rag@sha256:...
```

## 11. Anti-patterns

1. **`alpine:*` or `ubuntu:*` as the final stage.** Use distroless.
2. **`latest` tag in a manifest.** Always digest-pin.
3. **Root user in production container.** Use distroless `nonroot` (UID 65532).
4. **Cosign v2 scripts** (no `--bundle`). Update to v3.0.6+.
5. **Skipping the SBOM attestation** because "the build is internal."
   Every artifact has provenance.
6. **In-image `curl` / `wget`** for runtime tasks. Move to a sidecar or
   pre-bake into the binary.

## 12. Sources scanned

- [distroless](https://github.com/GoogleContainerTools/distroless),
  [Chainguard](https://edu.chainguard.dev/chainguard/chainguard-images/overview/).
- [Cosign](https://docs.sigstore.dev/signing/overview/),
  [Syft](https://github.com/anchore/syft),
  [SLSA](https://slsa.dev/).
- [muslrust](https://github.com/clux/muslrust),
  [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild).
- [Trivy](https://trivy.dev/).
- [`lts-versions-verified.md`](lts-versions-verified.md).
- [`hyperscaler-best-practices.md`](hyperscaler-best-practices.md)
  Domain 4 "Container images" + "Supply-chain".
