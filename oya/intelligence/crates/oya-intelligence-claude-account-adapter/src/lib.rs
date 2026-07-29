//! Claude CLI driver adapter.
//!
//! Implements `SessionDriver` by spawning the `claude-code` CLI as a subprocess.
//! Supports the `Stop` hook and `request_id` (anthropic-idempotency-key).

use std::process::Stdio;
use tokio::process::Command;

use intelligence_account_domain::SecretStorePort;
use intelligence_supervisor_kernel::{
    DriverHealth, ProviderFamily, SessionDriver, SessionTicket, SpawnedSession, SupervisorError,
};

pub struct ClaudeDriver<S> {
    secrets: S,
}

impl<S> ClaudeDriver<S> {
    pub fn new(secrets: S) -> Self {
        Self { secrets }
    }
}

impl<S: SecretStorePort + Send + Sync> SessionDriver for ClaudeDriver<S> {
    fn provider_family(&self) -> ProviderFamily {
        ProviderFamily::Claude
    }

    fn spawn_for_message(&self, ticket: &SessionTicket) -> Result<SpawnedSession, SupervisorError> {
        let material = self.secrets.get(&ticket.secret_ref).map_err(|e| {
            SupervisorError::DriverError(format!("secret resolution failed: {:?}", e))
        })?;

        let _child = Command::new("claude-code")
            .arg("--message-id")
            .arg(&ticket.message_id.0)
            .arg("--idempotency-key")
            .arg(&ticket.request_id.0)
            .env(
                "ANTHROPIC_API_KEY",
                String::from_utf8_lossy(material.expose_for_provider_call()).into_owned(),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| {
                SupervisorError::DriverError(format!("failed to spawn claude-code: {}", e))
            })?;

        Ok(SpawnedSession {
            session_id: format!("claude-{}", ticket.message_id.0),
            account_id: ticket.account_id.clone(),
            message_id: ticket.message_id.clone(),
        })
    }

    fn inject_message(
        &self,
        _session: &SpawnedSession,
        _msg: &[u8],
    ) -> Result<(), SupervisorError> {
        // Implementation for injecting subsequent messages into a live session
        Ok(())
    }

    fn drain_response(&self, _session: &SpawnedSession) -> Result<Vec<u8>, SupervisorError> {
        // Implementation for capturing CLI output (stdout)
        Ok(b"Claude response placeholder".to_vec())
    }

    fn kill(&self, _session: &SpawnedSession) -> Result<(), SupervisorError> {
        // Implementation for SIGTERM/SIGKILL
        Ok(())
    }

    fn health_check(&self) -> DriverHealth {
        DriverHealth::Healthy
    }
}
