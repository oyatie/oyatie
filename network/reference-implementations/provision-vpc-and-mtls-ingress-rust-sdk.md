# Reference implementation — Provision VPC + deploy service + create mTLS ingress via `cloud-network-sdk`

Runnable Rust program that provisions a tenant VPC, deploys two services, creates an mTLS-enforced HTTP/3 ingress, pushes a
Cedar L7 policy, and tails the flow-log allow/deny stream.

## `Cargo.toml`

```toml
[package]
name = "network-end-to-end-example"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
futures = "0.3"
cloud-network-sdk = "0.42.0"
trace = "0.42.0"
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
tokio = { version = "1.43", features = ["macros", "rt-multi-thread"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## `src/main.rs`

```rust
use anyhow::Result;
use futures::StreamExt;
use cloud_network_sdk::{
    Cidr, FlowVerdict, IngressTlsMode, NetworkClient, NetworkConfig, ServiceSpec, Tenant,
    VpcCreateRequest,
};
use trace::TraceContext;
use std::time::Duration;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let trace = TraceContext::new_root();

    let cfg = NetworkConfig::builder()
        .endpoint("https://loopback.cloud-network.oyatie.local".parse()?)
        .tenant(Tenant::parse("oyatie.b2b.smb.acme-software")?)
        .service_account_credentials_path("/etc/oya/network/sa-creds.json")
        .request_timeout(Duration::from_secs(20))
        .build()?;

    let client = NetworkClient::connect(cfg).await?;
    info!("connected to cloud-network");

    // 1. Provision VPC
    let vpc = client
        .vpc_create(
            VpcCreateRequest::builder()
                .region("loopback-us-east-2")
                .cidr(Cidr::parse("10.214.0.0/22")?)
                .availability_zones(3)
                .enable_ipv6(true)
                .build()?,
            trace.child(),
        )
        .await?;
    info!(
        vpc_id = %vpc.id(),
        cidr = %vpc.cidr(),
        ipv6_cidr = ?vpc.ipv6_cidr(),
        subnet_count = vpc.subnets().len(),
        "vpc provisioned"
    );

    // 2. Deploy webapp + pgproxy
    let webapp = client
        .service_apply(
            ServiceSpec::builder()
                .name("webapp")
                .image("ghcr.io/samples/echo:0.4.1")
                .replicas(2)
                .port("http", 8080)
                .spiffe_id("spiffe://oyatie.b2b.smb.acme-software/webapp")
                .build()?,
            trace.child(),
        )
        .await?;
    let pgproxy = client
        .service_apply(
            ServiceSpec::builder()
                .name("pgproxy")
                .image("ghcr.io/samples/pgproxy:0.2.0")
                .replicas(1)
                .port("postgres", 5432)
                .spiffe_id("spiffe://oyatie.b2b.smb.acme-software/pgproxy")
                .build()?,
            trace.child(),
        )
        .await?;
    info!(webapp_id = %webapp.id(), pgproxy_id = %pgproxy.id(), "services deployed");

    // 3. mTLS ingress (HTTP/3 default)
    let ingress = client
        .ingress_create(
            "webapp",
            "acme-webapp.loopback.oyatie.local",
            IngressTlsMode::Mtls {
                client_ca_source: "spiffe://oyatie.b2b.smb.acme-software/clients".into(),
                alpn_default: "h3".into(),
            },
            trace.child(),
        )
        .await?;
    info!(ingress_id = %ingress.id(), public_url = %ingress.public_url(), "ingress created");

    // 4. Push a Cedar policy
    let cedar = r#"
permit (
  principal in Workload::"oyatie.b2b.smb.acme-software/webapp",
  action == cloud_network::Action::EstablishFlow,
  resource == Service::"oyatie.b2b.smb.acme-software/pgproxy"
)
when {
  context.flow.destination_port == 5432 &&
  context.flow.protocol == "tcp" &&
  context.session.spiffe_svid_valid == true
};
"#;
    client.policy_push("webapp-to-pgproxy", cedar, /* enforce= */ true, trace.child()).await?;
    info!("cedar policy pushed");

    // 5. Tail flow log for 30 s, count allow/deny
    let mut allow = 0u64;
    let mut deny = 0u64;
    let mut stream = client.flow_log_subscribe(trace.child()).await?;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    while let Some(event) = tokio::time::timeout(
        deadline.duration_since(tokio::time::Instant::now()),
        stream.next(),
    )
    .await
    .ok()
    .flatten()
    {
        let event = event?;
        match event.verdict() {
            FlowVerdict::Allow => allow += 1,
            FlowVerdict::Deny => deny += 1,
        }
    }
    info!(allow, deny, "flow log tail complete");

    Ok(())
}
```

## Run it

```bash
cargo run --release
```

Expected output (trimmed):
```
INFO  connected to cloud-network
INFO  vpc provisioned vpc_id=vpc-… cidr=10.214.0.0/22 ipv6_cidr=Some("2400:…::/56") subnet_count=6
INFO  services deployed webapp_id=svc-webapp-… pgproxy_id=svc-pgproxy-…
INFO  ingress created ingress_id=ing-… public_url=https://acme-webapp.loopback.oyatie.local
INFO  cedar policy pushed
INFO  flow log tail complete allow=412 deny=3
```

## SDK correctness guarantees

1. `vpc_create(...)` validates the CIDR against the tenant's allocation budget (`/22` for paid, `/20` for paid, `/16` for
   paid). Refusal is `NetworkError::CidrBudgetExceeded`.
2. `service_apply(...)` is idempotent on the service `name + tenant`. Re-applying with a different `image` is treated as a
   rolling update.
3. `ingress_create(...)` defaults `alpn_default` to `h3` per ADR-0253; explicit `h2` requires
   `cloud_network::Action::AllowHttp2OnlyRoute`.
4. `policy_push(...)` lints Cedar before sending; the lint includes the `lean-a3-tenant-trace` check, refusing cross-tenant
   references.
5. `flow_log_subscribe(...)` returns a backpressured stream — slow consumers get sampled rather than blocked.

## Tests

```bash
cargo test --features hermetic
```

The `hermetic` feature uses `cloud_network_sdk::testkit::Hermetic` to spin a single-process loopback cell with Cilium in
kernel-bypass mode (XDP) and a SoftHSM-backed SPIFFE; tests finish in ≤ 60 s.

## Error budget

`NetworkError::PolicyDecisionSloBreached { took_micros }` indicates a policy lookup exceeded the 1.4 µs p95 target. Do not retry —
file a `cloud_network.slo.policy_decision_slow` event. Persistent breach signals eBPF map saturation; the on-call rotation rebalances.

`NetworkError::FlowLogBackpressureSampled { dropped_count }` is normal when consumers fall behind; it is a hint to add more
consumers or reduce subscription scope.
