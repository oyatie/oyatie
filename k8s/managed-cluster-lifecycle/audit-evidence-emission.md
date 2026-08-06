# Managed K8s Tenant Quota — Audit Evidence Emission

## Status: Unimplemented (wave follow-on)

Audit chain emission on quota check and quota set operations is tracked as:
`registry/placeholder-debt/adr-follow-ups.yaml#adr-0376-audit-chain-emission`

The `oya-managed-k8s-tenant-quota-app` crate exposes a typed
`Unimplemented::AuditChainEmission` placeholder. No stubbed `Ok(())` is used.

## When Implemented

Each `POST /tenants/{id}/quota/check` decision and each `PUT /tenants/{id}/quota`
change will emit a sealed audit event per ADR-0376 and the audit-chain substrate.
