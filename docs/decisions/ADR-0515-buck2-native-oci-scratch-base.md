---
id: ADR-0515
title: "buck2-native OCI image assembly + scratch base for fully-static binaries"
status: Accepted
planning_impact: true
date: 2026-05-31
deciders:
  - council-architecture
  - ops-sre-reliability
supersedes:
  - ADR-0146
  - ADR-0381
superseded_by: []
amends: []
amended_by: []
related:
  - ADR-0514
  - ADR-0513
  - ADR-0375
  - ADR-0254
---
# ADR-0515: buck2-native OCI image assembly + scratch base for fully-static binaries

## Status

Accepted — 2026-05-31 (founder-locked). Supersedes ADR-0146 (distroless static-debian12 base) and
the image-build portion of ADR-0381 (Kaniko→BuildKit migration); the multi-node Talos cell topology
decisions in ADR-0381 (D2–D4) are NOT affected. See §Supersession below.

## Context

Two converging pressures resolve into a single decision:

1. **Base image:** ADR-0146 chose `gcr.io/distroless/static-debian12:nonroot` over `scratch` on the
   grounds that (a) CA-cert bundle, (b) timezone data, and (c) `/etc/passwd` UID lookup were needed.
   Those grounds were valid when ADR-0146 was authored (2026-05-18) — they assumed that Rust binaries
   were compiled against glibc (dynamic, or at least needing the distroless runtime shims). ADR-0146
   itself stated: *"A single legitimate `scratch` exception requires its own ADR carve-out; no such
   exception exists today."* That exception now exists: the hermetic toolchain cell (ADR-0514 D1)
   emits `aarch64-unknown-linux-musl` fully-static binaries. A musl-static binary carries its own
   libc; it has no glibc dependency and no dynamic linker call. CA certificates, timezone data, and
   passwd lookup are embedded in the binary or injected via ConfigMap at runtime — not sourced from
   the container filesystem. The stated ADR-0146 blockers for scratch are all resolved.

2. **Image build substrate:** ADR-0381 D1 chose BuildKit (Moby, Apache 2) to replace the archived
   Kaniko. BuildKit is a correct hyperscaler-lens-clean choice and was the right interim target.
   However, it is not hermetic: BuildKit consumes a `Dockerfile` (an interpreted build script) that
   is executed at build time by an in-cluster daemon, with layer caching that depends on
   daemon-side content-addressing rather than buck2's CAS. This breaks the reproducibility
   invariant: the same source tree can produce different image layers depending on daemon state,
   cache invalidation, or base-image drift. A bespoke Rust OCI assembler in `tools/oci/` that
   reads buck2 build outputs (ELF binary + static assets) and assembles a content-addressed OCI
   manifest directly — with no `Dockerfile`, no daemon, no interpreter — achieves full hermeticity.
   It is also simpler: the final image for a musl-static binary is two layers (scratch root +
   binary), which a small Rust library can assemble in milliseconds without spawning a daemon.

The founder decision 2026-05-31: **musl-static → scratch**. This ADR formalises it.

## Decision

### (a) Base image: scratch

The canonical base image for all Rust binary containers is **`scratch`** (empty filesystem).

Every µservice binary MUST be compiled as a fully-static `aarch64-unknown-linux-musl` (or
`x86_64-unknown-linux-musl`) binary via the hermetic toolchain cell (ADR-0514 D1). The resulting
container image has:

- Layer 0: scratch (empty).
- Layer 1: the static binary at `/binary` (or the conventional path per the Helm chart convention).
- Optional layer 2: a baked-in CA-bundle ConfigMap mount or embedded cert pool (for services that
  perform outbound TLS — injected at runtime via K8s projected volumes, not sourced from base).

The `oya gate validate container-base-image` lane (established in ADR-0146) is updated to enforce
`scratch` as the canonical base rather than distroless.

**Hyperscaler lens:** scratch = 0 base-OS CVEs (nothing to CVE-track), smallest possible attack
surface, fully hermetic, pure-Rust endpoint with no libc/userland dependency. Every hyperscaler
internally uses scratch or equivalent distroless-minimal-scratch for their own statically-compiled
control-plane services (GCP's gRPC-health-probe containers, AWS's eksctl-utils, Cloudflare's
Rust-based edge binaries).

**INTERIM:** Until the hermetic toolchain cell musl-static target (ADR-0514 D1) lands and is
green for all µservices, `cc-debian12` (glibc dynamic, the existing build default) is the
documented temporary fallback. A µservice may ship with the interim base only if its
`microservices/<ms>/iac/build/Dockerfile*` includes a prominently-commented
`# INTERIM: pending musl-static hermetic toolchain (ADR-0514 D1)` marker. The
`container-base-image` gate enforces removal of interim markers once the toolchain cell lands.

### (b) Image assembly: buck2-native bespoke Rust OCI assembler

Container images are assembled **buck2-natively** by a bespoke Rust OCI assembler in `tools/oci/`.
The assembler:

- Accepts buck2 build outputs (ELF binary + optional static asset set) as explicit inputs.
- Produces a content-addressed OCI manifest + layer tarballs, written to the buck2 output tree.
- Uses no `Dockerfile`, no BuildKit daemon, no `docker build` invocation.
- Is deterministic: the same inputs always produce bit-identical OCI layers.
- Integrates with buck2's CAS: layer blobs are addressed by their content hash, enabling
  cross-build layer reuse without a daemon-side cache.

