//! Per-seat token state: access token, refresh token, expiry, and error classification.
// data_class: INTERNAL_ONLY throughout this module.

use std::fmt;

use crate::ports::AlertKind;

/// Duration in seconds before token expiry to trigger a proactive refresh.
pub const EXPIRES_LEAD_SECS: u64 = 300; // 5 minutes lead

/// Quarantine duration for terminal errors (24 hours in seconds).
pub const TERMINAL_BACKOFF_SECS: u64 = 86_400;

/// How token bytes are serialized for storage (JSON envelope; raw value stays internal).
// data_class: INTERNAL_ONLY
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct StoredTokenEnvelope {
    pub access_token: String,  // data_class: INTERNAL_ONLY
    pub refresh_token: String, // data_class: INTERNAL_ONLY
    pub expires_at: u64,       // unix epoch secs
    pub issued_at: u64,        // unix epoch secs
}

impl fmt::Debug for StoredTokenEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StoredTokenEnvelope(issued={}, expires={}, tokens=[REDACTED])",
            self.issued_at, self.expires_at
        )
    }
}

/// Classification of a refresh failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RefreshFailureKind {
    /// Terminal: no automatic retry; 24h quarantine; operator alert required.
    Terminal(AlertKind),
    /// Transient: retry with exponential backoff; no operator alert.
    Transient,
}

/// Classify the OAuth error field from the token endpoint into terminal vs transient.
/// Terminal errors per Anthropic OAuth spec: refresh_token_expired, reused, invalidated.
pub fn classify_oauth_error(error: &str) -> RefreshFailureKind {
    match error {
        "refresh_token_expired" => RefreshFailureKind::Terminal(AlertKind::RefreshTokenExpired),
        "refresh_token_reused" => RefreshFailureKind::Terminal(AlertKind::RefreshTokenReused),
        "refresh_token_invalidated" => {
            RefreshFailureKind::Terminal(AlertKind::RefreshTokenInvalidated)
        }
        _ => RefreshFailureKind::Transient,
    }
}

/// Live per-seat token state.
// data_class: INTERNAL_ONLY
#[derive(Clone)]
pub struct SeatTokenState {
    pub access_token: String,  // data_class: INTERNAL_ONLY
    pub refresh_token: String, // data_class: INTERNAL_ONLY
    pub expires_at: u64,       // unix epoch secs
    pub issued_at: u64,        // unix epoch secs
    /// When set, the seat is in terminal quarantine until this unix epoch.
    pub terminal_until: Option<u64>,
}

impl fmt::Debug for SeatTokenState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SeatTokenState(issued={}, expires={}, terminal_until={:?}, tokens=[REDACTED])",
            self.issued_at, self.expires_at, self.terminal_until
        )
    }
}

