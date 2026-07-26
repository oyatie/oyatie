#!/usr/bin/env bash
# Validate Kata cloud-hypervisor fidelity — the make-or-break nested-virt check.
#  1. /dev/kvm present on a Kata worker  -> nested virt reached the Linux guest.
#  2. A pod with runtimeClassName: kata-cloud-hypervisor reaches Running -> CLH booted a microVM
#     INSIDE the Talos node. This is the proof the whole substrate choice rests on.
set -euo pipefail
WORKDIR="${WORKDIR:-$HOME/talos-mac}"
export KUBECONFIG="${KUBECONFIG:-$WORKDIR/kubeconfig}"
export TALOSCONFIG="${TALOSCONFIG:-$WORKDIR/talosconfig}"
read -ra WORKER_IPS <<< "${WORKER_IPS:-192.168.64.21 192.168.64.22}"

echo "=== 1. /dev/kvm on worker ${WORKER_IPS[0]} (nested virt reached the guest)? ==="
if talosctl -n "${WORKER_IPS[0]}" ls /dev/kvm 2>/dev/null; then
  echo "  /dev/kvm present -> nested virt OK"
else
  echo "  /dev/kvm MISSING -> nested virt NOT exposed. On UTM: confirm the VM uses the Apple"
  echo "  Virtualization (vz) backend and UTM >= 4.6. On Parallels: enable nested virt + Hypervisor=Apple."
  exit 1
fi

echo "=== 2. kata-cloud-hypervisor pod boots a CLH microVM? ==="
kubectl delete pod kata-smoke --ignore-not-found >/dev/null 2>&1
kubectl apply -f - <<'EOF'
apiVersion: v1
kind: Pod
metadata:
  name: kata-smoke
spec:
  runtimeClassName: kata-cloud-hypervisor
  containers:
    - name: nginx
      image: nginx:1.29-alpine
      resources:
        requests: { cpu: 100m, memory: 64Mi }
        limits:   { cpu: 500m, memory: 256Mi }
EOF
if kubectl wait --for=condition=Ready pod/kata-smoke --timeout=180s; then
  echo "  kata-cloud-hypervisor pod Running -> CLH microVM booted. FIDELITY CONFIRMED."
  kubectl delete pod kata-smoke --ignore-not-found >/dev/null 2>&1
else
  echo "  pod not Ready — diagnostics:"; kubectl describe pod kata-smoke | tail -25; exit 1
fi
