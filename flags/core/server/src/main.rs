use flags_server::{config, observability};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    observability::init();
    let _cfg = config::load()?;
    // TODO(ADR-0481): wire subsystems
    Ok(())
}
