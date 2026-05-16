#!/usr/bin/env bash
# uninstall.sh — reverse of containerd/install.sh.
set -uo pipefail
systemctl disable --now containerd.service 2>/dev/null || true
rm -f /etc/systemd/system/containerd.service
systemctl daemon-reload
rm -f /usr/local/bin/containerd /usr/local/bin/containerd-shim* /usr/local/bin/ctr /usr/local/bin/runc
rm -f /usr/local/sbin/runc
rm -rf /opt/cni/bin /etc/containerd
echo "containerd + runc + CNI plugins uninstalled."
