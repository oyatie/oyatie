use oya_meter::{config, observability};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    observability::init();
    let _cfg = config::load()?;
    // TODO(ADR-0479): wire subsystems
    Ok(())
}
