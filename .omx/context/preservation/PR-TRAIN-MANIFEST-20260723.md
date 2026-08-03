# PR train preservation manifest

Status: `NON_AUTHORITY`

Planning state: `HOLD(Planning)`

Artifact disposition: `Proposed`

This manifest records local recovery artifacts only. It is not a protected-PR
receipt, required-check result, qualified-human disposition, Stage-1 control
receipt, planning authorization, or implementation-dispatch instruction.

## Current exact #1364 successor preservation receipt

Current candidate:
`cc08cc12be9318a11ae15da47f9ca39002109c73`

- Tree:
  `3ca56efd398a869a5a339a3a3da55ba545eb03bf`.
- Train base:
  `1026a65b707ce57693d9b830de33ee0ce228f16b`.
- Regression-first test-only RED:
  `d43053db4ebe473275ba3071003a9694e6469b01`.
- Exact docs-only parent:
  `85cc6dba57c6ec969b4a6cc8376e09d85d068ae6`.
- Candidate signature: valid ED25519 signature using key
  `SHA256:5grGNUtX9Zgmy1SWne6wF9DR8W1ElUQaF/Z8SYRz8E8`.
- Candidate worktree: clean; generated faces changed: `0`.
- Canonical Buck2 receipt:
  `/private/tmp/pr1364-cc08-final-green.2EGabg/buck2-test.txt`;
  SHA-256
  `5834913b0135b485110c312d1005b72543f0425d85503ad126a13b4c6565d088`;
  unit `1/1`, contract `106/106`, terminal `Pass 2, Fail 0`.
- Independent verifier Buck2 receipt:
  `/private/tmp/pr1364-cc08-verifier.6A8TNZ/buck2-test.txt`;
  SHA-256
  `05b6decdf5e38759b8a4c33082cd48f80bd54062104d5b4fb620776bbec56bb7`;
  all `107/107` tests passed.
- Supplemental direct Rust diagnostic:
  `/private/tmp/pr1364-cc08-rustc-diagnostic.JslGvd/rustc-diagnostic.txt`;
  SHA-256
  `4650ee95de593633b5f2c51a32358336efdc8aa8905371d7fe86ee40831f4a99`;
  contract `106/106`.
- Self-contained recovery bundle:
  `pr1364-final-local-candidate-cc08cc12b-20260723-self-contained.bundle`;
  SHA-256
  `1dc5d3a642c9c87f540d6a0b2b79393a7bfbea6c137f6d877b23508e65453452`;
  size `111449172` bytes.
- Bundle recovery verification: fresh bare clone, fresh fetch/checkout,
  non-shallow, no alternates, full `git fsck`, exact head/tree, and `74`
  signed commits passed. Signature log SHA-256:
  `492259557621096b9d009d007c139a5e21e9cf69170a3600e95f89c8f4f08cf1`.
- Exact-head NON_AUTHORITY receipts:
  - `reviews/pr1364-cc08cc12b-architecture.yaml` —
    `0d0b7e367ae27385d3e93e21cdfa8b294dbbf4289f5e05101e5134e3e348a242`;
    verdict `APPROVE_SOURCE_QUALITY_ONLY`.
  - `reviews/pr1364-cc08cc12b-code-security.yaml` —
    `dee21f05c20efc3fc5cbf180568045f968d92a0986796a43245e53dd5c307bef`;
    verdict `APPROVE_SOURCE_QUALITY_ONLY`.
  - `reviews/pr1364-cc08cc12b-authority.yaml` —
    `e2406219a968dd299564cb8e04e5bbb9f66500e0a56b52d7f41ae1d6c8768d4f`;
    verdict `NON_AUTHORITY_CONFIRMED`.
  - `reviews/pr1364-cc08cc12b-verifier.yaml` —
    `93a532155001d656c1646c5b4cafed27b528c50222a8bd20e33c4498170410a8`;
    verdict `VERIFIED_ENGINEERING_EVIDENCE_ONLY`.
