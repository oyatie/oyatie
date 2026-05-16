#!/usr/bin/env bash
# install.sh — install Istio (Envoy data plane) on top of the kubeadm cluster.
# Authority: ADR-0119 (kubeadm + containerd + Istio + Envoy).
# Per ADR-0044: service-mesh mTLS posture starts permissive, graduates to strict.
# Prereq: ../containerd/install.sh + ../kubeadm/install.sh have completed.
#
# Run as: sudo bash /home/oyatie/projects/oyatie/infra/onprem/istio/install.sh
set -euo pipefail

REAL_USER=${SUDO_USER:-oyatie}
# Istio 1.29.x is the current supported minor (released 2026-02-16; latest patch
# 1.29.2 Apr 2026). 1.27 support ended 2026-04-30; 1.23/1.24/1.25/1.26 are EOL.
# Per https://istio.io/latest/docs/releases/supported-releases/.
ISTIO_VERSION=${ISTIO_VERSION:-1.29.2}
ISTIO_DIR=${ISTIO_DIR:-/opt/istio}

# Precondition: the kubeadm cluster's API server must be reachable.
# Without this check, `istioctl install` partial-applies then dies on timeout.
if ! sudo -u "$REAL_USER" -H kubectl --kubeconfig="/home/$REAL_USER/.kube/config" cluster-info >/dev/null 2>&1; then
  echo "ERROR: Kubernetes API not reachable from $REAL_USER's kubeconfig." >&2
  echo "  Run first:  sudo bash /home/oyatie/projects/oyatie/infra/onprem/containerd/install.sh" >&2
  echo "  Then:       sudo bash /home/oyatie/projects/oyatie/infra/onprem/kubeadm/install.sh" >&2
  echo "  Then re-run this script." >&2
  exit 2
fi

echo "==> Step 1/4: download istioctl $ISTIO_VERSION"
if [ ! -x "$ISTIO_DIR/$ISTIO_VERSION/bin/istioctl" ]; then
  mkdir -p "$ISTIO_DIR"
  cd "$ISTIO_DIR"
  curl -fsSL "https://github.com/istio/istio/releases/download/$ISTIO_VERSION/istio-$ISTIO_VERSION-linux-amd64.tar.gz" | tar xz
  mv "istio-$ISTIO_VERSION" "$ISTIO_VERSION"
fi
# Istio's tarball ships restrictive perms; relax so non-root users can run
# istioctl (the kubeadm cluster admin user, sudo -u <user> below).
chmod 0755 "$ISTIO_DIR" "$ISTIO_DIR/$ISTIO_VERSION" "$ISTIO_DIR/$ISTIO_VERSION/bin"
chmod 0755 "$ISTIO_DIR/$ISTIO_VERSION/bin/istioctl"
ln -sfn "$ISTIO_DIR/$ISTIO_VERSION/bin/istioctl" /usr/local/bin/istioctl

echo "==> Step 2/4: install Istio on the kubeadm cluster (minimal profile)"
sudo -u "$REAL_USER" -H env PATH="/usr/local/bin:/usr/bin:/bin" KUBECONFIG="/home/$REAL_USER/.kube/config" \
  /usr/local/bin/istioctl install --set profile=minimal -y

echo "==> Step 3/4: label default namespace for sidecar injection"
sudo -u "$REAL_USER" -H env PATH="/usr/local/bin:/usr/bin:/bin" KUBECONFIG="/home/$REAL_USER/.kube/config" \
  /usr/local/bin/kubectl label namespace default istio-injection=enabled --overwrite

echo "==> Step 4/4: smoke"
sudo -u "$REAL_USER" -H env PATH="/usr/local/bin:/usr/bin:/bin" KUBECONFIG="/home/$REAL_USER/.kube/config" bash -c '
  /usr/local/bin/kubectl get pods -n istio-system
  /usr/local/bin/istioctl version
'

echo "==> done. Sidecar injection on namespace=default."
