# Residency policy — `comms-email` µservice

> ADR anchor: ADR-0201, ADR-0064, ADR-0171.

## Tenant binding to region

Per ADR-0064 each tenant binds to a pack. Each pack binds to a
region:

| Pack            | Region(s)             | Allowed providers       |
| --------------- | --------------------- | ----------------------- |
| canonical-base  | us-east-1, us-west-2  | ses, postal, mailgun, smtp |
| eu              | eu-central-1, eu-west-1 | ses (eu), postal, mailgun (eu), smtp |
| kr              | ap-northeast-2        | postal, smtp            |
| ksa             | me-south-1            | postal (sovereign-only) |
| uae             | me-south-1            | postal (sovereign-only) |
| us-healthcare   | us-east-1 (BAA)       | ses (BAA), postal       |

## Enforcement points

1. **Cedar (application tier)**: per-send authz rejects when
   message resource's pack does not match the principal's
   tenant binding.
2. **Routing (IP-013)**: tenant pack → region → adapter pool;
   cross-region routing rejected unless pack explicitly
   permits.
3. **Helm pack overlay (IP-014)**: config-load reject when
   `provider` ∉ `allowed_providers` for the pack.
4. **Kyverno (admission tier)**: cluster admission rejects
   Helm releases that target a sovereign pack with a
   non-Postal provider.

## Cross-region exceptions

Cross-region routing is allowed only when:

- The tenant pack explicitly declares
  `cross_region_allowed = true`.
- The destination region is in the pack's allow-list.
- The audit chain captures the cross-region routing decision.

Sovereign packs (ksa, uae) NEVER allow cross-region routing.

## Audit emission

Every send carries `processing_region` in the audit chain
entry. A CI compliance gate runs nightly and asserts:

- No `processing_region` outside the pack's allow-list for any
  send in the past 24h.
- No sovereign-pack send used a non-Postal provider.
- No EU-pack send used a non-EU SES region.

Violations are SEV-2 incidents per `incident-response.md`.

## GDPR Art. 44-50

EU pack tenants never have personal data transit non-EU
regions. The substrate enforces this at all four points above.

## KR PIPA

KR pack tenants use ap-northeast-2 Postal only — SES has no
KR-region as of 2026-05-18; the substrate enforces Postal-only
for the KR pack.
