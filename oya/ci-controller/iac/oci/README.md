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
  Push is intentionally blocked on darwin by `push-and-sign.sh`.
- **Linux CI (aarch64, Talos node):** the binary is `aarch64-linux` ELF — correct
  for deployment.  Run `push-and-sign.sh` after assembly.

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

## Base digest staging and bump procedure

The `sha256` in `BUCK` is the digest of the **OCI layout tarball** (not the image
manifest digest).  It is a placeholder (`000...000`) until the first real staging.

To stage or bump the base (requires `crane` on PATH):

```sh
# 1. Pull as OCI layout (NOT crane export — that yields a flat rootfs with no index.json)
crane pull --format oci \
  gcr.io/distroless/static-debian12:nonroot \
  /tmp/distroless-static-nonroot-aarch64-$(date +%Y%m%d)

# 2. Tar the layout directory
tar -cf distroless-static-nonroot-aarch64-$(date +%Y%m%d).tar \
  -C /tmp/distroless-static-nonroot-aarch64-$(date +%Y%m%d) .

# 3. Capture sha256
sha256sum distroless-static-nonroot-aarch64-$(date +%Y%m%d).tar

# 4. Push to in-cluster build-inputs registry
crane push distroless-static-nonroot-aarch64-$(date +%Y%m%d).tar \
  registry.oya-registry.svc.cluster.local:5000/build-inputs/distroless-static-nonroot-aarch64-$(date +%Y%m%d).tar

# 5. Update BUCK:
#   - urls[0]: new filename with date
#   - sha256: output of step 3
```

**Important:** use `crane pull --format oci`, not `crane export`.
`crane export` produces a flat merged rootfs tarball with no `index.json` or
`blobs/sha256/` tree.  The assembler (`oya-oci-assemble`) strictly requires an
OCI Image Layout and will bail with a clear error if `index.json` is missing.

## Reproducible layers

The application layer (`controller-layer.tar.gz`) is produced via:

```sh
tar --create --mtime="2026-01-01 00:00:00" --owner=0 --group=0 \
    --numeric-owner --sort=name -C "$layer_root" . | gzip -n > layer.tar.gz
```

The `gzip -n` flag zeroes the gzip header MTIME and omits the filename, so the
compressed digest is byte-stable across builds.  (`tar --gzip` sets the gzip
header MTIME to wall-clock time and must not be used here.)

## Push and sign

`push-and-sign.sh` is Linux CI only.  Before running it, export:

```sh
# From the ESO-projected cosign secret mount:
export COSIGN_PASSWORD="$(cat /var/run/secrets/cosign/password)"
# For unencrypted keys:
export COSIGN_PASSWORD=""
```

Then:

```sh
./push-and-sign.sh <oci-layout-dir> [<tag>]
# tag defaults to git short SHA; ADR-0181 target form: ${GIT_SHA}-dev
```

The script:
1. Packs the OCI layout and pushes via `crane push`.
2. Resolves the pushed digest via `crane digest`.
3. Signs by digest via `cosign sign --key k8s://oya-ci/cosign-key` (ADR-0181).
4. Writes the digest to `last-pushed-digest.txt` for the Helm values updater.

## Follow-up work (Linux CI lane)

- Emit tag as `${GIT_SHA}-dev` to match ADR-0181 tagging convention.
- Write the resolved digest into `values.yaml` `image.digest` and flip `cosign.required=true`.
- Add SBOM (`cosign attest --type spdxjson`) and Trivy scan evidence per ADR-0039/0181
  promotion ladder for dev→staging.
