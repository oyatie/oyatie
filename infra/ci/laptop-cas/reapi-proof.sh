#!/usr/bin/env bash
# REAPI proof against Access TCP forwarders (127.0.0.1:55051/55052).
# Expects start-access-tcp.sh already running and canary/proto present OR
# fetches protos into $RUNNER_TEMP.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
WRITE_PORT="${OYA_CAS_LOCAL_WRITE_PORT:-55051}"
READ_PORT="${OYA_CAS_LOCAL_READ_PORT:-55052}"
WRITE_HOST="${OYA_CAS_WRITE_HOST:-cw.oyatie.dev}"
READ_HOST="${OYA_CAS_READ_HOST:-cr.oyatie.dev}"
INSTANCE="${OYA_CAS_INSTANCE:-main}"
PROTO_DIR="${OYA_CAS_PROTO_DIR:-${RUNNER_TEMP:-/tmp}/oya-cas-proto}"

need() { command -v "$1" >/dev/null || { echo "missing: $1" >&2; exit 1; }; }
need grpcurl
need openssl
need git

mkdir -p "$PROTO_DIR"
if [[ ! -d "$PROTO_DIR/remote-apis/.git" ]]; then
  git clone --depth 1 https://github.com/bazelbuild/remote-apis.git "$PROTO_DIR/remote-apis"
fi
if [[ ! -d "$PROTO_DIR/googleapis/.git" ]]; then
  git clone --depth 1 https://github.com/googleapis/googleapis.git "$PROTO_DIR/googleapis"
fi

reapi() {
  local authority="$1" port="$2" method="$3" data="$4"
  grpcurl -insecure -authority "$authority" \
    -import-path "$PROTO_DIR/remote-apis" \
    -import-path "$PROTO_DIR/googleapis" \
    -proto build/bazel/remote/execution/v2/remote_execution.proto \
    -d "$data" \
    "127.0.0.1:${port}" \
    "$method"
}

CAPS=$(printf '{"instance_name":"%s"}' "$INSTANCE")
echo "--- GetCapabilities read ---"
reapi "$READ_HOST" "$READ_PORT" build.bazel.remote.execution.v2.Capabilities/GetCapabilities "$CAPS" \
  | tee "${RUNNER_TEMP:-/tmp}/oya-cas-caps-read.json"
echo "--- GetCapabilities write ---"
reapi "$WRITE_HOST" "$WRITE_PORT" build.bazel.remote.execution.v2.Capabilities/GetCapabilities "$CAPS" \
  | tee "${RUNNER_TEMP:-/tmp}/oya-cas-caps-write.json"

DIGEST=$(printf 'oya-gha-proof-%s-%s' "${GITHUB_RUN_ID:-local}" "$(date -u +%Y%m%dT%H%M%SZ)" \
  | openssl dgst -sha256 | awk '{print $NF}')
FMB=$(printf '{"instance_name":"%s","blob_digests":[{"hash":"%s","size_bytes":"32"}]}' "$INSTANCE" "$DIGEST")
echo "--- FindMissingBlobs digest=${DIGEST} ---"
reapi "$WRITE_HOST" "$WRITE_PORT" build.bazel.remote.execution.v2.ContentAddressableStorage/FindMissingBlobs "$FMB" \
  | tee "${RUNNER_TEMP:-/tmp}/oya-cas-fmb-write.json"
reapi "$READ_HOST" "$READ_PORT" build.bazel.remote.execution.v2.ContentAddressableStorage/FindMissingBlobs "$FMB" \
  | tee "${RUNNER_TEMP:-/tmp}/oya-cas-fmb-read.json"

echo "verdict: GREEN_REAPI_GHA"
echo "NOTE: does not flip warm_reads_licensed"
