# Copyright (c) Oyatie contributors.
# SPDX-License-Identifier: MIT OR Apache-2.0

"""Hermetic clang/lld C/C++ toolchain for aarch64-linux-gnu.

Pins a downloaded LLVM release (clang + lld) and a single coherent
aarch64-linux-gnu sysroot as buck2 http_archive build inputs.  The
toolchain injects --sysroot / --gcc-install-dir / -fuse-ld=lld to
collapse GCCInstallationDetector to one CRT search root, fixing the
double-CRT symbol clash surfaced by aws-lc-sys on the Linux gate pod.

Migration status:
  CUT-1 (this file): purely additive — target exists alongside the
         default system_cxx_toolchain, which is NOT replaced here.
  CUT-2: Linux link-proof via toolchain override flag (orchestrator-gated).
  CUT-3: Default flip to select() on prelude//os:linux (orchestrator-gated).
  CUT-4: Delete system_cxx_toolchain (orchestrator-gated).
"""

load(
    "@prelude//cxx:cxx_toolchain_types.bzl",
    "BinaryUtilitiesInfo",
    "CCompilerInfo",
    "CxxCompilerInfo",
    "CxxInternalTools",
    "LinkerInfo",
    "LinkerType",
    "ShlibInterfacesMode",
    "StripFlagsInfo",
    "cxx_toolchain_infos",
)
load(
    "@prelude//cxx:headers.bzl",
    "HeaderMode",
)
load(
    "@prelude//linking:link_info.bzl",
    "LinkStyle",
)
load(
    "@prelude//os_lookup:defs.bzl",
    "ScriptLanguage",
)
load(
    "@prelude//utils:cmd_script.bzl",
    "cmd_script",
)

# ---------------------------------------------------------------------------
# HermeticClangDistributionInfo — provider surfaced by :llvm-18-aarch64-linux
# ---------------------------------------------------------------------------

HermeticClangDistributionInfo = provider(
    # @unsorted-dict-items
    fields = {
        "clang": provider_field(typing.Any, default = None),  # path to clang binary
        "clangxx": provider_field(typing.Any, default = None),  # path to clang++ binary
        "lld": provider_field(typing.Any, default = None),  # path to ld.lld binary
        "llvm_ar": provider_field(typing.Any, default = None),  # path to llvm-ar binary
        "llvm_ranlib": provider_field(typing.Any, default = None),  # path to llvm-ranlib binary
        "llvm_nm": provider_field(typing.Any, default = None),  # path to llvm-nm binary
        "llvm_objcopy": provider_field(typing.Any, default = None),  # path to llvm-objcopy binary
        "llvm_strip": provider_field(typing.Any, default = None),  # path to llvm-strip binary
    },
)

def _hermetic_clang_dist_impl(ctx: AnalysisContext) -> list[Provider]:
    """Wraps an http_archive output into HermeticClangDistributionInfo.

    The archive's http_archive uses strip_prefix to remove the top-level
    clang+llvm-<ver>-<triple>/ directory, so the dist output's immediate
    children are bin/, lib/, include/, .... Tools are therefore referenced
    relative to the dist root (bin/clang, ...) with NO prefix re-prepended.
    (A prior bug double-counted strip_prefix AND a prefix attr, yielding
    <out>/clang+llvm-.../bin/clang while the real path is <out>/bin/clang.)
    """
    dist_out = ctx.attrs.dist[DefaultInfo].default_outputs[0]

    def _tool(rel: str) -> cmd_args:
        return cmd_args(
            dist_out,
            format = "{{}}/{}".format(rel),
            hidden = [
                ctx.attrs.dist[DefaultInfo].default_outputs,
                ctx.attrs.dist[DefaultInfo].other_outputs,
            ],
        )

    return [
        ctx.attrs.dist[DefaultInfo],
        HermeticClangDistributionInfo(
            clang = _tool("bin/clang"),
            clangxx = _tool("bin/clang++"),
            lld = _tool("bin/ld.lld"),
            llvm_ar = _tool("bin/llvm-ar"),
            llvm_ranlib = _tool("bin/llvm-ranlib"),
            llvm_nm = _tool("bin/llvm-nm"),
            llvm_objcopy = _tool("bin/llvm-objcopy"),
            llvm_strip = _tool("bin/llvm-strip"),
        ),
    ]

