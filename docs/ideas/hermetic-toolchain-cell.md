# Hermetic toolchain cell — buck2-native pinned clang + sysroot

Status: idea / design (first-cut buildable). Owner lane: feat/oya-ci-controller.
Date: 2026-05-30. Pairs with the (to-be-written) ADR on hermetic C++ toolchain.

## Problem

`third-party//:aws-lc-sys-0.41-build-script-main-run` PANICS on the aarch64-linux
CI gate at `cc_builder.rs:748`. Its compiler feature-test (`memcmp_check`) compiles
AND LINKS a tiny executable; the link fails:

```
ld.lld: error: duplicate symbol: _start / _init / _fini / __dso_handle /
  _IO_stdin_used / __data_start / __TMC_END__
  (from /lib/aarch64-linux-gnu/Scrt1.o, crti.o AND
        /usr/lib/gcc/aarch64-linux-gnu/14/crtbeginS.o/crtendS.o)
```

The FULL C-runtime startup set is double-linked: both the libc/glibc half
(`Scrt1.o`/`crti.o`) and the gcc half (`crtbeginS.o`/`crtendS.o`). Verified root
cause (research-agent-2, against the vendored crate + both prelude shims): the
prelude build-script shims (`__cc_shim.sh`/`__ld_shim.sh`) add NO CRT objects —
the duplication is **driver-side**. The host `/usr/bin/clang`, invoked with
`--target=aarch64-unknown-linux-gnu` but with NO `--sysroot` / `--gcc-toolchain` /
`--gcc-install-dir`, runs `GCCInstallationDetector` and on a Debian-multiarch host
resolves TWO coherent startup-object search roots, emitting BOTH onto one `ld.lld`
line (canonical case: LLVM #61355; mechanism: maskray.me compiler-driver).

The repo's `toolchains//:cxx` is config-CLEAN (`system_cxx_toolchain`, bare
`/usr/bin/clang`, `/usr/bin/ar`, no custom flags — `toolchains/BUCK:22-29`). So this
is NOT our flags; it is an unanchored host clang. The controller is merely the FIRST
aws-lc-sys-dependent BINARY built cold on the Linux gate, so it surfaced a LATENT,
SHARED blocker affecting ALL aws-lc-sys / ring / openssl-sys Linux builds.

Verified mechanism chain (all read from the bundled prelude in this repo):
- `rust/cargo_buildscript.bzl:313-347` splices `c_compiler_info.compiler` +
  `c_compiler_info.compiler_flags` + `["--target={triple}"]` into `$CC` (the
  `__cc_shim.sh`), and `linker_info.archiver` into `$AR`. So whatever flags the cxx
  toolchain puts in `compiler_flags` flow straight into aws-lc-sys's feature-test cc.
- `toolchains/rust.bzl:11` `_DEFAULT_TRIPLE` resolves to `aarch64-unknown-linux-gnu`
  on linux-arm64 (the repo's `system_rust_toolchain(name="rust")` does not override
  it), so the shim DOES emit `--target=aarch64-unknown-linux-gnu` — exactly the
  triple in the panic.

`prebuilt-nasm` is a RED HERRING: `use_prebuilt_nasm()` (aws-lc-sys `main.rs:833`)
is hard-gated to `windows && x86_64`; on aarch64-linux it is a no-op. The failing
path is CcBuilder→`memcmp_check`, which never references nasm. Dropping the feature
is optional hygiene, NOT the fix.

## Recommended direction

Build a buck2-native HERMETIC TOOLCHAIN CELL: pin a downloaded clang+lld
distribution and a single coherent aarch64-linux-gnu sysroot as `http_archive`
build inputs OWNED by the `toolchains//` cell, and expose them through a NEW
`hermetic_clang_toolchain` rule that emits the standard `cxx_toolchain_infos`
providers — REPLACING the host `/usr/bin/clang` only on aarch64-linux. This is the
Bazel `hermetic_cc_toolchain` / Buck2-fbcode in-repo-toolchain hyperscaler pattern;
the prelude's own Zig toolchain (`toolchains/cxx/zig/defs.bzl`) is the proven shape
to mirror (it is the ONLY prelude-native hermetic example).

### How it fixes the double-CRT (flag prescription tied to the error)