- Current decision/blocker packet:
  `STAGE1-DECISION-BLOCKER-PACKET-cc08cc12b-20260723.yaml`;
  SHA-256
  `00b953d5b2db7de2a68b0f4ba65be5c66401ef6e455f824ca8c3a1be791ff7f2`.
- Current handoff index:
  `STAGE1-DECISION-BLOCKER-HANDOFF-INDEX-cc08cc12b-20260723.yaml`.
- Current handoff envelope:
  `stage1-decision-blocker-handoff-00b953d5b2db-20260723.tar`;
  SHA-256
  `51c90ed2ab6933279e1f16a2a49477d1577df5ed2a64f0af5aef4d3e2c03e10e`.
- Envelope checksum sidecar SHA-256:
  `f192c79e9d0e4c31110ac0c35728cef9e6e569355b6c0b1831809fb8a824ec0b`.
- Envelope policy: the exact packet, packet sidecar, four review receipts, and
  bundle sidecar are embedded. The 111 MB bundle remains adjacent.
- GitHub read-only snapshot completed at `2026-07-24T02:47:31Z`: PRs
  #1361-#1364 remain open drafts, report `mergeable=false`, have no submitted
  reviews, and show CodeRabbit success without `oya-ci-required`. Every remote
  head differs from its exact signed local candidate. Observed `dev` remains
  exactly `30dd46c4dd7d12c085b77332d9ac5035d583edcf`; live branch-protection
  configuration was not exposed by the connector.
- Terminal decision:
  `BLOCKED_QUALIFIED_HUMAN_INPUT`.
- Planning state: `HOLD(Planning)`. Roadmap planning, binding plan approval,
  implementation dispatch, and merge are not authorized.
- Archive rule: retired bytes may remain in Git history or an adjacent recovery
  bundle, but not in a readable archived-information directory in the active
  tree. This is an authority-discovery boundary, not a confidentiality claim.
- The earlier `85cc6dba5` recovery bundle and all `781789573` packet/envelope
  artifacts remain immutable superseded provenance only.

## Frozen train

| PR | Local role | Head | Tree | Bundle SHA-256 |
| --- | --- | --- | --- | --- |
| #1361 | final local admission candidate | `aadb8c75878fff3c93f0858d5a758707a93f1205` | `fcbe2e60185520bae38d1e2a90ffa02e581c4d78` | `011acffdb17e1789d2a6563a1adbecefac6e47371943628212fe38dd0edffc09` |
| #1362 | frozen successor of #1361 | `d69a14831d8eb5f731bd74f60e74db13bb3792e8` | `7a6abf708fb3751799784fec35a59e9755d25bea` | `b2a81acde4480a06b877ef0bf90b3edd1804c21d3f6c42ac96f652117ac21bd0` |
| #1363 | frozen successor of #1362 | `1026a65b707ce57693d9b830de33ee0ce228f16b` | `d20c21163c6ec1ac2f6e1b581031e4431b54f401` | `13d76d8159f10a9629403d26179b71ee1ef993e0adbc79bc24ca7479b31b0079` |
| #1364 | current exact local preservation candidate; external admission blocked | `cc08cc12be9318a11ae15da47f9ca39002109c73` | `3ca56efd398a869a5a339a3a3da55ba545eb03bf` | `1dc5d3a642c9c87f540d6a0b2b79393a7bfbea6c137f6d877b23508e65453452` |

## Superseded #1364 preservation receipt (`781789573`)

Status: immutable provenance only; superseded by the exact `cc08cc12b`
successor receipt above.

Artifact:
`pr1364-final-local-candidate-781789573-20260723-self-contained.bundle`

- SHA-256:
  `8adb152d504c1725e7d1e4ed35f24bd1dd96c83b12bcd3b74308ae5903f14b0e`
