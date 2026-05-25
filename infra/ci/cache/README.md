# O3 — Warm shared cache + cached downloads

Realizes **ADR-0360 part O3**: a warm, shared **compilation** cache (sccache → S3)
and a warm **download** cache (Panamax sparse-registry mirror + a warm read-only
`CARGO_HOME`). sccache caches *compilation*; Panamax caches *downloads* — they are
complementary, and O3 wires both.

| File | Purpose |
|---|---|
| `README.md` | This file — the trust model, key-prefix scheme, path-normalization, RO/RW IAM split. |
| `panamax-mirror.yaml` | k8s Deployment + Service + PVC for a Panamax sparse-registry crate-download mirror in `oya-ci-jenkins`. |
| `cargo-config-sample.toml` | SAMPLE `.cargo/config.toml` (applied at the agent level, **not** committed to repo root) doing source-replacement to the sparse mirror. |

## Trust model — who may write the shared cache

The shared sccache prefix is a trust boundary: a poisoned cache entry would be
served to every later build. So **the write principal must equal the trust
boundary**:

- **Trunk / postsubmit** builds (reviewed, merged code) get **read-write** to the
  *blessed* sccache prefix. They populate the cache everyone reads.
- **PR / presubmit** builds are **read-through**: they **read** the blessed prefix
  (warm hits from trunk) but **write** only to a **PR-scoped prefix** (e.g. keyed
  by PR number). On merge, the now-trusted result is promoted into the blessed
  prefix by the trunk run — never by the PR principal directly.

This prevents an untrusted PR from poisoning the cache that gates everyone, while
still giving PRs warm reads. It mirrors the O5 agent-image RO/RW cred split.

### RO vs RW S3 IAM creds split

Two distinct S3 principals, delivered via external-secrets ← OpenBao (prod) /
the `seaweedfs-s3` Secret (local):

| Principal | sccache prefix access | Used by |
|---|---|---|
| **`sccache-ro`** | `Get`/`List` on blessed prefix; `Put` only on `pr/<id>/*` | PR / presubmit agents |
| **`sccache-rw`** | `Get`/`List`/`Put` on blessed prefix | trunk / postsubmit agents |

The PR principal physically **cannot** write the blessed prefix (IAM-enforced),
so read-through safety is structural, not convention.

## `SCCACHE_S3_KEY_PREFIX` — encode the toolchain identity

A cache key must never mix artifacts built by different toolchains (a `rustc`
bump can change codegen). So the S3 key prefix encodes the **toolchain identity**:

```
SCCACHE_S3_KEY_PREFIX="rust/${RUSTC_VERSION}/${TARGET}/${blessed|pr-<id>}"
```

Different `rustc`/target ⇒ different prefix ⇒ no cross-toolchain contamination.
The `blessed` vs `pr-<id>` suffix is what implements the read-through split above.
(sccache also hashes compiler inputs/flags into each object key; the prefix is the
coarse partition that keeps unrelated toolchains and trust tiers apart.)

## `CARGO_INCREMENTAL=0`

Cargo incremental artifacts are machine- and path-specific and would poison a
**shared** cache (they're never reused across agents anyway). Disable incremental
on the farm so every compile is a clean, cacheable unit. Already set in the agent
image (`Dockerfile`) and the pod templates (`values-local.yaml`).

## `SCCACHE_BASEDIR` — path-independent cache keys

By default **sccache includes absolute source paths in its cache key** (e.g. via
`-Cdebuginfo` paths, include args, and `file!()` expansions). Two agents that
check the same commit out at different absolute paths would therefore compute
**different keys for identical code** — a near-total cache miss across agents.

`SCCACHE_BASEDIR` tells sccache to treat paths **relative to that base** when
hashing, normalizing them so the key is path-independent:

```
SCCACHE_BASEDIR="$CARGO_TARGET_DIR/.."     # or the per-agent checkout root
```

Set it to each agent's checkout root so a build at `/home/a/repo` and one at
`/workspace/repo` hash identically and share cache hits. (Local cross-agent hits
in `evidence/ci-farm-local/cross-agent-cache-measure.txt` rely on consistent
paths; `SCCACHE_BASEDIR` is what makes that robust when paths differ.)

## Download cache (Panamax)

`panamax-mirror.yaml` runs a Panamax mirror serving the cargo **sparse** protocol.
Agents point `crates-io` source-replacement at it via `cargo-config-sample.toml`
(applied at the agent's `$CARGO_HOME/config.toml`, not the repo root). This serves
the registry index + `.crate` files from inside the cluster, so the hot path makes
no public crates.io round-trips. Combined with the warm RO `/opt/cargo` registry
baked in the agent image, cold-download cost on the build path approaches zero.

## Local-vs-production deltas (honest)

| Aspect | Production | Local k3s/colima profile |
|---|---|---|
| sccache backend | S3 (object store), OpenBao-bound creds | SeaweedFS S3 (`seaweedfs-local.yaml`), static Secret |
| RO/RW split | two IAM principals, external-secrets ← OpenBao | single admin identity (local Secret) |
| Panamax mirror | HA, persistent volume, periodic sync | single replica, one PVC (`panamax-mirror.yaml`) |
| Key-prefix tiers | `blessed` + `pr-<id>` enforced by IAM | single shared prefix (`oya-ci-sccache-shared-prod`) |
| `SCCACHE_BASEDIR` | per-agent checkout root | paths already consistent (hostPath mount) |

The local profile proves the *wiring* (sccache→S3 hits, sparse mirror serving).
**No cache hit-rate, download-latency, or build-speedup number is claimed** — all
remain `blocked_until_required_evidence_is_green` per ADR-0360. The local measured
cache evidence in `evidence/ci-farm-local/` is single-node and does not stand in
for farm-scale figures.
