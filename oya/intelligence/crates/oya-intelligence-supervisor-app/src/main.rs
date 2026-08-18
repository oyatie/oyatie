//! oya-intelligence-supervisor binary entry point.

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use intelligence_account_adapter_inmemory::InMemorySecretStoreAdapter;
use intelligence_autonomy_ceiling_domain::CeilingPolicy;
use intelligence_cli_session_driver::CliSessionDriver;
use intelligence_jsonl_supervisor_adapter::{JsonlInboxStore, JsonlOutboxSink};
use intelligence_settings_template_adapter::{MultiProviderRenderer, TemplateStore};
use intelligence_supervisor_kernel::{
    AccountId, AccountSnapshotProvider, AuditChainPort, ProviderFamily, RendererMode,
    SupervisorAccount, SupervisorConfig, SupervisorError, SupervisorEvent, UsageWindowPort,
    UsageWindowSnapshot,
};
use intelligence_supervisor_security_adapter::CedarAutonomyCeilingAdapter;
use oya_intelligence_supervisor_app::SupervisorApp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .json()
        .init();

    tracing::info!("Starting oya-intelligence-supervisor");

    // 2. Setup config
    let config = SupervisorConfig {
        max_in_flight: 10,
        blocking_pool_size: 20,
        default_cost_ceiling: 1000,
        watchdog_secs: 30,
        heartbeat_interval_secs: 10,
        settings_renderer_mode: RendererMode::Disabled,
        settings_verify_debounce_secs: 60,
        minimum_eligible_accounts: 1,
    };

    // 3. Setup security
    let secrets_adapter = InMemorySecretStoreAdapter::new();
    let ceiling_policy = CeilingPolicy::new();
    let ceiling = CedarAutonomyCeilingAdapter::new(ceiling_policy);
    let audit = LogAuditPort;
    let usage = NoopUsagePort;

    // 4. Setup drivers
    let drivers = vec![
        CliSessionDriver::claude(secrets_adapter.clone()),
        CliSessionDriver::codex(secrets_adapter.clone()),
        CliSessionDriver::gemini(secrets_adapter.clone()),
    ];

    // 5. Setup other ports
    let inbox = JsonlInboxStore::new(".omc/supervisor");
    let outbox = JsonlOutboxSink::new(".omc/supervisor/outbox");
    let accounts = FileAccountSnapshotProvider::new("registry/accounts");
    let renderer = MultiProviderRenderer::new();
    let templates = TemplateStore::new("templates/foundry-supervisor");

    // 6. Initialize App
    let app = SupervisorApp::new(
        drivers, inbox, accounts, outbox, renderer, templates, audit, ceiling, usage, config,
    );

    // 7. Run one tick as a demo
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    match app.tick_once(now).await {
        Ok(outcome) => tracing::info!(?outcome, "Tick completed"),
        Err(e) => tracing::error!(error = ?e, "Tick failed"),
    }

    Ok(())
}

// ── LogAuditPort ─────────────────────────────────────────────────────────────

struct LogAuditPort;
impl AuditChainPort for LogAuditPort {
    fn emit(&self, event: SupervisorEvent) -> Result<(), SupervisorError> {
        tracing::info!(
            event_class = ?event,
            capability = %event.capability_path(),
            "audit event emitted"
        );
        Ok(())
    }
}

// ── NoopUsagePort ────────────────────────────────────────────────────────────

struct NoopUsagePort;
impl UsageWindowPort for NoopUsagePort {
    fn check_usage(
        &self,
        _account_id: &AccountId,
        _now: u64,
    ) -> Result<UsageWindowSnapshot, SupervisorError> {
        Ok(UsageWindowSnapshot {
            started_at_epoch_secs: 0,
            ends_at_epoch_secs: u64::MAX,
            tokens_in: 0,
            tokens_out: 0,
            usage_limit_pct: 100,
            reserve_remaining_pct: 0,
        })
    }
}

// ── FileAccountSnapshotProvider ──────────────────────────────────────────────

struct FileAccountSnapshotProvider {
    dir: std::path::PathBuf,
}

impl FileAccountSnapshotProvider {
    fn new<P: AsRef<std::path::Path>>(path: P) -> Self {
        Self {
            dir: path.as_ref().to_path_buf(),
        }
    }
}

impl AccountSnapshotProvider for FileAccountSnapshotProvider {
    fn snapshot(&self) -> Vec<SupervisorAccount> {
        let mut accounts = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.dir) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("toml")
                    && let Ok(content) = fs::read_to_string(entry.path())
                {
                    let id = content
                        .lines()
                        .find(|l| l.starts_with("id = "))
                        .and_then(|l| l.split('"').nth(1))
                        .unwrap_or("unknown");
                    let family_str = content
                        .lines()
                        .find(|l| l.starts_with("provider_family = "))
                        .and_then(|l| l.split('"').nth(1))
                        .unwrap_or("Claude");
                    let sref_str = content
                        .lines()
                        .find(|l| l.starts_with("secret_ref = "))
                        .and_then(|l| l.split('"').nth(1))
                        .unwrap_or("sref://unknown");

                    let family = match family_str {
                        "Claude" => ProviderFamily::Claude,
                        "OpenAIOrCodex" => ProviderFamily::OpenAiOrCodex,
                        "Gemini" => ProviderFamily::Gemini,
                        _ => ProviderFamily::Claude,
                    };

                    // ADR-0083 Tier 1: SecretReference::new returns Result; on
                    // malformed input, skip the entry rather than `.unwrap()`-
                    // panicking the entire snapshot. Logged to stderr so the
                    // entry visibly drops out of the snapshot.
                    let secret_ref = match intelligence_account_domain::SecretReference::new(
                        sref_str.to_string(),
                    ) {
                        Ok(secret_ref) => secret_ref,
                        Err(error) => {
                            eprintln!(
                                "supervisor snapshot: skipping account {id} — secret_ref invalid: {error:?}"
                            );
                            continue;
                        }
                    };

                    accounts.push(SupervisorAccount {
                        id: AccountId(id.to_string()),
                        provider_family: family,
                        state: intelligence_account_domain::AccountState::Active,
                        secret_ref,
                    });
                }
            }
        }
        accounts
    }
}
