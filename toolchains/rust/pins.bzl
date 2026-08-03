# Content-addressed pins for the hermetic Rust toolchain (ADR-0392 buck2 canonical build
# graph). One cell per (component, triple): the immutable dated static.rust-lang.org dist URL
# plus its sha256. `ctx.actions.download_file` verifies the digest, so a wrong or re-pointed
# URL fails the download CLOSED — it can never silently substitute a foreign compiler.
#
# The channel is NOT duplicated here as a bare literal: every URL below carries it, and the
# `rust_toolchain_drift` freshness gate REDs any URL whose version diverges from
# rust-toolchain.toml (`toolchain.channel`), which stays the single version SSOT.
#
# The four component archives must ALL be pinned for every supported triple: rustc alone
# cannot find std, and clippy-driver's @loader_path/../lib rpath only resolves once the rustc
# driver dylib sits beside it in one composed tree (see toolchains/rust/defs.bzl).
#
# Windows has no cell: the ONE required windows-latest lane keeps the prelude's
# `system_rust_toolchain` (toolchains//:rust_system) via `toolchain_alias`. riscv64 likewise has
# no cell, so the archive `select()` in defs.bzl deliberately has no DEFAULT branch — an
# unpinned host fails at CONFIGURATION time (loud) instead of falling back to an ambient rustc.

# Directory inside each extracted archive that holds the rustup-shaped payload. rustup's own
# component names are `clippy-preview` / `rustfmt-preview` even though the FILE names are
# `clippy-` / `rustfmt-`; `{triple}` is substituted per cell.
RUST_TOOLCHAIN_COMPONENT_DIRS = {
    "clippy": "clippy-preview",
    "rust-std": "rust-std-{triple}",
    "rustc": "rustc",
    "rustfmt": "rustfmt-preview",
}

# component -> triple -> (url, sha256). Every digest below is the computed sha256 of the real
# downloaded bytes, cross-checked against the upstream channel manifest (`xz_hash`).
RUST_TOOLCHAIN_PINS = {
    "clippy": {
        "aarch64-apple-darwin": (
            "https://static.rust-lang.org/dist/2026-07-16/clippy-1.97.1-aarch64-apple-darwin.tar.xz",
            "5e44c0ac5ca9b6f14a3c9031a61f583348b902f908f46e95717aef1dbd2807db",
        ),
        "aarch64-unknown-linux-gnu": (
            "https://static.rust-lang.org/dist/2026-07-16/clippy-1.97.1-aarch64-unknown-linux-gnu.tar.xz",
            "d8bac7b0ba5ca9bb868ccb9e367a1d52f4837f3ebf4892eaf64cda37ce362bb5",
        ),
        "x86_64-apple-darwin": (
            "https://static.rust-lang.org/dist/2026-07-16/clippy-1.97.1-x86_64-apple-darwin.tar.xz",
            "6dad187a2210db93c63cdf21a376db8b7fe4f5e64d6ef4d404a74d166e59ad74",
        ),
        "x86_64-unknown-linux-gnu": (
            "https://static.rust-lang.org/dist/2026-07-16/clippy-1.97.1-x86_64-unknown-linux-gnu.tar.xz",
            "3441df8fb54db985f8c8a3e8356b8874a3f92cc8cca8565cfe36f1dc15935e72",
        ),
    },
    "rust-std": {
        "aarch64-apple-darwin": (
            "https://static.rust-lang.org/dist/2026-07-16/rust-std-1.97.1-aarch64-apple-darwin.tar.xz",
            "a4895f5c6995e83cab8687e46b14324592398049def71ce75ca308c981cf200d",
        ),
        "aarch64-unknown-linux-gnu": (
            "https://static.rust-lang.org/dist/2026-07-16/rust-std-1.97.1-aarch64-unknown-linux-gnu.tar.xz",
            "46aed8e63186350004d8ec6afca798811e6530b514352e5a8a26f3dc4939b3be",
        ),
        "x86_64-apple-darwin": (
            "https://static.rust-lang.org/dist/2026-07-16/rust-std-1.97.1-x86_64-apple-darwin.tar.xz",
            "0fa78653023be5bdfeb419edc82e3b1346ccaa23eaa036491cce084101c741dd",
        ),
        "x86_64-unknown-linux-gnu": (
            "https://static.rust-lang.org/dist/2026-07-16/rust-std-1.97.1-x86_64-unknown-linux-gnu.tar.xz",
            "1c1e704ae80126b7de34f72ea2825f7fd01736dec20732faed47374b95282fba",
        ),
    },
    "rustc": {
        "aarch64-apple-darwin": (
            "https://static.rust-lang.org/dist/2026-07-16/rustc-1.97.1-aarch64-apple-darwin.tar.xz",
            "6076cad38ccabaa24325f26a74080a363a2633a9cd34c473a8977255d8a593cb",
        ),
        "aarch64-unknown-linux-gnu": (
            "https://static.rust-lang.org/dist/2026-07-16/rustc-1.97.1-aarch64-unknown-linux-gnu.tar.xz",
            "b344b81f0cd4c2246c7da8b197fe7a339d7dd02bb15cb69b2524115d9c75224c",
        ),
        "x86_64-apple-darwin": (
            "https://static.rust-lang.org/dist/2026-07-16/rustc-1.97.1-x86_64-apple-darwin.tar.xz",
            "3c38289f319bf02fa1c8149ce3e00f261e4efd14813a99f7f7ae4f180c7d1173",
        ),
        "x86_64-unknown-linux-gnu": (
            "https://static.rust-lang.org/dist/2026-07-16/rustc-1.97.1-x86_64-unknown-linux-gnu.tar.xz",
            "9819d0a32d56bd339585319c80260e332779f5541fd66838ab7e016d6c814819",
        ),
    },
    "rustfmt": {
        "aarch64-apple-darwin": (
            "https://static.rust-lang.org/dist/2026-07-16/rustfmt-1.97.1-aarch64-apple-darwin.tar.xz",
            "358bbba5d0c7c37116ec15f67cfd3ac4da5d3c319cddb49389c26d3a0c65747a",
        ),
        "aarch64-unknown-linux-gnu": (
            "https://static.rust-lang.org/dist/2026-07-16/rustfmt-1.97.1-aarch64-unknown-linux-gnu.tar.xz",
            "3dbde15d30794924195ae446f3d2ceb542a131306d22ae7912c7634d414622a8",
        ),
        "x86_64-apple-darwin": (
            "https://static.rust-lang.org/dist/2026-07-16/rustfmt-1.97.1-x86_64-apple-darwin.tar.xz",
            "457c35a619207d35da2a3804940e620ad7cdc8e0808b17f2f6c2202f9e3f3d91",
        ),
        "x86_64-unknown-linux-gnu": (
            "https://static.rust-lang.org/dist/2026-07-16/rustfmt-1.97.1-x86_64-unknown-linux-gnu.tar.xz",
            "907fe97d6afbde1eca1b34c992c76e1406d422e2e6f137813d382acec7eb4d14",
        ),
    },
}
