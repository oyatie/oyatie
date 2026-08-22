# Reference implementation — Calculate batch tax + file a return via `cloud-billing-tax-sdk`

Runnable Rust program that calculates tax on a batch of cross-border transactions, aggregates EU OSS, generates a quarterly OSS
VAT return XML, and submits it to a loopback Revenue Online Service.

## `Cargo.toml`

```toml
[package]
name = "tax-end-to-end-example"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
cloud-billing-tax-sdk = "0.42.0"
trace = "0.42.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
time = { version = "0.3", features = ["macros"] }
tokio = { version = "1.43", features = ["macros", "rt-multi-thread"] }
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = { version = "1.10", features = ["v7"] }
```

## `src/main.rs`

```rust
use anyhow::Result;
use cloud_billing_tax_sdk::{
    BuyerType, CalculateRequest, FilingFormat, Jurisdiction, Line, Location, OssScheme, Period,
    TaxClient, TaxConfig, Tenant,
};
use trace::TraceContext;
use std::time::Duration;
use tracing::{info, warn};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let trace = TraceContext::new_root();

    let cfg = TaxConfig::builder()
        .endpoint("https://loopback.cloud-billing-tax.oyatie.local".parse()?)
        .tenant(Tenant::parse("oyatie.b2b.smb.acme-software")?)
        .service_account_credentials_path("/etc/oya/tax/sa-creds.json")
        .request_timeout(Duration::from_secs(15))
        .calculate_deadline(Duration::from_millis(60))
        .build()?;

    let client = TaxClient::connect(cfg).await?;
    info!("connected to cloud-billing-tax");

    // 1. Build the batch
    let requests = vec![
        CalculateRequest::builder()
            .calculation_id(Uuid::now_v7())
            .seller_location(Location::us_ca("94107"))
            .buyer_location(Location::us_tx("78701"))
            .buyer_type(BuyerType::Business)
            .transaction_currency("USD")
            .line(Line::new("SW054001", "Acme SaaS Pro", 1200.00, 1))
            .build()?,
        CalculateRequest::builder()
            .calculation_id(Uuid::now_v7())
            .seller_location(Location::us_ca("94107"))
            .buyer_location(Location::eu("DE", "10115"))
            .buyer_type(BuyerType::Consumer)
            .transaction_currency("EUR")
            .evidence(vec![
                "ip:5.1.2.3 (DE)".into(),
                "billing:DE".into(),
                "payment:DE".into(),
            ])
            .line(Line::new("SW054001", "Acme SaaS Pro", 79.00, 1))
            .build()?,
        CalculateRequest::builder()
            .calculation_id(Uuid::now_v7())
            .seller_location(Location::us_ca("94107"))
            .buyer_location(Location::eu("FR", "75001"))
            .buyer_type(BuyerType::Business)
            .buyer_vat_number("FR12345678901")
            .transaction_currency("EUR")
            .line(Line::new("SW054001", "Acme SaaS Pro", 999.00, 1))
            .build()?,
        CalculateRequest::builder()
            .calculation_id(Uuid::now_v7())
            .seller_location(Location::us_ca("94107"))
            .buyer_location(Location::kr("06236"))
            .buyer_type(BuyerType::Business)
            .buyer_brn("123-45-67890")
            .transaction_currency("KRW")
            .line(Line::new("SW054001", "Acme SaaS Pro", 1_500_000.0, 1))
            .build()?,
    ];

    // 2. Calculate
    let results = client.calculate_batch(&requests, trace.child()).await?;
    for r in &results {
        info!(
            calculation_id = %r.calculation_id(),
            total_tax = r.total_tax(),
            effective_rate = r.effective_rate(),
            tax_line_count = r.tax_lines().len(),
            buyer_obligation = ?r.buyer_obligation(),
            "calculated"
        );
    }

    // 3. EU OSS aggregation for Q2 2026
    let oss = client
        .oss_aggregate(
            OssScheme::EuUnion,
            Period::quarter(2026, 2)?,
            "IE",
            trace.child(),
        )
        .await?;
    info!(
        scheme = %oss.scheme(),
        applicable_lines = oss.applicable_lines(),
        country_count = oss.country_breakdown().len(),
        total_vat_eur = oss.total_vat(),
        "oss aggregate"
    );

    // 4. Generate the EU OSS VAT MOSS XML
    let xml_path = "filing/EU-OSS/2026-Q2/moss.xml";
    let gen = client
        .filing_artefact_generate(
            Jurisdiction::EuOssUnion,
            Period::quarter(2026, 2)?,
            FilingFormat::EuVatMossXml,
            xml_path,
            trace.child(),
        )
        .await?;
    info!(
        period = %gen.period(),
        country_lines = gen.country_lines(),
        total_vat_eur = gen.total_vat(),
        xsd_validation = gen.xsd_validation_status(),
        xml_path = %gen.output_path(),
        "filing artefact generated"
    );

    // 5. Submit to the loopback Revenue Online Service
    let submission = client
        .filing_submit_loopback(
            Jurisdiction::EuOssUnion,
            Period::quarter(2026, 2)?,
            xml_path,
            trace.child(),
        )
        .await?;
    info!(
        submission_id = %submission.id(),
        status = %submission.status(),
        ack = %submission.acknowledgement(),
        audit_chain_event_id = %submission.audit_chain_event_id(),
        "filing submitted"
    );

    Ok(())
}
```

