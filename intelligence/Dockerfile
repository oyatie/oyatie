# oya-cloud-intelligence — production multi-stage build (cargo-chef layer caching).
# Workspace member at crates/oya-cloud-intelligence-app (binary: cloud-intelligence).
#
# Build context is the WORKSPACE root (not this directory) because the binary
# depends on workspace siblings. Build invocation from the repo root:
#   docker build -f intelligence/Dockerfile \
#                -t registry.oyatie.dev/oya-cloud-intelligence:0.1.0 .
# Or via in-cluster BuildKit Job: see intelligence/k8s/
#
# Hyperscaler-lens: rust:1.97.1-bookworm (Apache 2/MIT) +
# gcr.io/distroless/cc-debian12:nonroot (Apache 2). Passes (a)-(d) of the lens.
#
# ADR-0039: supply-chain-security (cosign + SBOM + Trivy)
# ADR-0146: container-base-image-distroless-nonroot
# ADR-0181: container-image-promotion-pipeline

# ── Stage 1: planner ──────────────────────────────────────────────────────────
FROM rust:1.97.1-bookworm AS planner
WORKDIR /build
RUN cargo install cargo-chef --locked
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 2: cacher ───────────────────────────────────────────────────────────
FROM rust:1.97.1-bookworm AS cacher
WORKDIR /build
RUN cargo install cargo-chef --locked
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# ── Stage 3: builder ──────────────────────────────────────────────────────────
FROM rust:1.97.1-bookworm AS builder
WORKDIR /build
COPY . .
COPY --from=cacher /build/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release --locked --bin cloud-intelligence

# ── Stage 4: runtime ──────────────────────────────────────────────────────────
# Distroless cc runtime (Rust + libgcc); non-root (UID 65532).
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /build/target/release/cloud-intelligence /usr/local/bin/cloud-intelligence
# distroless:nonroot maps to UID 65532. SeccompProfile.RuntimeDefault is
# enforced at the Pod level (see iac/k8s/helm/templates/deployment.yaml).
USER nonroot
# Default service port; overridable via $GATEWAY_CONFIG (ConfigMap JSON).
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/cloud-intelligence"]