This retires BuildKit (ADR-0381 D1) as non-hermetic and replaces it with a pure-Rust,
buck2-native pipeline. The existing `infra/ci-webhook-gateway/buildkit-build.yaml` (if authored)
and any residual `kaniko-build.yaml` are deleted on ADR-0514 D4 cutover.

### (c) Hermetic toolchain dependency

This decision depends on the hermetic toolchain cell musl-static target referenced in ADR-0514
(the hermetic toolchain cell musl-static target, ADR-0514 D1). The OCI assembler and the scratch
base are only fully activated once that target is green. Until then, the INTERIM fallback applies.

## Supersession

- **Supersedes ADR-0146** ("Container base image: distroless static-debian12:nonroot"): the
  distroless base is replaced by scratch. ADR-0146's own stated exception criterion —
  *"no such [scratch] exception exists today"* — is satisfied by the musl-static hermetic
  toolchain. The CVE-tracking and weekly-rebuild benefits of distroless are superseded by the
  stronger guarantee of 0 base-OS CVEs (scratch contains nothing to track).

- **Supersedes the image-build portion of ADR-0381** ("Kaniko→BuildKit migration"): BuildKit
  was the correct interim target over archived Kaniko, but it is non-hermetic; the buck2-native
  bespoke Rust OCI assembler is the hermetic replacement. ADR-0381's multi-node Talos cell
  topology decisions (D2–D4: CP/Worker/Specialty pools, Cilium cell boundaries, SeaweedFS
  object-store) are NOT superseded and remain canonical. ADR-0381 is therefore `amended_by`
  this ADR (not fully `superseded_by`) because the topology half is independent and active.

## Consequences

**Positive:**
- 0 base-OS CVEs: scratch contains nothing; there is no base-layer CVE surface.
- Full hermeticity: same source tree produces bit-identical OCI image, eliminating base-image
  drift as a silent regression vector.
- Smaller images: a musl-static binary in a scratch image is typically under 15 MB compressed,
  vs. 17–20 MB with distroless overhead.
- No daemon dependency: no `buildkitd` to operate, restart, or patch.
- Native buck2 CAS: layer reuse is content-addressed through the same graph as the build.

**Negative / cost:**
- The hermetic toolchain cell musl-static target (ADR-0514 D1) must land before all µservices
  can migrate off the interim base. The migration is gated, not instantaneous.
- µservices that perform outbound TLS must explicitly embed or mount a CA-bundle; distroless
  provided this implicitly. This is a one-time per-µservice authoring task.
- The bespoke OCI assembler in `tools/oci/` is owned code; edge cases (multi-arch manifests,
  OCI index construction) must be implemented correctly. Mitigated by the narrow scope (scratch
  + one binary layer is the ~95% case) and by the existing test coverage obligation.

**Neutral:**
- The `oya gate validate container-base-image` lane logic changes (distroless → scratch) but
  the enforcement point is unchanged.
- ArgoCD's cosign-verify policy (ADR-0349 B2.009) and the audit-chain emitter are agnostic to
  the base image choice; no change to those surfaces.

## Rejected Alternatives

- **Retain distroless static-debian12 permanently:** rejected. ADR-0146's grounds for choosing
  distroless over scratch were correct at the time; those grounds no longer hold once
  musl-static is the compile target. Retaining distroless adds unnecessary base-layer CVE surface
  and breaks hermeticity (weekly Google rebuilds = base-layer churn outside our CAS).
- **Retain BuildKit as the assembler:** rejected. BuildKit is hyperscaler-lens-clean but
  non-hermetic; the buck2-native assembler is strictly superior for a buck2-first build graph.
  BuildKit remains available as a documented fallback if the bespoke assembler hits a hard
  blocker (e.g., a registry requiring a Dockerfile-only workflow).
- **Alpine base instead of scratch:** rejected. musl-static binaries do not need Alpine's
  userland; Alpine adds BusyBox + apk surface for zero benefit.
- **Distroless cc-debian12 (glibc) permanently:** rejected as destination. Acceptable as
  INTERIM only (see §Decision (a) above); the destination is scratch + musl-static.

## Verification

- `oya gate validate container-base-image` — enforces scratch base (or explicit INTERIM marker)
  on all µservice Dockerfiles post-toolchain-landing.
- `oya gate validate hermetic-toolchain-default` (ADR-0514 D1) — prerequisite gate; must be
  green before INTERIM markers are removed.
- OCI assembler test: `buck2 test //tools/oci/...` — unit tests covering manifest
  content-addressing, layer deduplication, and multi-arch index construction.
- End-to-end smoke: a CI presubmit run produces a scratch-based OCI image, pushes to the
  in-cluster registry (SeaweedFS-backed, ADR-0381 D4), and ArgoCD syncs it cleanly.

## References

- ADR-0146 — distroless static-debian12:nonroot (Accepted; **superseded here**).
- ADR-0381 — Kaniko→BuildKit + multi-node Talos topology; image-build portion **superseded
  here**; topology portion (D2–D4) remains canonical.
- ADR-0514 — Build/CI/CD pipeline target architecture; hermetic toolchain cell (D1) is the
  prerequisite for scratch + musl-static.
- ADR-0513 — oya-ci bespoke-Rust Prow platform (sibling; CI execution substrate).
- ADR-0375 — Talos + CAPI + ArgoCD fleet substrate (the CD surface that deploys images
  assembled by this ADR's pipeline).
- ADR-0254 — Kubernetes everywhere (runtime substrate on which containers run).
- Founder decision 2026-05-31: musl-static → scratch.