A single bundled sysroot supplies exactly ONE coherent CRT set, and the anchoring
flags collapse `GCCInstallationDetector` to exactly one search root (no host-GCC
discovery, no second `-B` prefix). The hermetic toolchain's `compiler_flags`
(C and C++) MUST carry, for aarch64-linux:

- `--target=aarch64-unknown-linux-gnu`  (matches the shim's `--target`; redundant
  but explicit so the toolchain is self-describing)
- `--sysroot=<sysroot-cell-out>`  → anchors the libc half (`Scrt1.o`/`crti.o`/`crtn.o`)
  to ONE root and redirects where the detector looks for GCC. Kills the
  `Scrt1.o`/`crti.o`/`__dso_handle`/`_IO_stdin_used`/`__data_start` duplicates.
- `--gcc-install-dir=<sysroot>/usr/lib/gcc/aarch64-linux-gnu/<ver>`  → PREFERRED over
  `--gcc-toolchain` (highest-priority, most-deterministic selector per maskray);
  pins the exact `crtbeginS.o`/`crtendS.o`/`libgcc` dir and short-circuits
  multi-install detection. Kills the `crtbeginS.o`/`__TMC_END__` duplicates.
  (If the bundled sysroot has no gcc tree, use `-resource-dir` + clang's own
  `crtbegin`; see Open Questions.)

`linker_info.linker_flags` MUST carry: `-fuse-ld=lld` (use the bundled `ld.lld`,
never host `ld`) and `--target=aarch64-unknown-linux-gnu`.

Do NOT use `-B` to anchor GCC (pre-clang-13 detection footgun → can RE-introduce a
second prefix). Strongly-recommended extra hermetic posture (defense-in-depth, can
be added once the base fix is proven): `-rtlib=compiler-rt`, `-unwindlib=libunwind`,
`--stdlib=libc++`, `-resource-dir=<clang>/lib/clang/<ver>`.

Because the cxx toolchain feeds `$CC`/`$CXX`/`$AR` to EVERY rust build script via
`cargo_buildscript.bzl`, no per-crate fixup changes are needed — the fix is at the
toolchain layer, per [[single-bootstrap-just-works]].

## Key assumptions

1. The Linux gate is a NATIVE aarch64→aarch64 build (HOST==TARGET), so `memcmp_check`
   not only links but RUNS the linked binary (`cc_builder.rs:764`). A native-aarch64
   sysroot runs fine on the native gate kernel. (Flag for executor: do NOT pick a
   sysroot whose libc the gate kernel can't run.)
2. `http_archive` is a NATIVE prelude global (`rules_impl.bzl:166`, declared in
   `decls/core_rules.bzl:902`) — callable in any BUCK file with NO load, with a
   default `exec_deps`. The repo already uses it ~hundreds of times in
   `third-party/BUCK` (e.g. line 9). We reuse it directly; we do NOT re-declare the
   zig-inline `_http_archive`.
3. `cquery` does NOT trigger the download AND does NOT analyze the archive-dep
   impls — so it passes even with a malformed sha. But `buck2 audit providers`
   (full ANALYSIS) DOES run http_archive's impl, which FORMAT-checks sha256 (must
   be 64 hex) at analysis time, NOT only at download. So CUT-1's real acceptance
   bar is `audit providers` clean (the rule impl actually runs), and the sysroot
   needs a 64-hex (placeholder-but-valid) sha to analyze. VERIFIED 2026-05-31:
   `audit providers cxx_hermetic_linux` constructs a full CxxToolchainInfo on
   darwin with a 64-hex-zero sysroot placeholder. The archive only DOWNLOADS at
   build/execution time. (zig's `download_zig_distribution` declares per-os
   archives the same way.)
4. LLVM 18.1.8 ships an official aarch64-linux asset:
   `clang+llvm-18.1.8-aarch64-linux-gnu.tar.xz` (verified via GitHub releases API),
   `strip_prefix = clang+llvm-18.1.8-aarch64-linux-gnu`. A matching
   `clang+llvm-18.1.8-arm64-apple-macos11.tar.xz` exists for an eventual darwin
   hermetic story (NOT in cut 1).
5. The new `hermetic_clang_toolchain` rule needs `is_toolchain_rule = True` and
   consumes the archive via `attrs.exec_dep(...)` (NOT `toolchain_dep`), mirroring
   zig's `"distribution": attrs.exec_dep(...)`.
