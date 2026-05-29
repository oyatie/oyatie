---
doc_class: IaCModuleManifest
microservice: contract-lifecycle-management
deployment_context: colo
related_memories: [feedback_zero_handroll_opentofu_only_2026_05_20]
date: 2026-05-21
---

# OpenTofu Module — Colocation

Deployment context: `colo` (colocation facility; mixed model with customer infrastructure + colocation provider).

Similar shape to `on-prem` but with assumptions about facility-level services (power, cooling, network) being provided by the colo operator (Equinix, Digital Realty, NTT, KT, KDDI, etc.).

## Usage

```bash
tofu apply \
  -var tenant_id=<tenant_id> \
  -var colo_provider=<provider> \
  -var colo_facility=<facility_id> \
  -var jurisdiction_packs='["gdpr", "eidas"]' \
  -var tenant_class=paid \
  -var billing_components='["per_seat"]'
```

## Resources

Inherits `iac/on-prem/` resources + colocation-specific:

- **Cross-connect** to public cloud (AWS Direct / OCI Fast/ Azure ExpressRoute) for hybrid cloud egress.
- **Colocation-provider managed DNS** (e.g. Equinix Network Edge).
- **Colocation-provider DDoS protection**.
- **Colocation-provider physical security attestation** (typically SOC-2 + ISO 27001 from the colo operator).

## Hybrid cross-connect

When a tenant runs CLM in a colo facility but uses cloud-native services (e.g. SES for email), the cross-connect provides low-latency egress to the cloud provider. The `colo/hybrid-cross-connect/` sub-module provisions:

- AWS Direct / OCI Fast/ Azure ExpressRoute.
- Private VLAN.
- BGP peering.

## Module structure (stub)

```
iac/colo/
  README.md
  main.tf
  variables.tf
  outputs.tf
  versions.tf
  (inherits iac/on-prem/* structure)
  hybrid-cross-connect/
    aws-direct-connect.tf
    oci-fastconnect.tf
    azure-expressroute.tf
  colo-providers/
    equinix.tf
    digital-realty.tf
    ntt.tf
    kt.tf
    kddi.tf
```