impl SeatTokenState {
    pub fn new(
        access_token: String,
        refresh_token: String,
        expires_at: u64,
        issued_at: u64,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            expires_at,
            issued_at,
            terminal_until: None,
        }
    }

    /// Return true if the token is still valid at `now_secs` (with optional lead window).
    pub fn is_valid_at(&self, now_secs: u64) -> bool {
        self.terminal_until.is_none() && now_secs < self.expires_at
    }

    /// Return true if a proactive refresh should be triggered (within lead window).
    pub fn needs_refresh_at(&self, now_secs: u64) -> bool {
        if self.terminal_until.is_some() {
            return false;
        }
        now_secs.saturating_add(EXPIRES_LEAD_SECS) >= self.expires_at
    }

    /// Unix epoch at which the next refresh should be triggered (expires_at - lead).
    pub fn next_refresh_due(&self) -> u64 {
        self.expires_at.saturating_sub(EXPIRES_LEAD_SECS)
    }

    /// Mark the seat as terminal-quarantined from `now_secs` for TERMINAL_BACKOFF_SECS.
    pub fn mark_terminal(&mut self, now_secs: u64) {
        self.terminal_until = Some(now_secs.saturating_add(TERMINAL_BACKOFF_SECS));
    }

    /// Encode to storage bytes (JSON envelope).
    pub fn to_storage_bytes(&self) -> Result<Vec<u8>, String> {
        let env = StoredTokenEnvelope {
            access_token: self.access_token.clone(),
            refresh_token: self.refresh_token.clone(),
            expires_at: self.expires_at,
            issued_at: self.issued_at,
        };
        serde_json::to_vec(&env).map_err(|e| e.to_string())
    }

    /// Decode from storage bytes.
    pub fn from_storage_bytes(bytes: &[u8]) -> Result<Self, String> {
        let env: StoredTokenEnvelope = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        Ok(Self::new(
            env.access_token,
            env.refresh_token,
            env.expires_at,
            env.issued_at,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_token_is_valid_before_expiry() {
        let s = SeatTokenState::new("a".into(), "r".into(), 1000, 0);
        assert!(s.is_valid_at(999));
        assert!(!s.is_valid_at(1000));
        assert!(!s.is_valid_at(2000));
    }

    #[test]
    fn needs_refresh_within_lead_window() {
        let lead = EXPIRES_LEAD_SECS;
        let s = SeatTokenState::new("a".into(), "r".into(), 1000, 0);
        // At now = 1000 - lead - 1: NOT yet in lead window.
        assert!(!s.needs_refresh_at(1000 - lead - 1));
        // At now = 1000 - lead: exactly at boundary → should refresh.
        assert!(s.needs_refresh_at(1000 - lead));
        // At now = 1000 - lead + 1: inside window → should refresh.
        assert!(s.needs_refresh_at(1000 - lead + 1));
    }

    #[test]
    fn terminal_state_suppresses_refresh() {
        let mut s = SeatTokenState::new("a".into(), "r".into(), 1000, 0);
        s.mark_terminal(0);
        assert!(!s.needs_refresh_at(0));
        assert!(!s.is_valid_at(500));
    }

    #[test]
    fn terminal_quarantine_duration() {
        let mut s = SeatTokenState::new("a".into(), "r".into(), 1000, 0);
        s.mark_terminal(100);
        assert_eq!(s.terminal_until, Some(100 + TERMINAL_BACKOFF_SECS));
    }

    #[test]
    fn debug_output_is_redacted() {
        let s = SeatTokenState::new(
            "super-secret-access".into(),
            "super-secret-refresh".into(),
            1000,
            0,
        );
        let dbg = format!("{s:?}");
        assert!(dbg.contains("[REDACTED]"));
        assert!(!dbg.contains("super-secret"));
    }

    #[test]
    fn roundtrip_storage_bytes() {
        let s = SeatTokenState::new("acc".into(), "ref".into(), 9999, 1000);
        let bytes = s.to_storage_bytes().unwrap();
        let s2 = SeatTokenState::from_storage_bytes(&bytes).unwrap();
        assert_eq!(s2.access_token, "acc");
        assert_eq!(s2.refresh_token, "ref");
        assert_eq!(s2.expires_at, 9999);
        assert_eq!(s2.issued_at, 1000);
    }

    #[test]
    fn classify_terminal_errors() {
        assert!(matches!(
            classify_oauth_error("refresh_token_expired"),
            RefreshFailureKind::Terminal(AlertKind::RefreshTokenExpired)
        ));
        assert!(matches!(
            classify_oauth_error("refresh_token_reused"),
            RefreshFailureKind::Terminal(AlertKind::RefreshTokenReused)
        ));
        assert!(matches!(
            classify_oauth_error("refresh_token_invalidated"),
            RefreshFailureKind::Terminal(AlertKind::RefreshTokenInvalidated)
        ));
    }

    #[test]
    fn classify_unknown_error_is_transient() {
        assert_eq!(
            classify_oauth_error("server_error"),
            RefreshFailureKind::Transient
        );
        assert_eq!(
            classify_oauth_error("network_unavailable"),
            RefreshFailureKind::Transient
        );
    }
}
