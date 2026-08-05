# Supply Chain Planning IAC Provenance Inventory

Kanban source: `t_192d3f33`.

Authority ceiling: `specs/microservices/supply-chain-planning.json` is a preview, metadata-only PRD. The `oya/supply-chain-planning/iac/**` tree is target/provenance inventory only. Nothing in this directory is evidence of live deployment, tenant namespace readiness, DR readiness, runtime audit-chain emission, SLO readiness, IAC activation, or GA status.

## ADR-0349 / deployment wording reconciliation

- Jenkins/Prow/legacy local verifier wording is historical provenance after ADR-0515; it is not current CI authority.
- GitHub Actions plus the protected `oya-ci-required` context is the current CI authority cited by the operating contract, but this inventory does not create or modify CI authority.
- ArgoCD, Helm, kubectl, Kubernetes `Deployment`, OpenTofu/Terraform, OpenBao, ECH, PQC, and WAF terms in this directory describe future target planning or preserved provenance only.
- Do not run `kubectl apply`, `helm install/upgrade`, `terraform apply`, `tofu apply`, OpenBao policy writes, cert issuance, or Argo sync from this directory until a separate activation card supplies measured evidence and review approval.

## File classification

| File | Surface | Classification | Rationale / claim boundary |
|---|---|---|---|
| `terraform-module/main.tf` | Terraform/OpenTofu wrapper stub | target/provenance | Variables and outputs only; no resources, module invocation, plan/apply, or cloud activation evidence. |
| `secret-bindings.yaml` | OpenBao secret binding target | target/provenance | Describes intended secret paths and lease/audit event class; no OpenBao runtime policy write, secret lease, or runtime audit emission is claimed. |
| `pqc-cert.yaml` | cert-manager/PQC certificate target | target/provenance | Describes certificate shape; no cert-manager install, certificate issuance, tenant namespace, or TLS readiness is claimed. |
| `openbao-policy.hcl` | OpenBao policy target | target/provenance | Describes future policy capabilities; no live OpenBao policy load, transit signer, or secret readiness is claimed. |
| `network-policy.yaml` | Kubernetes NetworkPolicy target | target/provenance | Describes intended ingress/egress posture; no Kubernetes namespace, pod, or network enforcement readiness is claimed. |
| `k8s/helm/values.yaml` | Helm values target | target/provenance | Values feed a target chart only; production/runtime/audit/SLO readiness is not claimed. |
| `k8s/helm/templates/service.yaml` | Helm Service template | target/provenance | Template describes future ClusterIP shape; no rendered manifest, Argo sync, or service readiness is claimed. |
| `k8s/helm/templates/deployment.yaml` | Helm Deployment template | target/provenance | Template describes future pod/runtime and probes; no signed image, runtime audit-chain emission, readiness, or deployment activation is claimed. |
| `k8s/helm/templates/configmap.yaml` | Helm ConfigMap template | target/provenance | Template records target metadata only; audit-chain target fields are not runtime emission evidence. |
| `k8s/helm/templates/cedar.yaml` | Helm Cedar ConfigMap template | target/provenance | Template records a future policy bundle shape; no runtime Cedar enforcement or tenant authorization readiness is claimed. |
| `k8s/helm/Chart.yaml` | Helm chart metadata | target/provenance | Chart metadata is retained for future planning; ArgoCD wording is target/provenance, not live application source authority. |
| `k8s-deployment.yaml` | Legacy static Kubernetes deployment sample | stale | Retained only as non-active provenance for earlier direct-manifest planning; Helm target files are the preferred future shape. Do not apply with kubectl. |
| `helm-values.yaml` | Legacy root Helm values sample | stale | Retained only as non-active benchmark/transport planning; `k8s/helm/values.yaml` is the preferred future chart values surface. |
| `edge-waf.yaml` | Edge WAF target | target/provenance | Describes future anti-abuse/security posture; no edge deployment, WAF enforcement, or production protection readiness is claimed. |
| `ech-config.yaml` | ECH target | target/provenance | Describes future ECH rotation/audit event class; no DNS HTTPS record, cert issuance, or runtime audit-chain emission is claimed. |

## Linked IP document classification

| File | Classification | Reconciliation |
|---|---|---|
| `../IPs/IP-ADR-0339-Shared-IaC-Modules.md` | target/provenance | PROPOSED IP; wrapper/module language is future service-owned planning. Older `microservices/supply-chain-planning/...` path strings are stale aliases unless later indexed activation evidence proves promotion. |
| `../IPs/IP-WAVE-15-ZD-sharding-automation.md` | target/provenance | Doctrine propagation only; ADR-0349 Jenkins language is historical, ArgoCD is future/separately authorized CD evidence, and audit-chain wording is a downstream implementation requirement rather than current runtime emission. |

## Active/stale summary

- Active IAC files: none.
- Target/provenance IAC files: 13.
- Stale-but-preserved IAC files: 2 (`k8s-deployment.yaml`, `helm-values.yaml`).
- Out-of-scope files: none within the inspected 15-file IAC inventory.
