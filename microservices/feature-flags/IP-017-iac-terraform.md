# IP-017 — IaC / Terraform Buildout

**microservice**: feature-flags
**bc**: infra
**layer**: iac
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0243, ADR-0248, ADR-0251, ADR-0253, ADR-0254, ADR-0295, ADR-0296, ADR-0297
**companion_ips**: IP-010, IP-018

## Scope

Full IaC for feature-flags µservice: Kubernetes deployment, HPA, PDB, CiliumNetworkPolicy, OpenBao policy + k8s auth backend, Kafka topics, ClickHouse database + table, cert-manager PQC certificate, ECH ConfigMap, edge WAF HTTPRoute + RateLimitPolicy, Helm chart values, ExternalSecret bindings, Terraform module.

## Deliverables

| # | Artifact | File | Acceptance Criterion |
|---|----------|------|---------------------|
| 1 | K8s Deployment | `iac/k8s-deployment.yaml` | `runtimeClassName: kata-clh`; SPIFFE annotation; OpenBao sidecar; `readOnlyRootFilesystem: true`; `runAsUser: 10001` |
| 2 | HPA | `iac/k8s-deployment.yaml` | min=4, max=50; CPU 70% + custom metric `oya_feature_flag_eval_queue_depth` |
| 3 | PDB | `iac/k8s-deployment.yaml` | `maxUnavailable: 1` |
| 4 | CiliumNetworkPolicy | `iac/network-policy.yaml` | Default-deny; ingress: SDK/admin/Prometheus/Foundry; egress: Postgres/Kafka/OpenBao/policy-engine/DNS |
| 5 | OpenBao policy | `iac/openbao-policy.hcl` | 7 paths; all `max_ttl = "60s"` per ADR-0296; explicit deny all other paths |
| 6 | ExternalSecret | `iac/secret-bindings.yaml` | `refreshInterval: 55s`; 5 secretKey mappings; SPIFFE ServiceAccount |
| 7 | Helm values | `iac/helm-values.yaml` | HTTP/3 annotations; ECH enabled; PQC enabled; `kata-clh`; pack overlays |
| 8 | ECH ConfigMap | `iac/ech-config.yaml` | 90d rotation; OpenBao key ref; DNS HTTPS RR example |
| 9 | PQC cert | `iac/pqc-cert.yaml` | TLS 1.3 floor; KEM preference X25519MLKEM768→X25519→P-256 |
| 10 | Edge WAF | `iac/edge-waf.yaml` | EMERGENCY_SERVICES bypass rule FIRST; 100k eval/min per tenant; bot_score≥95 block |
| 11 | Terraform module | `iac/terraform/main.tf` | K8s namespace; Helm release; OpenBao role TTL=60; Kafka 50+100 partitions; ClickHouse ReplicatedMergeTree |

## OpenBao TTL Invariant

Every credential path in `openbao-policy.hcl` and `terraform/main.tf` MUST set `max_ttl = "60s"`. CI gate `lean-a6-secret-ttl` rejects any path where `max_ttl > "60s"`.

## Definition of Done

- `terraform validate` on `iac/terraform/` passes
- `kubectl apply --dry-run=server` on all YAML files passes
- OpenBao TTL CI gate green (all paths ≤60s)
- Cilium policy: `cilium policy trace` confirms egress to non-allowlisted destinations is dropped
- Edge WAF: EMERGENCY_SERVICES request reaches evaluator with `X-Oya-WAF-Bypass: emergency-services` header
