#![cfg(all(unix, feature = "async"))]

use std::env;

use intelligence_codex_sdk::{AppServerConfig, AsyncAppCodex};

/// Opt-in smoke test against a real installed `codex app-server`.
///
/// This is ignored and additionally gated by `RUN_CODEX_LIVE_APP_SERVER=1` so
/// default CI never requires credentials, network, or an installed Codex CLI.
#[test]
#[ignore = "requires RUN_CODEX_LIVE_APP_SERVER=1 and a real codex CLI on PATH"]
fn live_app_server_initializes_and_lists_models() -> intelligence_codex_sdk::Result<()> {
    if env::var("RUN_CODEX_LIVE_APP_SERVER").as_deref() != Ok("1") {
        eprintln!("skipping live app-server smoke test; set RUN_CODEX_LIVE_APP_SERVER=1 to run");
        return Ok(());
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async {
        let codex = AsyncAppCodex::new(AppServerConfig::default()).await?;
        assert!(
            codex.metadata().server_info.is_some() || codex.metadata().user_agent.is_some(),
            "initialize should return server metadata"
        );
        let models = codex.models(false).await?;
        assert!(
            models.get("data").is_some(),
            "model/list should return data"
        );
        codex.close().await;
        Ok(())
    })
}