- Size: `111651963` bytes.
- Candidate head: `781789573560cd95556f2a9764e2262196708140`
- Candidate tree: `7e5395f2e3463c1c5f5e62d0da13ae4b4778a8ed`
- Required base: `1026a65b707ce57693d9b830de33ee0ce228f16b`
- Required base tree: `d20c21163c6ec1ac2f6e1b581031e4431b54f401`
- Architecture review: `APPROVE_SOURCE_QUALITY_ONLY`; architectural status
  `CLEAR`.
- Code/security review: `COMMENT`, with zero source findings; `APPROVE` was not
  issued because canonical Buck and clean modified-file workspace diagnostics
  were unavailable in this managed environment.
- Authority review: `NON_AUTHORITY_CONFIRMED`.
- Independent verifier: `PASS_WITH_DECLARED_BUCK_ENVIRONMENT_GAP`; canonical
  Buck exited before build work because its daemon directory could not be
  created. This is not a passing Buck result or merge authority.
- Bounded diagnostic: `66/66`; exact combined-output SHA-256
  `6b56ee329057266f3ef920b3469120e99b6ac9d71abe3090f334e0d535e77ab4`.
- SSH signatures: `43/43` candidate commits.
- Generated faces: `0`.
- Exact-head NON_AUTHORITY receipts:
  - `reviews/pr1364-781789573-architecture.yaml` —
    `eb4e53afb8eadf0ded04ea5d8f6d85f0187aeb88f0b75a7a84942163d746538b`;
    leader-materialized from the read-only architect's final message and not
    independently signed by that reviewer.
  - `reviews/pr1364-781789573-code-security.yaml` —
    `f1aa891ebc85904d155e7f2c60389a505ed5b863d238ab3b846b69b029b4da71`.
  - `reviews/pr1364-781789573-authority.yaml` —
    `4540fb201f07c9562d81eaca3c5260f8cc46683d002dcd9f590388c0abe537b9`.
  - `reviews/pr1364-781789573-verifier.yaml` —
    `6a73ac7a726f6e0d6b22e381b0f9dba3b0aa84897fe50b1e5fafef4a54d128dc`.
- Governance state: `HOLD(Planning)`; artifact disposition `Proposed`; status
  `NON_AUTHORITY`.
- Claim boundary: exact local recovery only. This receipt is not remote state,
  protected admission evidence, merge readiness, Stage-1 PASS, or product
  completion.

The final bundle is self-contained for these named local refs:

| Recovery ref | Object |
| --- | --- |
| `HEAD` and `refs/heads/recovery/pr1364-final-local-candidate` | `781789573560cd95556f2a9764e2262196708140` |
| `refs/heads/recovery/pr1363-base` | `1026a65b707ce57693d9b830de33ee0ce228f16b` |
| `refs/heads/recovery/pr1364-donor-baseline` | `2832aed7fa1240412a0a5d9bd854ab214a3c7dab` |
| `refs/heads/recovery/root-checkout-head` | `c52bdb09ea337de103b05317de0c120f2b7a3e45` |
| `refs/heads/recovery/root-local-dev` | `b0a42acbabfe82b900d7a64c8af52e6c9a80bfcc` |
| `refs/heads/recovery/root-origin-dev` | `30dd46c4dd7d12c085b77332d9ac5035d583edcf` |
| `refs/heads/recovery/root-origin-main` | `9dd19524155011ac9824bfee972029a3b95917a6` |

Independent verification exercised both supported recovery paths:

- A fresh clone checked out exact head
  `781789573560cd95556f2a9764e2262196708140` and exact tree
  `7e5395f2e3463c1c5f5e62d0da13ae4b4778a8ed`.
- A fresh initialized repository fetched
  `refs/heads/recovery/pr1364-final-local-candidate`, then checked out the same
  exact head and tree.
- Both repositories were non-shallow, had no object alternates, and returned
  exit zero from `git fsck --full`.
