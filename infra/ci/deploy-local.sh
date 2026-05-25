#!/usr/bin/env bash
# Deploy the Oyatie CI farm (Jenkins controller + ephemeral k8s agents) onto a
# LOCAL single-node k3s cluster (colima --kubernetes).
#
# Realizes EXE-CI-FARM-SUBSTRATE-CANONICAL / ADR-0349 for local execution.
# This is a local DEV cluster, not the production multi-region farm: the
# Kata runtime, Karpenter autoscaling, cosign-required agent image, and
# SeaweedFS remote cache backend are production deltas (see README.md).
#
# Prereqs: colima (running with --kubernetes), kubectl, helm.
set -euo pipefail

NS="oya-ci-jenkins"
RELEASE="oya-jenkins"
CHART="jenkins/jenkins"
VALUES="$(cd "$(dirname "$0")" && pwd)/jenkins/values-local.yaml"

echo "==> context: $(kubectl config current-context)"
[ "$(kubectl config current-context)" = "colima" ] || {
  echo "refusing to deploy: kube-context is not 'colima' (local cluster guard)"; exit 1; }

echo "==> namespace $NS"
kubectl create namespace "$NS" --dry-run=client -o yaml | kubectl apply -f -

echo "==> helm repo"
helm repo add jenkins https://charts.jenkins.io >/dev/null 2>&1 || true
helm repo update jenkins >/dev/null

echo "==> deploy $RELEASE"
helm upgrade --install "$RELEASE" "$CHART" \
  --namespace "$NS" \
  --values "$VALUES" \
  --wait --timeout 10m

echo "==> rollout status"
kubectl -n "$NS" rollout status statefulset/"$RELEASE" --timeout=8m

echo "==> done. admin password:"
kubectl -n "$NS" exec "statefulset/$RELEASE" -c jenkins -- \
  cat /run/secrets/additional/chart-admin-password 2>/dev/null || \
  kubectl -n "$NS" get secret "$RELEASE" -o jsonpath='{.data.jenkins-admin-password}' | base64 -d; echo
echo "==> UI: kubectl -n $NS port-forward svc/$RELEASE 8080:8080  then open http://localhost:8080"
