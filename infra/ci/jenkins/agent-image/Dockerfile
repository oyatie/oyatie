# rust-ci agent image — adds sccache to the rust:1-bookworm base so Jenkins
# agents can reuse compiler artifacts while Buck2 remains the CI build/test
# authority. This image never invokes Cargo during image construction.
FROM rust:1-bookworm

ARG SCCACHE_VERSION=0.8.2
ARG TARGETARCH
ARG BUCK2_RELEASE=2026-06-01

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends ca-certificates curl zstd; \
    rm -rf /var/lib/apt/lists/*; \
    case "${TARGETARCH:-amd64}" in \
      amd64) sccache_arch="x86_64-unknown-linux-musl"; buck2_arch="x86_64-unknown-linux-gnu" ;; \
      arm64) sccache_arch="aarch64-unknown-linux-musl"; buck2_arch="aarch64-unknown-linux-gnu" ;; \
      *) echo "unsupported TARGETARCH=${TARGETARCH}" >&2; exit 1 ;; \
    esac; \
    curl -fsSL -o /tmp/sccache.tgz \
      "https://github.com/mozilla/sccache/releases/download/v${SCCACHE_VERSION}/sccache-v${SCCACHE_VERSION}-${sccache_arch}.tar.gz"; \
    mkdir -p /tmp/sccache; \
    tar -xzf /tmp/sccache.tgz -C /tmp/sccache --strip-components=1; \
    install -m 0755 /tmp/sccache/sccache /usr/local/bin/sccache; \
    curl -fsSL -o /tmp/buck2.zst \
      "https://github.com/facebook/buck2/releases/download/${BUCK2_RELEASE}/buck2-${buck2_arch}.zst"; \
    zstd -d -c /tmp/buck2.zst > /usr/local/bin/buck2; \
    chmod 0755 /usr/local/bin/buck2; \
    sccache --version; \
    buck2 --version; \
    rm -rf /tmp/sccache /tmp/sccache.tgz /tmp/buck2.zst

# No ENTRYPOINT/CMD — Jenkins's kubernetes cloud sets container command/args
# to "sleep infinity" at pod creation time; this image just adds sccache.
