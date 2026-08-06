---
id: ADR-0146
status: Superseded
superseded_by: [ADR-709]
---

# ADR-0146 — Container base image: distroless `static-debian12:nonroot`

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-05-18 |
| Deciders | axis-governance, axis-cloud-k8s, axis-observability |
| Supersedes | — |
| Superseded by | — |
| Related | ADR-0064 (canonical-base + localization), ADR-0131 (per-µservice flat layout), ADR-0083 (kernel-tier invariants) |

## Context

Every Rust binary container that ships out of oyatie ultimately runs in
a hyperscaler-grade Kubernetes cluster (workers in gVisor sandboxes per
the foundry / mail / recordings tiering). User directive 2026-05-18
("distroless or scratch preference") asks for a single canonical base
image, applied uniformly across all 33 µservices, so the CVE attack
surface collapses to the smallest defensible set.

The four candidate bases were:

1. `scratch` — no userland at all.
2. `ubuntu-minimal` / `debian-slim` — full distro minus extras.
3. `alpine` — musl libc, BusyBox userland.
4. `gcr.io/distroless/static-debian12:nonroot` — Google-maintained
   distroless static.

## Decision

The canonical base image for all Rust binary containers is
**`gcr.io/distroless/static-debian12:nonroot`** (with the
`:debug-nonroot` variant accepted only for explicit dev builds).

Every µservice's `microservices/<ms>/iac/build/Dockerfile*` MUST:

- Use the canonical base on the final stage.
- Declare `USER 65532:65532` (or `USER 65532`) on the final stage.
- Be validated by the
  `oya gate validate container-base-image` lane in pre-merge CI.

A single legitimate `scratch` exception requires its own ADR carve-out;
no such exception exists today.

## Alternatives considered

### (a) `scratch` — rejected

- Missing CA-cert bundle. `foundry-providers` calls the Anthropic /
  OpenAI / Google APIs over TLS and would refuse handshakes without
  `/etc/ssl/certs/ca-certificates.crt`.
- Missing timezone data. mail / calendar / recordings emit ICS, RFC
  5322 timestamps, and human-readable durations that need
  `/usr/share/zoneinfo`.
- Missing `/etc/passwd` UID lookup. nonroot UID 65532 has no shell
  entry, so any code that resolves UID to a name (Go's `os/user`
  cross-runtime probes, the Rust `whoami` crate, etc.) breaks.

### (b) Ubuntu minimal / debian-slim — rejected

- 30 MB+ base layer; ships apt + dpkg + bash. Every package manager
  binary is reachable CVE surface that we never use post-build.
- Adopting it would invert the agentic SLO-gated promotion direction
  (ADR-0139): smaller-but-Google-CVE-tracked > larger-but-self-tracked.

### (c) Alpine — rejected

- musl libc vs. glibc binary-compat risk. We compile Rust against
  `x86_64-unknown-linux-musl` for static binaries anyway, so the
  alpine userland adds zero ergonomic value.
- Google does not publish Alpine variants of its distroless images.
  Standardizing on Alpine forks the dependency tree.

### (d) `gcr.io/distroless/static-debian12:nonroot` — accepted

- 2.5 MB compressed; only the runtime libs a static binary actually
  needs.
- Google-maintained CVE tracking with weekly rebuilds.
- Ships CA-cert bundle, `/etc/passwd` with the `nonroot` UID 65532
  pre-baked, and timezone data.
- nonroot UID baked in: aligns with the
  `oya.securityContext.podStandard65534` standard (the policy already
  accepts UID 65532 as the only distroless-native nonroot value).
- `:debug-nonroot` adds BusyBox for ephemeral `kubectl exec`-style
  debugging without rebuilding charts.

## Industry citations

- Google distroless project (the upstream that invented the pattern
  and uses it internally for GCP control-plane components).
- AWS Well-Architected Operational Excellence Pillar (2024) recommends
  distroless or minimal-distro bases.
- Stripe security engineering blog: "distroless for everything".
- Cloudflare engineering blog: distroless used for their Rust
  workloads.
- Anthropic public statements about their training-stack containers.
- CIS Kubernetes Benchmark v1.10 restricted profile compliance is
  satisfied by the baked-in nonroot UID.

## Consequences

1. **All 33 µservice Dockerfiles standardize on the canonical base.**
   Existing Dockerfiles get migrated via mechanical sed of the FROM
   line; the new `oya gate validate container-base-image` lane fails
   pre-merge CI if any file drifts.
2. **Workers in gVisor still use distroless.** Layered defense:
   gVisor = host-kernel isolation, distroless = container minimal
   userland. Neither replaces the other.
3. **The single legitimate `scratch` exception requires explicit ADR
   carve-out.** Reserved for truly self-contained binaries that
   perform no TLS, no timezone math, and no UID lookups (currently
   zero µservices qualify).
4. **Helm `values.yaml` image stanzas reference the canonical base via
   the `oya.dockerfile.distroless-rust` chart helper** documented in
   `docs/standards/container-image-convention.md`.
5. **Future bump to Debian 13 (trixie) flows through this ADR.** A
   replacement ADR will be authored when Google promotes the trixie
   variant.

## Compliance

The convention is enforced by:

- `crates/oya-check-container-base-image/` (Layer-1 kernel-tier
  validator per ADR-0083).
- `cargo run -p oya-dev-cli -- gate validate container-base-image`
  (CLI integration).
- `microservices/governance/iac/build/Dockerfile.distroless-rust`
  (canonical multi-stage template).
- `docs/standards/container-image-convention.md` (humanreadable
  convention).
