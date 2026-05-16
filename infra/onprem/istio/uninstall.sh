#!/usr/bin/env bash
# uninstall.sh — reverse of istio/install.sh.
set -uo pipefail
REAL_USER=${SUDO_USER:-oyatie}
if [ -f "/home/$REAL_USER/.kube/config" ] && [ -x /usr/local/bin/istioctl ]; then
  sudo -u "$REAL_USER" -H env KUBECONFIG="/home/$REAL_USER/.kube/config" \
    /usr/local/bin/istioctl uninstall --purge -y 2>/dev/null || true
  sudo -u "$REAL_USER" -H env KUBECONFIG="/home/$REAL_USER/.kube/config" \
    /usr/local/bin/kubectl delete namespace istio-system 2>/dev/null || true
  sudo -u "$REAL_USER" -H env KUBECONFIG="/home/$REAL_USER/.kube/config" \
    /usr/local/bin/kubectl label namespace default istio-injection- 2>/dev/null || true
fi
rm -f /usr/local/bin/istioctl
rm -rf /opt/istio
echo "istio uninstalled."
