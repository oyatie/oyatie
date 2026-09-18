#!/usr/bin/env bash
# Live FoundationDB lane for the mail product.
#
# Starts one pinned FoundationDB server container, gives the Rust builder a
# libfdb_c from the same release, writes a cluster file, and runs the live
# lane command once per crate in LANE_CRATES. Stops only the container it
# started. Runs unchanged on a GitHub runner and on a developer machine:
#
#     OYATIE_MAIL_LIVE_FDB=1 .github/scripts/live-fdb.sh
#
# Requirements: docker, cargo + nextest, curl (macOS only, for the client pkg).
# Optional: OYATIE_MAIL_FDB_PORT (default 4500), OYATIE_MAIL_LIBFDB_DIR to
# point at an already-installed libfdb_c instead of provisioning one.
set -euo pipefail

if [ "${OYATIE_MAIL_LIVE_FDB:-}" != "1" ]; then
  echo "refusing: OYATIE_MAIL_LIVE_FDB=1 is required to run the live FoundationDB lane" >&2
  exit 2
fi

# --- Pins -------------------------------------------------------------------
# foundationdb/foundationdb:7.3.79, multi-arch OCI index (linux/amd64 needs AVX,
# linux/arm64). Pinned by the index digest so both runners resolve one release.
FDB_RELEASE="7.3.79"
FDB_IMAGE="foundationdb/foundationdb@sha256:d3530c3066f94abffb61facac527c9c3517f6553ee0e75efa69d54296290156a"
# macOS client packages from the same GitHub release, for developer machines.
FDB_MACOS_ARM64_SHA256="5104ade94d1e1b62119f49e3e16d43bd9ffb8b5ec604b1730f46f680c0c1890e"
FDB_MACOS_X86_64_SHA256="0bcd0f9430984ab72d7ba47f8ed95c85f14d237f3878ab1521feff14e074dbc4"

# --- Lane rows: one `-p <crate> [--features fdb]` per crate ------------------
# Extended by later steps in lock-step with .github/workflows/live-fdb.yml.
LANE_CRATES=(
  "mail-store-fdb --features fdb"
)

PORT="${OYATIE_MAIL_FDB_PORT:-4500}"
CONTAINER="oyatie-live-fdb-$$"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/live-fdb.XXXXXX")"
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

cleanup() {
  docker rm -f "$CONTAINER" >/dev/null 2>&1 || true
  rm -rf "$WORK"
}
trap cleanup EXIT

sha256_check() {
  local expected="$1" file="$2" actual
  if command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "$file" | cut -d' ' -f1)"
  else
    actual="$(shasum -a 256 "$file" | cut -d' ' -f1)"
  fi
  if [ "$actual" != "$expected" ]; then
    echo "checksum mismatch for $file: want $expected got $actual" >&2
    exit 1
  fi
}

# --- Server ------------------------------------------------------------------
echo "starting FoundationDB $FDB_RELEASE as $CONTAINER on 127.0.0.1:$PORT"
docker run -d --name "$CONTAINER" \
  -p "127.0.0.1:${PORT}:${PORT}" \
  -e FDB_NETWORKING_MODE=host -e "FDB_PORT=${PORT}" \
  --entrypoint /usr/bin/tini "$FDB_IMAGE" -g -- /var/fdb/scripts/fdb_single.bash >/dev/null

FDB_CLUSTER_FILE="$WORK/fdb.cluster"
printf 'docker:docker@127.0.0.1:%s\n' "$PORT" > "$FDB_CLUSTER_FILE"
export FDB_CLUSTER_FILE

for attempt in $(seq 1 90); do
  if docker exec "$CONTAINER" fdbcli --exec 'status minimal' 2>/dev/null | grep -q 'The database is available'; then
    echo "database available after ${attempt}s"
    break
  fi
  if [ "$attempt" = 90 ]; then
    echo "FoundationDB did not become available; container logs follow" >&2
    docker logs "$CONTAINER" >&2 || true
    exit 1
  fi
  sleep 1
done

# --- Client library for the builder -----------------------------------------
LIBDIR="${OYATIE_MAIL_LIBFDB_DIR:-}"
if [ -z "$LIBDIR" ]; then
  LIBDIR="$WORK/lib"
  mkdir -p "$LIBDIR"
  case "$(uname -s)-$(uname -m)" in
    Linux-*)
      # Same release as the server: the image ships libfdb_c.so.
      docker cp "$CONTAINER:/usr/lib/libfdb_c.so" "$LIBDIR/libfdb_c.so"
      ;;
    Darwin-arm64 | Darwin-x86_64)
      arch="$(uname -m)"
      if [ "$arch" = "arm64" ]; then sum="$FDB_MACOS_ARM64_SHA256"; else sum="$FDB_MACOS_X86_64_SHA256"; fi
      pkg="$WORK/FoundationDB-${FDB_RELEASE}_${arch}.pkg"
      curl --fail --silent --show-error --location --retry 2 --connect-timeout 10 --max-time 300 \
        "https://github.com/apple/foundationdb/releases/download/${FDB_RELEASE}/FoundationDB-${FDB_RELEASE}_${arch}.pkg" \
        --output "$pkg"
      sha256_check "$sum" "$pkg"
      pkgutil --expand-full "$pkg" "$WORK/pkg"
      dylib="$(find "$WORK/pkg" -name 'libfdb_c.dylib' -type f | head -n 1)"
      test -n "$dylib" || { echo "libfdb_c.dylib not found in $pkg" >&2; exit 1; }
      cp "$dylib" "$LIBDIR/libfdb_c.dylib"
      ;;
    *)
      echo "unsupported host $(uname -s)-$(uname -m); set OYATIE_MAIL_LIBFDB_DIR to a libfdb_c directory" >&2
      exit 1
      ;;
  esac
fi
echo "libfdb_c from $LIBDIR"
export RUSTFLAGS="${RUSTFLAGS:-} -L native=${LIBDIR} -C link-arg=-Wl,-rpath,${LIBDIR}"
export LD_LIBRARY_PATH="${LIBDIR}${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export DYLD_LIBRARY_PATH="${LIBDIR}${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"

# --- Lane --------------------------------------------------------------------
cd "$REPO_ROOT"
for row in "${LANE_CRATES[@]}"; do
  # shellcheck disable=SC2086 # each row is a fixed `<crate> [--features fdb]` word list
  cargo nextest run --locked --profile live --run-ignored only --no-tests=error -p $row
done
echo "live FoundationDB lane passed for: ${LANE_CRATES[*]}"
