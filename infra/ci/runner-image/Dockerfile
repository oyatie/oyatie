# Owned CI runner image — buck2, the pinned Rust toolchain, clang/lld and pwsh baked in.
#
# WHY THIS FILE IS COMMITTED: the live arm64 fleet
# (registry.registry.svc.cluster.local:5000/oya/runner:arm64) was built from a copy of
# this recipe that existed ONLY in a scratchpad directory, which made the image that holds
# merge authority unreproducible. A digest-pinned image whose Dockerfile is not in the repo
# is a declaration wired to nothing.
#
# WHY BAKED AT ALL: the stock actions-runner image is Ubuntu with python3/git/curl but NO
# compiler, NO rustup, NO zstd and NO pwsh (probed, not assumed). Installing them per job
# costs ~2 min on every run. Hyperscaler practice is a golden image; this is that.
#
# ARCH-AGNOSTIC BY CONSTRUCTION: nothing below hardcodes a single architecture. apt-get and
# rustup dispatch on the host; install-buck2.sh switches on `uname -m` and carries a digest
# pin for BOTH x86_64 and aarch64; the pwsh layer does the same. The same Dockerfile
# therefore produces an amd64 image on an amd64 builder with no edit — which is the
# requirement for the amd64 box that joins this cluster later.
#
# BUILD CONTEXT IS THE REPO ROOT (it needs /rust-toolchain.toml and
# /infra/ci/install-buck2.sh). See README.md in this directory.
#
# PINNED BY DIGEST, not by tag. A floating :latest makes the image irreproducible and
# silently re-bases on every rebuild — the same discipline install-buck2.sh applies to
# buck2. Bump this digest deliberately, never implicitly.
FROM ghcr.io/actions/actions-runner:latest@sha256:0cfdcc701ce933c6d243c6b0b2da767366dc9f2e99961d4c3754b0b78084cdda

USER root

