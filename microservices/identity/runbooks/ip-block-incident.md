---
doc_class: Runbook
runbook_id: identity-ip-block-incident
microservice: identity
sev: Sev-2 (specific block) / Sev-3 (single-tenant complaint)
owner_team: ops-security + axis-identity
date: 2026-05-18
---

# Runbook: IP block incident (false positive OR confirmed bad-actor)

## When to use

- A tenant complains "my users can't sign in from <country>".
- A specific IP/CIDR is unintentionally blocked.
- A confirmed bad-actor IP needs adding to the block list.

## Diagnostic

1. **Check edge block list**:
   - `oya identity edge list-blocks --pack <pack>` — enumerate active blocks.

2. **Check WAF / Coraza activity for affected IP**:
   - `oya identity edge waf-history --ip <ip> --since 24h`.

3. **Check GeoIP database freshness**:
   - `oya identity edge geoip-version` — MaxMind DB version; should be ≤30 days old.

## Case A — confirmed false positive

1. **Verify the legitimate source**: contact tenant IT; confirm IP is theirs.
2. **Remove block**: 
   - `oya identity edge remove-block --ip <cidr> --reason "false-positive-confirmed" --ticket <t>`.
3. **Propagate**: Envoy config reload (auto-rolling, ≤60s).
4. **Verify**: sign-in test from the IP succeeds.
5. **Add tenant-edge-policy exemption** (if recurring):
   - `oya identity edge tenant-allowlist --tenant <t> --cidr <cidr>`.

## Case B — confirmed bad-actor

1. **Confirm abuse**: pull WAF history; cross-reference threat-intel.
2. **Add long-term block**:
   - `oya identity edge add-block --ip <cidr> --duration 30d --reason "<abuse-summary>" --ticket <t>`.
3. **Share with peer packs**:
   - `oya identity edge propagate-block --pack-list "kr,eu,us,..." --cidr <cidr>` — apply to all packs.
4. **Document in shared abuse-ledger**: `evidence/shared/abuse-ledger.jsonl` append-only.

## Case C — geo / ASN class block

1. Pack-policy review with council-compliance.
2. Update `microservices/identity/iac/kustomize/components/edge-authz-rules/values-pack-<x>.yaml`.
3. Roll via the standard IaC promotion.

## Communication

- Per affected tenant: email within 1h confirming resolution + audit-ticket reference.
- Per peer-pack propagation: ops-security cross-pack channel notification.

## Postmortem trigger

False-positive blocks affecting > 5 tenants or > 24h duration → blameless postmortem.
