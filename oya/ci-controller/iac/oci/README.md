# oya-ci-controller OCI image (buck2-native)

Buck2-native OCI image for `oya-ci-controller`.  Retires the BuildKit/Dockerfile
path for this service per ADR-0514.  ADDITIVE: the gateway Dockerfile is untouched.

## Targets

| Target | Description |
|---|---|
| `root//oya/ci-controller/iac/oci:distroless-base` | http_archive: fetches base OCI layout tarball |
| `root//oya/ci-controller/iac/oci:controller-layer` | genrule: packages the buck2-built binary as a reproducible tar.gz layer |
| `root//oya/ci-controller/iac/oci:controller-oci` | oci_image: assembles the final OCI Image Layout directory |

## Darwin / Linux split

- **Darwin (aarch64):** `buck2 audit providers` and `buck2 cquery` resolve cleanly.
  The assembled layout is inspectable (valid OCI Image Layout directory).
  The binary inside is Mach-O (host arch) — NOT runnable on Linux.
  Push is intentionally blocked on darwin because the assembled binary is a
  host-arch inspection artifact.
- **Linux CI (aarch64, Talos node):** the binary is `aarch64-linux` ELF — correct
  for deployment.  Publish with `//tools/oci:oya-oci-push` after assembly.

## Base image

`gcr.io/distroless/static-debian12:nonroot` — the `static` variant carries no
libc or libgcc, which is correct for a fully-static musl binary.  The controller
is compiled with `aarch64-unknown-linux-musl` + `link-self-contained=yes`
(`+crt-static` is the musl-target default), so it has zero shared-library
dependencies.  Founder decision 2026-05-31; ADR-0146 canonical.

## Building the musl image

The OCI target must be built with the musl platform so the transitive
`rust_binary` dep resolves to the musl toolchain:

```sh
buck2 build root//oya/ci-controller/iac/oci:controller-oci \
  --target-platforms //platforms:linux-aarch64-musl
```

The `--target-platforms` flag propagates to all transitive deps; the
`//oya/ci-controller/crates/oya-ci-controller-app:oya-ci-controller`
`rust_binary` is automatically compiled with
`aarch64-unknown-linux-musl` + `-Clink-self-contained=yes -Clinker=rust-lld`.

## Base digest verification and bump procedure

The `sha256:` in `BUCK` is the immutable linux/arm64 manifest digest passed to
`//tools/oci:oya-oci-pull-base`.  The Rust puller verifies
`sha256(manifest_bytes) == DIGEST`, so a stale or wrong digest fails the build
deterministically.

To bump to a new CVE-patched rebuild:

1. Resolve the linux/arm64 child manifest digest from trusted upstream release
   evidence.
2. Update the `DIGEST` argument in the `:distroless-base` genrule.
3. Verify with `buck2 build root//oya/ci-controller/iac/oci:controller-oci
   --target-platforms //platforms:linux-aarch64-musl` in the Linux CI lane.

The puller writes an OCI Image Layout (`oci-layout`, `index.json`, and
`blobs/sha256/`).  The assembler (`oya-oci-assemble`) requires that layout and
will bail with a clear error if `index.json` is missing.

## Reproducible layers

The application layer (`controller-layer.tar.gz`) is produced via:

```sh
tar --create --mtime="2026-01-01 00:00:00" --owner=0 --group=0 \
    --numeric-owner --sort=name -C "$layer_root" . | gzip -n > layer.tar.gz
```

The `gzip -n` flag zeroes the gzip header MTIME and omits the filename, so the
compressed digest is byte-stable across builds.  (`tar --gzip` sets the gzip
header MTIME to wall-clock time and must not be used here.)

## Push, sign, and promote

The prior local shell wrapper is retired.  The active push surface is the Rust
Buck2 tool:

```sh
buck2 run //tools/oci:oya-oci-push -- \
  <oci-layout-dir> \
  registry.oya-registry.svc.cluster.local:5000 \
  oya-ci-controller \
  ${GIT_SHA}-dev \
  --insecure
```

The command prints the pushed manifest digest to stdout.  Prow/native promotion
then consumes that digest for signing, SBOM/attestation, admission policy, and
deployment.  This directory no longer writes local digest sidecar files, and no
Helm adapter update is part of the authority path.

## Follow-up work (native promotion lane)

- Keep tag form `${GIT_SHA}-dev`.
- Sign by digest in the native Prow promotion worker with the pinned Cosign
  version and key/identity policy from the supply-chain standard.
- Attach SBOM and vulnerability scan evidence before promotion past dev.
- Feed the digest into CUE-owned desired state; any Helm artifact remains an
  adapter/export, not the canonical deployment source.
