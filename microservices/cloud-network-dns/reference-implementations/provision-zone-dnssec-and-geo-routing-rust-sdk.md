# Reference implementation — Provision zone + DNSSEC + geo-routing + health-checks via `oya-cloud-network-dns-sdk`

Runnable Rust program that creates a tenant zone, enables DNSSEC, configures a geo-routed apex record set with three regional
answers + health-checks, and verifies responses via DoH/3 + DoQ.

## `Cargo.toml`

```toml
[package]
name = "dns-end-to-end-example"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
oya-cloud-network-dns-sdk = "0.42.0"
oya-trace = "0.42.0"
serde = { version = "1", features = ["derive"] }
tokio = { version = "1.43", features = ["macros", "rt-multi-thread"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## `src/main.rs`

```rust
use anyhow::Result;
use oya_cloud_network_dns_sdk::{
    DnssecAlgorithm, DnssecEnableRequest, DnsClient, DnsConfig, GeoContinent, HealthCheckRequest,
    NsecMode, RecordKind, RecordRequest, RoutingPolicy, Tenant, Transport, ZoneCreateRequest,
};
use oya_trace::TraceContext;
use std::time::Duration;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let trace = TraceContext::new_root();

    let cfg = DnsConfig::builder()
        .endpoint("https://loopback.cloud-network-dns.oyatie.local".parse()?)
        .tenant(Tenant::parse("oyatie.b2b.smb.acme-software")?)
        .service_account_credentials_path("/etc/oya/dns/sa-creds.json")
        .request_timeout(Duration::from_secs(5))
        .build()?;

    let client = DnsClient::connect(cfg).await?;
    info!("connected to cloud-network-dns");

    // 1. Create zone
    let zone = client
        .zone_create(
            ZoneCreateRequest::builder()
                .zone("acme-software.io")
                .soa_mname("ns1.oyatie.dns.net")
                .soa_rname("hostmaster.acme-software.io")
                .ttl_default(60)
                .build()?,
            trace.child(),
        )
        .await?;
    info!(zone_id = %zone.id(), ns_count = zone.ns_records().len(), "zone created");

    // 2. Add CAA + MX + TXT for SPF
    for rec in [
        RecordRequest::caa("@", "0 issue \"letsencrypt.org;account=acme-software\"", 86400),
        RecordRequest::caa("@", "0 issue \"digicert.com;account=acme-software-ev\"", 86400),
        RecordRequest::mx("@", "10 mail.acme-software.io", 3600),
        RecordRequest::txt("@", "\"v=spf1 mx -all\"", 3600),
        RecordRequest::txt(
            "_dmarc",
            "\"v=DMARC1; p=quarantine; rua=mailto:dmarc-reports@acme-software.io\"",
            3600,
        ),
    ] {
        client.record_create(zone.id(), rec, trace.child()).await?;
    }
    info!("baseline records created");

    // 3. Enable DNSSEC (ECDSAP256SHA256 + NSEC3)
    let dnssec = client
        .dnssec_enable(
            zone.id(),
            DnssecEnableRequest::builder()
                .algorithm(DnssecAlgorithm::EcdsaP256Sha256)
                .nsec_mode(NsecMode::Nsec3 { salt_length: 8, iterations: 10 })
                .ksk_rotation_days(180)
                .zsk_rotation_days(30)
                .build()?,
            trace.child(),
        )
        .await?;
    info!(
        ksk_keytag = dnssec.ksk_keytag(),
        zsk_keytag = dnssec.zsk_keytag(),
        ds_for_registrar = %dnssec.ds_record_text(),
        "dnssec enabled"
    );

    // 4. Geo-routed apex
    for (continent, ip, set_id) in [
        (GeoContinent::NorthAmerica, "203.0.113.42", "api-na-primary"),
        (GeoContinent::Europe, "198.51.100.42", "api-eu-primary"),
        (GeoContinent::Asia, "192.0.2.42", "api-apac-primary"),
    ] {
        client
            .record_create(
                zone.id(),
                RecordRequest::builder()
                    .name("api")
                    .kind(RecordKind::A)
                    .value(ip)
                    .ttl(60)
                    .routing_policy(RoutingPolicy::Geo { continent: Some(continent), country: None })
                    .set_identifier(set_id)
                    .build()?,
                trace.child(),
            )
            .await?;
    }
    info!("geo-routed apex created");

    // 5. Health-checks per region
    for (region, ip, set_id) in [
        ("na", "203.0.113.42", "api-na-primary"),
        ("eu", "198.51.100.42", "api-eu-primary"),
        ("apac", "192.0.2.42", "api-apac-primary"),
    ] {
        client
            .health_check_create(
                HealthCheckRequest::builder()
                    .name(format!("api-{region}-health"))
                    .target_url(format!("https://{ip}/healthz"))
                    .expected_status(200)
                    .expected_body_contains("\"ok\":true")
                    .interval(Duration::from_secs(10))
                    .failure_threshold(3)
                    .success_threshold(1)
                    .attach_to_set_identifier(set_id)
                    .build()?,
                trace.child(),
            )
            .await?;
    }
    info!("health-checks created");

    // 6. Query via DoH/3
    let answer_doh3 = client
        .query(
            "api.acme-software.io",
            RecordKind::A,
            Transport::DohHttp3,
            trace.child(),
        )
        .await?;
    info!(
        answers = ?answer_doh3.answers(),
        rcode = ?answer_doh3.rcode(),
        ad_bit = answer_doh3.ad_bit(),
        latency_micros = answer_doh3.latency_micros(),
        "doh/3 query"
    );

    // 7. Query via DoQ
    let answer_doq = client
        .query("api.acme-software.io", RecordKind::A, Transport::Doq, trace.child())
        .await?;
    info!(
        answers = ?answer_doq.answers(),
        ad_bit = answer_doq.ad_bit(),
        latency_micros = answer_doq.latency_micros(),
        "doq query"
    );

    // 8. Simulate NA health-check failure + re-query
    client
        .health_check_simulate_failure("api-na-health", Duration::from_secs(60), trace.child())
        .await?;
    tokio::time::sleep(Duration::from_secs(30)).await;

    let answer_after_failure = client
        .query_with_client_location(
            "api.acme-software.io",
            RecordKind::A,
            GeoContinent::NorthAmerica,
            Transport::DohHttp3,
            trace.child(),
        )
        .await?;
    if answer_after_failure.answers().first().map(|s| s.as_str()) == Some("198.51.100.42") {
        info!("failover to EU answer confirmed");
    } else {
        warn!(answers = ?answer_after_failure.answers(), "unexpected failover");
    }

    Ok(())
}
```

## Run it

```bash
cargo run --release
```

Expected output (trimmed):
```
INFO  connected to cloud-network-dns
INFO  zone created zone_id=zone-… ns_count=4
INFO  baseline records created
INFO  dnssec enabled ksk_keytag=19273 zsk_keytag=47102 ds_for_registrar=acme-software.io. IN DS 19273 13 2 7c8d…
INFO  geo-routed apex created
INFO  health-checks created
INFO  doh/3 query answers=["203.0.113.42"] rcode=NOError ad_bit=true latency_micros=2418
INFO  doq query answers=["203.0.113.42"] ad_bit=true latency_micros=1612
INFO  failover to EU answer confirmed
```

## SDK correctness guarantees

1. `zone_create(...)` validates ownership of the apex domain via DNS-01 TXT challenge (`oya dns verify ...`) before activating the zone.
2. `record_create(...)` is idempotent on `(zone_id, name, kind, set_identifier)`; replays are no-ops.
3. `dnssec_enable(...)` is atomic — either the zone is fully signed with chain-of-trust ready, or no change.
4. `health_check_create(...)` validates the target URL is reachable from at least one cell before creating; refuses otherwise.
5. `query(...)` returns the `ad_bit` only when the signed chain validates end-to-end; otherwise `ad_bit=false` even on signed zones.
6. `query_with_client_location(...)` simulates resolver location for geo-routing testing; this is the recommended testing path.

## Tests

```bash
cargo test --features hermetic
```

The `hermetic` feature uses `oya_cloud_network_dns_sdk::testkit::Hermetic` with an in-process Knot DNS resolver + SoftHSM-backed
DNSSEC; tests finish in ≤ 30 s.

## Error budget

`DnsError::QueryLatencySloBreached { took_micros }` indicates the resolver exceeded the tier SLO. Do not retry — file
`cloud_network_dns.slo.query_slow` for on-call. Persistent breach signals cell cache miss pathology.