hermetic_clang_distribution = rule(
    impl = _hermetic_clang_dist_impl,
    attrs = {
        "dist": attrs.dep(providers = [DefaultInfo]),
    },
)

# ---------------------------------------------------------------------------
# hermetic_clang_toolchain — the actual CxxToolchain rule
# ---------------------------------------------------------------------------

def _hermetic_clang_toolchain_impl(ctx: AnalysisContext) -> list[Provider]:
    dist = ctx.attrs.distribution[HermeticClangDistributionInfo]
    sysroot_out = ctx.attrs.sysroot[DefaultInfo].default_outputs[0]

    # --sysroot and --gcc-install-dir collapse GCCInstallationDetector to
    # exactly one CRT search root, eliminating the double-CRT link error.
    # hidden: ensure the sysroot dir (+ any other_outputs) materializes into
    # every compile/link action carrying these flags (esp. under remote exec).
    _sysroot_hidden = [
        ctx.attrs.sysroot[DefaultInfo].default_outputs,
        ctx.attrs.sysroot[DefaultInfo].other_outputs,
    ]
    sysroot_arg = cmd_args(sysroot_out, format = "--sysroot={}", hidden = _sysroot_hidden)
    gcc_install_arg = cmd_args(sysroot_out, format = "--gcc-install-dir={}/usr/lib/gcc/aarch64-linux-gnu/{}".format("{}", ctx.attrs.gcc_version), hidden = _sysroot_hidden)

    target_flag = "--target={}".format(ctx.attrs.target)

    # Wrappers so the prelude's cc-shim receives a single executable path.
    clang_cc = cmd_script(
        actions = ctx.actions,
        name = "hermetic_clang_cc",
        cmd = cmd_args(dist.clang),
        language = ScriptLanguage("sh"),
    )
    clang_cxx = cmd_script(
        actions = ctx.actions,
        name = "hermetic_clang_cxx",
        cmd = cmd_args(dist.clangxx),
        language = ScriptLanguage("sh"),
    )
    lld_ar = cmd_script(
        actions = ctx.actions,
        name = "hermetic_lld_ar",
        cmd = cmd_args(dist.llvm_ar),
        language = ScriptLanguage("sh"),
    )
    lld_ranlib = cmd_script(
        actions = ctx.actions,
        name = "hermetic_lld_ranlib",
        cmd = cmd_args(dist.llvm_ranlib),
        language = ScriptLanguage("sh"),
    )

    # Compiler flags: target triple + sysroot + gcc-install-dir
    base_compiler_flags = cmd_args([
        target_flag,
        sysroot_arg,
        gcc_install_arg,
    ])

    # Linker flags: use lld + target triple (no duplicate CRT objects)
    base_linker_flags = cmd_args([
        "-fuse-ld=lld",
        target_flag,
        sysroot_arg,
        gcc_install_arg,
    ])

    return [ctx.attrs.distribution[DefaultInfo]] + cxx_toolchain_infos(
        internal_tools = ctx.attrs._cxx_internal_tools[CxxInternalTools],
        platform_name = "aarch64",
        c_compiler_info = CCompilerInfo(
            compiler = RunInfo(args = cmd_args(clang_cc)),
            compiler_type = "clang",
            compiler_flags = cmd_args(base_compiler_flags, ctx.attrs.c_compiler_flags),
            preprocessor_flags = cmd_args(ctx.attrs.c_preprocessor_flags),
        ),
        cxx_compiler_info = CxxCompilerInfo(
            compiler = RunInfo(args = cmd_args(clang_cxx)),
            compiler_type = "clang",
            compiler_flags = cmd_args(base_compiler_flags, ctx.attrs.cxx_compiler_flags),
            preprocessor_flags = cmd_args(ctx.attrs.cxx_preprocessor_flags),
        ),
        linker_info = LinkerInfo(
            archiver = RunInfo(args = cmd_args(lld_ar)),
            archiver_type = "gnu",
            archiver_supports_argfiles = True,
            archive_objects_locally = False,
            binary_extension = "",
            generate_linker_maps = False,
            link_binaries_locally = False,
            link_libraries_locally = False,
            link_style = LinkStyle(ctx.attrs.link_style),
            link_weight = 1,
            linker = RunInfo(args = cmd_args(clang_cxx)),
            linker_flags = cmd_args(base_linker_flags, ctx.attrs.linker_flags),
            object_file_extension = "o",
            shlib_interfaces = ShlibInterfacesMode("disabled"),
            shared_dep_runtime_ld_flags = ctx.attrs.shared_dep_runtime_ld_flags,
            shared_library_name_default_prefix = "lib",
            shared_library_name_format = "{}.so",
            shared_library_versioned_name_format = "{}.so.{}",
            static_dep_runtime_ld_flags = ctx.attrs.static_dep_runtime_ld_flags,
            static_library_extension = "a",
            static_pic_dep_runtime_ld_flags = ctx.attrs.static_pic_dep_runtime_ld_flags,
            independent_shlib_interface_linker_flags = ctx.attrs.shared_library_interface_flags,
            type = LinkerType("gnu"),
            use_archiver_flags = True,
            is_pdb_generated = False,
        ),
        binary_utilities_info = BinaryUtilitiesInfo(
            bolt_msdk = None,
            dwp = None,
            nm = RunInfo(args = cmd_args(dist.llvm_nm)),
            objcopy = RunInfo(args = cmd_args(dist.llvm_objcopy)),
            ranlib = RunInfo(args = cmd_args(lld_ranlib)),
            strip = RunInfo(args = cmd_args(dist.llvm_strip)),
        ),
        header_mode = HeaderMode("symlink_tree_only"),
        strip_flags_info = StripFlagsInfo(
            strip_debug_flags = ctx.attrs.strip_debug_flags,
            strip_non_global_flags = ctx.attrs.strip_non_global_flags,
            strip_all_flags = ctx.attrs.strip_all_flags,
        ),
    )

