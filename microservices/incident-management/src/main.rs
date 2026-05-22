use clap::Parser;
use oya_incident_management_sre_incident_command_app::adapter::AdapterRegistry;
use oya_incident_management_sre_incident_command_app::config::ServiceConfig;
use oya_incident_management_sre_incident_command_app::error::Result;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(name = "oya-incident-management-sre-incident-command")]
#[command(about = "Incident Management µservice scaffold entrypoint")]
struct CliArgs {
    #[arg(long)]
    config: Option<PathBuf>,

    #[arg(long, default_value_t = 8080)]
    port: u16,

    #[arg(long, default_value = "local-tenant")]
    tenant_id: String,
}

fn main() -> ExitCode {
    match run(CliArgs::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: CliArgs) -> Result<()> {
    let mut config = if let Some(path) = args.config {
        ServiceConfig::from_toml_file(path)?
    } else {
        ServiceConfig::local_default(args.tenant_id.clone(), args.port)
    };
    config.inbound.port = args.port;
    config.tenant.tenant_id = args.tenant_id;
    config.validate()?;

    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .try_init()
        .ok();

    let registry = AdapterRegistry::scaffolded();
    registry.validate()?;
    tracing::info!(
        service = config.service_name,
        port = config.inbound.port,
        rest_routes = registry.http_routes.len(),
        grpc_methods = registry.grpc_methods.len(),
        asyncapi_channels = registry.asyncapi_channels.len(),
        "incident management scaffold validated"
    );
    Ok(())
}
