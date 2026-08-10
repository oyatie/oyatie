---
doc_class: IaCModuleManifest
microservice: contract-lifecycle-management
deployment_context: on-prem
related_memories: [feedback_zero_handroll_opentofu_only_2026_05_20]
date: 2026-05-21
---

# OpenTofu Module — On-Prem

Deployment context: `on-prem` (customer's bare-metal infrastructure; air-gap deployments supported).

Particularly relevant for KR-PIPA sovereign tenants, EU eIDAS QES tenants with customer-controlled HSM, and government tenants requiring air-gap.

## Usage

```bash
tofu apply \
  -var tenant_id=<tenant_id> \
  -var cluster_endpoint=<kubernetes_api_endpoint> \
  -var jurisdiction_packs='["eidas", "eu-eidas-qes", "gdpr"]' \
  -var tenant_class=paid \
  -var billing_components='["per_seat"]' \
  -var hsm_byok=true \
  -var hsm_byok_required_by_pack=true \
  -var hsm_provider="thales-luna-7-a790"
```

## Resources

- **Kubernetes cluster** (Talos preferred for hardened deployments; customer-managed).
- **Local PostgreSQL** with WAL streaming to local replica.
- **Local SeaweedFS** with Compliance mode for WORM.
- **Customer-provided HSM** (Thales Luna 7 A790 / Utimaco / Entrust nShield XC) for QES.
- **MetalLB** for service load balancing.
- **OpenBao** for credential management.
- **Local OTEL collector** for observability (data stays on-prem).

## Air-gap mode

```bash
tofu apply \
  -var air_gap=true \
  -var outbound_egress_allowed=false
```

In air-gap mode:

- No outbound calls except to customer-controlled NTP / DNS.
- AI redlining via on-prem Llama-3.1-70B inference server (BYOK model).
- TSA via customer-controlled Trust Service Provider.
- All updates via Oyatie OCI image bundles transferred via removable media.

## Sovereign-cell variants

EU sovereign tenant with customer-owned data center:

```bash
tofu apply \
  -var data_center_country=DE \
  -var jurisdiction_packs='["gdpr", "eidas", "eu-eidas-qes"]' \
  -var tsa_provider="d-trust-qualified-tsa" \
  -var hsm_provider="thales-luna-7-a790"
```

KR sovereign tenant:

```bash
tofu apply \
  -var data_center_country=KR \
  -var jurisdiction_packs='["kr-pipa", "kr-pipa-sovereign"]' \
  -var tsa_provider="kisa-tsa" \
  -var qes_certificate_authority="yessign"
```

## State backend

Local state backend (customer-managed):

```hcl
terraform {
  backend "local" {
    path = "/opt/oyatie/tofu/clm/on-prem/state.tfstate"
  }
}
```

## Module structure (stub)

```
iac/on-prem/
  README.md
  main.tf
  variables.tf
  outputs.tf
  versions.tf
  kubernetes.tf
  postgres.tf
  seaweedfs.tf
  hsm.tf              # customer-provided
  metallb.tf
  openbao.tf
  observability.tf
  air-gap/
    main.tf
  sovereign-eu/
    main.tf
    d-trust-tsa.tf
  sovereign-kr/
    main.tf
    kisa-tsa.tf
    yessign-cert.tf
```
