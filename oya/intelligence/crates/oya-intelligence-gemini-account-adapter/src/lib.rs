//! Gemini CLI driver adapter.
//!
//! Implements `SessionDriver` by spawning the `gemini` CLI as a subprocess.
//! Note: Gemini currently lacks documented idempotency-key support; demoted to T2.

use std::process::Stdio;
use tokio::process::Command;

use intelligence_account_domain::SecretStorePort;
use oya_intelligence_supervisor_kernel::{
    DriverHealth, ProviderFamily, SessionDriver, SessionTicket, SpawnedSession, SupervisorError,
};

pub struct GeminiDriver<S> {
    secrets: S,
}

impl<S> GeminiDriver<S> {
    pub fn new(secrets: S) -> Self {
        Self { secrets }
    }
}

impl<S: SecretStorePort + Send + Sync> SessionDriver for GeminiDriver<S> {
    fn provider_family(&self) -> ProviderFamily {
        ProviderFamily::Gemini
    }

    fn spawn_for_message(&self, ticket: &SessionTicket) -> Result<SpawnedSession, SupervisorError> {
        let material = self.secrets.get(&ticket.secret_ref).map_err(|e| {
            SupervisorError::DriverError(format!("secret resolution failed: {:?}", e))
        })?;

        let _child = Command::new("gemini")
            .arg("--message-id")
            .arg(&ticket.message_id.0)
            .env(
                "GOOGLE_API_KEY",
                String::from_utf8_lossy(material.expose_for_provider_call()).into_owned(),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| SupervisorError::DriverError(format!("failed to spawn gemini: {}", e)))?;

        Ok(SpawnedSession {
            session_id: format!("gemini-{}", ticket.message_id.0),
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
        Ok(b"Gemini response placeholder".to_vec())
    }

    fn kill(&self, _session: &SpawnedSession) -> Result<(), SupervisorError> {
        Ok(())
    }

    fn health_check(&self) -> DriverHealth {
        DriverHealth::Healthy
    }
}
