use billing_cost_service::{config, observability};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    observability::init();
    let _cfg = config::load()?;
    // TODO(ADR-0480): wire subsystems
    Ok(())
}
