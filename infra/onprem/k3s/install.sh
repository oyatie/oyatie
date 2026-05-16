#!/usr/bin/env bash
# RETIRED 2026-05-16. k3s is no longer the on-prem Kubernetes choice.
# See ADR-0118: vanilla kubeadm + containerd is the canonical stack.
# Edge cells (M07+) may still sanction k3s under a separate future ADR.
set -e
echo "ERROR: k3s install retired per ADR-0118 (on-prem stack: kubeadm + containerd + Istio + Envoy)." >&2
echo "Use: sudo bash /home/oyatie/projects/oyatie/infra/onprem/containerd/install.sh && \\" >&2
echo "     sudo bash /home/oyatie/projects/oyatie/infra/onprem/kubeadm/install.sh && \\" >&2
echo "     sudo bash /home/oyatie/projects/oyatie/infra/onprem/istio/install.sh" >&2
exit 64
