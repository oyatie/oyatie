#!/usr/bin/env bash
# Materialize cloud-ci generated faces from the checked-out candidate tree.
#
# Generated faces are CI/controller outputs, not contributor-owned merge surfaces. This boundary
# step regenerates the declared SCM snapshot and accounting faces before gates consume them, so
# PRs and merge queues validate the final candidate tree without hand-merging generated JSON.
set -euo pipefail

repo_root="${1:-.}"
faces_dir="${repo_root%/}/cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app"
scm_facts="${faces_dir}/scm-facts.generated.json"

toolchain="$(awk -F'"' '/channel/ {print $2; exit}' "${repo_root%/}/rust-toolchain.toml")"
rustup toolchain install "$toolchain" --profile minimal --component rustfmt --component clippy

# Build all three targets in one buck2 invocation and capture output paths in the same call.
# --show-output prints "<target> <path>" per line; we match each target name to extract its path.
show_out="$(buck2 build \
  //cloud/cloud-ci/gates/oya-cloud-ci-scm-facts-emitter-app:oya-cloud-ci-scm-facts-emitter-app \
  //cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app:oya-cloud-ci-accounting-registry-app-bin \
  //tools/oya-reorg-codemod-app:oya-reorg-codemod \
  --show-output 2>/dev/null)"

emitter="$(printf '%s\n' "$show_out" | awk '/oya-cloud-ci-scm-facts-emitter-app:oya-cloud-ci-scm-facts-emitter-app/ {print $2}')"
producer="$(printf '%s\n' "$show_out" | awk '/oya-cloud-ci-accounting-registry-app:oya-cloud-ci-accounting-registry-app-bin/ {print $2}')"
codemod="$(printf '%s\n' "$show_out" | awk '/oya-reorg-codemod-app:oya-reorg-codemod[[:space:]]/ {print $2}')"

# RENAME-AWARE PATH-KEYED BASELINE RELABEL (task #64). ORDERING is load-bearing:
#   1. codemod(manifest) — regenerate the committed move-manifest from the committed plan (if
#      any) + the candidate tracked tree, so the emitter consumes a FRESH, drift-checked copy;
#   2. emitter(--merge-base-baseline, relabel) — snapshot the frozen face at the merge-base AND
#      content-aware relabel its path-keyed keys per that manifest (fail-closed; strict no-op
#      when there are no renames);
#   3. producer(faces) — regenerate the candidate-tree faces the firewall differences against.
# The manifest is per-PR-overwrite, regenerated each run; with no committed plan it is the
# canonical EMPTY manifest (identity relabel — a no-move PR is gate-green and byte-stable).
"$codemod" manifest --repo-root "$repo_root" --out "${repo_root%/}/specs/reorg/move-manifest.generated.json"

# --merge-base-baseline (ADR-0551, FRIC-1781112000): the emitter also materializes the
# firewall's FROZEN reference — the gate-baseline face at `git merge-base <bootstrap> HEAD`.
# FROZEN-POLICY-WINS (FRIC-1781280000): the policy facts selecting that reference
# (base_ref, face_path) are read from ratchet-policy.json AS COMMITTED AT THE MERGE-BASE,
# located via the emitter's out-of-band bootstrap ref (--frozen-base-ref, default
# origin/dev) — never the candidate tree, so a same-PR base_ref repoint cannot select this
# PR's own frozen reference. The emitter ALSO relabels the frozen face's path-keyed keys per
# the move-manifest above (task #64). Untracked + gitignored; the firewall gate consumes it
# instead of the PR-local face, so a same-PR baseline regen can no longer launder new debt
# past the ratchet.
"$emitter" --repo-root "$repo_root" --out "$scm_facts" --merge-base-baseline
"$producer" --repo-root "$repo_root" --scm-facts "$scm_facts"
