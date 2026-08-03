# Hermetic (downloaded + digest-pinned) buck2 Rust toolchain.
#
# WHY IT HAS TO BE AUTHORED: the bundled prelude offers no download-rust rule.
# prelude//toolchains/rust.bzl is `system_rust_toolchain` alone, and a system toolchain
# contributes rustc's PATH — not rustc's CONTENT — to the action key. Swap /usr/bin's rustup
# default and every action key stays UNCHANGED, so a shared CAS happily serves artifacts
# built by a different compiler as current. Here the component sha256s are inputs, so the
# same swap changes the keys.
#
# BEHAVIOUR PARITY is structural, not asserted: this rule sets only the RustToolchainInfo
# fields that genuinely differ from the ambient path (the three tools, the sysroot, the
# triple, the edition, the panic runtime). Every other field is left at its
# `rust_toolchain_attrs` provider default, and those defaults are exactly what
# `system_rust_toolchain` forwards when the declaration carries nothing but
# `default_edition` — which is all toolchains//:rust-system carries. So no lint set, flag
# list, or doctest setting can drift between the two paths without editing both.
#
# CONFIGURATION SCOPE: host == target. The component archives are `exec_dep`s (they run on
# the exec platform) while `rustc_target_triple` selects on the target configuration; those
# agree for every lane this overlay supports. Cross-compiling would need the std archive
# split onto the target configuration — deliberately not built until something asks for it.

load("@prelude//rust:rust_toolchain.bzl", "PanicRuntime", "RustToolchainInfo")
load(
    ":releases.bzl",
    "RUST_DIST_SHA256",
    "RUST_TRIPLE_BY_PLATFORM",
    "rust_dist_strip_prefix",
    "rust_dist_url",
)

def _download_rust_toolchain_impl(ctx: AnalysisContext) -> list[Provider]:
    triple = ctx.attrs.rustc_target_triple
    rustc_dist = ctx.attrs.rustc_dist[DefaultInfo].default_outputs[0]
    std_dist = ctx.attrs.std_dist[DefaultInfo].default_outputs[0]
    clippy_dist = ctx.attrs.clippy_dist[DefaultInfo].default_outputs[0]

    # The component tarballs are UNMERGED: `rustc` ships bin/ + lib/ but nothing under
    # lib/rustlib/<triple>/lib, and `rust-std` ships only that directory. Left unmerged,
    # rustc cannot find std at all. Recompose the two halves into one rustup-shaped sysroot.
    # rustc only READS through --sysroot, so symlinks are sufficient here.
    sysroot = ctx.actions.symlinked_dir("sysroot", {
        "lib/rustlib/{}/bin".format(triple): rustc_dist.project(
            "rustc/lib/rustlib/{}/bin".format(triple),
        ),
        "lib/rustlib/{}/lib".format(triple): std_dist.project(
            "rust-std-{}/lib/rustlib/{}/lib".format(triple, triple),
        ),
    })

    # clippy-driver's only rpath is `@loader_path/../lib` (verified with otool) and the clippy
    # tarball ships bin/ alone, so run in place it dyld-fails on librustc_driver. A SYMLINK
    # does not fix that: dyld and ld.so expand @loader_path / $ORIGIN from the realpath, which
    # lands back in the clippy tarball — measured, same "Library not loaded" failure. The
    # driver therefore has to be a REAL file beside a lib/ holding librustc_driver, which is
    # what copied_dir gives. rustc and rustdoc need none of this: they run from inside the
    # rustc tarball, where ../lib already is their own lib/.
    clippy_root = ctx.actions.copied_dir("clippy-root", {
        "bin/clippy-driver": clippy_dist.project("clippy-preview/bin/clippy-driver"),
        "lib": rustc_dist.project("rustc/lib"),
    })

    return [
        DefaultInfo(default_output = sysroot),
        RustToolchainInfo(
            clippy_driver = RunInfo(args = cmd_args(
                clippy_root.project("bin/clippy-driver"),
                hidden = [clippy_root],
            )),
            compiler = RunInfo(args = cmd_args(
                rustc_dist.project("rustc/bin/rustc"),
                hidden = [rustc_dist],
            )),
            default_edition = ctx.attrs.default_edition,
            panic_runtime = PanicRuntime("unwind"),
            rustc_target_triple = triple,
            rustdoc = RunInfo(args = cmd_args(
                rustc_dist.project("rustc/bin/rustdoc"),
                hidden = [rustc_dist],
            )),
            sysroot_path = sysroot,
        ),
    ]

_download_rust_toolchain = rule(
    impl = _download_rust_toolchain_impl,
    attrs = {
        "clippy_dist": attrs.exec_dep(providers = [DefaultInfo]),
        "default_edition": attrs.option(attrs.string(), default = None),
        "rustc_dist": attrs.exec_dep(providers = [DefaultInfo]),
        "rustc_target_triple": attrs.string(),
        "std_dist": attrs.exec_dep(providers = [DefaultInfo]),
    },
    is_toolchain_rule = True,
)

def _platform_select(template: str):
    """Select `template` formatted with the host/target rust triple, per prelude os+cpu."""
    return select({
        "prelude//os:" + os: select({
            "prelude//cpu:" + cpu: template.format(triple)
            for cpu, triple in cpus.items()
        })
        for os, cpus in RUST_TRIPLE_BY_PLATFORM.items()
    })

def download_rust_toolchain(name: str, default_edition = None, visibility = None):
    """Declare the digest-pinned component archives plus the toolchain that composes them.

    Only the three archives matching the resolved platform are ever configured, so the other
    nine declarations cost parse time and zero bytes of download.
    """
    for package, digests in RUST_DIST_SHA256.items():
        for triple, sha256 in digests.items():
            native.http_archive(
                name = "{}-{}-{}".format(name, package, triple),
                urls = [rust_dist_url(package, triple)],
                sha256 = sha256,
                strip_prefix = rust_dist_strip_prefix(package, triple),
            )

    _download_rust_toolchain(
        name = name,
        clippy_dist = _platform_select(":" + name + "-clippy-{}"),
        default_edition = default_edition,
        rustc_dist = _platform_select(":" + name + "-rustc-{}"),
        rustc_target_triple = _platform_select("{}"),
        std_dist = _platform_select(":" + name + "-rust-std-{}"),
        visibility = visibility,
    )
