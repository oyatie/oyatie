# Mesh-wide Istio + Envoy hardening

Authority: user directive 2026-05-16 ("just general hardening with istio and
envoy all together").

## Posture

| Layer | Control | File |
|-------|---------|------|
| L4 CNI | NetworkPolicy default-deny-ingress per ns | `40-default-network-policy.yaml` |
| L4 mesh | PeerAuthentication STRICT mTLS mesh-wide | `10-peer-authentication-strict.yaml` |
| L7 mesh | AuthorizationPolicy default-deny per ns | `20-default-deny.yaml` |
| L7 mesh | Telemetry CR (access logs + tracing + metrics) | `30-telemetry.yaml` |
| L7 mesh | Per-ingressgateway allow NetworkPolicy | `50-allow-from-ingressgateway.yaml` |

## Apply

Canonical bootstrap from `infra/onprem/setup.sh` will apply these once they
land. Until then:

```bash
kubectl apply -f infra/k8s/mesh-hardening/
kubectl apply -f infra/k8s/api-gateway/
```

## Rust takeover (ADR-0120 Phase C)

This YAML-apply step is a placeholder for an `oya-onprem` Component impl that
applies manifests via the `kube-rs` crate. Tracked in ADR-0120 Phase C.