6. Sysroot packaging: there is NO off-the-shelf prelude sysroot rule. First-cut
   sysroot = a SHA256-pinned `.tar.xz` of the CI pod's
   `/lib/aarch64-linux-gnu` + `/usr/lib/aarch64-linux-gnu` + `/usr/include` +
   `/usr/lib/gcc/aarch64-linux-gnu/<ver>`, uploaded to an internal artifact store or
   a repo GitHub release, consumed via `http_archive`. (Alternative: a
   debootstrap-trixie sysroot. Decision deferred — see Open Questions.)

## MVP scope (first cut — what the executor does NOW)

Goal of cut 1: ADD the hermetic toolchain target ALONGSIDE `system_cxx_toolchain`,
prove it CONFIGURES on darwin, change NOTHING about the default toolchain. Minimal,
additive, reversible. NO download is exercised on darwin; the Linux link-proof is a
later, orchestrator-driven step.

Files (all NEW or additive — no existing target is mutated):

1. NEW `toolchains/cxx/clang_hermetic/defs.bzl` — the `hermetic_clang_toolchain`
   rule, mirroring `prelude//toolchains/cxx/zig/defs.bzl`. Shape:
   - Loads from `@prelude//cxx:cxx_toolchain_types.bzl` (`BinaryUtilitiesInfo`,
     `CCompilerInfo`, `CxxCompilerInfo`, `CxxInternalTools`, `LinkerInfo`,
     `LinkerType`, `ShlibInterfacesMode`, `cxx_toolchain_infos`),
     `@prelude//cxx:headers.bzl` (`HeaderMode`),
     `@prelude//linking:link_info.bzl` (`LinkStyle`),
     `@prelude//os_lookup:defs.bzl` (`ScriptLanguage`),
     `@prelude//utils:cmd_script.bzl` (`cmd_script`).
   - `_impl(ctx)`:
       clang_dist = ctx.attrs.clang_distribution[DefaultInfo].default_outputs[0]
       sysroot    = ctx.attrs.sysroot[DefaultInfo].default_outputs[0]
       cc  = cmd_script(ctx.actions,"clang",  cmd_args(clang_dist,format="{}/bin/clang"),  ScriptLanguage("sh"))
       cxx = cmd_script(ctx.actions,"clangpp",cmd_args(clang_dist,format="{}/bin/clang++"),ScriptLanguage("sh"))
       ld  = cmd_script(ctx.actions,"lld",    cmd_args(clang_dist,format="{}/bin/ld.lld"), ScriptLanguage("sh"))
       ar  = cmd_args(clang_dist, format="{}/bin/llvm-ar")
       sysroot_flag = cmd_args(sysroot, format="--sysroot={}")
       target = ["--target=aarch64-unknown-linux-gnu"]
       compiler_flags = cmd_args(target, sysroot_flag, ctx.attrs.gcc_install_dir_flag, ctx.attrs.extra_compiler_flags)
       return cxx_toolchain_infos(
           internal_tools = ctx.attrs._cxx_internal_tools[CxxInternalTools],
           platform_name  = "aarch64",
           c_compiler_info   = CCompilerInfo(compiler=RunInfo(args=cmd_args(cc)),  compiler_type="clang", compiler_flags=compiler_flags, preprocessor_flags=cmd_args()),
           cxx_compiler_info = CxxCompilerInfo(compiler=RunInfo(args=cmd_args(cxx)),compiler_type="clang", compiler_flags=compiler_flags, preprocessor_flags=cmd_args()),
           linker_info = LinkerInfo(
               linker=RunInfo(args=cmd_args(ld)),
               linker_flags=cmd_args(["-fuse-ld=lld"], target, ctx.attrs.extra_linker_flags),
               archiver=RunInfo(args=ar), archiver_type="gnu", type=LinkerType("gnu"),
               link_style=LinkStyle("static_pic"), static_library_extension="a",
               object_file_extension="o", shared_library_name_default_prefix="lib",
               shared_library_name_format="{}.so", shared_library_versioned_name_format="{}.so.{}",
               shlib_interfaces=ShlibInterfacesMode("disabled"), use_archiver_flags=True,
               link_weight=1, binary_extension="", generate_linker_maps=False,
               link_binaries_locally=False, link_libraries_locally=False,
               archive_objects_locally=False, archiver_supports_argfiles=True,
               independent_shlib_interface_linker_flags=[],
               shared_dep_runtime_ld_flags=[], static_dep_runtime_ld_flags=[],
               static_pic_dep_runtime_ld_flags=[], is_pdb_generated=False),
           binary_utilities_info = BinaryUtilitiesInfo(
               nm=RunInfo(args=cmd_args(clang_dist,format="{}/bin/llvm-nm")),
               objcopy=RunInfo(args=cmd_args(clang_dist,format="{}/bin/llvm-objcopy")),
               ranlib=RunInfo(args=cmd_args(ar,"s")),
               strip=RunInfo(args=cmd_args(clang_dist,format="{}/bin/llvm-strip")),
               dwp=None, bolt_msdk=None),
           header_mode = HeaderMode("symlink_tree_only"))
     NOTE: cross-check every LinkerInfo / BinaryUtilitiesInfo field name + requiredness
     against `prelude//cxx/cxx_toolchain_types.bzl` and `zig/defs.bzl:353-414` at build
     time — the exact required set is prelude-version-specific; the zig impl in THIS
     repo's bundled prelude is the authoritative template to copy field-for-field.
   - rule attrs:
       clang_distribution: attrs.exec_dep(providers=[DefaultInfo]),
       sysroot:            attrs.exec_dep(providers=[DefaultInfo]),
       gcc_install_dir_flag: attrs.list(attrs.arg(), default=[]),  # filled once sysroot gcc ver known
       extra_compiler_flags: attrs.list(attrs.arg(), default=[]),
       extra_linker_flags:   attrs.list(attrs.arg(), default=[]),
       _cxx_internal_tools: attrs.default_only(attrs.dep(providers=[CxxInternalTools],
           default="prelude//cxx/tools:internal_tools")),
     rule(..., is_toolchain_rule=True)

