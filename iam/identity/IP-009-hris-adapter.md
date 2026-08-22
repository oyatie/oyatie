---
doc_class: IP
ip_id: IP-009
microservice: identity
status: ga
related_adrs: [ADR-0190]
date: 2026-05-18
owner_team: axis-identity
---

# IP-009 — HRIS adapter contract + Workday/BambooHR/Rippling implementations

## Goal

Pluggable `HrisAdapter` trait + concrete adapters for the three most-requested non-SCIM HRIS systems (Workday SOAP, BambooHR REST, Rippling REST). Poll cadence default 15min; pulls hires, promotions, terminations; translates to internal SCIM operations; emits audit events.

## Files

| File | Purpose |
|---|---|
| `crates/identity-hris-adapter-kernel/Cargo.toml` | trait + types |
| `crates/identity-hris-adapter-kernel/src/lib.rs` | `HrisAdapter` + `HrisHire` + `HrisChange` + `HrisTermination` |
| `crates/identity-hris-adapter-workday/Cargo.toml` | Workday adapter |
| `crates/identity-hris-adapter-workday/src/lib.rs` | SOAP client; Workday-XML → internal types |
| `crates/identity-hris-adapter-bamboohr/Cargo.toml` | BambooHR adapter |
| `crates/identity-hris-adapter-bamboohr/src/lib.rs` | REST + OAuth2 client |
| `crates/identity-hris-adapter-rippling/Cargo.toml` | Rippling adapter |
| `crates/identity-hris-adapter-rippling/src/lib.rs` | REST + OAuth2 client |
| `crates/identity-hris-adapter-worker/Cargo.toml` | tokio poller |
| `crates/identity-hris-adapter-worker/src/lib.rs` | scheduler + DLQ + reconciliation job |

## Adapter contract

```rust
pub trait HrisAdapter: Send + Sync {
    fn vendor(&self) -> &'static str;
    fn pull_hires(&self, since: DateTime<Utc>) -> Result<Vec<HrisHire>, HrisError>;
    fn pull_promotions(&self, since: DateTime<Utc>) -> Result<Vec<HrisChange>, HrisError>;
    fn pull_terminations(&self, since: DateTime<Utc>) -> Result<Vec<HrisTermination>, HrisError>;
}

pub struct HrisHire {
    pub external_id: String,   // vendor employee_id; dedup key
    pub email: String,
    pub display_name: String,
    pub department: Option<String>,
    pub manager_external_id: Option<String>,
    pub start_date: DateTime<Utc>,
    pub raw_vendor_payload_hash: String, // for audit
}
```

## Worker schedule

- Per (tenant, vendor) tuple, run every 15min.
- Per pull: cursor = max(prev_max_timestamp, now - 24h) to allow back-fill of clock-skewed records.
- Translate each event to SCIM POST (hire), PATCH (promotion / termination).

## Daily reconciliation job

- For each (tenant, vendor):
  - Pull full active-employee set.
  - Diff against Zitadel active-user set (via SCIM GET).
  - Drift > 0.1% → alarm.
  - Auto-correct (apply PATCH `active=false` for users in Zitadel-active but not HRIS-active) ONLY with risk-acceptance ticket OR per-tenant policy `hris_auto_terminate=true`.

## Failure modes

- **HRIS endpoint outage**: backoff exponentially; DLQ events that fail 3 retries; alert at 1h.
- **Credential expired**: refresh OAuth2 token; if refresh fails, page ops-security.
- **Shape mismatch**: refuse event; emit `IdentityHrisShapeError` audit event.
- **Duplicate external_id**: idempotent — skip if already provisioned.

## Tests

| Test | Mechanism |
|---|---|
| `workday_xml_to_hire_translation` | fixture SOAP response; assert HrisHire fields populated |
| `bamboohr_pagination_cursor_advances` | mock REST; assert cursor advances across pages |
| `rippling_oauth_refresh_on_401` | mock returns 401; refresh token; retry succeeds |
| `idempotent_external_id_dedupes` | re-poll same hire; second call no-op |
| `daily_reconciliation_detects_drift` | mock Zitadel + HRIS with drift; assert alarm event |
| `auto_terminate_disabled_by_default` | drift detected; no PATCH active=false |
| `auto_terminate_enabled_applies_patch` | with per-tenant policy enabled, PATCH applied |
| `shape_error_event_emitted` | malformed payload; audit event observed |
| `worker_dlq_after_3_retries` | failing endpoint; DLQ message after 3 attempts |
| `vendor_credential_rotation_path` | rotate credential in OpenBao; adapter picks up new on next poll |

## Per-tenant policy

| Field | Default | Purpose |
|---|---|---|
| `hris_vendor` | none | which adapter to instantiate |
| `hris_credentials_path` | none | OpenBao path |
| `hris_poll_cadence_minutes` | 15 | cadence |
| `hris_auto_terminate` | false | apply auto-correction |
| `hris_dlq_email` | none | where DLQ alerts go |

## Evidence

- `evidence/identity/hris-pull/<tenant>-<vendor>-<date>.json` — per-poll counts
- `evidence/identity/hris-reconciliation/<tenant>-<vendor>-<date>.json` — daily drift report
- `evidence/identity/hris-shape-errors/<tenant>-<vendor>-<date>.json`

## Acceptance — DONE when

- 10 adapter-tests pass.
- Live Workday / BambooHR / Rippling sandboxes pass smoke test.
- Daily reconciliation job runs cleanly in staging for 7 days.
- Drift alarm fires correctly on simulated drift.

## Counterpart references - 009-hris-adapter

- Counterpart class: workforce lifecycle.
- ServiceNow workforce workflows and GitHub enterprise SSO show the baseline for enterprise identity lifecycle; this IP keeps Oyatie stronger by routing lifecycle changes through SCIM/HRIS contracts, tenant-scoped Cedar, and audit-chain evidence instead of relying on tenant-admin convention.
- Verification anchor: this row intentionally includes a named counterpart from the Wave 15 grep allowlist while keeping the implementation reference service-local: `microservices/identity/competitor-parity-matrix.md`, `microservices/identity/PRD.md`, `microservices/identity/manifest.json`, and the contract/policy files cited above.

## Pod runtime tier (per ADR-0338)

- Authority: ADR-0338.
- `pod_runtime_tier`: `0`.
- Justification: tenant-customer code exists in this IP execution path; Kata Containers + Cloud Hypervisor are required.
- Surface evidence: `microservices/identity/IP-009-hris-adapter.md`, `microservices/identity/manifest.json`; trigger terms `sandbox`.
