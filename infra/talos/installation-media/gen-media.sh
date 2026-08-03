#!/usr/bin/env bash
# gen-media.sh — build Talos installation media and enforce bootstrap PKI custody guardrails.
#
#   control-plane : config BAKED into the ISO (offline). Boot the control plane machine(s) -> auto-install ->
#          the CAPI management cluster forms itself. Needs CONTROLPLANE_ENDPOINT.
#   node : GENERIC image (Kata baked, no secrets). Boots with talos.config=$CONFIG_URL.
#          Production worker zero-touch also requires a hub config-serving plane that maps
#          attested MAC/UUID identity to generated Talos machineconfig; until then this is
#          experimental for more than hand-assigned nodes.
#   verify-backup : verify an operator-created sealed-backup receipt for secrets/secrets.yaml.
#   shred : remove bootstrap ISO artifacts from _out after the control plane is verified up.
#   rotate-pki : print the bounded compromise-rotation workflow without exposing secrets.
#
# Cloud spokes (OCI/AWS) are NOT built here — CAPI provisions them with platform images.
#
# Uses the Talos `imager` container (no Image Factory round-trip; bakes extensions + config).
# Output ISO -> write to install media (USB stick):  sudo dd if=_out/<preset>-metal-amd64.iso of=/dev/sdX bs=4M oflag=sync status=progress
set -euo pipefail

SCRIPT_PATH="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
HERE="${OYATIE_INSTALL_MEDIA_HOME:-$(cd "$(dirname "$0")" && pwd)}"
OUT="${OYATIE_INSTALL_MEDIA_OUT:-$HERE/_out}"
SECRETS="${OYATIE_INSTALL_MEDIA_SECRETS:-$HERE/secrets}"

TALOS_VERSION="${TALOS_VERSION:-v1.13.3}"
K8S_VERSION="${K8S_VERSION:-1.36.1}"
ARCH="${ARCH:-amd64}"
INSTALL_DISK="${INSTALL_DISK:-/dev/sda}"
INSTALL_IMAGE="${INSTALL_IMAGE:-ghcr.io/siderolabs/installer:${TALOS_VERSION}}"
IMAGER="ghcr.io/siderolabs/imager:${TALOS_VERSION}"

usage() {
  cat <<'USAGE'
Usage:
  gen-media.sh control-plane [--backup-receipt <path>] [--dry-run]
  gen-media.sh node [--dry-run]
  gen-media.sh verify-backup [--receipt <path>]
  gen-media.sh shred --confirm-control-plane-up [--dry-run] [--all|<iso>...]
  gen-media.sh rotate-pki --dry-run --ack-compromise --confirm-destructive-rebootstrap
  gen-media.sh self-test

Custody guardrails:
  - control-plane re-generation reuses secrets/secrets.yaml only after verify-backup passes.
  - backup receipts must prove the sealed backup matches secrets/secrets.yaml by sha256, while keeping
    the receipt itself free of secret values.
  - shred only accepts ISO paths under _out and refuses symlinks / non-ISO paths.
  - rotate-pki is an explicit dry-run runbook because compromise recovery is destructive re-bootstrap.
USAGE
}

die() {
  printf 'ERROR: %s\n' "$*" >&2
  exit 1
}

init_dirs() {
  mkdir -p "$OUT" "$SECRETS"
  chmod 700 "$SECRETS"
}

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    die "sha256sum or shasum is required to verify PKI backup receipts"
  fi
}

receipt_field() {
  local key="$1"
  local file="$2"
  local line_key line_value

  while IFS='=' read -r line_key line_value || [ -n "${line_key:-}" ]; do
    [ "$line_key" = "$key" ] || continue
    printf '%s\n' "$line_value"
    return 0
  done <"$file"

  return 1
}

validate_backup_receipt_shape() {
  local file="$1"
  local line_key line_value line_no=0

  while IFS='=' read -r line_key line_value || [ -n "${line_key:-}" ]; do
    line_no=$((line_no + 1))
    case "$line_key" in
      created_at|sealed_backup_uri|secrets_sha256)
        [ -n "${line_value:-}" ] || die "backup receipt field $line_key must not be empty"
        ;;
      *)
        die "backup receipt contains unsupported field at line $line_no: $line_key"
        ;;
    esac
  done <"$file"
}

