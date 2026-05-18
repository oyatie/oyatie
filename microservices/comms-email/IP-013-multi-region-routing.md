# IP-013 — Multi-region routing

> ADR anchor: ADR-0201, ADR-0171, ADR-0180.
> Owner: `oya-substrate-comms`.
> Estimate: 4 days.

## Goal

Route email sends through the regional provider closest to the
tenant's data-residency boundary. EU tenants send via EU-region
SES (or EU-hosted Postal); KR tenants via KR-region; etc.

## Why this IP

Data-residency regulations (GDPR Art. 44-50, KR PIPA) prohibit
processing personal data outside the bound region. Even
transactional email content (subject, body) frequently contains
PII (names, account references). Routing must honor residency.

## Pre-conditions

- ADR-0171 multi-cluster federation.
- ADR-0180 DR / business-continuity portfolio policy.
- Adapter IPs (IP-001 / IP-002 / IP-004) land.

## Tasks

### 1. Regional adapter pools

- Each region runs its own adapter instances:
  - `us-east-1` → SES `us-east-1`, Postal `us-east-1`.
  - `eu-central-1` → SES `eu-central-1`, Postal `eu-central-1`.
  - `ap-northeast-2` (KR) → Postal `ap-northeast-2` (SES has
    no KR-region as of 2026-05-18; sovereign pack defaults).
- Each tenant's pack (ADR-0064) declares its bound region.

### 2. Routing logic

- On send: look up tenant pack → bound region → regional
  adapter pool → adapter instance.
- Cross-region routing is rejected unless the tenant explicitly
  opts in via `cross_region_allowed = true` in the pack.

### 3. Failover

- If the bound region's adapter is degraded, retry within the
  same region first, then escalate to a sibling region only
  if the tenant pack permits.

### 4. Audit chain region tag

- Every audit event carries `processing_region` so auditors
  can prove residency posture.

### 5. Tests

- Unit tests for the routing decision.
- Integration test asserting EU pack never reaches non-EU
  adapter.
- Failover test asserting cross-region escalation respects
  the pack flag.

## Failure modes

- Bound region fully down + cross-region disallowed: sends
  reject with explicit error; tenant is alerted; on-call
  paged.
- Region tagging drift: `processing_region` mismatch with
  tenant pack triggers a CI lint that compares pack
  declarations vs deployed regional adapters.

## Acceptance criteria

- 100% of sends for residency-bound tenants land in the
  bound region's adapter.
- Cross-region routing reject works end-to-end.
- Audit `processing_region` populated on every event.

## Rollback

Parent reverts the routing config to a single-region
default; degrades multi-region but keeps sending.

## References

- ADR-0201.
- ADR-0171 multi-cluster federation.
- ADR-0180 DR / BC.
- ADR-0064 packs.
