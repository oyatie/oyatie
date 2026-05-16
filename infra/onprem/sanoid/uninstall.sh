#!/usr/bin/env bash
# uninstall.sh — reverse of sanoid/install.sh.
set -uo pipefail
systemctl disable --now sanoid.timer sanoid.service 2>/dev/null || true
apt-get -y remove --purge sanoid 2>/dev/null || true
rm -rf /etc/sanoid
echo "sanoid uninstalled."
