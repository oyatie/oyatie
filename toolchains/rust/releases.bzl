# Digest-pinned Rust distribution tarballs for the hermetic buck2 Rust toolchain
# (`download_rust_toolchain` in toolchains//rust:defs.bzl).
#
# VERSION SSOT stays rust-toolchain.toml. `RUST_VERSION` here must equal its
# `toolchain.channel`, and it is gate-pinned rather than hand-audited: the freshness gate's
# rust_toolchain_drift `check_ci_text` REDs every `1.NN.N` literal under `toolchains/` that
# differs from `rust-toolchain.toml`, so no second pin gate is introduced by this file.
#
# The digests are the `xz_hash` values published in
# https://static.rust-lang.org/dist/channel-rust-1.97.1.toml (manifest-version 2,
# date 2026-07-16, git_commit_hash 8bab26f4f68e0e26f0bb7960be334d5b520ea452) — the SAME
# upstream build that rustup installs today, so moving buck2 onto these tarballs cannot
# change codegen. A wrong digest fails the download closed and loud; it can never be
# silently accepted, which is the whole point relative to a PATH-keyed system toolchain.
#
# COMPONENTS: `rustc` (rustc, rustdoc, librustc_driver), `rust-std` (the target sysroot
# rlibs/dylibs), `clippy` (clippy-driver). rustfmt is deliberately absent: RustToolchainInfo
# has no rustfmt field and no BUCK/.bzl in this repo invokes rustfmt, so a fourth tarball
# per triple would be an unreferenced download plus four digests to maintain.
# ponytail: add a rustfmt row the day a buck2 rule actually runs rustfmt.
#
# PLATFORMS: arm64 + x86_64 on both Linux and macOS are first-class. Windows and riscv64 are
# absent because the hermetic path is opt-in behind
# infra/ci/buckconfig/hermetic-rust.buckconfig (Linux/macOS lanes) and the Windows lane keeps
# the ambient prelude toolchain; an absent triple fails loud at configuration time rather
# than silently resolving to a wrong arch.

RUST_VERSION = "1.97.1"

_DIST_DATE = "2026-07-16"

# prelude os constraint -> prelude cpu constraint -> rust target triple. Mirrors the
# linux/macos branches of `_DEFAULT_TRIPLE` in prelude//toolchains/rust.bzl, which is private
# there and cannot be loaded. Already arch-agnostic — no host arch is hardcoded anywhere.
RUST_TRIPLE_BY_PLATFORM = {
    "linux": {
        "arm64": "aarch64-unknown-linux-gnu",
        "x86_64": "x86_64-unknown-linux-gnu",
    },
    "macos": {
        "arm64": "aarch64-apple-darwin",
        "x86_64": "x86_64-apple-darwin",
    },
}

# package -> triple -> sha256 of the .tar.xz named by `rust_dist_url()`. The package set is
# read off this table's keys, so a package can never be requested without a digest row.
RUST_DIST_SHA256 = {
    "clippy": {
        "aarch64-apple-darwin": "5e44c0ac5ca9b6f14a3c9031a61f583348b902f908f46e95717aef1dbd2807db",
        "aarch64-unknown-linux-gnu": "d8bac7b0ba5ca9bb868ccb9e367a1d52f4837f3ebf4892eaf64cda37ce362bb5",
        "x86_64-apple-darwin": "6dad187a2210db93c63cdf21a376db8b7fe4f5e64d6ef4d404a74d166e59ad74",
        "x86_64-unknown-linux-gnu": "3441df8fb54db985f8c8a3e8356b8874a3f92cc8cca8565cfe36f1dc15935e72",
    },
    "rust-std": {
        "aarch64-apple-darwin": "a4895f5c6995e83cab8687e46b14324592398049def71ce75ca308c981cf200d",
        "aarch64-unknown-linux-gnu": "46aed8e63186350004d8ec6afca798811e6530b514352e5a8a26f3dc4939b3be",
        "x86_64-apple-darwin": "0fa78653023be5bdfeb419edc82e3b1346ccaa23eaa036491cce084101c741dd",
        "x86_64-unknown-linux-gnu": "1c1e704ae80126b7de34f72ea2825f7fd01736dec20732faed47374b95282fba",
    },
    "rustc": {
        "aarch64-apple-darwin": "6076cad38ccabaa24325f26a74080a363a2633a9cd34c473a8977255d8a593cb",
        "aarch64-unknown-linux-gnu": "b344b81f0cd4c2246c7da8b197fe7a339d7dd02bb15cb69b2524115d9c75224c",
        "x86_64-apple-darwin": "3c38289f319bf02fa1c8149ce3e00f261e4efd14813a99f7f7ae4f180c7d1173",
        "x86_64-unknown-linux-gnu": "9819d0a32d56bd339585319c80260e332779f5541fd66838ab7e016d6c814819",
    },
}

def rust_dist_url(package: str, triple: str) -> str:
    return "https://static.rust-lang.org/dist/{}/{}-{}-{}.tar.xz".format(
        _DIST_DATE,
        package,
        RUST_VERSION,
        triple,
    )

def rust_dist_strip_prefix(package: str, triple: str) -> str:
    # Every rust component tarball unpacks under `<package>-<version>-<triple>/`.
    return "{}-{}-{}".format(package, RUST_VERSION, triple)
