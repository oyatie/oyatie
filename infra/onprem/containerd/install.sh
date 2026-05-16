#!/usr/bin/env bash
# install.sh — containerd CRI runtime for the on-prem KR primary cell.
# Authority: ADR-0119.
# Run as: sudo bash /home/oyatie/projects/oyatie/infra/onprem/containerd/install.sh
set -euo pipefail

CONTAINERD_VERSION=${CONTAINERD_VERSION:-2.3.0}     # first annual LTS (Apr 2026)
RUNC_VERSION=${RUNC_VERSION:-1.4.0}                 # current stable (containerd 2.3.0 LTS-bundled track)
CNI_PLUGINS_VERSION=${CNI_PLUGINS_VERSION:-1.6.0}   # current stable

echo "==> Step 1/6: prereqs"
apt-get update
apt-get install -y --no-install-recommends ca-certificates curl gnupg

echo "==> Step 2/6: install containerd $CONTAINERD_VERSION"
curl -fsSL "https://github.com/containerd/containerd/releases/download/v${CONTAINERD_VERSION}/containerd-${CONTAINERD_VERSION}-linux-amd64.tar.gz" \
  | tar -C /usr/local -xz

mkdir -p /usr/local/lib/systemd/system
curl -fsSL https://raw.githubusercontent.com/containerd/containerd/main/containerd.service \
  -o /etc/systemd/system/containerd.service

echo "==> Step 3/6: install runc $RUNC_VERSION"
curl -fsSL "https://github.com/opencontainers/runc/releases/download/v${RUNC_VERSION}/runc.amd64" \
  -o /usr/local/sbin/runc
chmod 0755 /usr/local/sbin/runc

echo "==> Step 4/6: install CNI plugins $CNI_PLUGINS_VERSION"
mkdir -p /opt/cni/bin
curl -fsSL "https://github.com/containernetworking/plugins/releases/download/v${CNI_PLUGINS_VERSION}/cni-plugins-linux-amd64-v${CNI_PLUGINS_VERSION}.tgz" \
  | tar -C /opt/cni/bin -xz

echo "==> Step 5/6: containerd config (systemd cgroup driver — required for kubelet parity)"
mkdir -p /etc/containerd
containerd config default > /etc/containerd/config.toml
sed -i 's/SystemdCgroup = false/SystemdCgroup = true/' /etc/containerd/config.toml

echo "==> Step 6/6: enable + start"
systemctl daemon-reload
systemctl enable --now containerd
sleep 2
systemctl --no-pager status containerd | head -10 || true
/usr/local/bin/containerd --version
/usr/local/sbin/runc --version | head -3

echo "==> done. Ready for kubeadm."
