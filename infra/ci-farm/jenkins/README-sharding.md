# O4 — Distributed test sharding (`Jenkinsfile.sharded`)

Realizes **ADR-0360 part O4**: fan a `cargo nextest run` across N ephemeral
agents, each running a disjoint partition of the test set, then aggregate. The
pipeline is `infra/ci-farm/jenkins/Jenkinsfile.sharded`.

```
cargo nextest run --partition slice:${SHARD}/${SHARDS}
```

## Partition modes: `slice` vs `hash` vs `count`

`cargo nextest` supports test **partitioning** so independent machines run
disjoint subsets. There are two supported strategies and one deprecated alias:

| Mode | Form | How it assigns a test to a bucket | Use when |
|---|---|---|---|
| **`slice`** (used here) | `slice:m/n` | Round-robins the *ordered, enumerated* test list into `n` buckets; bucket `m` runs. Assignment depends only on position in the list. | Default for CI sharding — even fill, simple, stable for a fixed test set. |
| **`hash`** | `hash:m/n` | Hashes each test's name into one of `n` buckets; bucket `m` runs. Assignment is a pure function of the test *name*. | When you need a test to always land on the same shard regardless of how many other tests exist (stable per-test placement across runs where the set changes). |
| **`count`** | `count:m/n` | Historical alias. **Deprecated** — do not use in new pipelines. | Never (kept only for back-compat). ADR-0360 mandates `slice:`. |

We use **`slice:`** because the O4 contract is even distribution of a known
test set across a known shard count, and `slice` gives the most even fill for
that. `hash` would be preferred only if we needed a *specific test* pinned to a
*specific shard* independent of the rest of the set (e.g. shard-local fixtures);
we don't, so `slice` wins on balance. `count:` is forbidden by the ADR.

Partitioning is applied at the **list** step: nextest enumerates all matching
tests, assigns each to a bucket, and only runs the tests in this shard's bucket
— so `slice:1/4 .. slice:4/4` together run exactly the full set, once each.

## Composition with O1 (affected-scope)

Sharding and affected-scope are **orthogonal** and compose by construction:

1. **O1 picks the SCOPE** (which crates' tests run). `oya verify --affected`
   classifies the diff and, for the `Crates` class, produces a `-p <crate> …`
   selection covering the transitive reverse-dependency closure. For `NoRust`
   diffs there is no cargo scope at all (gates only). For the `Full` class (or
   `--ci-required` on trunk) the scope is `--workspace`.
2. **O4 shards that SCOPE** (how those tests are distributed). The same `-p`
   selection is handed to `Jenkinsfile.sharded` via the `AFFECTED_ARGS` param;
   each shard runs `cargo nextest run $AFFECTED_ARGS --partition slice:m/n`. When
   `AFFECTED_ARGS` is empty the scope falls back to `--workspace` (the trunk
   full-mirror backstop).

```
diff ──O1 classify──▶ scope: { --workspace | -p A -p B … | (none, gates only) }
                              │
                              ▼
            O4: nextest run <scope> --partition slice:m/n  ×  N agents
```

Correctness invariant (from ADR-0360): **sharding never changes WHAT runs, only
HOW it is distributed.** The authoritative scopes are still O1's affected closure
on presubmit and the unchanged `--ci-required` whole-workspace mirror on trunk;
O4 is purely a distribution layer over whichever scope it is given.

## Parameters

| Param | Default | Meaning |
|---|---|---|
| `SHARDS` | `4` | Number of parallel slices / agents. Local-friendly default. |
| `AFFECTED_ARGS` | `` (empty) | O1 `-p` selection; empty ⇒ `--workspace` full mirror. |
| `NEXTEST_VERSION` | `0.9.96` | Pinned nextest version (baked in the prod agent image). |

## Local-vs-production deltas (honest)

- **Agent image**: locally the `oya-rust-build` template uses public
  `rust:1-bookworm` and installs sccache/nextest at runtime. Production uses the
  cosign-signed, digest-pinned image from `infra/ci-farm/agent-image/` with the
  toolchain + `cargo-nextest` + `sccache` **baked in** — no per-run install, no
  network fetch of binaries on the hot path.
- **Shard count**: `SHARDS=4` is a single-node-friendly default, NOT a measured
  optimum. Production derives the shard count from the measured per-test wall
  distribution against Karpenter-elastic capacity (ADR-0198).
- **No speedup claimed**: this pipeline establishes the partition + composition
  contract. End-to-end wall-clock speedup and shard-balance figures remain
  `blocked_until_required_evidence_is_green` per ADR-0360 — nothing is claimed
  as measured here.
