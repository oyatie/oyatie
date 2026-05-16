# api.oyatie.com — Istio Ingress Gateway + Envoy hardening

Authority: ADR-0119 (k8s + Istio + Envoy), ADR-0044 (service mesh), user
directive 2026-05-16 ("envoy + istio should harden api.oyatie.com").

## Shape

```
client → Cloudflare edge (DNS + WAF + Bot)
       → Cloudflare Tunnel (oyatie-onprem-kr)
       → cloudflared on on-prem (sends to a TBD local address)
       → istio-ingressgateway service (Envoy at the cluster edge)
       │   ├─ AuthorizationPolicy (allow read-only public; require JWT for mutating verbs)
       │   ├─ RequestAuthentication (JWT issuer validation)
       │   └─ EnvoyFilter local_ratelimit (100 req/min per source IP)
       → VirtualService api.oyatie.com → workspace-shell.api-gateway.svc:8080
       → Endpoints subset (host 220.84.203.169:8080)
       → workspace-shell systemd service on host
```

## Apply

```bash
kubectl apply -f infra/k8s/api-gateway/00-namespace.yaml
kubectl apply -f infra/k8s/api-gateway/10-workspace-shell-upstream.yaml
kubectl apply -f infra/k8s/api-gateway/20-istio-gateway.yaml
kubectl apply -f infra/k8s/api-gateway/30-virtualservice.yaml
kubectl apply -f infra/k8s/api-gateway/40-authorization-policy.yaml
kubectl apply -f infra/k8s/api-gateway/50-rate-limit.yaml
```

## Repoint cloudflared tunnel

Update `infra/cloudflare/main.tf` ingress rule for `api.oyatie.com` to point at
the istio-ingressgateway service. The service runs in cluster IP space; cloudflared
must reach it. Two options:

1. **NodePort**: patch `istio-ingressgateway` service to `type: NodePort`, take
   the nodePort, and point cloudflared at `http://127.0.0.1:<nodePort>` on the
   on-prem host (cluster + cloudflared are on the same host).
2. **LoadBalancer + MetalLB**: install MetalLB (free) so the ingressgateway gets
   a host-reachable IP. Cloudflared then targets that IP:80.

For stage-0 we use #1 (NodePort) because it's a single-host cluster and avoids
MetalLB setup. Get the nodePort:

```bash
kubectl -n istio-system get svc istio-ingressgateway -o jsonpath='{.spec.ports[?(@.name=="http2")].nodePort}'
```

Then update the tofu Cloudflare module:
```hcl
# infra/cloudflare/main.tf, api ingress
service  = "http://127.0.0.1:<nodePort>"
```

## Migration to in-mesh workspace-shell

Currently workspace-shell is on systemd, exposed to the cluster via an Endpoints
object. For full mTLS through Envoy sidecar, move workspace-shell into the
cluster as a Deployment (M3-P06 territory) or add the host as a VM workload to
the Istio mesh via `istioctl x workload entry configure` (intermediate path).
