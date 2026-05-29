# Reference implementation — Hire + onboard + payroll cycle with `oya-workplace-integration-sdk`

A runnable Rust program that walks a single-employee lifecycle: hire, send 3 onboarding docs, run E-Verify, activate, simulate
clock-in for one biweekly period, manager-approve, run payroll, print stub summary. Designed for both dev cells and production.

## `Cargo.toml`

```toml
[package]
name = "workplace-flow-example"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
chrono = { version = "0.4", features = ["serde"] }
oya-workplace-integration-sdk = "0.42.0"
oya-trace = "0.42.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1.43", features = ["macros", "rt-multi-thread"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## `src/main.rs`

```rust
use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use oya_trace::TraceContext;
use oya_workplace_integration_sdk::{
    ClockInAttestation, DocumentKind, EmployeeHire, EmploymentType, FilingStatus, PayFrequency,
    PayPeriod, SignatureLevel, Tenant, WorkplaceClient, WorkplaceConfig,
};
use std::time::Duration;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let trace = TraceContext::new_root();
    let tenant = Tenant::parse("oyatie.b2b.smb.acme-software")?;

    let cfg = WorkplaceConfig::builder()
        .endpoint("https://loopback.workplace-integration.oyatie.local".parse()?)
        .api_key(std::env::var("OYA_API_KEY").context("OYA_API_KEY missing")?)
        .request_timeout(Duration::from_secs(15))
        .build()?;
    let client = WorkplaceClient::connect(cfg).await?;
    info!("connected to workplace-integration");

    // 1. Hire
    let employee = client
        .hire(
            &tenant,
            EmployeeHire {
                legal_name: "Alice Aaronson".into(),
                email: "alice@acme-software.io".into(),
                employment_type: EmploymentType::W2,
                start_date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
                work_state: "CA".into(),
                residence_state: "CA".into(),
                role: "Senior Software Engineer".into(),
                comp_base_minor: 165_000_00,
                comp_currency: "USD".into(),
                pay_frequency: PayFrequency::Biweekly,
                first_payday: NaiveDate::from_ymd_opt(2026, 6, 12).unwrap(),
                filing_status: FilingStatus::Single,
                federal_allowances: 0,
                state_allowances_overrides: Default::default(),
            },
            trace.child(),
        )
        .await
        .context("hire failed")?;
    info!(employee_id = %employee.id(), stage = ?employee.stage(), "hired");

    // 2. Onboarding docs (offer letter + I-9 + W-4)
    for doc_kind in [
        DocumentKind::OfferLetter,
        DocumentKind::I9,
        DocumentKind::W4_2026,
    ] {
        let sig_level = match doc_kind {
            DocumentKind::OfferLetter => SignatureLevel::EidasSimple,
            _ => SignatureLevel::EsignAct,
        };
        let sent = client
            .esign_send(&tenant, employee.id(), doc_kind, sig_level, trace.child())
            .await?;
        info!(doc = ?doc_kind, sig_id = %sent.signature_id(), "esign sent");
    }

    // 3. Wait for all sigs to land (dev cell auto-signs within ~5 s)
    client
        .wait_for_onboarding_docs_signed(&tenant, employee.id(), Duration::from_secs(30), trace.child())
        .await?;
    info!("all onboarding docs signed");

    // 4. E-Verify
    let everify = client
        .e_verify_run(&tenant, employee.id(), trace.child())
        .await
        .context("e-verify failed")?;
    info!(
        case = %everify.case_number(),
        status = ?everify.status(),
        "e-verify result"
    );

    // 5. Activate on start date
    let activated = client
        .activate(&tenant, employee.id(), trace.child())
        .await
        .context("activate failed")?;
    info!(stage = ?activated.stage(), "active");

    // 6. Simulate 10 working days of clock-in
    for day in 0..10 {
        let attestation = ClockInAttestation::all_passed(
            "dev-iphone-mock-alice",
            "f0:9f:c2:11:23:ac",
            format!("ed25519-sig-mock-day-{day}"),
        );
        client
            .clock_in(&tenant, employee.id(), attestation.clone(), trace.child())
            .await?;
        client
            .clock_out(&tenant, employee.id(), attestation, trace.child())
            .await?;
        info!(day, "clocked in + out");
    }

    // 7. Manager approves timecard
    let approval = client
        .timecard_approve(
            &tenant,
            employee.id(),
            PayPeriod::biweekly_iso_week(2026, 23),
            "mgr-eve-evergreen",
            trace.child(),
        )
        .await
        .context("timecard approve failed")?;
    info!(approval_id = %approval.id(), "timecard approved");

    // 8. Run payroll cycle for the tenant
    let payroll = client
        .payroll_run(
            &tenant,
            PayPeriod::biweekly_iso_week(2026, 23),
            NaiveDate::from_ymd_opt(2026, 6, 12).unwrap(),
            trace.child(),
        )
        .await
        .context("payroll run failed")?;
    let stub = payroll
        .stub_for(employee.id())
        .context("expected stub for our employee")?;
    info!(
        gross_minor = stub.gross_minor(),
        fed_withhold_minor = stub.federal_withhold_minor(),
        state_withhold_minor = stub.state_withhold_minor(),
        ssn_minor = stub.ssn_minor(),
        medicare_minor = stub.medicare_minor(),
        ca_sdi_minor = stub.state_disability_insurance_minor(),
        net_minor = stub.net_minor(),
        "paystub"
    );

    if stub.net_minor() <= 0 {
        warn!("net pay non-positive; check withholding setup");
    }

    Ok(())
}
```

## Run it

```bash
OYA_API_KEY=$(./bin/oya creds dev-token --tenant oyatie.b2b.smb.acme-software) \
  cargo run --release
```

Expected stdout:
```
INFO  connected to workplace-integration
INFO  hired employee_id=emp-… stage=Hired
INFO  esign sent doc=OfferLetter sig_id=sig-… etc.
INFO  all onboarding docs signed
INFO  e-verify result case=2026… status=EmploymentAuthorized
INFO  active stage=Active
INFO  clocked in + out day=0 … day=9
INFO  timecard approved approval_id=ap-…
INFO  paystub gross_minor=634615 fed_withhold_minor=76271 state_withhold_minor=29281 ssn_minor=39346 medicare_minor=9202 ca_sdi_minor=5712 net_minor=474797
```

## SDK correctness guarantees

1. `EmploymentType` is a closed enum — adding new types requires ADR.
2. `EmployeeHire` validates date + state codes at construction.
3. `DocumentKind` + `SignatureLevel` are paired by Cedar policy; mismatched pairs fail compile in `client.esign_send`.
4. `e_verify_run` is idempotent on `(tenant, employee, case_uuid)`.
5. `clock_in` enforces 3-signal attestation; missing signals return `InsufficientAttestation`.
6. `payroll_run` is idempotent on `(tenant, pay_period, pay_date)`.
7. Every API call carries `traceparent`; audit chain links the events.

## Tests

```bash
cargo test --features hermetic
```

The hermetic feature spins a single-process loopback workplace-integration cell with mock E-Verify + mock ACH rails for fast tests.
