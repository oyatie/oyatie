#!/usr/bin/env bash
# uninstall.sh — k3s is retired per ADR-0119. If k3s was installed via the
# tombstoned install.sh OR via the upstream `curl get.k3s.io | sh` path,
# this script delegates to the upstream uninstaller and clears the ZFS dataset.
set -uo pipefail
if [ -x /usr/local/bin/k3s-uninstall.sh ]; then
  /usr/local/bin/k3s-uninstall.sh
fi
if zfs list oyatie-bulk/srv/k3s >/dev/null 2>&1; then
  zfs destroy -r oyatie-bulk/srv/k3s 2>/dev/null || rm -rf /srv/oyatie/k3s
fi
rm -rf /etc/rancher /var/lib/rancher /srv/oyatie/k3s 2>/dev/null || true
echo "k3s residue cleared."
