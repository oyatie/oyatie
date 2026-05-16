#!/usr/bin/env bash
# uninstall.sh — reverse of podman/install.sh.
set -uo pipefail
REAL_USER=${SUDO_USER:-oyatie}
sudo -u "$REAL_USER" -H bash -c 'systemctl --user disable --now podman.socket 2>/dev/null || true'
loginctl disable-linger "$REAL_USER" 2>/dev/null || true
apt-get -y remove --purge podman buildah skopeo uidmap slirp4netns fuse-overlayfs 2>/dev/null || true
apt-get -y autoremove --purge 2>/dev/null || true
echo "podman uninstalled."
