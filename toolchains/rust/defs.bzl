# Hermetic, content-addressed Rust toolchain for buck2.
#
# THE DEFECT THIS REPLACES: `system_rust_toolchain` publishes `RunInfo(args = ["rustc"])`, so the
# compiler enters the action key by PATH, never by CONTENT. Swapping the rustup default leaves
# every action key UNCHANGED, and a shared CAS would then serve foreign-compiler artifacts as
# current. Here the compiler is a digest-pinned archive, so the compiler bytes are part of the
# action key.
#
# THE SYSROOT CONTRACT (the one thing that must not be got wrong):
# `prelude//rust/cargo_buildscript.bzl` `_make_rustc_shim` branches ONLY on
# `explicit_sysroot_deps` and NEVER reads `sysroot_path`, so every `buildscript_run` action gets a
# bare `$RUSTC` with NO `--sysroot`. And `prelude//rust/context.bzl` sets
# `skip_setting_sysroot` once `sysroot_path` is set, so the clippy wrapper stops exporting
# SYSROOT. Both therefore require the composed tree to be SELF-SUFFICIENT: rustc derives its
# sysroot from the CANONICALISED path of the loaded librustc_driver dylib, and dyld/`$ORIGIN`
# resolve symlinks, so `bin/{rustc,rustdoc,clippy-driver,rustfmt}` and `lib/**` are COPIED, not
# symlinked. A symlink farm yields `E0463: can't find crate for std` while buck2 still reports
# BUILD SUCCEEDED — measured — and a symlinked clippy-driver cannot even load (the clippy tarball
# ships no `lib/`, so its `@loader_path/../lib` rpath dangles).
#
# `sysroot_path` is then set as well, which is what covers the compile/rustdoc actions. It is
# deliberately NOT baked into the `compiler` RunInfo: `prelude//rust/tools/tool_rules.bzl` appends
# an unconditional `cmd_args("--sysroot=")`, and rustc hard-errors on a duplicate `sysroot`
# option. `explicit_sysroot_deps` is likewise NOT used — it would demand `Dependency`s for the
# hash-suffixed sysroot rlibs, which are unknowable at analysis time.
#
# The `compiler` RunInfo must also be EXACTLY ONE argument: `prelude//rust/build.bzl` builds
# `cmd_args("--test-builder=", compiler, delimiter = "")`, which glues a two-element cmd_args into
# one broken token. The composed directory rides along as `hidden` so it materialises without
# contributing an argument.

load("@prelude//rust:rust_toolchain.bzl", "PanicRuntime", "RustToolchainInfo")
load(
    ":pins.bzl",
    "RUST_TOOLCHAIN_COMPONENT_DIRS",
    "RUST_TOOLCHAIN_PINS",
)

_ARCHIVE_SUFFIX = ".tar.xz"

# Copied VERBATIM from the private `_DEFAULT_TRIPLE` in `prelude//toolchains/rust.bzl` so the
# hermetic toolchain reports exactly the triple the system toolchain reported. It is also the
# precedent proving `select()` resolves inside a toolchain rule's attrs.
_DEFAULT_TRIPLE = select({
    "prelude//os:linux": select({
        "prelude//cpu:arm64": "aarch64-unknown-linux-gnu",
        "prelude//cpu:riscv64": "riscv64gc-unknown-linux-gnu",
        "prelude//cpu:x86_64": "x86_64-unknown-linux-gnu",
    }),
    "prelude//os:macos": select({
        "prelude//cpu:arm64": "aarch64-apple-darwin",
        "prelude//cpu:x86_64": "x86_64-apple-darwin",
    }),
    "prelude//os:windows": select({
        "prelude//cpu:arm64": select({
            # Rustup's default ABI for the host on Windows is MSVC, not GNU.
            # When you do `rustup install stable` that's the one you get. It
            # makes you opt in to GNU by `rustup install stable-gnu`.
            "DEFAULT": "aarch64-pc-windows-msvc",
            "prelude//abi:gnu": "aarch64-pc-windows-gnu",
            "prelude//abi:msvc": "aarch64-pc-windows-msvc",
        }),
        "prelude//cpu:x86_64": select({
            "DEFAULT": "x86_64-pc-windows-msvc",
            "prelude//abi:gnu": "x86_64-pc-windows-gnu",
            "prelude//abi:msvc": "x86_64-pc-windows-msvc",
        }),
    }),
})

def _archive_root(url):
    return url.rsplit("/", 1)[1][:-len(_ARCHIVE_SUFFIX)]

def _sub_target_paths(component, triple):
    if component == "rustc":
        # `lib` wholesale keeps librustc_driver-<hash>.{so,dylib} a real file (so rustc's
        # canonicalised sysroot lands inside the composed tree AND clippy-driver's rpath
        # resolves) without naming the per-triple hash, and keeps lib/rustlib/<triple>/bin
        # (rust-lld and friends) real too.
        return ["bin/rustc", "bin/rustdoc", "lib"]
    if component == "rust-std":
        return ["lib/rustlib/{}/lib".format(triple)]
    if component == "clippy":
        return ["bin/clippy-driver"]
    if component == "rustfmt":
        return ["bin/rustfmt"]
    fail("unpinned Rust toolchain component: {}".format(component))

def rust_toolchain_archives():
    """Declare one digest-pinned `http_archive` per (component, triple) pin cell."""
    for component, cells in RUST_TOOLCHAIN_PINS.items():
        for triple, (url, sha256) in cells.items():
            native.http_archive(
                name = "{}-{}".format(component, triple),
                urls = [url],
                sha256 = sha256,
                strip_prefix = "{}/{}".format(
                    _archive_root(url),
                    RUST_TOOLCHAIN_COMPONENT_DIRS[component].format(triple = triple),
                ),
                sub_targets = _sub_target_paths(component, triple),
                visibility = ["toolchains//..."],
            )