verify_backup_receipt() {
  local receipt="${1:-}"
  local secrets_file="$SECRETS/secrets.yaml"
  local expected actual sealed_uri created_at

  init_dirs
  [ -f "$secrets_file" ] || die "cannot verify backup: $secrets_file is absent"
  if [ -z "$receipt" ]; then
    receipt="${PKI_BACKUP_RECEIPT:-$SECRETS/secrets.yaml.backup.receipt}"
  fi
  [ -f "$receipt" ] || die "backup receipt not found: $receipt"
  validate_backup_receipt_shape "$receipt"

  expected="$(receipt_field secrets_sha256 "$receipt" || true)"
  sealed_uri="$(receipt_field sealed_backup_uri "$receipt" || true)"
  created_at="$(receipt_field created_at "$receipt" || true)"
  [[ "$expected" =~ ^[0-9a-f]{64}$ ]] || die "backup receipt must contain lowercase hex secrets_sha256=<sha256>"
  [ -n "$sealed_uri" ] || die "backup receipt must contain sealed_backup_uri=<vault-or-envelope-id>"
  [ -n "$created_at" ] || die "backup receipt must contain created_at=<timestamp>"

  actual="$(sha256_file "$secrets_file")"
  [ "$actual" = "$expected" ] || die "backup receipt digest does not match secrets/secrets.yaml"

  printf 'Verified sealed backup receipt for secrets/secrets.yaml (digest redacted; receipt=%s).\n' "$receipt"
}

parse_receipt_flag() {
  local __result_var="$1"
  shift
  local parsed_receipt="${PKI_BACKUP_RECEIPT:-}"

  while [ "$#" -gt 0 ]; do
    case "$1" in
      --receipt|--backup-receipt)
        [ "$#" -ge 2 ] || die "$1 requires a path"
        parsed_receipt="$2"
        shift 2
        ;;
      --help|-h)
        usage
        exit 0
        ;;
      *)
        die "unknown verify-backup flag: $1"
        ;;
    esac
  done

  printf -v "$__result_var" '%s' "$parsed_receipt"
}

run_verify_backup() {
  local receipt
  parse_receipt_flag receipt "$@"
  verify_backup_receipt "$receipt"
}

# imager needs /dev + privileged to build images; outputs to the mounted /out.
imager() { docker run --rm -t --privileged -v /dev:/dev -v "$OUT:/out" -v "$SECRETS:/secrets:ro" "$IMAGER" "$@"; }

run_control_plane() {
  local dry_run=0
  local backup_receipt="${PKI_BACKUP_RECEIPT:-}"
  local had_secrets=0

  while [ "$#" -gt 0 ]; do
    case "$1" in
      --backup-receipt)
        [ "$#" -ge 2 ] || die "--backup-receipt requires a path"
        backup_receipt="$2"
        shift 2
        ;;
      --dry-run)
        dry_run=1
        shift
        ;;
      --help|-h)
        usage
        exit 0
        ;;
      *)
        die "unknown control-plane flag: $1"
        ;;
    esac
  done

  init_dirs
  : "${CONTROLPLANE_ENDPOINT:?set CONTROLPLANE_ENDPOINT=https://<control-plane-ip-or-vip>:6443}"
  CLUSTER="${CLUSTER:-oya-control-plane}"

  if [ -f "$SECRETS/secrets.yaml" ]; then
    had_secrets=1
    verify_backup_receipt "$backup_receipt"
  else
    printf 'No existing secrets/secrets.yaml found; first generation will create bootstrap PKI on this host.\n'
    printf 'Before any future re-generation, create a sealed backup and run: %s verify-backup --receipt <receipt>\n' "$0"
  fi

  if [ "$dry_run" -eq 1 ]; then
    MEDIA_DRY_RUN=1
    printf 'DRY-RUN control-plane: would render Talos control-plane config and build %s/control-plane-metal-%s.iso.\n' "$OUT" "$ARCH"
    [ "$had_secrets" -eq 1 ] && printf 'DRY-RUN guard: existing bootstrap PKI backup receipt verified before re-generation.\n'
    return 0
  fi

  # Stable cluster secrets (reused across re-gen so the baked PKI is consistent).
  [ -f "$SECRETS/secrets.yaml" ] || talosctl gen secrets -o "$SECRETS/secrets.yaml"
  talosctl gen config "$CLUSTER" "$CONTROLPLANE_ENDPOINT" \
    --with-secrets "$SECRETS/secrets.yaml" \
    --kubernetes-version "$K8S_VERSION" \
    --talos-version "$TALOS_VERSION" \
    --install-disk "$INSTALL_DISK" \
    --install-image "$INSTALL_IMAGE" \
    --output-types controlplane \
    --output "$SECRETS/control-plane-config.yaml" \
    --config-patch "@$HERE/patches/control-plane.yaml" \
    --force
  echo ">> building control-plane ISO (vanilla + embedded config)"
  imager iso --arch "$ARCH" --embedded-config-path /secrets/control-plane-config.yaml
  mv -f "$OUT/metal-${ARCH}.iso" "$OUT/control-plane-metal-${ARCH}.iso" 2>/dev/null || true

  if [ "$had_secrets" -eq 0 ]; then
    printf '\nWARNING: new bootstrap PKI was generated. Seal a backup, write a receipt, and run verify-backup before any re-generation.\n' >&2
  fi
}

