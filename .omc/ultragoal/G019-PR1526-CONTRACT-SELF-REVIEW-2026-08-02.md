# PR #1526 contract self-review — 2026-08-02

Candidate head: `fd2cb9d2f0d47f4bcd84c1c76e1953e7be440ecc`
Merge-base with origin/dev: `e26f2cc488c4a3ecab85be6bd7215f70decfa595`
Role: **coordinator self-review only**. Independent reviewer transport failed (`encrypted_content` decrypt). This is **not** an independent APPROVE and does **not** authorize admission.

## What changed
- `governance/corpus/extract/yaml_facts.bzl`: `corpus_yaml_facts_shards(srcs, shard_size)` — sorted inputs, shard_size>0, non-empty, ordinal 0000 grammar, first face keeps stable names `corpus-yaml-facts` / `yaml-facts.json`.
- `oya/BUCK`: root package uses macro with `shard_size = 256`.
- Nested package faces added: authn-device-firmware, billing, cost, flags, identity, meter.
- `ci/facade/corpus-index-coverage`: declaration parser + face derivation + exact-union/limit checks; policy freezes `expected_yaml_files: 4103` and face limits 512 / 1MiB.
- `Cargo.lock`: +1 dep edge `ci-corpus-index-coverage` → `oya-buck-syntax-kernel` only.

## Contract checks reproduced
| Check | Result |
|---|---|
| Exact head | `fd2cb9d2…` matches worktree + PR |
| No generated faces committed | true (policy/data + source only) |
| Nested packages non-empty | 1,2,2,1,2,2 YAML each — macro will not fail empty |
| Nearest-package Oya YAML union | 4103 = root 4082 + nested 10 + ci-webhook-gateway 11 |
| Root shard estimate | 16 faces (15×256 + 242) |
| Live consumer of full single-label output | none outside coverage gate / affected-set fixture strings |
| Label semantics change | `//oya:corpus-yaml-facts` becomes **first shard only** (≤256). Union coverage is enforced by the gate, not by that single label. Acceptable for current consumers; future full-union consumers must depend on all shard labels or a new aggregator. |
| Local Buck2 unittest | 44/44 PASS |
| Local Buck2 gate | 19/19 PASS |
| Placeholder / skip / unimplemented in diff | none found |

## Residual risks (not blockers of the repair class)
1. Source-derived face derivation is not a Starlark evaluator — policy already records this; configured Buck2 build remains mandatory proof.
2. Independent review still missing (transport).
3. Candidate protected CI attempt 3 still `QUEUED_NO_VERDICT` (affected-set job `91524468086`).
4. Local green ≠ candidate green ≠ promoted green.

## Verdict
- Self-review structural quality: **APPROVE-quality structure** for the intended ARG_MAX / oversized-face repair.
- Independent review: **FAILED_TRANSPORT / absent**.
- Admission: **NOT PERMITTED**.
- Merge: **NOT PERMITTED**.