def rust_toolchain_for_mode(mode):
    """Resolve the [oya_toolchain] rust mode to a toolchain target in toolchains//.

    An ABSENT config yields "hermetic" (the caller's `read_root_config` default), and an
    unrecognised value FAILS here at parse time — an unknown mode must never degrade to the
    ambient system compiler the way a missing `--config-file` silently does.
    """
    if mode == "hermetic":
        return ":rust_hermetic"
    if mode == "system":
        return ":rust_system"
    fail("[oya_toolchain] rust must be \"hermetic\" or \"system\", got {}".format(repr(mode)))

def rust_toolchain_archive_select(component):
    """The pinned archive for the configured host.

    Deliberately NO `DEFAULT` branch and no riscv64/windows branch: an unpinned platform must
    fail at CONFIGURATION time (loud) rather than degrade to an ambient compiler.
    """
    return select({
        "prelude//os:linux": select({
            "prelude//cpu:arm64": "toolchains//rust:{}-aarch64-unknown-linux-gnu".format(component),
            "prelude//cpu:x86_64": "toolchains//rust:{}-x86_64-unknown-linux-gnu".format(component),
        }),
        "prelude//os:macos": select({
            "prelude//cpu:arm64": "toolchains//rust:{}-aarch64-apple-darwin".format(component),
            "prelude//cpu:x86_64": "toolchains//rust:{}-x86_64-apple-darwin".format(component),
        }),
    })

def _projection(dep, path):
    return dep[DefaultInfo].sub_targets[path][DefaultInfo].default_outputs[0]

def _hermetic_rust_toolchain_impl(ctx):
    triple = ctx.attrs.rustc_target_triple

    # ONE composed directory shaped like a rustup toolchain. rustc owns lib/rustlib/<triple>/bin
    # and rust-std owns lib/rustlib/<triple>/lib, so the prefix-overlapping `lib` keys MERGE with
    # no file-level collision.
    composed = ctx.actions.copied_dir("toolchain", {
        "bin/clippy-driver": _projection(ctx.attrs.clippy, "bin/clippy-driver"),
        "bin/rustc": _projection(ctx.attrs.rustc, "bin/rustc"),
        "bin/rustdoc": _projection(ctx.attrs.rustc, "bin/rustdoc"),
        "bin/rustfmt": _projection(ctx.attrs.rustfmt, "bin/rustfmt"),
        "lib": _projection(ctx.attrs.rustc, "lib"),
        "lib/rustlib/{}/lib".format(triple): _projection(
            ctx.attrs.rust_std,
            "lib/rustlib/{}/lib".format(triple),
        ),
    })

    def tool(relative_path):
        return RunInfo(args = cmd_args(composed.project(relative_path), hidden = composed))

    return [
        DefaultInfo(default_output = composed),
        RustToolchainInfo(
            allow_lints = ctx.attrs.allow_lints,
            clippy_driver = tool("bin/clippy-driver"),
            clippy_toml = ctx.attrs.clippy_toml[DefaultInfo].default_outputs[0] if ctx.attrs.clippy_toml else None,
            compiler = tool("bin/rustc"),
            default_edition = ctx.attrs.default_edition,
            panic_runtime = PanicRuntime("unwind"),
            deny_lints = ctx.attrs.deny_lints,
            doctests = ctx.attrs.doctests,
            nightly_features = ctx.attrs.nightly_features,
            report_unused_deps = ctx.attrs.report_unused_deps,
            rustc_binary_flags = ctx.attrs.rustc_binary_flags,
            rustc_flags = ctx.attrs.rustc_flags,
            rustc_target_triple = triple,
            rustc_test_flags = ctx.attrs.rustc_test_flags,
            rustdoc = tool("bin/rustdoc"),
            rustdoc_flags = ctx.attrs.rustdoc_flags,
            sysroot_path = composed,
            warn_lints = ctx.attrs.warn_lints,
        ),
    ]

# Field-for-field the same provider the prelude's `system_rust_toolchain` publishes — same attr
# names, same defaults, `explicit_sysroot_deps` left None and `configuration_hash` left unset —
# plus `sysroot_path`. Every other provider field stays at its prelude default on purpose:
# setting `configuration_hash` alone would change every `-Cmetadata`.
hermetic_rust_toolchain = rule(
    impl = _hermetic_rust_toolchain_impl,
    attrs = {
        "allow_lints": attrs.list(attrs.string(), default = []),
        "clippy": attrs.dep(providers = [DefaultInfo]),
        "clippy_toml": attrs.option(attrs.dep(providers = [DefaultInfo]), default = None),
        "default_edition": attrs.option(attrs.string(), default = None),
        "deny_lints": attrs.list(attrs.string(), default = []),
        "doctests": attrs.bool(default = False),
        "nightly_features": attrs.bool(default = False),
        "report_unused_deps": attrs.bool(default = False),
        "rust_std": attrs.dep(providers = [DefaultInfo]),
        "rustc": attrs.dep(providers = [DefaultInfo]),
        "rustc_binary_flags": attrs.list(attrs.arg(), default = []),
        "rustc_flags": attrs.list(attrs.arg(), default = []),
        "rustc_target_triple": attrs.string(default = _DEFAULT_TRIPLE),
        "rustc_test_flags": attrs.list(attrs.arg(), default = []),
        "rustdoc_flags": attrs.list(attrs.arg(), default = []),
        "rustfmt": attrs.dep(providers = [DefaultInfo]),
        "warn_lints": attrs.list(attrs.string(), default = []),
    },
    is_toolchain_rule = True,
)
