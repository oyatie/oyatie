//! oya-meter binary entry point (ADR-0479).

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().json().init();
    oya_meter_app::run().await;
}
