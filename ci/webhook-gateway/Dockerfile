# CI Webhook Gateway — multi-stage build. Standalone crate (own Cargo.lock); build
# context is this directory. Built in-cluster by BuildKit (ADR-0381 D1; Kaniko archived
# 2024; host docker retired per ADR-0378) and pushed to the in-cluster registry;
# deployed in namespace oya-ci.
FROM rust:1.97.1-bookworm AS builder
WORKDIR /build
COPY . .
RUN cargo build --release --locked --bin oya-ci-webhook-gateway

# Distroless cc runtime (Rust needs libgcc); non-root.
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /build/target/release/oya-ci-webhook-gateway /usr/local/bin/oya-ci-webhook-gateway
EXPOSE 8099
USER nonroot
ENTRYPOINT ["/usr/local/bin/oya-ci-webhook-gateway"]
