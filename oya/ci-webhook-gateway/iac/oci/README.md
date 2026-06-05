# oya-ci-webhook-gateway OCI image (buck2-native)

Buck2-native OCI image for `oya-ci-webhook-gateway`.  Mirrors
`oya/ci-controller/iac/oci/BUCK`.  Retires the BuildKit/Dockerfile path for this
service per ADR-0514.  ADDITIVE: the gateway `Dockerfile` is untouched until this
target is wired into push/deploy; it migrates separately.

## Targets

| Target | Description |
|---|---|
| `root//oya/ci-webhook-gateway/iac/oci:distroless-base` | genrule: pulls the base OCI layout via the OCI distribution API (`//tools/oci:oya-oci-pull-base`, pinned arm64 manifest digest) |
| `root//oya/ci-webhook-gateway/iac/oci:gateway-layer` | genrule: packages the buck2-built binary as a reproducible tar.gz layer |
| `root//oya/ci-webhook-gateway/iac/oci:gateway-oci` | oci_image: assembles the final OCI Image Layout directory |

## Binary target

The image packages `//oya/ci-webhook-gateway:ci-webhook-gateway-v2` — the
`rust_binary` whose `crate_root` is `src/main.rs`, i.e. the same entrypoint the
legacy `Dockerfile` compiled as `--bin oya-ci-webhook-gateway`.  It binds
`0.0.0.0:8099` by default (`OYA_CI_WEBHOOK_*` config; see `src/config.rs`,
`DEFAULT_BIND_ADDR`).  Inside the image the binary lives at
`/usr/local/bin/oya-ci-webhook-gateway`, preserving the existing ENTRYPOINT and
k8s Deployment contract.

## Darwin / Linux split

- **Darwin (aarch64):** `buck2 audit providers` and `buck2 cquery` resolve cleanly.
  The assembled layout is inspectable (valid OCI Image Layout directory).
  The binary inside is Mach-O (host arch) — NOT runnable on Linux.
  Push is intentionally blocked on darwin (linux CI only).
- **Linux CI (aarch64, Talos node):** the binary is `aarch64-linux` ELF — correct
  for deployment.  Publish with `//tools/oci:oya-oci-push` after assembly.

## Base image

`gcr.io/distroless/static-debian12:nonroot` — the `static` variant carries no
libc or libgcc, which is correct for a fully-static musl binary.  The gateway is
compiled with `aarch64-unknown-linux-musl` + `link-self-contained=yes`
(`+crt-static` is the musl-target default), so it has zero shared-library
dependencies.  Founder decision 2026-05-31; ADR-0146 canonical.

> The legacy `Dockerfile` used `gcr.io/distroless/cc-debian12` (glibc + libgcc)
> because it compiled a dynamically linked glibc binary.  The buck2-native path
> compiles fully-static against musl, so the smaller `static` base is correct.

The pinned arm64 manifest digest
(`sha256:82043e1cb4913437a9e90c2425b164d96031469894edc6011eed045001ba7181`) is
intentionally identical to the controller's so both images share one base layer.

## Building the musl image

The OCI target must be built with the musl platform so the transitive
`rust_binary` dep resolves to the musl toolchain:

```sh
buck2 build root//oya/ci-webhook-gateway/iac/oci:gateway-oci \
  --target-platforms //platforms:linux-aarch64-musl
```

The `--target-platforms` flag propagates to all transitive deps; the
`//oya/ci-webhook-gateway:ci-webhook-gateway-v2` `rust_binary` is automatically
compiled with `aarch64-unknown-linux-musl` + `-Clink-self-contained=yes`.

## Base digest bump procedure

The `sha256:` in `BUCK` is the **immutable arm64 manifest digest** passed to
`//tools/oci:oya-oci-pull-base`.  The puller verifies
`sha256(manifest_bytes) == DIGEST`, so a stale or wrong digest fails the build
deterministically.

To bump to a new CVE-patched weekly rebuild, resolve the linux/arm64 child
manifest digest from trusted upstream release evidence, then update the
`DIGEST` argument in the `:distroless-base` genrule.

Keep this digest in lock-step with `oya/ci-controller/iac/oci/BUCK`.

## Reproducible layers

The application layer (`gateway-layer.tar.gz`) is produced via:

```sh
tar --create --mtime="2026-01-01 00:00:00" --owner=0 --group=0 \
    --numeric-owner --sort=name -C "$layer_root" . | gzip -n > layer.tar.gz
```

The `gzip -n` flag zeroes the gzip header MTIME and omits the filename, so the
compressed digest is byte-stable across builds.  (`tar --gzip` sets the gzip
header MTIME to wall-clock time and must not be used here.)

## Image metadata

| Field | Value |
|---|---|
| `entrypoint` | `["/usr/local/bin/oya-ci-webhook-gateway"]` |
| `user` | `65532:65532` (nonroot) |
| `exposed_ports` | `["8099/tcp"]` |
| `image_title` | `oya-ci-webhook-gateway` |

## Push and sign

Linux CI only.  Publish with the Rust Buck2 tool:

```sh
buck2 run //tools/oci:oya-oci-push -- \
  <oci-layout-dir> \
  registry.oya-registry.svc.cluster.local:5000 \
  oya-ci-webhook-gateway \
  ${GIT_SHA}-dev \
  --insecure
```

The command prints the pushed manifest digest to stdout.  Signing, SBOM,
vulnerability scan evidence, and deployment admission stay in the native Prow
promotion lane.  Helm artifacts, if emitted, are compatibility adapters only;
CUE-owned desired state is the canonical deployment input.
