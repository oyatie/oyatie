# infra/ci-farm/agent-image/Dockerfile
# ============================================================================
# O5 — Pinned, signed CI agent image (ADR-0360, part O5).
#
# A prebuilt CI agent image: digest-pinned base, baked toolchain + cargo-nextest
# + sccache + git + ca-certificates, a warm read-only crate registry, non-root
# numeric UID. Built and cosign-signed BY DIGEST (build-and-sign.sh) and admitted
# only via the scoped Kyverno verifyImages policy (kyverno-verify-agent-image.yaml).
#
# Why digest-pin the base (not a tag): a tag like `rust:1-bookworm` is mutable —
# the same tag resolves to different content over time, so two builds are not
# reproducible and the supply chain is unpinned. Pinning `@sha256:<digest>` makes
# the base byte-identical on every build; the signature then attests THAT content.
#
# >>> FILL THE REAL DIGEST <<< Resolve with:
#     docker buildx imagetools inspect rust:1.86-bookworm --format '{{.Manifest.Digest}}'
# or  crane digest docker.io/library/rust:1.86-bookworm
# then replace BOTH placeholders below. Keep the human-readable tag in the comment
# so reviewers can see which release the digest corresponds to.
# ----------------------------------------------------------------------------

# ---- stage 1: builder — install pinned tools into a relocatable prefix --------
# rust:1.86-bookworm  (replace @sha256 placeholder with the real resolved digest)
FROM docker.io/library/rust@sha256:<PINNED>__REPLACE_ME_rust_1_86_bookworm__ AS builder

# Pinned tool versions — bump deliberately, never float.
ARG NEXTEST_VERSION=0.9.96
ARG SCCACHE_VERSION=0.8.2

# cargo-install both tools with --locked so their own Cargo.lock pins transitive
# deps. They are installed into a throwaway CARGO_HOME and copied into the runtime
# stage, so none of the build toolchain leaks into the final image.
ENV CARGO_HOME=/tmp/cargo-build
RUN cargo install cargo-nextest --locked --version "${NEXTEST_VERSION}" --root /opt/tools \
 && cargo install sccache       --locked --version "${SCCACHE_VERSION}" --root /opt/tools

# ---- stage 2: runtime — minimal, non-root, warm registry ----------------------
# rust:1.86-bookworm  (same pinned digest as the builder stage)
FROM docker.io/library/rust@sha256:<PINNED>__REPLACE_ME_rust_1_86_bookworm__

# --no-install-recommends keeps the image lean; git + ca-certificates only.
# (curl is intentionally NOT installed: the prod image fetches no binaries at
# runtime — everything needed is baked here.)
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      git \
      ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Baked, pinned tools from the builder stage.
COPY --from=builder /opt/tools/bin/cargo-nextest /usr/local/bin/cargo-nextest
COPY --from=builder /opt/tools/bin/sccache       /usr/local/bin/sccache

# --- shared CARGO_HOME with a WARM, READ-ONLY registry -------------------------
# CARGO_HOME lives at a well-known path so the registry index + cached crate
# downloads can be baked warm and mounted read-only at runtime. sccache caches
# COMPILATION; this caches DOWNLOADS (the registry index + .crate files). The two
# are complementary (ADR-0360 O3). The warm-registry contents are populated by a
# `cargo fetch` against the pinned Cargo.lock during the farm image bake; the
# placeholder dir below is the mount point + ownership contract.
ENV CARGO_HOME=/opt/cargo
RUN mkdir -p /opt/cargo/registry \
 # WARM REGISTRY PLACEHOLDER: in the farm bake, `cargo fetch --locked` (run against
 # the repo's Cargo.lock) populates /opt/cargo/registry/{cache,index}. It is then
 # served READ-ONLY (agents never write the blessed registry; PR-scoped writes go
 # to the per-agent workspace CARGO_HOME overlay — see cache/README.md). For the
 # sparse source-replacement that points cargo at the Panamax mirror, agents drop
 # cache/cargo-config-sample.toml at $CARGO_HOME/config.toml at the agent level.
 && :

# --- sccache as the rustc wrapper; deterministic, path-independent cache keys ---
# RUSTC_WRAPPER=sccache         -> every rustc invocation goes through sccache.
# CARGO_INCREMENTAL=0           -> incremental artifacts are per-machine and poison
#                                  a shared cache; disable for cache-friendliness.
# SCCACHE_DIR is intentionally NOT set here: the S3 backend + SCCACHE_BUCKET/
# ENDPOINT/REGION + SCCACHE_S3_KEY_PREFIX + SCCACHE_BASEDIR are injected per-lane
# (see values-local.yaml / cache/README.md) so the same image works for any farm.
ENV RUSTC_WRAPPER=sccache \
    CARGO_INCREMENTAL=0

# --- non-root numeric UID (10001), group 0 -------------------------------------
# Numeric UID satisfies runAsNonRoot admission without a username lookup. Group 0
# (root group) so an OpenShift-style arbitrary-UID runtime can still read group-
# readable baked content. The agent writes only to its mounted workspace; the
# baked /opt/cargo registry is read-only to it.
RUN useradd --uid 10001 --gid 0 --no-create-home --shell /usr/sbin/nologin ciagent || true \
 && chown -R 10001:0 /opt/cargo \
 && chmod -R g=u /opt/cargo
USER 10001:0

# Sanity: the baked tools resolve. (Build-time check; no network.)
RUN cargo --version && sccache --version && cargo-nextest --version && git --version

# No ENTRYPOINT/CMD: the Jenkins kubernetes plugin overrides the container command
# (`sleep infinity`) and drives the agent via the durable-task shell, exactly like
# the values-local.yaml templates.