2. NEW `toolchains/cxx/clang_hermetic/BUCK` — declares the two `http_archive`
   inputs + the hermetic target (additive; PUBLIC visibility):
       http_archive(
           name = "llvm-18-aarch64-linux",
           urls = ["https://github.com/llvm/llvm-project/releases/download/llvmorg-18.1.8/clang+llvm-18.1.8-aarch64-linux-gnu.tar.xz"],
           sha256 = "<FILL: sha256 of the asset>",
           strip_prefix = "clang+llvm-18.1.8-aarch64-linux-gnu",
           visibility = ["PUBLIC"],
       )
       http_archive(
           name = "aarch64-linux-sysroot",
           urls = ["<FILL: internal artifact-store or repo-release URL>"],
           sha256 = "<FILL>",
           strip_prefix = "<FILL if the tar has a top dir>",
           visibility = ["PUBLIC"],
       )
       load("//toolchains/cxx/clang_hermetic:defs.bzl", "hermetic_clang_toolchain")
       hermetic_clang_toolchain(
           name = "cxx_hermetic_linux",
           clang_distribution = ":llvm-18-aarch64-linux",
           sysroot = ":aarch64-linux-sysroot",
           # gcc_install_dir_flag filled after sysroot gcc ver is known, e.g.
           # ["--gcc-install-dir=.../usr/lib/gcc/aarch64-linux-gnu/14"]
           visibility = ["PUBLIC"],
       )
   For the darwin-NOW configure check, the two `http_archive` sha256/urls may be
   placeholders ONLY IF cquery does not resolve them; if cquery complains, use a
   real LLVM sha256 (downloadable) and a throwaway-but-real sysroot url, since
   cquery still must parse the targets. Prefer: fill the real LLVM sha256 now (it is
   public), leave the sysroot as a real-but-internal url to be finalized at proof time.

3. NO CHANGE to `toolchains/BUCK` `:cxx` / `:cxx_no_default_deps` in cut 1. The
   default toolchain stays the host `system_cxx_toolchain`. (The flip is step 3,
   orchestrator-gated.) Keeping the new target in its OWN package
   (`toolchains/cxx/clang_hermetic/`) means `toolchains/BUCK` is untouched →
   maximally reversible (delete one directory to revert).