run_node() {
  local dry_run=0

  while [ "$#" -gt 0 ]; do
    case "$1" in
      --dry-run)
        dry_run=1
        shift
        ;;
      --help|-h)
        usage
        exit 0
        ;;
      *)
        die "unknown node flag: $1"
        ;;
    esac
  done

  init_dirs
  : "${CONFIG_URL:?set CONFIG_URL=https://join.oyatie.dev/config (hub config-serving endpoint; production zero-touch requires MAC/UUID assignment registry)}"
  if [ "$dry_run" -eq 1 ]; then
    MEDIA_DRY_RUN=1
    printf 'DRY-RUN node: would request Image Factory schematic with talos.config=%s and write %s/node-metal-%s.iso.\n' "$CONFIG_URL" "$OUT" "$ARCH"
    printf 'DRY-RUN node: production zero-touch still requires node_config_serving_plane MAC/UUID assignment evidence; no endpoint is contacted.\n'
    return 0
  fi

  # No secrets in the node image (config is FETCHED), so build via Image Factory — it
  # resolves the Kata extension version for the Talos release automatically. The schematic
  # bakes the Kata extension + a talos.config kernel arg pointing at the hub config-serving endpoint.
  echo ">> requesting Image Factory schematic (kata + talos.config=$CONFIG_URL)"
  SCHEMATIC_DOC=$(printf 'customization:\n  systemExtensions:\n    officialExtensions:\n      - siderolabs/kata-containers\n  extraKernelArgs:\n    - talos.config=%s\n' "$CONFIG_URL")
  SCHEMATIC_ID=$(curl -fsSL -X POST --data-binary "$SCHEMATIC_DOC" https://factory.talos.dev/schematics \
                 | sed -E 's/.*"id":"([a-f0-9]+)".*/\1/')
  [ -n "$SCHEMATIC_ID" ] || { echo "schematic POST failed" >&2; exit 1; }
  echo "   schematic id: $SCHEMATIC_ID"
  echo ">> downloading node ISO from Image Factory"
  curl -fSL -o "$OUT/node-metal-${ARCH}.iso" \
    "https://factory.talos.dev/image/${SCHEMATIC_ID}/${TALOS_VERSION}/metal-${ARCH}.iso"
}