- The required base and formerly missing commit
  `651d66800ebfa8d2706100fddc67e411f95a6fcf` were present.
- All 43 candidate commits in
  `1026a65b707ce57693d9b830de33ee0ce228f16b..781789573560cd95556f2a9764e2262196708140`
  verified against the configured SSH allowed-signers file.
- The checksum sidecar is
  `pr1364-final-local-candidate-781789573-20260723-self-contained.bundle.sha256`.

### Superseded #1364 snapshots

These older artifacts remain provenance snapshots only and are superseded by
the final local preservation receipt above:

- `pr1364-final-local-candidate-455ca5577-20260723-self-contained.bundle`:
  valid self-contained predecessor snapshot with SHA-256
  `b36b938fe49780e32c8ed088d7a1da08d9c7dc57ca442e563181c6537a6e35b8`;
  superseded after regression-first SCM namespace, merge-base, sibling-boundary,
  and QPA-causality repairs.
- `pr1364-candidate-912c1223c-20260723-self-contained.bundle`: valid
  self-contained earlier local snapshot; not the final candidate.
- `pr1364-baseline-2832aed7f-20260723.bundle`: thin baseline snapshot with
  SHA-256
  `acb5c1521d561165c2ae4d0aad60800e1091d9dec903c904b1c1725045298f52`;
  requires predecessor `1026a65b707ce57693d9b830de33ee0ce228f16b`.
- `pr1364-baseline-2832aed7f-20260723-standalone.bundle`: misleading historical
  filename, not standalone or complete-history. A fresh non-shallow clone
  followed by `git fsck --full` exits 2 because commit
  `33134e055d96326ccb0e4b76602d93a285673dad` references missing parent
  `651d66800ebfa8d2706100fddc67e411f95a6fcf`. Do not use it for recovery.

## #1364 rebuild procedure

The safe procedure never copies a source `shallow` file. It exposes the root
and candidate object databases to a temporary bare repository, pins explicit
recovery refs, copies every reachable object into a new local pack, removes
the alternates, and requires a full object-graph check before bundle creation:

