# O5 — Pinned, signed CI agent image

Realizes **ADR-0360 part O5**: a prebuilt CI agent image with a digest-pinned
base, baked toolchain (`cargo` + `cargo-nextest` + `sccache` + `git` +
`ca-certificates`), a warm read-only crate registry, a non-root numeric UID, and
a cosign signature on its digest — admitted by a scoped Kyverno verifyImages
policy.

| File | Purpose |
|---|---|
| `Dockerfile` | Multi-stage, digest-pinned rust base; bakes pinned nextest + sccache; warm RO `CARGO_HOME=/opt/cargo`; non-root `10001:0`. |
| `build-and-sign.sh` | Build → push → capture digest → `cosign sign` **by digest** (keyless OIDC preferred, key fallback); guards for missing cosign/builder. |
| `kyverno-verify-agent-image.yaml` | NEW scoped `ClusterPolicy` (verifyImages, `required: true`, `mutateDigest: true`) for the agent repo only, in `oya-ci-jenkins`. |

## Recipe

```bash
# 1. Resolve + pin the base digest in Dockerfile (replace the @sha256:<PINNED> placeholders):
crane digest docker.io/library/rust:1.86-bookworm        # or: docker buildx imagetools inspect …
#    -> paste the sha256 into BOTH FROM lines, keep the human tag in the comment.

# 2. Build + push + sign by digest:
REGISTRY=registry.oyatie.dev ./build-and-sign.sh          # keyless OIDC (CI identity)
# or, key fallback (colo/air-gapped):
COSIGN_KEY=k8s://oya-ci-jenkins/cosign-key ./build-and-sign.sh

# 3. Apply the scoped admission policy on the PRODUCTION farm cluster:
kubectl apply -f kyverno-verify-agent-image.yaml          # fill the cosign key / keyless block first

# 4. Point the agent pod template at the DIGEST-pinned ref the script prints:
#    registry.oyatie.dev/ci/rust-agent@sha256:<digest>
```

## Why digest-pin (base AND the agent image)

A tag (`rust:1-bookworm`, `…/rust-agent:latest`) is **mutable** — the same name
can resolve to different bytes over time. That breaks reproducibility and lets an
attacker (or an innocent re-push) swap content after review. Pinning
`@sha256:<digest>` makes the content byte-identical on every pull, and the cosign
signature attests **that exact digest**. The Kyverno policy's `mutateDigest: true`
closes the last gap: it rewrites any admitted tag to the verified digest at
admission time, so what runs is exactly what was signed — no tag-re-point TOCTOU.

`build-and-sign.sh` therefore signs the **digest**, never the tag, and the policy
runs `required: true` (fail-closed) so an unsigned or unknown agent image is
rejected rather than silently admitted.

## RO/RW credential split (external-secrets / OpenBao)

The image bakes **no** credentials. At runtime the agent gets S3 creds for the
sccache backend, and the trust boundary follows the O3 split (see
`../cache/README.md`):

- **PR / presubmit agents** get **read-only** S3 creds to the blessed sccache
  prefix (read-through) and write only a PR-scoped prefix. The warm `/opt/cargo`
  registry baked in the image is mounted **read-only** to the agent.
- **Trunk / postsubmit agents** get **read-write** creds to the blessed prefix so
  their results promote into the shared cache.

Creds are delivered by **external-secrets** syncing from **OpenBao** into a
namespace Secret, then bound into the pod env (mirrors the `seaweedfs-s3` Secret
binding in `values-local.yaml`, but OpenBao-backed and split by principal). The
write principal MUST equal the trust boundary — a PR agent must never hold a key
that can write the blessed prefix.

## Local-vs-production deltas (honest)

| Aspect | Production | Local k3s/colima profile |
|---|---|---|
| Base image | digest-pinned `rust@sha256:…` | public `rust:1-bookworm` tag (values-local.yaml) |
| Toolchain | nextest + sccache **baked** in image | installed at runtime via curl/`cargo install` |
| Warm registry | `/opt/cargo` populated by `cargo fetch`, mounted RO | none; cargo fetches live |
| Signing | cosign keyless OIDC (or KMS key) | none; image unsigned |
| Admission | `kyverno-verify-agent-image.yaml` **Enforce** | Kyverno not installed; not enforced |
| S3 creds | external-secrets ← OpenBao, RO/RW split | static `seaweedfs-s3` Secret |

None of these deltas change the *contract* (non-root agent, baked deterministic
toolchain, signed-by-digest, scoped admission). They are the hardening layer the
real farm provides. **No build time, image size, or signing latency is claimed** —
these stay `blocked_until_required_evidence_is_green` per ADR-0360.

## Production wiring (FB / ADR-0360 O5)

The local proof built + cosign-signed + verified this image (evidence in
`evidence/ci-farm-local/agent-image-build-sign.txt`). To go to production:

1. **Build + push by digest** to the real registry, capture the digest:
   `build-and-sign.sh` builds, pushes, and signs `registry.oyatie.dev/ci/rust-agent`
   BY DIGEST. Record the pushed `@sha256:...`.
2. **Pin the agent pod template** (`infra/ci-farm/jenkins/values-local.yaml`
   `rust-build`/`rust-parallel` templates) to that digest in production overlays —
   local keeps stock `rust:1-bookworm` since the private registry isn't reachable here.
3. **Key**: replace the demo public key in `kyverno-verify-agent-image.yaml` with the
   production cosign public key; the **private half is held in OpenBao** and injected
   into `build-and-sign.sh` via external-secrets (never committed). Public keys are
   safe to commit; the committed key here is the local demo key.
4. **Admission**: apply `kyverno-verify-agent-image.yaml` (scoped to
   `registry.oyatie.dev/ci/rust-agent*` in `oya-ci-jenkins`); unsigned/unknown images
   fail closed (`required: true`, `mutateDigest: true`).

Local-vs-prod delta: the demo signed against an offline key (no Rekor tlog;
`ctlog.ignoreTlog: true`); production keyed/keyless signing logs to Rekor — re-enable
tlog verification then.
