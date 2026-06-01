---
id: ADR-0515
title: "buck2-native OCI image assembly + static-musl binaries on distroless static base"
status: Accepted
planning_impact: true
date: 2026-05-31
deciders:
  - council-architecture
  - ops-sre-reliability
supersedes: []
superseded_by: []
amends:
  - ADR-0146
  - ADR-0381
amended_by: []
related:
  - ADR-0514
  - ADR-0513
  - ADR-0375
  - ADR-0254
---
# ADR-0515: buck2-native OCI image assembly + static-musl binaries on distroless static base

## Status

Accepted — 2026-05-31 (founder-locked). Amends ADR-0146 (distroless static-debian12 base —
REAFFIRMED, with the binary static requirement clarified) and the image-build portion of ADR-0381
(Kaniko→BuildKit migration — image assembly is now buck2-native bespoke Rust); the multi-node
Talos cell topology decisions in ADR-0381 (D2–D4) are NOT affected. See §Amendment below.

## Context

Two converging pressures resolve into a single decision:

1. **Binary linkage:** ADR-0146 chose `gcr.io/distroless/static-debian12:nonroot` as the canonical
   base image, and implicitly assumed that Rust binaries would be compiled statically. However, the
   buck2 build had been emitting dynamically-linked glibc binaries (`aarch64-unknown-linux-gnu`
   with dynamic-linker calls), not static binaries. This is the actual defect ADR-0146 caught
   but did not fully address: the base image was correct for a TLS-capable, nonroot-safe service;
   the compilation target was wrong. The fix is to mandate `aarch64-unknown-linux-musl`
   fully-static binaries (rust-lld self-contained linking via the hermetic toolchain cell,
   ADR-0514 D1), not to change the base image.

2. **Image build substrate:** ADR-0381 D1 chose BuildKit (Moby, Apache 2) to replace the archived
   Kaniko. BuildKit is a correct hyperscaler-lens-clean choice and was the right interim target.
   However, it is not hermetic: BuildKit consumes a `Dockerfile` (an interpreted build script)
   executed at build time by an in-cluster daemon, with layer caching that depends on
   daemon-side content-addressing rather than buck2's CAS. This breaks the reproducibility
   invariant: the same source tree can produce different image layers depending on daemon state,
   cache invalidation, or base-image drift. A bespoke Rust OCI assembler in `tools/oci/` that
   reads buck2 build outputs (ELF binary + static assets) and assembles a content-addressed OCI
   manifest directly — with no `Dockerfile`, no daemon, no interpreter — achieves full hermeticity.

The founder decision 2026-05-31: **musl-static binaries on distroless static base, assembled
buck2-natively**. This ADR formalises it.

## Decision

### (a) Base image: `gcr.io/distroless/static-debian12:nonroot` — REAFFIRMED