```sh
ROOT=/Users/jasonlee/Developer/oyatie
CANDIDATE="$ROOT/.omx/worktrees/codex-pr1364-prep-20260723-v2"
TARGET_HEAD="$(git -C "$CANDIDATE" rev-parse HEAD)"
TARGET_TREE="$(git -C "$CANDIDATE" rev-parse "$TARGET_HEAD^{tree}")"
STAGE_ROOT="$(mktemp -d /private/tmp/pr1364-preservation.XXXXXX)"
STAGE="$STAGE_ROOT/stage.git"
OUTPUT="$STAGE_ROOT/pr1364-final-local-candidate-${TARGET_HEAD}-self-contained.bundle"

git init --bare "$STAGE"
printf '%s\n%s\n' \
  "$(git -C "$ROOT" rev-parse --path-format=absolute --git-common-dir)/objects" \
  "$(git -C "$CANDIDATE" rev-parse --path-format=absolute --git-common-dir)/objects" \
  >"$STAGE/objects/info/alternates"

git -C "$STAGE" update-ref \
  refs/heads/recovery/pr1364-final-local-candidate "$TARGET_HEAD"
git -C "$STAGE" update-ref refs/heads/recovery/pr1363-base \
  1026a65b707ce57693d9b830de33ee0ce228f16b
git -C "$STAGE" update-ref refs/heads/recovery/pr1364-donor-baseline \
  2832aed7fa1240412a0a5d9bd854ab214a3c7dab
git -C "$STAGE" update-ref refs/heads/recovery/root-checkout-head \
  "$(git -C "$ROOT" rev-parse HEAD)"
git -C "$STAGE" update-ref refs/heads/recovery/root-local-dev \
  "$(git -C "$ROOT" rev-parse refs/heads/dev)"
git -C "$STAGE" update-ref refs/heads/recovery/root-origin-dev \
  "$(git -C "$ROOT" rev-parse refs/remotes/origin/dev)"
git -C "$STAGE" update-ref refs/heads/recovery/root-origin-main \
  "$(git -C "$ROOT" rev-parse refs/remotes/origin/main)"
git -C "$STAGE" symbolic-ref HEAD \
  refs/heads/recovery/pr1364-final-local-candidate

git -C "$STAGE" fsck --full --no-dangling
git -C "$STAGE" repack -a -d
rm "$STAGE/objects/info/alternates"
git -C "$STAGE" fsck --full --no-dangling
git -C "$STAGE" bundle create "$OUTPUT" --all

VERIFY_ROOT="$(mktemp -d /private/tmp/pr1364-preservation-verify.XXXXXX)"
git clone --bare "$OUTPUT" "$VERIFY_ROOT/recovered.git"
git -C "$VERIFY_ROOT/recovered.git" fsck --full --no-dangling
test "$(git -C "$VERIFY_ROOT/recovered.git" rev-parse HEAD)" = "$TARGET_HEAD"
test "$(git -C "$VERIFY_ROOT/recovered.git" rev-parse 'HEAD^{tree}')" = "$TARGET_TREE"
git -C "$VERIFY_ROOT/recovered.git" cat-file -e \
  1026a65b707ce57693d9b830de33ee0ce228f16b^{commit}
git -C "$VERIFY_ROOT/recovered.git" cat-file -e \
  2832aed7fa1240412a0a5d9bd854ab214a3c7dab^{commit}
git -C "$VERIFY_ROOT/recovered.git" cat-file -e \
  651d66800ebfa8d2706100fddc67e411f95a6fcf^{commit}
git -C "$VERIFY_ROOT/recovered.git" rev-list --reverse \
  1026a65b707ce57693d9b830de33ee0ce228f16b.."$TARGET_HEAD" \
  >"$VERIFY_ROOT/candidate-commits"
while read -r commit; do
  git -C "$VERIFY_ROOT/recovered.git" \
    -c gpg.format=ssh \
    -c gpg.ssh.allowedSignersFile=/Users/jasonlee/.ssh/allowed_signers \
    verify-commit "$commit" || exit 1
done <"$VERIFY_ROOT/candidate-commits"

FETCH_ROOT="$(mktemp -d /private/tmp/pr1364-preservation-fetch.XXXXXX)"
git init "$FETCH_ROOT/recovered"
git -C "$FETCH_ROOT/recovered" fetch "$OUTPUT" \
  '+refs/heads/*:refs/heads/*'
git -C "$FETCH_ROOT/recovered" checkout --detach \
  refs/heads/recovery/pr1364-final-local-candidate
test "$(git -C "$FETCH_ROOT/recovered" rev-parse HEAD)" = "$TARGET_HEAD"
test "$(git -C "$FETCH_ROOT/recovered" rev-parse 'HEAD^{tree}')" = "$TARGET_TREE"
git -C "$FETCH_ROOT/recovered" fsck --full

shasum -a 256 "$OUTPUT"
```

`git bundle verify` is necessary but not sufficient for a source created from
a shallow repository. The fresh non-shallow clone plus `git fsck --full` is
the required completeness test.

## Admission state