4. NO CHANGE to `third-party/fixups/*` and NO CHANGE to `.buckconfig` in cut 1. The
   existing cells/parser spec already cover `toolchains//...`. (External cells are
   NOT needed — `http_archive` targets live inside the existing `toolchains` cell.)

Executor's NOW acceptance check (darwin, must stay green):
- `buck2 cquery 'toolchains//cxx/clang_hermetic:cxx_hermetic_linux'` SUCCEEDS
  (configures; no download triggered).
- `buck2 cquery 'toolchains//:cxx'` STILL succeeds (unchanged).
- `buck2 build third-party//:aws-lc-sys-0.41` on darwin STILL BUILD SUCCEEDED
  (proves the default path is untouched).
- Working tree changes confined to `toolchains/cxx/clang_hermetic/` (+ this doc).

## Incremental migration (NON-NEGOTIABLE: never break green-darwin or the live gate)

- STEP 1 (DONE + VERIFIED 2026-05-31, branch feat/hermetic-toolchain-cell): create
  the package in MVP scope above. Additive only; default unchanged. Verified via
  `buck2 audit providers` (full analysis, clean CxxToolchainInfo) on darwin, NOT
  just cquery. Per [[canonical-monorepo-pattern]] structural migrations are
  exclusive/incremental — a big-bang flip broke dev before.
  LANDING NOTE: CUT-1 does NOT land alone. The affected-gate builds the changed
  package's targets, which include the two `http_archive` targets; the sysroot
  archive (placeholder sha) FAILS its download on the gate. So CUT-1 lands TOGETHER
  WITH CUT-2 (once the real sysroot tarball + both real sha256s exist and the Linux
  proof passes). Until then the branch holds CUT-1.
- STEP 2a (CHEAP hypothesis-validation FIRST, on a rust-ci Linux pod — do BEFORE
  provisioning/hosting any sysroot): the whole approach rests on the open-question
  HIGH risk "do --sysroot + --gcc-install-dir + -fuse-ld=lld actually collapse to
  ONE CRT?". Validate cheaply with the POD's OWN clang+sysroot (no download): run
  the exact failing aws-lc-sys feature-test compile+link
  (`clang --target=aarch64-unknown-linux-gnu memcmp_invalid_stripped_check.c -o x`)
  (a) BARE → reproduce the duplicate _start/_init, then (b) WITH
  `--sysroot=/ --gcc-install-dir=/usr/lib/gcc/aarch64-linux-gnu/14 -fuse-ld=lld` →
  confirm it links clean and `clang -v`/`-print-search-dirs` shows exactly ONE
  Scrt1.o/crtbeginS.o root. If (b) fails, rethink flags (--gcc-toolchain,
  -nostartfiles control) BEFORE investing in the hosted sysroot.
- STEP 2b (after 2a confirms): provision the real trixie/gcc-14 sysroot tarball,
  host it (self-hosted artifact store / a Forgejo release), fill both sha256s, then
  build aws-lc-sys against the hermetic toolchain via an explicit override, e.g.:
    `buck2 build third-party//:aws-lc-sys-0.41-build-script-main-run \
       --target-platforms <linux-aarch64> \
       -c toolchain.cxx=toolchains//cxx/clang_hermetic:cxx_hermetic_linux`
  (or a temporary `select()` on `prelude//os:linux` in a throwaway branch). SUCCESS
  = the `duplicate symbol` link error is GONE and the feature-test binary links+runs.
- STEP 3 (orchestrator, after STEP 2 proof): flip the DEFAULT cxx toolchain on
  Linux only — make `toolchains//:cxx` a `select({ "prelude//os:linux":
  ":cxx_hermetic_linux" (re-exported), DEFAULT: <host system_cxx_toolchain> })`, or
  set the equivalent `[toolchain]`/build override. darwin keeps the host toolchain.
  Land via the normal dev PR + gate (the gate now runs on the hermetic toolchain).
- STEP 4 (cleanup, later): once Linux is fully hermetic and darwin is migrated (or
  a darwin hermetic toolchain is added with the macos11 LLVM asset), delete the host
  `system_cxx_toolchain` and any now-unneeded crypto fixup env / `prebuilt-nasm`
  hygiene. Each as its OWN exclusive migration IP, never bundled into a drain.

## Not doing (first cut)