# clang is NOT optional and NOT interchangeable with gcc here: toolchains/BUCK sets the
# buck2 cxx toolchain to an ABSOLUTE path —
#   "compiler": "/usr/bin/clang", "compiler_type": "clang", "linker": "/usr/bin/clang"
# so a gcc-only image fails every C/C++ compile action with a missing binary. An earlier
# build of this image shipped with build-essential alone and passed a `cc --version` check,
# because cc resolves to gcc — the check verified a proxy, not the requirement.
#
# lld ships alongside because clang is also the LINKER driver here, and lld is the linker
# Google/Meta standardise on (materially faster than GNU ld on large link steps).
# build-essential is retained: rustc's own default linker driver is `cc`, and some -sys
# crates' build scripts assume gcc/make exist.
# zstd = install-buck2.sh decompresses a .zst release asset.
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      clang lld build-essential zstd ca-certificates curl \
 && rm -rf /var/lib/apt/lists/* \
 && test -x /usr/bin/clang && test -x /usr/bin/ld.lld

# PowerShell is a HARD requirement of the merge-authority lane, not a convenience:
# .github/workflows/presubmit.yml runs the gate matrix's main step under
# `shell: pwsh` for its self-hosted Linux gate leg. The stock actions-runner image has no pwsh — probed
# in a pod from the pulled image, all of pwsh/powershell/pwsh-preview absent and no
# /opt/microsoft install dir — so without this layer the gate job fails on this fleet
# with a missing-shell error, which looks like a gate failure and is not one.
#
# GitHub-HOSTED ubuntu images ship pwsh preinstalled, which is exactly why this dependency
# was invisible until the fleet moved off hosted runners.
ARG PWSH_VERSION=7.6.4
ARG PWSH_SHA256_ARM64=d4ef2382fa452f2ccbdb48a01adbbce9ed64954872123970c16be6d086d1224b
ARG PWSH_SHA256_X64=4471b5a36bfe86ec7af8525d36bb1cacba0128e7aac22d05cc064bc00e604721
RUN set -eux; \
    case "$(uname -m)" in \
      aarch64|arm64)  pwsh_arch=linux-arm64; pwsh_sha="${PWSH_SHA256_ARM64}" ;; \
      x86_64|amd64)   pwsh_arch=linux-x64;   pwsh_sha="${PWSH_SHA256_X64}" ;; \
      *) echo "unsupported architecture for pwsh: $(uname -m)" >&2; exit 1 ;; \
    esac; \
    url="https://github.com/PowerShell/PowerShell/releases/download/v${PWSH_VERSION}/powershell-${PWSH_VERSION}-${pwsh_arch}.tar.gz"; \
    curl --proto '=https' --tlsv1.2 -fsSL -o /tmp/pwsh.tar.gz "$url"; \
    echo "${pwsh_sha}  /tmp/pwsh.tar.gz" | sha256sum -c -; \
    mkdir -p /opt/microsoft/powershell/7; \
    tar zxf /tmp/pwsh.tar.gz -C /opt/microsoft/powershell/7; \
    chmod +x /opt/microsoft/powershell/7/pwsh; \
    ln -sf /opt/microsoft/powershell/7/pwsh /usr/local/bin/pwsh; \
    rm -f /tmp/pwsh.tar.gz; \
    pwsh --version

ENV RUSTUP_HOME=/opt/rust/rustup \
    CARGO_HOME=/opt/rust/cargo \
    PATH=/opt/rust/cargo/bin:/usr/local/bin:$PATH

# rust-toolchain.toml is the SSOT for the channel + components; installing with
# --default-toolchain none then `rustup toolchain install` in the repo dir makes rustup read
# that file rather than duplicating the version here.
COPY rust-toolchain.toml /tmp/toolchain/rust-toolchain.toml
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --profile minimal --default-toolchain none --no-modify-path \
 && cd /tmp/toolchain && rustup toolchain install \
 # Set a DEFAULT toolchain, still sourced from rust-toolchain.toml (no version duplicated
 # here). Without this, `rustc` works only inside a directory that HAS a
 # rust-toolchain.toml — the build-time check passed for exactly that reason while the
 # shipped image was broken for every other caller. Verified by running the toolchain in a
 # pod from the PULLED image, not at build time.
 && rustup default "$(rustup show active-toolchain | awk '{print $1}')" \
 && cd / && rustc --version && cargo --version \
 && /usr/bin/clang --version | head -1

# Digest-pinned buck2, the same adapter edge CI uses. Two non-obvious details:
#   - the script is bash (it uses `local`), so it must be run with bash, not sh;
#   - it installs to ${BUCK2_INSTALL_DIR}/sha256-<digest>/buck2, and the digest differs per
#     architecture, so the stable PATH entry has to be a symlink discovered after the fact
#     rather than a hardcoded directory.
COPY infra/ci/install-buck2.sh /tmp/install-buck2.sh
RUN BUCK2_INSTALL_DIR=/opt/buck2 bash /tmp/install-buck2.sh \
 && ln -sf "$(find /opt/buck2 -type f -name buck2 | head -1)" /usr/local/bin/buck2 \
 && buck2 --version

# Fail the BUILD, not a job, if any baked tool is missing or unreachable on PATH. Every
# assertion here corresponds to something the required workflow actually invokes: rustc and
# cargo (11 rustup steps become no-ops), buck2 (every gate leg), /usr/bin/clang by absolute
# path (toolchains/BUCK), ld.lld (linker), pwsh (`shell: pwsh` gate matrix), and the two
# bundled node runtimes every `uses:` step needs.
RUN set -eux; \
    rustc --version; cargo --version; buck2 --version; pwsh --version; \
    /usr/bin/clang --version | head -1; \
    test -x /usr/bin/ld.lld; \
    test -x /home/runner/externals/node20/bin/node; \
    test -x /home/runner/externals/node24/bin/node; \
    test -x /home/runner/run.sh; \
    echo BAKED-IMAGE-SELFCHECK-OK

RUN chown -R runner:runner /opt/rust /opt/buck2
USER runner