- Bounded GitHub connector snapshot on 2026-07-23: remote PRs #1361-#1364
  were open drafts, reported `mergeable=false`, and had no submitted reviews.
  Their remote heads were `7915baa3` (#1361), `f5ebd909` (#1362),
  `d422aa41` (#1363), and `6c149e8b` (#1364). Combined status on each head
  contained CodeRabbit success only and no protected `oya-ci-required`.
- That connector snapshot is drift-prone `NON_AUTHORITY` context. In
  particular, remote #1364 head `6c149e8b` is not the local preservation
  target `781789573560cd95556f2a9764e2262196708140`.
- The protected train order is #1361, #1362, #1363, then #1364.
- Preparation and read-only review may overlap; restack freshness, exact-head
  review, protected `oya-ci-required`, merge, and post-merge proof serialize
  against the actual protected parent.
- Shell transport is blocked by DNS resolution failure for `github.com`.
- At `2026-07-24T00:01:50Z`, one bounded force-with-lease admission attempt for
  exact signed #1361 candidate
  `aadb8c75878fff3c93f0858d5a758707a93f1205`, pinned to observed remote head
  `7915baa3d95ae3ea6a4d35c0a1fc750ee303c51c`, failed before any remote
  mutation with:
  `fatal: unable to access 'https://github.com/jason931225/oyatie.git/': Could not resolve host: github.com`.
- No GitHub connector substitute was used to recreate commits because it would
  not preserve the exact local SSH-signed commit objects.

## Process packet

`bun-derived-non-regression-pipeline-20260723.md` has SHA-256
`0b21b63c5b1c83068c43bd7c110b03c26e0abb723c4a78f516f528b46b3ec8a1`.
It is process preparation only and is not C05 comparator evidence. Its
restricted-history rule explicitly separates agent-readable Git/bundle recovery
from qualified external custody and makes no retroactive confidentiality claim.

## Sealed decision/blocker handoff

- Status: `SUPERSEDED_BY_POSTSEAL_ARCHITECTURE_FINDINGS`.
- Packet:
  `STAGE1-DECISION-BLOCKER-PACKET-20260723.yaml`
- Packet SHA-256:
  `21d05b4d3a1dafa568ce14ecba7af812d9e7620c80a14aa62e570a1f8e1a5b29`
- Handoff envelope:
  `stage1-decision-blocker-handoff-21d05b4d3a1d-20260723.tar`
- Handoff envelope SHA-256:
  `9b6cf3e40b875a138d04c69b414a1aef1a4e008301edac43e07c8949ea7f798b`
- Envelope index:
  `STAGE1-DECISION-BLOCKER-HANDOFF-INDEX-20260723.yaml`
- Recovery verification: fresh extraction reproduced the exact packet digest
  and every embedded receipt digest.
- Dependency direction: this manifest is a mutable one-way index of the sealed
  packet and envelope. Neither sealed artifact embeds a digest of this manifest.
- Claim boundary: the sealed verdict is
  `BLOCKED_QUALIFIED_HUMAN_INPUT`, not Stage-1 PASS. It preserves
  `HOLD(Planning)` and grants no roadmap-planning, plan-approval, dispatch, or
  merge authority.
- Supersession cause: a fresh exact-head architectural audit found that the
  proposed source contract still accepted protected-head/base identity collapse,
  serialized oracle before blind reader despite their parallel DAG edge, and
  kept full mutation/context-exposure invalidation only in this NON_AUTHORITY
  packet. Regression-first repair is active; the packet and envelope remain
  immutable provenance but are not the current terminal handoff.
- Follow-up candidate `4eab21df0ff8a03f201a07e75998695dd1e9e4c2`
  closed those three findings and passed 68/68 bounded diagnostics, but a new
  fresh exact-head architectural audit found that the declared epoch-opening
  `materializer` domain had only a principal identity binding, not an exact
  implementation-artifact binding in protected facts and the immutable
  successor. A focused regression-first repair is active; `4eab21df…` is also
  superseded as a final candidate.

## Supersession

The final local #1364 preservation receipt is recorded above. This closes only
the exact local recovery obligation for candidate
`781789573560cd95556f2a9764e2262196708140`. H1-H6, protected admission,
remote-head freshness, merge readiness, and product completion remain outside
this manifest's authority. Preserve `HOLD(Planning)`, `Proposed`, and
`NON_AUTHORITY`; do not relabel this receipt or any baseline as Stage-1 PASS.
