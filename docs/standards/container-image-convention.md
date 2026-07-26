# Container image convention (ADR-0146)

Authority: ADR-0146 — `gcr.io/distroless/static-debian12:nonroot` is the
canonical base for every Rust binary container shipped from oyatie.

This standard codifies the build pattern that produces a CI-gateable,
2.5 MB, CVE-trackable, nonroot container per µservice.

## Canonical multi-stage Dockerfile

```dockerfile
FROM rust:1.97.1-slim AS builder
WORKDIR /build
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM gcr.io/distroless/static-debian12:nonroot
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/<bin> /app
USER 65532:65532
ENTRYPOINT ["/app"]
```

## Required directives

| Directive | Required value | Validator |
| --- | --- | --- |
| Final-stage `FROM` | `gcr.io/distroless/static-debian12:nonroot` | `oya gate validate container-base-image` |
| `USER` | `65532:65532` (or bare `65532`) | same |

The `:debug-nonroot` variant is the only other accepted final-stage
image. It ships BusyBox so a human can `kubectl exec` into a running
container for ephemeral debugging without rebuilding the chart. Pre-prod
builds may declare it; production images must not.

## How to adopt for a new µservice

1. Copy `microservices/governance/iac/build/Dockerfile.distroless-rust`
   into `microservices/<ms>/iac/build/Dockerfile` and replace the
   `<bin>` token with the µservice's binary name.
2. Wire the build target into the µservice's CI workflow under
   `.github/workflows/<ms>-build.yml` (when the workflow exists).
3. Re-run `oya gate validate container-base-image` locally to confirm
   the lane passes before opening the PR.

## How to migrate an existing µservice

1. Replace the final-stage `FROM` line with the canonical image.
2. Add `USER 65532:65532` if it is missing.
3. Ensure the binary is built with
   `cargo build --release --target x86_64-unknown-linux-musl` so it
   does not link against glibc.
4. Run `oya gate validate container-base-image` and address any
   reported violations.

## Why distroless and not scratch

`scratch` removes the CA-cert bundle, `/etc/passwd`, and timezone data.
The platform µservices (foundry-providers, mail, calendar, recordings,
notes) require all three. ADR-0146 documents the full alternatives
analysis.

## Why distroless and not Alpine

musl/glibc binary-compat risk plus Alpine variants are not
Google-maintained. We compile Rust against `x86_64-unknown-linux-musl`
already; pairing static binaries with distroless static gets the
smallest defensible surface.

## CI lane

The pre-merge `oya gate validate container-base-image` lane scans every
`microservices/*/iac/build/Dockerfile*` plus any Helm `values.yaml`
image stanzas. Violations are surfaced as `NonCanonicalBase`,
`MissingUser`, or `NonCanonicalUser`.

The lane is also surfaced as a row in
`registry/quality/lanes.yaml` so the quality-lanes registry stays
authoritative.