resolve_iso_target() {
  local candidate="$1"
  local out_real dir base full

  if [[ "$candidate" != */* ]]; then
    candidate="$OUT/$candidate"
  fi
  [ -e "$candidate" ] || die "ISO target not found: $candidate"
  [ -f "$candidate" ] || die "ISO target is not a regular file: $candidate"
  [ ! -L "$candidate" ] || die "refusing to shred symlink: $candidate"
  [[ "$candidate" = *.iso ]] || die "refusing non-ISO target: $candidate"

  out_real="$(cd "$OUT" && pwd -P)"
  dir="$(cd "$(dirname "$candidate")" && pwd -P)"
  base="$(basename "$candidate")"
  full="$dir/$base"
  case "$full" in
    "$out_real"/*.iso) printf '%s\n' "$full" ;;
    *) die "refusing to shred outside _out: $candidate" ;;
  esac
}

secure_delete_iso() {
  local target="$1"
  local dry_run="$2"

  if [ "$dry_run" -eq 1 ]; then
    printf 'DRY-RUN shred: would overwrite/unlink %s\n' "$target"
    return 0
  fi

  if command -v shred >/dev/null 2>&1; then
    shred -f -u -z "$target"
  elif command -v srm >/dev/null 2>&1; then
    srm -sz "$target"
  elif rm -P -f "$target" 2>/dev/null; then
    :
  else
    die "no secure deletion primitive found (need shred, srm, or rm -P); move _out to encrypted media and crypto-erase it"
  fi
  printf 'Removed bootstrap ISO artifact: %s\n' "$target"
}

run_shred() {
  local dry_run=0
  local confirmed=0
  local all=0
  local targets=()
  local resolved=()
  local target

  init_dirs
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --dry-run)
        dry_run=1
        shift
        ;;
      --confirm-control-plane-up)
        confirmed=1
        shift
        ;;
      --all)
        all=1
        shift
        ;;
      --help|-h)
        usage
        exit 0
        ;;
      --*)
        die "unknown shred flag: $1"
        ;;
      *)
        targets+=("$1")
        shift
        ;;
    esac
  done

  [ "$confirmed" -eq 1 ] || [ "${CONTROL_PLANE_UP_CONFIRMED:-0}" = "1" ] || die "refusing shred until --confirm-control-plane-up is supplied"
  if [ "$all" -eq 1 ]; then
    for target in "$OUT"/*.iso; do
      [ -e "$target" ] || continue
      targets+=("$target")
    done
  fi
  [ "${#targets[@]}" -gt 0 ] || die "shred requires --all or at least one ISO target under _out"

  for target in "${targets[@]}"; do
    resolved+=("$(resolve_iso_target "$target")")
  done
  for target in "${resolved[@]}"; do
    secure_delete_iso "$target" "$dry_run"
  done
  printf 'Custody note: SSD/APFS recovery guarantees depend on the encrypted volume. For high-trust cells, use encrypted scratch media and destroy the key or physically destroy removable media.\n'
}

run_rotate_pki() {
  local dry_run=0
  local ack=0
  local confirm=0

  while [ "$#" -gt 0 ]; do
    case "$1" in
      --dry-run)
        dry_run=1
        shift
        ;;
      --ack-compromise)
        ack=1
        shift
        ;;
      --confirm-destructive-rebootstrap)
        confirm=1
        shift
        ;;
      --help|-h)
        usage
        exit 0
        ;;
      *)
        die "unknown rotate-pki flag: $1"
        ;;
    esac
  done

  [ "$dry_run" -eq 1 ] || die "rotate-pki is a dry-run runbook only; pass --dry-run"
  [ "$ack" -eq 1 ] || die "rotate-pki requires --ack-compromise"
  [ "$confirm" -eq 1 ] || die "rotate-pki requires --confirm-destructive-rebootstrap"

  cat <<'RUNBOOK'
PKI rotation-on-compromise workflow (no secret values printed):
  1. Treat every existing control-plane ISO, USB stick, secrets/secrets.yaml, and local backup as compromised.
  2. Isolate affected artifacts. Run `gen-media.sh shred --confirm-control-plane-up --all` for _out/*.iso
     once the replacement control plane is reachable, and physically destroy removable media for high-trust cells.
  3. Preserve incident/audit evidence outside this repo. Do not commit receipts, ISO files, or secret material.
  4. Move compromised secrets out of service only after operator approval for destructive re-bootstrap.
  5. Generate fresh bootstrap PKI with `gen-media.sh control-plane`, seal a new backup, and verify it with
     `gen-media.sh verify-backup --receipt <receipt>` before any later re-generation.
  6. Rebuild/reflash control-plane media, re-bootstrap the management cluster, then let CAPI + Argo CD reconcile
     nodes/apps through the normal zero-touch path. Do not add manual SSH troubleshooting.
RUNBOOK
}

expect_success() {
  local name="$1"
  shift
  if "$@" >/dev/null 2>&1; then
    printf 'ok - %s\n' "$name"
  else
    printf 'not ok - %s\n' "$name" >&2
    return 1
  fi
}

expect_failure() {
  local name="$1"
  shift
  if "$@" >/dev/null 2>&1; then
    printf 'not ok - %s (unexpected success)\n' "$name" >&2
    return 1
  else
    printf 'ok - %s\n' "$name"
  fi
}

run_self_test() {
  local tmp receipt receipt_with_extra_field digest outside_iso

  tmp="$(mktemp -d "${TMPDIR:-/tmp}/oyatie-gen-media-test.XXXXXX")"
  SELF_TEST_TMP="$tmp"
  trap 'rm -rf "$SELF_TEST_TMP"' EXIT
  mkdir -p "$tmp/secrets" "$tmp/_out"
  chmod 700 "$tmp/secrets"
  printf 'fixture-bootstrap-secret\n' >"$tmp/secrets/secrets.yaml"
  digest="$(sha256_file "$tmp/secrets/secrets.yaml")"
  receipt="$tmp/secrets/secrets.yaml.backup.receipt"
  cat >"$receipt" <<EOF
created_at=2026-07-01T00:00:00Z
sealed_backup_uri=age://fixture/off-host-envelope
secrets_sha256=$digest
EOF

  expect_success "verify-backup accepts matching sealed-backup receipt" \
    env OYATIE_INSTALL_MEDIA_HOME="$tmp" bash "$SCRIPT_PATH" verify-backup --receipt "$receipt"
  expect_failure "verify-backup rejects missing receipt" \
    env OYATIE_INSTALL_MEDIA_HOME="$tmp" bash "$SCRIPT_PATH" verify-backup --receipt "$tmp/missing.receipt"
  receipt_with_extra_field="$tmp/secrets/secrets.yaml.bad.receipt"
  cp "$receipt" "$receipt_with_extra_field"
  printf 'secret_value=do-not-accept-extra-fields\n' >>"$receipt_with_extra_field"
  expect_failure "verify-backup rejects receipt fields outside the metadata allowlist" \
    env OYATIE_INSTALL_MEDIA_HOME="$tmp" bash "$SCRIPT_PATH" verify-backup --receipt "$receipt_with_extra_field"
  expect_failure "control-plane dry-run refuses existing PKI without backup receipt" \
    env CONTROLPLANE_ENDPOINT="https://127.0.0.1:6443" PKI_BACKUP_RECEIPT="$tmp/missing.receipt" OYATIE_INSTALL_MEDIA_HOME="$tmp" bash "$SCRIPT_PATH" control-plane --dry-run
  expect_success "control-plane dry-run accepts verified backup receipt" \
    env CONTROLPLANE_ENDPOINT="https://127.0.0.1:6443" OYATIE_INSTALL_MEDIA_HOME="$tmp" bash "$SCRIPT_PATH" control-plane --backup-receipt "$receipt" --dry-run

  printf 'fake iso bytes\n' >"$tmp/_out/test.iso"
  outside_iso="$tmp/outside.iso"
  printf 'fake iso bytes\n' >"$outside_iso"
  expect_success "shred dry-run accepts ISO under _out with confirmation" \
    env OYATIE_INSTALL_MEDIA_HOME="$tmp" bash "$SCRIPT_PATH" shred --dry-run --confirm-control-plane-up test.iso
  expect_failure "shred dry-run refuses ISO outside _out" \
    env OYATIE_INSTALL_MEDIA_HOME="$tmp" bash "$SCRIPT_PATH" shred --dry-run --confirm-control-plane-up "$outside_iso"
  expect_success "rotate-pki dry-run requires explicit compromise acknowledgements" \
    bash "$SCRIPT_PATH" rotate-pki --dry-run --ack-compromise --confirm-destructive-rebootstrap
  expect_failure "rotate-pki refuses missing destructive acknowledgement" \
    bash "$SCRIPT_PATH" rotate-pki --dry-run --ack-compromise

  printf 'self-test passed\n'
}

finish_media_command() {
  local preset="$1"

  echo
  echo "DONE. Image(s) in $OUT:"
  ls -lh "$OUT"/*.iso 2>/dev/null | awk '{print "  "$5,$NF}' || true
  echo "Write to install media (USB stick):  sudo dd if=$OUT/${preset}-metal-${ARCH}.iso of=/dev/sdX bs=4M oflag=sync status=progress"
}

COMMAND="${1:-}"
[ -n "$COMMAND" ] || { usage >&2; exit 2; }
shift || true

case "$COMMAND" in
  control-plane)
    run_control_plane "$@"
    [ "${MEDIA_DRY_RUN:-0}" -eq 1 ] || finish_media_command control-plane
    ;;
  node)
    run_node "$@"
    [ "${MEDIA_DRY_RUN:-0}" -eq 1 ] || finish_media_command node
    ;;
  verify-backup)
    run_verify_backup "$@"
    ;;
  shred)
    run_shred "$@"
    ;;
  rotate-pki)
    run_rotate_pki "$@"
    ;;
  self-test)
    run_self_test
    ;;
  --help|-h|help)
    usage
    ;;
  *)
    usage >&2
    die "unknown command: $COMMAND"
    ;;
esac
