#!/usr/bin/env bash
# uninstall.sh — reverse of kubeadm/install.sh. Tears down the cluster cleanly.
set -uo pipefail
PURGE=${PURGE:-0}
kubeadm reset -f 2>/dev/null || true
apt-mark unhold kubeadm kubelet kubectl 2>/dev/null || true
apt-get -y remove --purge kubeadm kubelet kubectl 2>/dev/null || true
apt-get -y autoremove --purge 2>/dev/null || true
rm -f /etc/apt/sources.list.d/kubernetes.list /etc/apt/keyrings/kubernetes-apt-keyring.gpg
rm -f /etc/modules-load.d/k8s.conf /etc/sysctl.d/99-k8s.conf
rm -rf /etc/kubernetes /var/lib/kubelet /var/lib/dockershim /var/lib/etcd /etc/cni /opt/cni
rm -rf "$HOME/.kube" 2>/dev/null || true
REAL_USER=${SUDO_USER:-oyatie}
[ -n "$REAL_USER" ] && rm -rf "/home/$REAL_USER/.kube" 2>/dev/null || true
if [ "$PURGE" = "1" ]; then
  zfs destroy -r oyatie-bulk/srv/k3s 2>/dev/null || true
  rm -rf /srv/oyatie/k3s
fi
echo "kubeadm + cluster uninstalled."