The canonical base image for all Rust binary containers remains
**`gcr.io/distroless/static-debian12:nonroot`** (ADR-0146's choice, reaffirmed here).

ADR-0146 rejected scratch on three grounds:
- Missing CA-cert bundle (outbound TLS needs `/etc/ssl/certs/ca-certificates.crt`).
- Missing timezone data (ICS / RFC 5322 timestamps need `/usr/share/zoneinfo`).
- Missing `/etc/passwd` UID lookup (nonroot UID 65532 needs a `passwd` entry for
  `whoami`-style crate calls and container-runtime UID resolution).

Those grounds remain valid for Rust services that perform outbound TLS, emit calendar
data, or resolve UIDs — i.e., the vast majority of µservices. The distroless base
provides all three at ~2.5 MB compressed, with Google-maintained CVE tracking and
weekly rebuilds. The nonroot UID 65532 is baked in, satisfying the
`oya.securityContext.podStandard65534` policy.

**Pure scratch** remains a future option ONLY for services that are (a) fully
self-contained with no outbound TLS, no timezone math, and no UID lookups, AND (b)
carry an explicit ADR-0146 carve-out. No such carve-out exists today; no µservice
currently qualifies. A scratch exception requires its own ADR amendment per ADR-0146's
standing rule.

### (b) Binary target: fully-static `aarch64-unknown-linux-musl`

Every µservice binary MUST be compiled as a fully-static
`aarch64-unknown-linux-musl` (or `x86_64-unknown-linux-musl`) binary via the hermetic
toolchain cell (ADR-0514 D1). Requirements:

- Compiled against musl libc (not glibc): no dynamic linker call, no `ld-linux-*.so`
  dependency.
- Linked with rust-lld in self-contained mode: no external shared libraries.
- The resulting binary runs correctly on the distroless static base, which carries no
  glibc but does carry the CA bundle, tzdata, and passwd — exactly what a musl-static
  binary needs from the container filesystem.

This is the actual fix to ADR-0146's implicit static assumption: the base was correct;
the compilation target was wrong. The hermetic toolchain cell (ADR-0514 D1) is the
prerequisite for universal musl-static coverage.

**INTERIM:** Until ADR-0514 D1 lands and is green for all µservices,
`cc-debian12` (glibc dynamic, the existing build default) is the documented
temporary fallback. A µservice may ship with the interim base only if its
`microservices/<ms>/iac/build/Dockerfile*` includes a prominently-commented
`# INTERIM: pending musl-static hermetic toolchain (ADR-0514 D1)` marker. The
`container-base-image` gate enforces removal of interim markers once the toolchain
cell lands.

### (c) Image assembly: buck2-native bespoke Rust OCI assembler

Container images are assembled **buck2-natively** by a bespoke Rust OCI assembler
in `tools/oci/`. The assembler:

- Accepts buck2 build outputs (ELF binary + optional static asset set) as explicit
  inputs.
- Produces a content-addressed OCI manifest + layer tarballs, written to the buck2
  output tree.
- Uses no `Dockerfile`, no BuildKit daemon, no `docker build` invocation.
- Is deterministic: the same inputs always produce bit-identical OCI layers.
- Integrates with buck2's CAS: layer blobs are addressed by their content hash,
  enabling cross-build layer reuse without a daemon-side cache.

This retires BuildKit (ADR-0381 D1) as non-hermetic and replaces it with a pure-Rust,
buck2-native pipeline. The existing `infra/ci-webhook-gateway/buildkit-build.yaml` (if
authored) and any residual `kaniko-build.yaml` are deleted on ADR-0514 D4 cutover.

### (d) Hermetic toolchain dependency

This decision depends on the hermetic toolchain cell musl-static target referenced in
ADR-0514 (D1). The OCI assembler and the musl-static binary requirement are only fully
activated once that target is green. Until then, the INTERIM fallback applies.

## Amendment

### Amends ADR-0146 — base image REAFFIRMED; binary static requirement CLARIFIED

ADR-0146's choice of `gcr.io/distroless/static-debian12:nonroot` as the canonical base
is **reaffirmed**. This ADR does NOT supersede ADR-0146.

What this ADR adds to ADR-0146:
- The binary MUST be `aarch64-unknown-linux-musl` fully-static. ADR-0146 assumed static
  binaries but did not mandate the compilation target; the buck2 build had been producing
  dynamic-glibc binaries. That gap is closed here.
- Image assembly is now buck2-native (bespoke Rust assembler), not BuildKit — so the
  base-image check now applies to OCI manifests assembled by the bespoke assembler, not
  Dockerfiles.
- The `oya gate validate container-base-image` lane continues to enforce the
  distroless base (unchanged from ADR-0146's intent).

ADR-0146's own scratch-exception criterion — *"A single legitimate `scratch` exception
requires its own ADR carve-out; no such exception exists today"* — remains in force.

### Amends ADR-0381 — image-build portion (D1) only

ADR-0381 D1 (Kaniko→BuildKit) is superseded by the buck2-native assembler. The
multi-node Talos cell topology decisions (D2: CP/Worker/Specialty pools; D3: Cilium
cell boundaries; D4: SeaweedFS object-store) are NOT affected and remain canonical.
ADR-0381 is therefore `amended_by` this ADR, not `superseded_by`.

## Consequences

**Positive:**
- Correct static linkage: musl-static on distroless static-debian12 is the right
  pairing — the binary carries its own libc; the base provides CA certs, tzdata, and
  passwd. No CA bundle injection needed; no runtime TLS failures.
- Full hermeticity: same source tree produces bit-identical OCI image, eliminating
  base-image drift as a silent regression vector.
- Small images: a musl-static binary on distroless static-debian12 is ~5–8 MB
  compressed (base ~2.5 MB + binary).
- No daemon dependency: no `buildkitd` to operate, restart, or patch.
- Native buck2 CAS: layer reuse is content-addressed through the same graph as the
  build.

**Negative / cost:**
- The hermetic toolchain cell musl-static target (ADR-0514 D1) must land before all
  µservices can migrate off the interim base. The migration is gated, not instantaneous.
- The bespoke OCI assembler in `tools/oci/` is owned code; edge cases (multi-arch
  manifests, OCI index construction) must be implemented correctly. Mitigated by the
  narrow scope (distroless static + one binary layer is the ~95% case) and by the
  existing test coverage obligation.

**Neutral:**
- The `oya gate validate container-base-image` lane logic is UNCHANGED (still enforces
  distroless static-debian12 base or explicit INTERIM marker).
- ArgoCD's cosign-verify policy (ADR-0349 B2.009) and the audit-chain emitter are
  agnostic to the image-assembly mechanism; no change to those surfaces.

## Rejected Alternatives

- **Change the base to scratch:** rejected. The distroless base provides CA certs,
  tzdata, and `/etc/passwd` that TLS-capable Rust services need. Scratch would require
  embedding those at build time (extra toolchain complexity) or injecting via ConfigMap
  (runtime coupling). ADR-0146 correctly rejected scratch for µservices with TLS,
  timezone math, or UID lookups; that rejection stands. Scratch remains a valid future
  carve-out for fully self-contained binaries (per ADR-0146).
- **Retain glibc dynamic binaries on distroless cc-debian12:** rejected as destination.
  The cc-debian12 base includes glibc; the `static` base does not. Using glibc-dynamic
  binaries on the wrong base image silently breaks. The correct pair is musl-static +
  distroless-static. glibc-dynamic + distroless-cc is also technically valid but adds
  unnecessary glibc CVE surface — acceptable only as INTERIM.
- **Retain BuildKit as the assembler:** rejected. BuildKit is hyperscaler-lens-clean but
  non-hermetic; the buck2-native assembler is strictly superior for a buck2-first build
  graph. BuildKit remains available as a documented fallback if the bespoke assembler
  hits a hard blocker (e.g., a registry requiring a Dockerfile-only workflow).
- **Alpine base instead of distroless:** rejected. Alpine adds BusyBox + apk surface for
  zero benefit over distroless. ADR-0146's rejection of Alpine stands.

## Verification

- `oya gate validate container-base-image` — enforces distroless static-debian12:nonroot
  base (or explicit INTERIM marker) on all µservice images post-toolchain-landing.
- `oya gate validate hermetic-toolchain-default` (ADR-0514 D1) — prerequisite gate; must
  be green before INTERIM markers are removed.
- `file <binary>` or `ldd <binary>` on the built artifact confirms `statically linked`
  with musl (not glibc).
- OCI assembler test: `buck2 test //tools/oci/...` — unit tests covering manifest
  content-addressing, layer deduplication, and multi-arch index construction.
- End-to-end smoke: a CI presubmit run produces a distroless-static-based OCI image,
  pushes to the in-cluster registry (SeaweedFS-backed, ADR-0381 D4), and ArgoCD syncs
  it cleanly.

## References

- ADR-0146 — distroless static-debian12:nonroot (Accepted; **REAFFIRMED and amended
  here** — base is correct, binary static requirement clarified).
- ADR-0381 — Kaniko→BuildKit + multi-node Talos topology; image-build portion (D1)
  **amended here**; topology portion (D2–D4) remains canonical.
- ADR-0514 — Build/CI/CD pipeline target architecture; hermetic toolchain cell (D1) is
  the prerequisite for universal musl-static binaries.
- ADR-0513 — oya-ci bespoke-Rust Prow platform (sibling; CI execution substrate).
- ADR-0375 — Talos + CAPI + ArgoCD fleet substrate (the CD surface that deploys images
  assembled by this ADR's pipeline).
- ADR-0254 — Kubernetes everywhere (runtime substrate on which containers run).
- Founder decision 2026-05-31: distroless static base REAFFIRMED; binary MUST be
  musl-static; image assembly buck2-native.
