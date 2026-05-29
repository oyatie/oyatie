//! oya-identity binary entrypoint.

use oya_identity::{config, observability};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    observability::init();
    let _cfg = config::load()?;
    // TODO(ADR-0476): wire subsystems + start rest/grpc servers
    Ok(())
}