- NOT vendoring all of LLVM source — a pinned RELEASE tarball
  (`clang+llvm-18.1.8-aarch64-linux-gnu.tar.xz`) suffices. Bespoke-LLVM is a
  far-future destination, not this fix.
- NOT Nix / external package managers. `http_archive` + SHA256 pin is the
  buck2-native, self-host, zero-drift mechanism.
- NOT hermeticizing the RUST toolchain in cut 1. cxx is the blocker; rust stays on
  the rustup shim. (Hermetic rust is a separate later IP.)
- NOT flipping the default toolchain in cut 1 (steps 3-4 are orchestrator-gated on
  the Linux link-proof).
- NOT a darwin hermetic toolchain in cut 1. darwin stays on host `/usr/bin/clang`
  (green today). The macos11 LLVM asset is noted for later.
- NOT touching `third-party/fixups/*`, `.buckconfig`, or `external_cells`.
- NOT dropping `prebuilt-nasm` (red herring; optional later hygiene).

## Open questions

1. Sysroot source: package the CI pod's sysroot (exactly matches the gate, lowest
   risk) vs a debootstrap-trixie sysroot (cleaner provenance, reproducible). Need
   the gcc version under `/usr/lib/gcc/aarch64-linux-gnu/` (the panic shows `14`) to
   fill `--gcc-install-dir`.
2. Does the LLVM 18.1.8 prebuilt bundle `compiler-rt`/`libc++`/`libunwind` such that
   `-rtlib=compiler-rt -unwindlib=libunwind --stdlib=libc++` works WITHOUT the gcc
   tree at all? If yes, the sysroot can omit the gcc CRT entirely → even cleaner
   single-CRT (clang's own `crtbegin`), and `--gcc-install-dir` becomes unnecessary.
   Decide at STEP 2 by trying the gcc-free posture first.
3. Exact `LinkerInfo`/`BinaryUtilitiesInfo` required-field set for THIS bundled
   prelude version — copy field-for-field from `zig/defs.bzl:353-414` at build time;
   do not trust the abstract list above verbatim.
4. Artifact hosting: internal artifact store vs a GitHub release on the repo's own
   toolchain-assets repo (must be self-hostable per hyperscaler-lens). LLVM upstream
   URL is fine for the clang dist; the sysroot needs a home we control.
5. Does the gate run with RE/NativeLink or local exec? Affects whether
   `link_binaries_locally`/`archive_objects_locally` should be True; first cut uses
   False (matches zig default) — revisit if RE lands.

## References (verified against this repo's bundled prelude)

- `toolchains/BUCK:22-43` — current host `system_cxx_toolchain(:cxx, :cxx_no_default_deps)`.
- `buck-out/v2/external_cells/bundled/prelude/toolchains/cxx/zig/defs.bzl` —
  canonical hermetic pattern: `cmd_script` + `exec_dep` distribution +
  `cxx_toolchain_infos` (impl `:325-461`), `is_toolchain_rule=True` (`:461`).
- `.../prelude/rust/cargo_buildscript.bzl:313-347` — `$CC`/`$CXX`/`$AR` injection:
  `c_compiler_info.compiler_flags` (where `--sysroot` lands) is spliced into the cc-shim.
- `.../prelude/toolchains/rust.bzl:11` — `_DEFAULT_TRIPLE` → `aarch64-unknown-linux-gnu`
  on linux-arm64 (drives the shim's `--target`).
- `.../prelude/cxx/cxx_toolchain_types.bzl:320` — `cxx_toolchain_infos` signature.
- `.../prelude/rules_impl.bzl:166` + `decls/core_rules.bzl:902` — `http_archive`
  native global rule (no load needed).
- `third-party/BUCK:9` — repo's existing `http_archive` convention.
- `third-party/BUCK:1083,1150-1161` — aws-lc-sys-0.41 target + `prebuilt-nasm` feature.
- LLVM release asset `clang+llvm-18.1.8-aarch64-linux-gnu.tar.xz` (GitHub releases
  API, llvmorg-18.1.8) + darwin counterpart `clang+llvm-18.1.8-arm64-apple-macos11.tar.xz`.
- maskray.me/blog/2021-03-28-compiler-driver-and-cross-compilation;
  LLVM issue #61355 (duplicate `__dso_handle` from two CRT providers).
