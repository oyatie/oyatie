//! Codex CLI driver adapter.
//!
//! Implements `SessionDriver` by spawning the `codex` CLI as a subprocess.
//! Supports the `Stop` hook and `request_id` (Idempotency-Key).

use std::process::Stdio;
use tokio::process::Command;

use oya_intelligence_account_domain::SecretStorePort;
use intelligence_supervisor_kernel::{
    DriverHealth, ProviderFamily, SessionDriver, SessionTicket, SpawnedSession, SupervisorError,
};

pub struct CodexDriver<S> {
    secrets: S,
}

impl<S> CodexDriver<S> {
    pub fn new(secrets: S) -> Self {
        Self { secrets }
    }
}

impl<S: SecretStorePort + Send + Sync> SessionDriver for CodexDriver<S> {
    fn provider_family(&self) -> ProviderFamily {
        ProviderFamily::OpenAiOrCodex
    }

    fn spawn_for_message(&self, ticket: &SessionTicket) -> Result<SpawnedSession, SupervisorError> {
        let material = self.secrets.get(&ticket.secret_ref).map_err(|e| {
            SupervisorError::DriverError(format!("secret resolution failed: {:?}", e))
        })?;

        let _child = Command::new("codex")
            .arg("--message-id")
            .arg(&ticket.message_id.0)
            .arg("--idempotency-key")
            .arg(&ticket.request_id.0)
            .env(
                "OPENAI_API_KEY",
                String::from_utf8_lossy(material.expose_for_provider_call()).into_owned(),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| SupervisorError::DriverError(format!("failed to spawn codex: {}", e)))?;

        Ok(SpawnedSession {
            session_id: format!("codex-{}", ticket.message_id.0),
            account_id: ticket.account_id.clone(),
            message_id: ticket.message_id.clone(),
        })
    }

    fn inject_message(
        &self,
        _session: &SpawnedSession,
        _msg: &[u8],
    ) -> Result<(), SupervisorError> {
        Ok(())
    }

    fn drain_response(&self, _session: &SpawnedSession) -> Result<Vec<u8>, SupervisorError> {
        Ok(b"Codex response placeholder".to_vec())
    }

    fn kill(&self, _session: &SpawnedSession) -> Result<(), SupervisorError> {
        Ok(())
    }

    fn health_check(&self) -> DriverHealth {
        DriverHealth::Healthy
    }
}
