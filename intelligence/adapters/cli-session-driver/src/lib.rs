//! Provider-CLI session driver.
//!
//! ONE `SessionDriver` implementation for every provider whose account is driven by spawning a
//! vendor CLI as a subprocess. This crate replaces the three crates
//! `intelligence-{claude,codex,gemini}-account-adapter`, whose sources were identical apart
//! from the five values now carried by [`CliDriverSpec`] — a triplication that made every future
//! change to the spawn path a three-place edit and that a straight capability relocation would
//! have carried into the new capability root unchanged.
//!
//! SUBPROCESS, NOT API: spawning a vendor CLI contradicts the cloud-native-API doctrine for the
//! intelligence capability. Collapsing to one crate does not discharge that debt, it localises it:
//! the CLI→typed-HTTP rewrite now has exactly ONE `spawn_for_message` to replace instead of three.
//! `inject_message`/`drain_response`/`kill` remain the placeholders the three source crates
//! shipped; they are preserved verbatim rather than silently "fixed" during a reorg.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` / `panic!()` to assert
// invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::process::Stdio;
use tokio::process::Command;

use intelligence_account_domain::SecretStorePort;
use intelligence_supervisor_kernel::{
    DriverHealth, ProviderFamily, SessionDriver, SessionTicket, SpawnedSession, SupervisorError,
};

/// Everything that differs between one provider CLI and the next.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CliDriverSpec {
    /// Provider family this CLI serves.
    pub family: ProviderFamily,
    /// Executable spawned once per message.
    pub program: &'static str,
    /// Environment variable the resolved secret is handed to the child through.
    pub api_key_env: &'static str,
    /// Prefix of the synthesised session id, and of the placeholder drain payload.
    pub session_prefix: &'static str,
    /// Whether the CLI documents an idempotency-key flag.
    ///
    /// Gemini does NOT (it was demoted to T2 for exactly this reason), so its invocation omits
    /// the flag. This is a real per-provider behavioural difference and the one thing a careless
    /// three-into-one collapse would flatten — `spec_table_preserves_per_provider_behaviour`
    /// fails if it is ever flattened.
    pub idempotency_key_flag: bool,
}

impl CliDriverSpec {
    /// `claude-code`, keyed by `ANTHROPIC_API_KEY`; supports `anthropic-idempotency-key`.
    pub const CLAUDE: Self = Self {
        family: ProviderFamily::Claude,
        program: "claude-code",
        api_key_env: "ANTHROPIC_API_KEY",
        session_prefix: "claude",
        idempotency_key_flag: true,
    };

    /// `codex`, keyed by `OPENAI_API_KEY`; supports `Idempotency-Key`.
    pub const CODEX: Self = Self {
        family: ProviderFamily::OpenAiOrCodex,
        program: "codex",
        api_key_env: "OPENAI_API_KEY",
        session_prefix: "codex",
        idempotency_key_flag: true,
    };

    /// `gemini`, keyed by `GOOGLE_API_KEY`; NO documented idempotency-key support (T2).
    pub const GEMINI: Self = Self {
        family: ProviderFamily::Gemini,
        program: "gemini",
        api_key_env: "GOOGLE_API_KEY",
        session_prefix: "gemini",
        idempotency_key_flag: false,
    };
}

/// A `SessionDriver` that spawns the vendor CLI described by its [`CliDriverSpec`].
pub struct CliSessionDriver<S> {
    spec: CliDriverSpec,
    secrets: S,
}

impl<S> CliSessionDriver<S> {
    pub fn new(spec: CliDriverSpec, secrets: S) -> Self {
        Self { spec, secrets }
    }

    pub fn claude(secrets: S) -> Self {
        Self::new(CliDriverSpec::CLAUDE, secrets)
    }

    pub fn codex(secrets: S) -> Self {
        Self::new(CliDriverSpec::CODEX, secrets)
    }

    pub fn gemini(secrets: S) -> Self {
        Self::new(CliDriverSpec::GEMINI, secrets)
    }

