#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use oya_learning_management_course_progress_service::{
    ServiceConfig, descriptor, validate_scaffold,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServiceConfig::from_env()?;
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .json()
        .init();

    validate_scaffold()?;
    let descriptor = descriptor();
    tracing::info!(
        service = config.service_name,
        profile = config.runtime_profile.as_str(),
        layers = descriptor.layer_count(),
        contracts = descriptor.contract_count(),
        "learning-management scaffold booted"
    );
    Ok(())
}
