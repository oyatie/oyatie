# Buck2 Native-FFI Investigation Notes

**Branch**: chore/buck2-native-ffi  
**Date**: 2026-05-29  
**Host**: macOS aarch64 (arm64-apple-darwin)

## Crate dependency chains

```
ring v0.17.14
  ← rustls v0.23 ← hyper-rustls, reqwest, kube-client, ...

aws-lc-rs v1.17.0
  ← rustls v0.23 ← (same chain)
  ← identity-workload-oidc-adapter

openssl-sys v0.9.116
  ← openssl v0.10.80 ← webauthn-rs ← identity
```

## Pre-existing bug: getrandom-0.2 bogus buildscript

`getrandom-0.2.17` has `build = false` in its Cargo.toml — no build.rs at all.
But `third-party/fixups/getrandom/fixups.toml` has `run = true`, causing reindeer to
generate a `getrandom-0.2-build-script-build` rule pointing at
`getrandom-0.2.17.crate/build.rs` which does not exist.

This causes `BUILD FAILED` whenever ring (or anything) pulls getrandom-0.2 because buck2
tries to compile the non-existent build.rs.

**Fix required**: Replace getrandom fixup `run = true` with `run = false` and manually
patch the generated BUCK to remove the bogus buildscript_run references from the
getrandom-0.2 library rule.

## ring-0.17 analysis

- `links = "ring_core_0_17_14_"`
- `build = "build.rs"` — real build script using `cc` crate
- Build strategy: when `.git` is absent (packaged crate), uses `pregenerated/` asm
  instead of invoking perl to regenerate. For macOS aarch64, the pregenerated files
  are `pregenerated/*-ios64.S` (ios64 perlasm format = Apple aarch64 ABI).
- C sources: ~12 platform-independent .c files + aarch64-specific .c files
  (p256-nistz.c, curve25519_64_adx.c) + asm from pregenerated/
- Build deps: `cc` crate (no cmake, no bindgen)

**Fixup strategy**: `buildscript.run = true` in fixups.toml. The build script will use
the system Clang (available via system_cxx_toolchain) to compile C + asm, linking
`libring_core_0_17_14_.a` into OUT_DIR and emitting `cargo:rustc-link-lib=static=...`.

**Challenge**: The buildscript_run approach means buck2 runs ring's build.rs in a
sandboxed environment. The build.rs uses the `cc` crate which invokes clang. On macOS
Seatbelt this works as long as clang is in PATH. The buildscript needs `CARGO_MANIFEST_DIR`
pointing into the extracted crate dir (buck2 sets this via http_archive).

## aws-lc-sys-0.41 analysis

- `links = "aws_lc_0_41_0"`
- `build = "builder/main.rs"` — complex build script
- Features in BUCK: `["prebuilt-nasm"]`
- **Key**: pregenerated Rust bindings exist at `src/aarch64_apple_darwin_crypto.rs` —
  no bindgen needed at all for this host.
- **Builder selection**: `get_builder()` tries CcBuilder first (not fips, no cmake env
  var, no sanitizer, bindgen not required). CcBuilder compiles ~800 C files using `cc`
  crate directly. No cmake required on macOS aarch64 with prebuilt-nasm feature.
- CcBuilder path for aarch64 apple: uses `cc_builder/apple_aarch64.rs` source list.

**Fixup strategy**: `buildscript.run = true`. The CcBuilder path avoids cmake, just
needs clang. This is the same model as ring — cc-driven native compilation via buildscript.

**Constraint**: `default-features = true` (founder constraint). The BUCK already has
`features = ["prebuilt-nasm"]` which is fine (subset of default features).

**Concern**: ~800 C files takes significant time. Buck2 will run this as a single
buildscript_run action. Action timeout may be a concern for CI.

## aws-lc-rs-1.17 analysis

- `links = "aws_lc_rs_1_17_0_sys"`
- Has its own `build.rs` that delegates linking info from aws-lc-sys via DEP_ env vars
- Fixup: `buildscript.run = true`

## openssl-sys-0.9 analysis

- `links = "openssl"`
- `build = "build/main.rs"` — looks for system OpenSSL via pkg-config or OPENSSL_DIR
- System OpenSSL@3 is at `/opt/homebrew/opt/openssl@3` (libcrypto.a + libssl.a present)
- Used only by webauthn-rs chain → identity
- **Current state**: BUCK has no buildscript — pure Rust check passes but link will fail
  because the `links = "openssl"` declaration means Cargo expects the build script to
  emit `cargo:rustc-link-lib=ssl` etc.

**Fixup strategy**: `buildscript.run = true` + inject env vars:
```toml
[buildscript]
run = true
[buildscript.env]
OPENSSL_DIR = "/opt/homebrew/opt/openssl@3"
OPENSSL_STATIC = "1"
```

## reindeer fixup schema summary

All existing working fixups use `[buildscript] run = true`. The reindeer-generated BUCK
will add `buildscript_run(...)` rule + wire `env.OUT_DIR` and `rustc_flags` into the
library rule. For env injection, use `[buildscript.env]` section.

## Action plan

1. Fix getrandom-0.2 fixup (`run = false`) + patch BUCK to remove bogus buildscript rules
2. Add ring fixup (`run = true`) → re-buckify → verify `buck2 build third-party//:ring-0.17`
3. Add aws-lc-sys fixup (`run = true`) → re-buckify → verify
4. Add aws-lc-rs fixup (`run = true`) → re-buckify → verify
5. Add openssl-sys fixup (`run = true` + OPENSSL_DIR env) → re-buckify → verify
6. Prove with a first-party target that uses TLS (e.g. identity or a minimal probe binary)

## Estimated remaining work for full `buck2 build //...`

- First-party BUCK generation: mechanical via generator, ~716 crates reported
- Other hard transitive FFI deps:
  - `k8s-openapi`: pure Rust, no FFI — tractable
  - `wasm-bindgen`: has wasm-specific fixup already in place
  - `prost`/`tonic` codegen: prost-build uses protoc — needs protoc in PATH or
    protoc-bin-vendored. Already blessed dep. Fixup: buildscript.run=true + PATH injection
  - `zstd-sys`: cc-based, same pattern as ring
  - `libz-sys`: links system zlib, same pattern as openssl-sys
  - `brotli-sys`: cc-based
- Estimate: ~3-5 more cc/links crates need fixups after the main 4 are done.
  Full //... clean build estimate: 1-2 days of fixup work after ring+aws-lc-sys+openssl are green.
