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

# Build both targets in one buck2 invocation and capture output paths in the same call.
# --show-output prints "<target> <path>" per line; we match each target name to extract its path.
# This replaces the previous pattern of 1 build + 2 separate --show-output queries (3 round-trips).
show_out="$(buck2 build \
  //cloud/cloud-ci/gates/oya-cloud-ci-scm-facts-emitter-app:oya-cloud-ci-scm-facts-emitter-app \
  //cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app:oya-cloud-ci-accounting-registry-app-bin \
  --show-output 2>/dev/null)"

emitter="$(printf '%s\n' "$show_out" | awk '/oya-cloud-ci-scm-facts-emitter-app:oya-cloud-ci-scm-facts-emitter-app/ {print $2}')"
producer="$(printf '%s\n' "$show_out" | awk '/oya-cloud-ci-accounting-registry-app:oya-cloud-ci-accounting-registry-app-bin/ {print $2}')"

"$emitter" --repo-root "$repo_root" --out "$scm_facts"
"$producer" --repo-root "$repo_root" --scm-facts "$scm_facts"
