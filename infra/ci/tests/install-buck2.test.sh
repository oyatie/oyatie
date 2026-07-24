#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
installer="${repo_root}/infra/ci/install-buck2.sh"
fixture_dir="$(mktemp -d)"
mock_bin="${fixture_dir}/bin"
server_pid=""

cleanup() {
  if [ -n "${server_pid}" ]; then
    kill "${server_pid}" 2>/dev/null || true
    wait "${server_pid}" 2>/dev/null || true
  fi
  rm -rf "${fixture_dir}"
}
trap cleanup EXIT

mkdir -p "${mock_bin}"
printf '#!/usr/bin/env bash\necho "buck2 test fixture"\n' > "${fixture_dir}/buck2-payload"
payload_sha="$(sha256sum "${fixture_dir}/buck2-payload" | awk '{print $1}')"

cat > "${mock_bin}/curl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

printf '%s\n' "$@" > "${CURL_ARGS_LOG}"
args=()
for arg in "$@"; do
  case "${arg}" in
    https://github.com/facebook/buck2/releases/download/*)
      args+=("http://127.0.0.1:${BUCK2_TEST_PORT}/buck2-fixture")
      ;;
    *) args+=("${arg}") ;;
  esac
done
exec "${REAL_CURL}" "${args[@]}"
EOF

cat > "${mock_bin}/zstd" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

input=""
output=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    -o)
      output="$2"
      shift 2
      ;;
    -d|-f)
      shift
      ;;
    *)
      input="$1"
      shift
      ;;
  esac
done
cp "${input}" "${output}"
EOF
chmod +x "${mock_bin}/curl" "${mock_bin}/zstd"

python3 - "${fixture_dir}" <<'PY' > "${fixture_dir}/port" 2>"${fixture_dir}/server.log" &
import http.server
import pathlib
import socketserver
import sys

root = pathlib.Path(sys.argv[1])
attempts = 0

class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        global attempts
        attempts += 1
        (root / "attempt-count").write_text(str(attempts))
        if attempts <= 6:
            self.send_response(504)
            self.end_headers()
            self.wfile.write(b"temporary gateway failure\n")
            return
        payload = (root / "buck2-payload").read_bytes()
        self.send_response(200)
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def log_message(self, _format, *_args):
        pass

with socketserver.TCPServer(("127.0.0.1", 0), Handler) as server:
    print(server.server_address[1], flush=True)
    server.serve_forever()
PY
server_pid=$!

for _ in $(seq 1 50); do
  [ -s "${fixture_dir}/port" ] && break
  sleep 0.1
done
[ -s "${fixture_dir}/port" ] || { cat "${fixture_dir}/server.log" >&2; exit 1; }

buck2_test_port="$(cat "${fixture_dir}/port")"
real_curl="$(command -v curl)"
export BUCK2_TEST_PORT="${buck2_test_port}"
export REAL_CURL="${real_curl}"
export CURL_ARGS_LOG="${fixture_dir}/curl-args.log"
export PATH="${mock_bin}:${PATH}"

install_dir="${fixture_dir}/install"
BUCK2_INSTALL_DIR="${install_dir}" BUCK2_ASSET="buck2-fixture.zst" BUCK2_SHA256="${payload_sha}" "${installer}" >"${fixture_dir}/success.log" 2>&1
[ -x "${install_dir}/buck2" ]
[ "$(cat "${fixture_dir}/attempt-count")" = "7" ]
rg -Fx -- '--retry' "${CURL_ARGS_LOG}"
rg -Fx -- '8' "${CURL_ARGS_LOG}"
rg -Fx -- '--retry-all-errors' "${CURL_ARGS_LOG}"
rg -Fx -- '--retry-max-time' "${CURL_ARGS_LOG}"
rg -Fx -- '180' "${CURL_ARGS_LOG}"
rg -Fx -- '--connect-timeout' "${CURL_ARGS_LOG}"
rg -Fx -- '20' "${CURL_ARGS_LOG}"
rg -Fx -- '--max-time' "${CURL_ARGS_LOG}"
rg -Fx -- '60' "${CURL_ARGS_LOG}"
if rg -Fx -- '--retry-delay' "${CURL_ARGS_LOG}"; then
  echo "curl retry delay must remain unset to preserve exponential backoff" >&2
  exit 1
fi

# A fresh digest mismatch must fail closed and leave neither promoted asset nor partial files.
mismatch_dir="${fixture_dir}/mismatch"
if BUCK2_INSTALL_DIR="${mismatch_dir}" BUCK2_ASSET="buck2-fixture.zst" BUCK2_SHA256="$(printf '0%.0s' {1..64})" "${installer}" >"${fixture_dir}/mismatch.log" 2>&1; then
  echo "expected digest mismatch to fail" >&2
  exit 1
fi
[ ! -e "${mismatch_dir}/buck2-fixture.zst" ]
if compgen -G "${mismatch_dir}/buck2-fixture.zst.part.*" > /dev/null; then
  echo "digest mismatch left a release-asset partial" >&2
  exit 1
fi
if compgen -G "${mismatch_dir}/buck2.part.*" > /dev/null; then
  echo "digest mismatch left a decompressed-binary partial" >&2
  exit 1
fi

# A verified cache hit must not contact the network, even when the curl shim fails.
cat > "${mock_bin}/curl" <<'EOF'
#!/usr/bin/env bash
exit 99
EOF
chmod +x "${mock_bin}/curl"
BUCK2_INSTALL_DIR="${install_dir}" BUCK2_ASSET="buck2-fixture.zst" BUCK2_SHA256="${payload_sha}" "${installer}" >"${fixture_dir}/cache-hit.log" 2>&1
rg -q 'cache hit \(SHA-256 verified\)' "${fixture_dir}/cache-hit.log"

echo "install-buck2 retry/cache/digest regression checks passed"