    /// The spec this driver was built from.
    pub fn spec(&self) -> CliDriverSpec {
        self.spec
    }
}

impl<S: SecretStorePort + Send + Sync> SessionDriver for CliSessionDriver<S> {
    fn provider_family(&self) -> ProviderFamily {
        self.spec.family
    }

    fn spawn_for_message(&self, ticket: &SessionTicket) -> Result<SpawnedSession, SupervisorError> {
        let material = self.secrets.get(&ticket.secret_ref).map_err(|e| {
            SupervisorError::DriverError(format!("secret resolution failed: {:?}", e))
        })?;

        let mut command = Command::new(self.spec.program);
        command.arg("--message-id").arg(&ticket.message_id.0);
        if self.spec.idempotency_key_flag {
            command.arg("--idempotency-key").arg(&ticket.request_id.0);
        }
        let _child = command
            .env(
                self.spec.api_key_env,
                String::from_utf8_lossy(material.expose_for_provider_call()).into_owned(),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| {
                SupervisorError::DriverError(format!(
                    "failed to spawn {}: {}",
                    self.spec.program, e
                ))
            })?;

        Ok(SpawnedSession {
            session_id: format!("{}-{}", self.spec.session_prefix, ticket.message_id.0),
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
        Ok(format!("{} response placeholder", self.spec.session_prefix).into_bytes())
    }

    fn kill(&self, _session: &SpawnedSession) -> Result<(), SupervisorError> {
        // Implementation for SIGTERM/SIGKILL
        Ok(())
    }

    fn health_check(&self) -> DriverHealth {
        DriverHealth::Healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The collapse is only correct if the spec table still says exactly what the three separate
    /// crates said. The failure mode this guards is a collapse that unifies the providers by
    /// quietly picking one provider's behaviour for all three — in particular Gemini's MISSING
    /// idempotency-key support, which is a documented T2 demotion and not an oversight.
    #[test]
    fn spec_table_preserves_per_provider_behaviour() {
        for (spec, family, program, env, prefix, idempotent) in [
            (
                CliDriverSpec::CLAUDE,
                ProviderFamily::Claude,
                "claude-code",
                "ANTHROPIC_API_KEY",
                "claude",
                true,
            ),
            (
                CliDriverSpec::CODEX,
                ProviderFamily::OpenAiOrCodex,
                "codex",
                "OPENAI_API_KEY",
                "codex",
                true,
            ),
            (
                CliDriverSpec::GEMINI,
                ProviderFamily::Gemini,
                "gemini",
                "GOOGLE_API_KEY",
                "gemini",
                false,
            ),
        ] {
            assert_eq!(spec.family, family, "{program}: provider family");
            assert_eq!(spec.program, program, "{program}: executable");
            assert_eq!(spec.api_key_env, env, "{program}: api key env var");
            assert_eq!(spec.session_prefix, prefix, "{program}: session prefix");
            assert_eq!(
                spec.idempotency_key_flag, idempotent,
                "{program}: idempotency-key support"
            );
        }
    }

    /// Every provider must map to a DISTINCT family/program/env triple: a copy-paste slip in the
    /// spec table would otherwise route two providers at one CLI, which no per-constant assertion
    /// above would catch on its own if the same wrong value were pasted twice.
    #[test]
    fn specs_are_pairwise_distinct() {
        let specs = [
            CliDriverSpec::CLAUDE,
            CliDriverSpec::CODEX,
            CliDriverSpec::GEMINI,
        ];
        for (i, a) in specs.iter().enumerate() {
            for b in &specs[i + 1..] {
                assert_ne!(a.family, b.family, "duplicate provider family");
                assert_ne!(a.program, b.program, "duplicate executable");
                assert_ne!(a.api_key_env, b.api_key_env, "duplicate api key env var");
                assert_ne!(a.session_prefix, b.session_prefix, "duplicate prefix");
            }
        }
    }
}
