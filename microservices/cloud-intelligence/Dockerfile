# oya-cloud-intelligence-rest — agent-dispatch gateway binary (the composition root of
# the cloud-intelligence microservice). Workspace member at crates/oya-cloud-intelligence-rest.
#
# Build context is the WORKSPACE root (not this directory) because the binary
# depends on workspace siblings. Build invocation from the repo root:
#   docker build -f microservices/cloud-intelligence/Dockerfile -t cloud-intelligence:dev .
# Or via in-cluster BuildKit (preferred): see microservices/cloud-intelligence/k8s/
# (the BuildKit Job mirrors infra/ci-webhook-gateway/buildkit-build.yaml).
#
# Hyperscaler-lens: rust:1-bookworm (Rust toolchain — Apache 2/MIT) +
# gcr.io/distroless/cc-debian12:nonroot (distroless base — Apache 2). Passes
# (a)-(d) of the lens.
FROM rust:1-bookworm AS builder
WORKDIR /build
COPY . .
RUN cargo build --release --locked --bin oya-cloud-intelligence-rest

# Distroless cc runtime (Rust + libgcc); non-root.
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /build/target/release/oya-cloud-intelligence-rest /usr/local/bin/oya-cloud-intelligence-rest
# distroless:nonroot maps to UID 65532. SeccompProfile.RuntimeDefault is
# enforced at the Pod level (see k8s/cloud-intelligence.yaml).
USER nonroot
# Default service port; can be overridden via $GATEWAY_CONFIG (ConfigMap JSON).
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/oya-cloud-intelligence-rest"]
