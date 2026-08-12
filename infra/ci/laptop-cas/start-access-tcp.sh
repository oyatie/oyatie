#!/usr/bin/env bash
# Start Cloudflare Access TCP forwarders to the laptop CAS public hostnames.
# Listens on 127.0.0.1:55051 (write/cw) and :55052 (read/cr).
#
# Required env (from GHA secrets or local ~/oyatie-cas/secrets):
#   CF_ACCESS_WRITE_CLIENT_ID / CF_ACCESS_WRITE_CLIENT_SECRET
#   CF_ACCESS_READ_CLIENT_ID  / CF_ACCESS_READ_CLIENT_SECRET
# Optional:
#   OYA_CAS_WRITE_HOST (default cw.oyatie.dev)
#   OYA_CAS_READ_HOST  (default cr.oyatie.dev)
#   OYA_CAS_CLOUDFLARED_IMAGE (default cloudflare/cloudflared:2026.7.3)
set -euo pipefail

WRITE_HOST="${OYA_CAS_WRITE_HOST:-cw.oyatie.dev}"
READ_HOST="${OYA_CAS_READ_HOST:-cr.oyatie.dev}"
CF_IMAGE="${OYA_CAS_CLOUDFLARED_IMAGE:-cloudflare/cloudflared:2026.7.3}"
WRITE_PORT="${OYA_CAS_LOCAL_WRITE_PORT:-55051}"
READ_PORT="${OYA_CAS_LOCAL_READ_PORT:-55052}"

need() { command -v "$1" >/dev/null || { echo "missing: $1" >&2; exit 1; }; }
need docker
need curl
need python3

: "${CF_ACCESS_WRITE_CLIENT_ID:?}"
: "${CF_ACCESS_WRITE_CLIENT_SECRET:?}"
: "${CF_ACCESS_READ_CLIENT_ID:?}"
: "${CF_ACCESS_READ_CLIENT_SECRET:?}"

resolve_a() {
  curl -fsS -H 'accept: application/dns-json' "https://1.1.1.1/dns-query?name=$1&type=A" \
    | python3 -c 'import sys,json; a=json.load(sys.stdin).get("Answer") or []; print(next(x["data"] for x in a if x.get("type")==1))'
}

start() {
  local name="$1" host="$2" ip="$3" port="$4" id="$5" secret="$6"
  docker rm -f "$name" >/dev/null 2>&1 || true
  docker run -d --name "$name" --restart unless-stopped \
    --add-host "${host}:${ip}" \
    -p "127.0.0.1:${port}:${port}" \
    -e "TUNNEL_SERVICE_TOKEN_ID=${id}" \
    -e "TUNNEL_SERVICE_TOKEN_SECRET=${secret}" \
    "$CF_IMAGE" \
    access tcp --hostname "$host" --url "0.0.0.0:${port}" >/dev/null
  echo "started ${name} -> ${host} via ${ip} on 127.0.0.1:${port}"
}

WRITE_IP="$(resolve_a "$WRITE_HOST")"
READ_IP="$(resolve_a "$READ_HOST")"
echo "dns ${WRITE_HOST}=${WRITE_IP} ${READ_HOST}=${READ_IP}"

start oya-cf-access-cw "$WRITE_HOST" "$WRITE_IP" "$WRITE_PORT" \
  "$CF_ACCESS_WRITE_CLIENT_ID" "$CF_ACCESS_WRITE_CLIENT_SECRET"
start oya-cf-access-cr "$READ_HOST" "$READ_IP" "$READ_PORT" \
  "$CF_ACCESS_READ_CLIENT_ID" "$CF_ACCESS_READ_CLIENT_SECRET"

# Wait until TLS handshake to origin succeeds through the forwarder
for i in 1 2 3 4 5 6 7 8 9 10; do
  if openssl s_client -connect "127.0.0.1:${READ_PORT}" -servername "$READ_HOST" </dev/null 2>/dev/null \
    | grep -q 'BEGIN CERTIFICATE'; then
    echo "access-tcp ready"
    exit 0
  fi
  sleep 1
done
echo "access-tcp failed to become ready" >&2
docker logs oya-cf-access-cr 2>&1 | sed -E 's/(Secret|secret)[=:].*/\1=REDACTED/g' | tail -n 40 || true
exit 1
