#!/usr/bin/env bash
# Stop Access TCP forwarders started by start-access-tcp.sh
set -euo pipefail
docker rm -f oya-cf-access-cw oya-cf-access-cr >/dev/null 2>&1 || true
echo "stopped oya-cf-access-cw oya-cf-access-cr"
