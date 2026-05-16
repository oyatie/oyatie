#!/usr/bin/env bash
# install.sh — install Podman on Debian 13 with rootless config for oyatie user.
# Run as: sudo bash /home/oyatie/projects/oyatie/infra/onprem/podman/install.sh
set -euo pipefail

REAL_USER=${SUDO_USER:-oyatie}

echo "==> Step 1/4: apt install podman + helpers"
apt-get update
apt-get install -y --no-install-recommends \
  podman \
  buildah \
  skopeo \
  uidmap \
  slirp4netns \
  fuse-overlayfs \
  ca-certificates

echo "==> Step 2/4: enable lingering for $REAL_USER (rootless socket survives logout)"
loginctl enable-linger "$REAL_USER" || true

echo "==> Step 3/4: enable user podman socket (Docker-compat API at unix:/run/user/<uid>/podman/podman.sock)"
sudo -u "$REAL_USER" -H bash -c 'systemctl --user daemon-reload; systemctl --user enable --now podman.socket'

echo "==> Step 4/4: smoke + version"
sudo -u "$REAL_USER" -H bash -c 'podman version | head -10; podman info --format "{{.Host.Conmon.Path}}"; podman run --rm docker.io/library/hello-world 2>&1 | tail -10 || true'

echo "==> done. Rootless socket: /run/user/$(id -u "$REAL_USER")/podman/podman.sock"
