#!/usr/bin/env bash
set -euo pipefail

# Configure optional CI-only Buck2 cache/RE overlay via .buckconfig.local.
#
# Supported mode values:
#  - off (default): no overlay; standard local-only .buckconfig.
#  - ro: read-only cache (no uploads)
#  - rw: read-write cache (uploads allowed)
#
# Optional overrides:
#   OYA_CI_RE_ENDPOINT             full gRPC URI (e.g. grpc://host:50051)
#   OYA_CI_RE_INSTANCE_NAME        RE instance name (default: main)
#   OYA_CI_RE_TLS                  true/false (default: true)
#   OYA_CI_RE_TLS_CLIENT_CERT       path to mTLS client cert PEM (appended if set)
#   OYA_CI_RE_REMOTE_CACHE_ENABLED true/false (default: true)
#   OYA_CI_RE_ALLOW_UPLOADS        true/false (default: ro/false for ro, rw/true for rw)
#
MODE="${OYA_CI_RE_CACHE_MODE:-${BUCK2_RE_CACHE_MODE:-off}}"
MODE="$(printf '%s' "$MODE" | tr 'A-Z' 'a-z')"
OVERLAY_FILE=".buckconfig.local"

remove_overlay() {
  rm -f "$OVERLAY_FILE"
}

case "$MODE" in
  ""|off)
    remove_overlay
    echo "buck2 cache/re overlay: OFF (default)"
    exit 0
    ;;
  ro|rw)
    :
    ;;
  *)
    echo "buck2 cache/re overlay: invalid mode '$MODE'; expected off|ro|rw" >&2
    exit 1
    ;;
esac

if [ "$MODE" = "ro" ]; then
  DEFAULT_ENDPOINT="grpc://nativelink-cas-reader.oya-ci.svc.cluster.local:50052"
  DEFAULT_UPLOADS="false"
  DEFAULT_REMOTE_CACHE="true"
elif [ "$MODE" = "rw" ]; then
  DEFAULT_ENDPOINT="grpc://nativelink-cas-writer.oya-ci.svc.cluster.local:50051"
  DEFAULT_UPLOADS="true"
  DEFAULT_REMOTE_CACHE="true"
fi

ENDPOINT="${OYA_CI_RE_ENDPOINT:-$DEFAULT_ENDPOINT}"
INSTANCE="${OYA_CI_RE_INSTANCE_NAME:-main}"
REMOTE_CACHE_ENABLED="${OYA_CI_RE_REMOTE_CACHE_ENABLED:-$DEFAULT_REMOTE_CACHE}"
UPLOADS="${OYA_CI_RE_ALLOW_UPLOADS:-$DEFAULT_UPLOADS}"
TLS="${OYA_CI_RE_TLS:-true}"
CERT_PATH="${OYA_CI_RE_TLS_CLIENT_CERT:-}"

if [ -z "$REMOTE_CACHE_ENABLED" ]; then
  REMOTE_CACHE_ENABLED="$DEFAULT_REMOTE_CACHE"
fi

if [ -z "$ENDPOINT" ]; then
  echo "buck2 cache/re overlay: MODE=$MODE requires a non-empty endpoint (OYA_CI_RE_ENDPOINT)." >&2
  exit 1
fi

if [[ "$ENDPOINT" == *://* ]]; then
  ENDPOINT_URL="$ENDPOINT"
else
  ENDPOINT_URL="grpc://$ENDPOINT"
fi

cat > "$OVERLAY_FILE" <<EOF2
[cache]
  # CI-local artifact cache (kept for reproducibility + offline fallback).
  mode = dir
  dir = ~/.cache/oya-buck2
  dir_mode = readwrite
  dir_max_size = 20GB

[buck2_re_client]
  engine_address = $ENDPOINT_URL
  cas_address = $ENDPOINT_URL
  action_cache_address = $ENDPOINT_URL
  instance_name = $INSTANCE
  tls = $TLS
EOF2

if [ -z "$UPLOADS" ]; then
  UPLOADS="$DEFAULT_UPLOADS"
fi

if [ "$REMOTE_CACHE_ENABLED" != "true" ] && [ "$REMOTE_CACHE_ENABLED" != "false" ]; then
  echo "buck2 cache/re overlay: invalid OYA_CI_RE_REMOTE_CACHE_ENABLED='$REMOTE_CACHE_ENABLED'; expected true|false" >&2
  exit 1
fi

if [ "$UPLOADS" != "true" ] && [ "$UPLOADS" != "false" ]; then
  echo "buck2 cache/re overlay: invalid OYA_CI_RE_ALLOW_UPLOADS='$UPLOADS'; expected true|false" >&2
  exit 1
fi

if [ "$REMOTE_CACHE_ENABLED" = "false" ] && [ "$UPLOADS" = "true" ]; then
  echo "buck2 cache/re overlay: REMOTE_CACHE_ENABLED=false with uploads=true is contradictory. forcing uploads=false." >&2
  UPLOADS="false"
fi

if [ -n "$CERT_PATH" ]; then
  echo "  tls_client_cert = $CERT_PATH" >> "$OVERLAY_FILE"
fi

cat >> "$OVERLAY_FILE" <<EOF2

[oya_cache]
  remote_cache_enabled = $REMOTE_CACHE_ENABLED
  allow_cache_uploads = false
EOF2

if [ "$UPLOADS" = "true" ]; then
  perl -0777 -i -pe 's/allow_cache_uploads = false/allow_cache_uploads = true/' "$OVERLAY_FILE"
fi

cat >> "$OVERLAY_FILE" <<'EOF2'

[build]
  execution_platforms = toolchains//cache:cache-platform
EOF2

rm -f "$OVERLAY_FILE.tmp"

if [ -n "$CERT_PATH" ] && [ ! -f "$CERT_PATH" ]; then
  echo "buck2 cache/re overlay: tls client cert path does not exist: $CERT_PATH" >&2
  echo "buck2 cache/re overlay: continuing because cert-based TLS endpoints commonly mount at runtime." >&2
fi

echo "buck2 cache/re overlay: mode=$MODE endpoint=$ENDPOINT_URL uploads=$UPLOADS instance=$INSTANCE tls=$TLS"
echo "--- .buckconfig.local ---"
sed -n '1,80p' "$OVERLAY_FILE"