hermetic_clang_toolchain = rule(
    impl = _hermetic_clang_toolchain_impl,
    attrs = {
        # Required deps
        "distribution": attrs.exec_dep(providers = [HermeticClangDistributionInfo]),
        "sysroot": attrs.exec_dep(providers = [DefaultInfo]),
        # Cross-compilation target triple — default is the Linux gate target.
        "target": attrs.string(default = "aarch64-unknown-linux-gnu"),
        # GCC version suffix inside the sysroot's usr/lib/gcc/aarch64-linux-gnu/<ver>/
        "gcc_version": attrs.string(default = "14"),
        # Passthrough flags
        "c_compiler_flags": attrs.list(attrs.arg(), default = []),
        "c_preprocessor_flags": attrs.list(attrs.arg(), default = []),
        "cxx_compiler_flags": attrs.list(attrs.arg(), default = []),
        "cxx_preprocessor_flags": attrs.list(attrs.arg(), default = []),
        "link_style": attrs.enum(
            LinkStyle.values(),
            default = "static",
            doc = "Default link_style for rules that consume this toolchain.",
        ),
        "linker_flags": attrs.list(attrs.arg(), default = []),
        "shared_dep_runtime_ld_flags": attrs.list(attrs.arg(), default = []),
        "shared_library_interface_flags": attrs.list(attrs.string(), default = []),
        "static_dep_runtime_ld_flags": attrs.list(attrs.arg(), default = []),
        "static_pic_dep_runtime_ld_flags": attrs.list(attrs.arg(), default = []),
        "strip_all_flags": attrs.option(attrs.list(attrs.arg()), default = None),
        "strip_debug_flags": attrs.option(attrs.list(attrs.arg()), default = None),
        "strip_non_global_flags": attrs.option(attrs.list(attrs.arg()), default = None),
        "_cxx_internal_tools": attrs.default_only(attrs.dep(
            providers = [CxxInternalTools],
            default = "prelude//cxx/tools:internal_tools",
        )),
    },
    is_toolchain_rule = True,
)
