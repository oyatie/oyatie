#!/usr/bin/env bash
# Generate Omni's secret material OUT OF GIT and stage it next to the compose file at run time.
#   1) GPG key   — encrypts Omni's etcd-at-rest (no passphrase, as Omni requires).
#   2) TLS cert  — server-chain.pem + server-key.pem for the public endpoint.
#   3) OIDC secret — shared client secret between Omni and the bundled Dex.
#
# Secrets are written to ./secrets/ (gitignored) AND mirrored into the macOS Keychain so they can be
# re-materialised on a fresh host. On the OCI host, prefer OpenBao; Keychain mirroring is skipped off-darwin.
# Canonical secret discipline: ADR-0043 (no plaintext secrets in git) / ADR-0371-D5.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
SECRETS="$HERE/secrets"
mkdir -p "$SECRETS"
chmod 700 "$SECRETS"

[ -f "$HERE/.env" ] && set -a && . "$HERE/.env" && set +a
: "${OMNI_ENDPOINT:?set OMNI_ENDPOINT in .env}"
: "${OMNI_PUBLIC_IP:?set OMNI_PUBLIC_IP in .env}"

step() { printf '\n=== %s ===\n' "$*"; }

step "1/3 GPG key for etcd encryption (no passphrase — Omni requirement)"
if [ ! -f "$SECRETS/omni.asc" ]; then
  GNUPGHOME="$(mktemp -d)"; export GNUPGHOME
  gpg --batch --passphrase '' --quick-generate-key \
    "Omni (etcd data encryption) omni@oyatie.local" rsa4096 cert never
  FPR=$(gpg --with-colons --list-keys omni@oyatie.local | awk -F: '$1=="fpr"{print $10; exit}')
  gpg --batch --passphrase '' --quick-add-key "$FPR" rsa4096 encr never
  gpg --export-secret-key --armor omni@oyatie.local > "$SECRETS/omni.asc"
  rm -rf "$GNUPGHOME"; unset GNUPGHOME
  echo "  wrote secrets/omni.asc"
else echo "  secrets/omni.asc exists — keeping"; fi

step "2/3 TLS server cert for https://$OMNI_ENDPOINT"
if [ ! -f "$SECRETS/server-key.pem" ]; then
  echo "  >> PRODUCTION: drop a real cert here as server-chain.pem + server-key.pem"
  echo "     (Let's Encrypt for $OMNI_ENDPOINT, or a Cloudflare origin cert)."
  echo "  >> Generating a SELF-SIGNED cert for first-boot/testing only:"
  openssl req -x509 -newkey rsa:4096 -nodes -days 825 \
    -keyout "$SECRETS/server-key.pem" -out "$SECRETS/server-chain.pem" \
    -subj "/CN=$OMNI_ENDPOINT" \
    -addext "subjectAltName=DNS:$OMNI_ENDPOINT,IP:$OMNI_PUBLIC_IP"
  cp "$SECRETS/server-chain.pem" "$SECRETS/ca.pem"
  echo "  wrote self-signed secrets/{server-chain,server-key,ca}.pem"
else echo "  secrets/server-key.pem exists — keeping"; fi

step "3/3 OIDC client secret (Omni <-> Dex)"
if [ ! -f "$SECRETS/oidc-client-secret" ]; then
  openssl rand -hex 32 > "$SECRETS/oidc-client-secret"
  echo "  wrote secrets/oidc-client-secret"
else echo "  secrets/oidc-client-secret exists — keeping"; fi

if command -v security >/dev/null 2>&1; then
  step "mirror -> macOS Keychain (oyatie-omni-*)"
  for f in omni.asc server-key.pem server-chain.pem oidc-client-secret; do
    security add-generic-password -U -s "oyatie-omni-$f" -a omni -w "$(cat "$SECRETS/$f")" >/dev/null 2>&1 || true
  done
  echo "  mirrored. Recover with: security find-generic-password -s oyatie-omni-<file> -w"
fi

echo
echo "DONE. secrets/ is gitignored. Next: docker compose --env-file .env up -d"
