# Owned CI runner image — buck2 + the pinned Rust toolchain baked in.
#
# WHY: the stock actions-runner image is Ubuntu with python3/git/curl but NO
# compiler, NO rustup and NO zstd (probed, not assumed). Installing them per job
# costs ~2 min on every run. Hyperscaler practice is a golden image; this is that.
#
# BUILD CONTEXT IS THE REPO ROOT (see build.yaml — no context-sub-path), so both
# COPYs below read the repo's real files instead of copies of them. The first build
# of this image was fed by a hand-made ConfigMap holding /tmp copies of
# rust-toolchain.toml and install-buck2.sh, which is why the shipped digest was
# unreproducible; a root context is what removes that copy.
#
# ARCH-AGNOSTIC BY CONSTRUCTION: nothing below names an architecture. rustup and
# apt-get dispatch on the host, and install-buck2.sh switches on `uname -m` and
# carries a digest pin for BOTH x86_64 and aarch64. The same Dockerfile therefore
# produces an amd64 image on an amd64 builder with no edit.
# PINNED BY DIGEST, not by tag. A floating :latest makes the image
# irreproducible and silently re-bases on every rebuild — the same discipline
# install-buck2.sh applies to buck2 (SHA-256) and the NativeLink manifest
# applies to its image. Bump this digest deliberately, via the scheduled
# base-refresh, never implicitly.
FROM ghcr.io/actions/actions-runner:latest@sha256:0cfdcc701ce933c6d243c6b0b2da767366dc9f2e99961d4c3754b0b78084cdda

USER root

# clang is NOT optional and NOT interchangeable with gcc here: toolchains/BUCK
# sets the buck2 cxx toolchain to an ABSOLUTE path —
#   "compiler": "/usr/bin/clang", "compiler_type": "clang", "linker": "/usr/bin/clang"
# so a gcc-only image fails every C/C++ compile action with a missing binary. An
# earlier build of this image shipped with build-essential alone and passed a
# `cc --version` check, because cc resolves to gcc — the check verified a proxy,
# not the requirement.
#
# lld ships alongside because clang is also the LINKER driver here, and lld is
# the linker Google/Meta standardise on (materially faster than GNU ld on large
# link steps). build-essential is retained: rustc's own default linker driver is
# `cc`, and some -sys crates' build scripts assume gcc/make exist.
# zstd = install-buck2.sh decompresses a .zst release asset.
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      clang lld build-essential zstd ca-certificates curl \
 && rm -rf /var/lib/apt/lists/* \
 && test -x /usr/bin/clang && test -x /usr/bin/ld.lld

ENV RUSTUP_HOME=/opt/rust/rustup \
    CARGO_HOME=/opt/rust/cargo \
    PATH=/opt/rust/cargo/bin:/usr/local/bin:$PATH

# The REPO-ROOT rust-toolchain.toml is the SSOT for the channel + components;
# installing with --default-toolchain none then `rustup toolchain install` in a dir
# holding that file makes rustup read it rather than duplicating the version here.
COPY rust-toolchain.toml /tmp/toolchain/rust-toolchain.toml
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --profile minimal --default-toolchain none --no-modify-path \
 && cd /tmp/toolchain && rustup toolchain install \
 # Set a DEFAULT toolchain, still sourced from rust-toolchain.toml (no version
 # duplicated here). Without this, `rustc` works only inside a directory that
 # HAS a rust-toolchain.toml — the build-time check passed for exactly that
 # reason while the shipped image was broken for every other caller. Verified by
 # running the toolchain in a pod from the PULLED image, not at build time.
 && rustup default "$(rustup show active-toolchain | awk '{print $1}')" \
 && cd / && rustc --version && cargo --version \
 && /usr/bin/clang --version | head -1

# Digest-pinned buck2, via the repo's own installer — the SAME file CI's workflow
# runs (infra/ci/install-buck2.sh), not a copy of it. Two non-obvious details:
#   - the script is bash (it uses `local`), so it must be run with bash, not sh;
#   - it installs to ${BUCK2_INSTALL_DIR}/sha256-<digest>/buck2, and the digest
#     differs per architecture, so the stable PATH entry has to be a symlink
#     discovered after the fact rather than a hardcoded directory.
COPY infra/ci/install-buck2.sh /tmp/install-buck2.sh
RUN BUCK2_INSTALL_DIR=/opt/buck2 bash /tmp/install-buck2.sh \
 && ln -sf "$(find /opt/buck2 -type f -name buck2 | head -1)" /usr/local/bin/buck2 \
 && buck2 --version

RUN chown -R runner:runner /opt/rust /opt/buck2
USER runner
