#!/usr/bin/env bash
# install.sh — vanilla Kubernetes via kubeadm on the on-prem KR primary cell.
# Authority: ADR-0118. Requires containerd already installed (see ../containerd/install.sh).
# Run as: sudo bash /home/oyatie/projects/oyatie/infra/onprem/kubeadm/install.sh
set -euo pipefail

REAL_USER=${SUDO_USER:-oyatie}
K8S_VERSION=${K8S_VERSION:-1.35}                            # N-1 (most-stable patch coverage; N=1.36, N-2=1.34)
POD_NETWORK_CIDR=${POD_NETWORK_CIDR:-10.244.0.0/16}         # flannel default; calico picks own
# kubeadm rejects loopback as advertise address — pick the primary non-loopback
# IPv4 (the route to the public internet decides). Override with
# APISERVER_ADVERTISE_ADDRESS=<ip> if a different NIC should host the API.
APISERVER_ADVERTISE_ADDRESS=${APISERVER_ADVERTISE_ADDRESS:-$(ip -4 route get 1.1.1.1 2>/dev/null | awk '/src/ {for (i=1;i<=NF;i++) if ($i=="src") print $(i+1)}')}
[ -n "${APISERVER_ADVERTISE_ADDRESS:-}" ] || APISERVER_ADVERTISE_ADDRESS=$(hostname -I | awk '{print $1}')
CNI_FLAVOR=${CNI_FLAVOR:-flannel}                           # or 'calico' for richer netpols

echo "==> using APISERVER_ADVERTISE_ADDRESS=$APISERVER_ADVERTISE_ADDRESS"

echo "==> Step 1/8: sysctl + kernel modules + iptables backend"
cat > /etc/modules-load.d/k8s.conf <<EOF
overlay
br_netfilter
EOF
modprobe overlay
modprobe br_netfilter

cat > /etc/sysctl.d/99-k8s.conf <<EOF
net.bridge.bridge-nf-call-iptables  = 1
net.bridge.bridge-nf-call-ip6tables = 1
net.ipv4.ip_forward                 = 1
EOF
sysctl --system >/dev/null

# Debian 13 (trixie) defaults `iptables` -> `iptables-nft`. kube-proxy 1.35's
# nftables mode segfaults on this kernel; iptables-nft shim leaves orphan rules
# from prior nft-mode attempts that break pod networking. Pin to iptables-legacy
# and flush any nftables ruleset that may have been programmed earlier.
if command -v update-alternatives >/dev/null && [ -x /usr/sbin/iptables-legacy ]; then
  update-alternatives --set iptables  /usr/sbin/iptables-legacy  >/dev/null 2>&1 || true
  update-alternatives --set ip6tables /usr/sbin/ip6tables-legacy >/dev/null 2>&1 || true
fi
if command -v nft >/dev/null; then
  nft flush ruleset 2>/dev/null || true
fi

echo "==> Step 2/8: disable swap (kubelet requirement)"
swapoff -a
sed -i '/\sswap\s/s/^/#/' /etc/fstab

echo "==> Step 3/8: apt repo for kubeadm/kubelet/kubectl v$K8S_VERSION"
apt-get update
apt-get install -y --no-install-recommends apt-transport-https ca-certificates curl gpg
mkdir -p -m 755 /etc/apt/keyrings
curl -fsSL "https://pkgs.k8s.io/core:/stable:/v${K8S_VERSION}/deb/Release.key" \
  | gpg --dearmor -o /etc/apt/keyrings/kubernetes-apt-keyring.gpg
echo "deb [signed-by=/etc/apt/keyrings/kubernetes-apt-keyring.gpg] https://pkgs.k8s.io/core:/stable:/v${K8S_VERSION}/deb/ /" \
  > /etc/apt/sources.list.d/kubernetes.list
apt-get update
apt-get install -y --no-install-recommends kubelet kubeadm kubectl
apt-mark hold kubelet kubeadm kubectl

echo "==> Step 4/8: kubeadm init (single-node control plane)"
kubeadm config images pull --kubernetes-version "v${K8S_VERSION}.0" >/dev/null 2>&1 || true
kubeadm init \
  --kubernetes-version="v${K8S_VERSION}.0" \
  --pod-network-cidr="$POD_NETWORK_CIDR" \
  --apiserver-advertise-address="$APISERVER_ADVERTISE_ADDRESS" \
  --cri-socket=unix:///var/run/containerd/containerd.sock

echo "==> Step 5/8: untaint control-plane (single-node cluster)"
kubectl --kubeconfig=/etc/kubernetes/admin.conf taint nodes --all node-role.kubernetes.io/control-plane- 2>/dev/null || true

echo "==> Step 6/8: install CNI ($CNI_FLAVOR)"
case "$CNI_FLAVOR" in
  flannel)
    kubectl --kubeconfig=/etc/kubernetes/admin.conf apply -f \
      https://raw.githubusercontent.com/flannel-io/flannel/master/Documentation/kube-flannel.yml
    ;;
  calico)
    kubectl --kubeconfig=/etc/kubernetes/admin.conf apply -f \
      https://raw.githubusercontent.com/projectcalico/calico/v3.28.1/manifests/calico.yaml
    ;;
  *) echo "unknown CNI_FLAVOR=$CNI_FLAVOR" >&2; exit 2;;
esac

echo "==> Step 7/8: kubeconfig for $REAL_USER"
sudo -u "$REAL_USER" -H bash -c '
  mkdir -p ~/.kube
  sudo cp /etc/kubernetes/admin.conf ~/.kube/config 2>/dev/null || true
  sudo chown $(id -u):$(id -g) ~/.kube/config 2>/dev/null || true
  chmod 0600 ~/.kube/config 2>/dev/null || true
'

# Fallback if the user can't sudo within the heredoc.
install -m 0644 /etc/kubernetes/admin.conf "/home/${REAL_USER}/.kube/config"
chown "$REAL_USER":"$REAL_USER" "/home/${REAL_USER}/.kube/config"

echo "==> Step 8/8: smoke"
sleep 8
kubectl --kubeconfig=/etc/kubernetes/admin.conf get nodes -o wide
kubectl --kubeconfig=/etc/kubernetes/admin.conf get pods -A | head -20

echo "==> done. kubeconfig at ~$REAL_USER/.kube/config. Next: bash ../istio/install.sh"
