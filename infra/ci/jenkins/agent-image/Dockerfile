# rust-ci agent image — adds sccache to the rust:1-bookworm base so Jenkins
# agents (per ADR-0380 D2) can pull/push to SeaweedFS-on-Talos (ADR-0381 D4)
# for incremental cargo build cache. Closes the sccache-restore half of
# ADR-0380 D6 (cache deferred there because rust:1-bookworm doesn't ship sccache).
#
# Hyperscaler-lens (per memory hyperscaler-lens-architectural-filter):
#   - sccache: Mozilla project, Apache 2 / MIT dual-licensed, actively
#     maintained (0.8+ in 2024-2025). Passes (a)-(d).
#   - rust:1-bookworm: official Rust + Debian; Apache 2 + MIT toolchain. Passes.
#
# Build (via BuildKit per ADR-0381 D1):
#   kubectl apply -f infra/ci/jenkins/agent-image/buildkit-build.yaml
# Resulting image: registry.oya-registry.svc.cluster.local:5000/rust-ci:dev
FROM rust:1-bookworm

# sccache binary install via cargo. --locked pins the embedded Cargo.lock for
# reproducibility; clearing the registry cache after install keeps the layer
# small.
RUN cargo install sccache --version "^0.8" --locked \
    && rm -rf /root/.cargo/registry/src \
    && rm -rf /root/.cargo/registry/cache

# Validate sccache is on PATH and prints its version (sanity check at build time).
RUN sccache --version

# No ENTRYPOINT/CMD — Jenkins's kubernetes cloud sets container command/args
# to "sleep infinity" at pod creation time; this image just adds the binary.