## Run it

```bash
cargo run --release
```

Expected output (trimmed):
```
INFO  connected to cloud-billing-tax
INFO  calculated calculation_id=cal-1 total_tax=99.00 effective_rate=0.0825 tax_line_count=3 buyer_obligation=None
INFO  calculated calculation_id=cal-2 total_tax=15.01 effective_rate=0.19 tax_line_count=1 buyer_obligation=None
INFO  calculated calculation_id=cal-3 total_tax=0 effective_rate=0 tax_line_count=1 buyer_obligation=Some(ReverseChargeBuyer)
INFO  calculated calculation_id=cal-4 total_tax=150000 effective_rate=0.10 tax_line_count=1 buyer_obligation=None
INFO  oss aggregate scheme=EuUnion applicable_lines=1 country_count=1 total_vat_eur=15.01
INFO  filing artefact generated period=2026-Q2 country_lines=1 total_vat_eur=15.01 xsd_validation=Passed xml_path=filing/EU-OSS/2026-Q2/moss.xml
INFO  filing submitted submission_id=sub-… status=Accepted ack=ROS-ACK-… audit_chain_event_id=ce-…
```

## SDK correctness guarantees

1. `calculate_batch(...)` is **strict on UUID v7 calculation IDs**; reuse of an ID returns the cached result (idempotent).
2. `calculate_*(...)` returns `TaxError::RateMissing` if any (jurisdiction, tax_code) tuple cannot be resolved — never a default
   rate, never a silent zero.
3. VAT number validation calls VIES (EU) / GSTIN portal (IN) / NTS (KR) before applying reverse-charge or B2B exemption. Failed
   validation returns `TaxError::BuyerIdentifierInvalid`.
4. `oss_aggregate(...)` honors the period's rate-card version — recomputation in the future yields the same numbers.
5. `filing_artefact_generate(...)` runs pre-file reconciliation against `cloud-billing` raw ledger; mismatch returns
   `TaxError::ReconciliationFailed`.
6. `filing_submit_loopback(...)` returns when the gateway acknowledges (≤ 2 s p95 in dev profile); production gateways may be
   asynchronous — the SDK's `filing_submit(...)` for live mode polls until terminal status.

## Tests

```bash
cargo test --features hermetic
```

The `hermetic` feature uses `cloud_billing_tax_sdk::testkit::Hermetic` with an in-process rate-card snapshot + a loopback
Revenue Online Service simulator; tests finish in ≤ 90 s.

## Error budget

`TaxError::CalculationSloBreached { took_micros }` indicates a calculation exceeded the tier SLO. Do not retry — file a
`cloud_billing_tax.slo.calculate_slow` event so the on-call rotation engages. Persistent breach signals a rate-card publish
pathology or HSM-DB latency.
