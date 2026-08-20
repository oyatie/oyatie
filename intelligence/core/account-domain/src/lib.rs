//! M02-P00-IP-002 — ProviderAccount domain types + state machine.
//! SecretReference inner value is NEVER exposed; sref:// scheme enforced.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;

// ── Identity types (live in kernel; re-exported here for back-compat) ──────
// Per ADR-0056 port-in-kernel: types that cross product boundaries belong
// in the kernel. Existing call sites continue to import from domain.
pub use intelligence_account_kernel::{
    AccountId, ProviderFamily, ProviderFamilyError, SecretReference, SecretReferenceError,
    SessionId,
};

// ── AccountState + transitions ───────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccountState {
    Draft,
    Verified,
    Active,
    Degraded { reason: String },
    Disabled { reason: String },
    Revoked,
}

impl AccountState {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Draft => "Draft",
            Self::Verified => "Verified",
            Self::Active => "Active",
            Self::Degraded { .. } => "Degraded",
            Self::Disabled { .. } => "Disabled",
            Self::Revoked => "Revoked",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccountError {
    InvalidTransition {
        from: &'static str,
        to: &'static str,
    },
    RevokedIsTerminal,
    MustVerifyBeforeActivate,
    SilentSwitchPrevented,
}

impl fmt::Display for AccountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransition { from, to } => write!(f, "invalid transition: {from} → {to}"),
            Self::RevokedIsTerminal => write!(f, "Revoked is terminal; no transitions allowed"),
            Self::MustVerifyBeforeActivate => {
                write!(f, "must reach Verified state before activating")
            }
            Self::SilentSwitchPrevented => write!(
                f,
                "silent account switch prevented; explicit audit required"
            ),
        }
    }
}

// ── ProviderAccount ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderAccount {
    pub id: AccountId,
    pub provider_family: ProviderFamily,
    pub state: AccountState,
    pub subscription_id: Option<String>,
}

impl ProviderAccount {
    pub fn new(id: AccountId, provider_family: ProviderFamily) -> Self {
        Self {
            id,
            provider_family,
            state: AccountState::Draft,
            subscription_id: None,
        }
    }

    pub fn with_subscription(mut self, subscription_id: String) -> Self {
        self.subscription_id = Some(subscription_id);
        self
    }

    pub fn verify(&mut self, _secret: SecretReference) -> Result<(), AccountError> {
        match &self.state {
            AccountState::Draft => {
                self.state = AccountState::Verified;
                Ok(())
            }
            AccountState::Revoked => Err(AccountError::RevokedIsTerminal),
            s => Err(AccountError::InvalidTransition {
                from: s.label(),
                to: "Verified",
            }),
        }
    }

    pub fn activate(&mut self) -> Result<(), AccountError> {
        match &self.state {
            AccountState::Draft => Err(AccountError::MustVerifyBeforeActivate),
            AccountState::Verified | AccountState::Degraded { .. } => {
                self.state = AccountState::Active;
                Ok(())
            }
            AccountState::Revoked => Err(AccountError::RevokedIsTerminal),
            s => Err(AccountError::InvalidTransition {
                from: s.label(),
                to: "Active",
            }),
        }
    }

    pub fn degrade(&mut self, reason: String) -> Result<(), AccountError> {
        match &self.state {
            AccountState::Active => {
                self.state = AccountState::Degraded { reason };
                Ok(())
            }
            AccountState::Revoked => Err(AccountError::RevokedIsTerminal),
            s => Err(AccountError::InvalidTransition {
                from: s.label(),
                to: "Degraded",
            }),
        }
    }

    pub fn recover(&mut self) -> Result<(), AccountError> {
        match &self.state {
            AccountState::Degraded { .. } => {
                self.state = AccountState::Active;
                Ok(())
            }
            AccountState::Revoked => Err(AccountError::RevokedIsTerminal),
            s => Err(AccountError::InvalidTransition {
                from: s.label(),
                to: "Active",
            }),
        }
    }

    pub fn disable(&mut self, reason: String) -> Result<(), AccountError> {
        match &self.state {
            AccountState::Revoked => Err(AccountError::RevokedIsTerminal),
            _ => {
                self.state = AccountState::Disabled { reason };
                Ok(())
            }
        }
    }

    pub fn revoke(&mut self) -> Result<(), AccountError> {
        match &self.state {
            AccountState::Revoked => Err(AccountError::RevokedIsTerminal),
            _ => {
                self.state = AccountState::Revoked;
                Ok(())
            }
        }
    }
}

// ── Silent-switch guard ──────────────────────────────────────────────────────

pub fn check_silent_switch(
    existing: &[&ProviderAccount],
    candidate: &ProviderAccount,
) -> Result<(), AccountError> {
    for acc in existing {
        if acc.id != candidate.id
            && acc.provider_family == candidate.provider_family
            && acc.subscription_id == candidate.subscription_id
            && acc.state == AccountState::Active
            && candidate.state == AccountState::Active
        {
            return Err(AccountError::SilentSwitchPrevented);
        }
    }
    Ok(())
}

// ── AuthSession ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityGrant(pub String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivacyBoundary(pub String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthSession {
    pub id: SessionId,
    pub account_id: AccountId,
    pub provider_family: ProviderFamily,
    pub started_at_epoch_secs: u64,
    pub expires_at_epoch_secs: u64,
    pub capability_grant: CapabilityGrant,
    pub privacy_boundary: PrivacyBoundary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionError(pub String);

impl AuthSession {
    pub fn new(
        id: SessionId,
        account_id: AccountId,
        provider_family: ProviderFamily,
        started_at_epoch_secs: u64,
        expires_at_epoch_secs: u64,
        capability_grant: CapabilityGrant,
        privacy_boundary: PrivacyBoundary,
    ) -> Result<Self, SessionError> {
        if expires_at_epoch_secs <= started_at_epoch_secs {
            return Err(SessionError(
                "expires_at must be after started_at".to_owned(),
            ));
        }
        if privacy_boundary.0.is_empty() {
            return Err(SessionError(
                "privacy_boundary must not be empty".to_owned(),
            ));
        }
        Ok(Self {
            id,
            account_id,
            provider_family,
            started_at_epoch_secs,
            expires_at_epoch_secs,
            capability_grant,
            privacy_boundary,
        })
    }
}

// ── UsageWindow ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UsageWindowKind {
    FiveHour,
    OneWeek,
    Project,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsageWindow {
    pub kind: UsageWindowKind,
    pub started_at_epoch_secs: u64,
    pub ends_at_epoch_secs: u64,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub cache_hits: u64,
    pub estimated_cost_micros: u64,
    pub usage_limit_pct: u8,
    pub reserve_remaining_pct: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsageWindowError(pub String);

impl UsageWindow {
    pub fn new(
        kind: UsageWindowKind,
        started_at_epoch_secs: u64,
        ends_at_epoch_secs: u64,
        usage_limit_pct: u8,
        reserve_remaining_pct: u8,
    ) -> Result<Self, UsageWindowError> {
        if ends_at_epoch_secs <= started_at_epoch_secs {
            return Err(UsageWindowError(
                "ends_at must be after started_at".to_owned(),
            ));
        }
        if usage_limit_pct > 100 {
            return Err(UsageWindowError(
                "usage_limit_pct must be in [0, 100]".to_owned(),
            ));
        }
        if reserve_remaining_pct > 100 {
            return Err(UsageWindowError(
                "reserve_remaining_pct must be in [0, 100]".to_owned(),
            ));
        }
        Ok(Self {
            kind,
            started_at_epoch_secs,
            ends_at_epoch_secs,
            tokens_in: 0,
            tokens_out: 0,
            cache_hits: 0,
            estimated_cost_micros: 0,
            usage_limit_pct,
            reserve_remaining_pct,
        })
    }
}

// ── Supporting types ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountHealth {
    pub account_id: AccountId,
    pub is_healthy: bool,
    pub reason: Option<String>,
    pub last_check_at_epoch_secs: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Quota {
    pub limit_tokens: u64,
    pub current_tokens: u64,
    pub reserve_remaining_pct: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteExplanation {
    pub chosen_provider: ProviderFamily,
    pub chosen_account_id: AccountId,
    pub chosen_model: String,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenCost {
    pub estimated_micros: u64,
    pub actual_micros: Option<u64>,
    pub model: String,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub cache_hit_pct: u8,
}

// ── SecretStorePort + SecretMaterial ─────────────────────────────────────────
// SECURITY: SecretMaterial inner bytes MUST NOT be exposed. No Display. Debug shows redacted.

#[derive(Clone, Eq, PartialEq)]
pub struct SecretMaterial(Vec<u8>);

impl SecretMaterial {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
    pub fn expose_for_provider_call(&self) -> &[u8] {
        &self.0
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for SecretMaterial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretMaterial([REDACTED; {} bytes])", self.0.len())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SecretStoreError {
    NotFound,
    Backend(String),
    InvalidReference,
}

impl fmt::Display for SecretStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "secret reference not found"),
            Self::Backend(s) => write!(f, "secret store backend error: {s}"),
            Self::InvalidReference => write!(f, "invalid secret reference"),
        }
    }
}

pub trait SecretStorePort {
    fn put(
        &mut self,
        sref: &SecretReference,
        material: SecretMaterial,
    ) -> Result<(), SecretStoreError>;
    fn get(&self, sref: &SecretReference) -> Result<SecretMaterial, SecretStoreError>;
    fn rotate(
        &mut self,
        sref: &SecretReference,
        new_material: SecretMaterial,
    ) -> Result<(), SecretStoreError>;
    fn delete(&mut self, sref: &SecretReference) -> Result<(), SecretStoreError>;
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn aid(s: &str) -> AccountId {
        AccountId(s.to_owned())
    }
    fn sid(s: &str) -> SessionId {
        SessionId(s.to_owned())
    }
    fn sref(s: &str) -> SecretReference {
        SecretReference::new(s.to_owned()).unwrap()
    }
    fn new_account(family: ProviderFamily) -> ProviderAccount {
        ProviderAccount::new(aid("acct-1"), family)
    }

    #[test]
    fn provider_family_aws() {
        assert_eq!(ProviderFamily::try_from("AWS"), Ok(ProviderFamily::Aws));
    }
    #[test]
    fn provider_family_oci() {
        assert_eq!(ProviderFamily::try_from("OCI"), Ok(ProviderFamily::Oci));
    }
    #[test]
    fn provider_family_claude() {
        assert_eq!(
            ProviderFamily::try_from("Claude"),
            Ok(ProviderFamily::Claude)
        );
    }
    #[test]
    fn provider_family_openai_codex() {
        assert_eq!(
            ProviderFamily::try_from("OpenAIOrCodex"),
            Ok(ProviderFamily::OpenAiOrCodex)
        );
    }
    #[test]
    fn provider_family_gemini() {
        assert_eq!(
            ProviderFamily::try_from("Gemini"),
            Ok(ProviderFamily::Gemini)
        );
    }
    #[test]
    fn provider_family_rejects_unknown() {
        assert!(ProviderFamily::try_from("Anthropic").is_err());
    }
    #[test]
    fn provider_family_rejects_empty() {
        assert!(ProviderFamily::try_from("").is_err());
    }
    #[test]
    fn provider_family_rejects_lowercase() {
        assert!(ProviderFamily::try_from("aws").is_err());
    }
    #[test]
    fn provider_family_rejects_partial() {
        assert!(ProviderFamily::try_from("Claud").is_err());
    }
    #[test]
    fn provider_family_error_contains_name() {
        let err = ProviderFamily::try_from("BadProvider").unwrap_err();
        assert!(err.0.contains("BadProvider"));
    }

    #[test]
    fn secret_ref_valid_sref_scheme() {
        assert!(SecretReference::new("sref://my-secret-id".to_owned()).is_ok());
    }
    #[test]
    fn secret_ref_rejects_non_sref_scheme() {
        assert!(SecretReference::new("http://my-secret".to_owned()).is_err());
    }
    #[test]
    fn secret_ref_rejects_bare_sref() {
        assert!(SecretReference::new("sref://".to_owned()).is_err());
    }
    #[test]
    fn secret_ref_debug_is_redacted() {
        let sr = sref("sref://super-secret-value");
        let dbg = format!("{sr:?}");
        assert!(dbg.contains("[REDACTED]"));
        assert!(!dbg.contains("super-secret"));
    }

    #[test]
    fn state_initial_is_draft() {
        let acc = new_account(ProviderFamily::Claude);
        assert_eq!(acc.state, AccountState::Draft);
    }

    #[test]
    fn state_draft_to_verified() {
        let mut acc = new_account(ProviderFamily::Claude);
        acc.verify(sref("sref://s1")).unwrap();
        assert_eq!(acc.state, AccountState::Verified);
    }

    #[test]
    fn state_verified_to_active() {
        let mut acc = new_account(ProviderFamily::Claude);
        acc.verify(sref("sref://s1")).unwrap();
        acc.activate().unwrap();
        assert_eq!(acc.state, AccountState::Active);
    }

    #[test]
    fn state_active_to_degraded() {
        let mut acc = new_account(ProviderFamily::Gemini);
        acc.verify(sref("sref://s1")).unwrap();
        acc.activate().unwrap();
        acc.degrade("quota exceeded".to_owned()).unwrap();
        assert_eq!(
            acc.state,
            AccountState::Degraded {
                reason: "quota exceeded".to_owned()
            }
        );
    }

    #[test]
    fn state_degraded_to_active_via_recover() {
        let mut acc = new_account(ProviderFamily::Gemini);
        acc.verify(sref("sref://s1")).unwrap();
        acc.activate().unwrap();
        acc.degrade("temp outage".to_owned()).unwrap();
        acc.recover().unwrap();
        assert_eq!(acc.state, AccountState::Active);
    }

    #[test]
    fn state_active_to_disabled() {
        let mut acc = new_account(ProviderFamily::Claude);
        acc.verify(sref("sref://s1")).unwrap();
        acc.activate().unwrap();
        acc.disable("user request".to_owned()).unwrap();
        assert_eq!(
            acc.state,
            AccountState::Disabled {
                reason: "user request".to_owned()
            }
        );
    }

    #[test]
    fn state_draft_to_disabled() {
        let mut acc = new_account(ProviderFamily::Aws);
        acc.disable("invalid credentials".to_owned()).unwrap();
        assert_eq!(
            acc.state,
            AccountState::Disabled {
                reason: "invalid credentials".to_owned()
            }
        );
    }

    #[test]
    fn state_verified_to_disabled() {
        let mut acc = new_account(ProviderFamily::Aws);
        acc.verify(sref("sref://s1")).unwrap();
        acc.disable("policy violation".to_owned()).unwrap();
        assert_eq!(
            acc.state,
            AccountState::Disabled {
                reason: "policy violation".to_owned()
            }
        );
    }

    #[test]
    fn state_degraded_to_disabled() {
        let mut acc = new_account(ProviderFamily::Oci);
        acc.verify(sref("sref://s1")).unwrap();
        acc.activate().unwrap();
        acc.degrade("network issue".to_owned()).unwrap();
        acc.disable("persistent failure".to_owned()).unwrap();
        assert!(matches!(acc.state, AccountState::Disabled { .. }));
    }

    #[test]
    fn state_active_to_revoked() {
        let mut acc = new_account(ProviderFamily::OpenAiOrCodex);
        acc.verify(sref("sref://s1")).unwrap();
        acc.activate().unwrap();
        acc.revoke().unwrap();
        assert_eq!(acc.state, AccountState::Revoked);
    }

    #[test]
    fn state_draft_to_revoked() {
        let mut acc = new_account(ProviderFamily::Claude);
        acc.revoke().unwrap();
        assert_eq!(acc.state, AccountState::Revoked);
    }

    #[test]
    fn state_verified_to_revoked() {
        let mut acc = new_account(ProviderFamily::Gemini);
        acc.verify(sref("sref://s1")).unwrap();
        acc.revoke().unwrap();
        assert_eq!(acc.state, AccountState::Revoked);
    }

    #[test]
    fn state_degraded_to_revoked() {
        let mut acc = new_account(ProviderFamily::Claude);
        acc.verify(sref("sref://s1")).unwrap();
        acc.activate().unwrap();
        acc.degrade("issue".to_owned()).unwrap();
        acc.revoke().unwrap();
        assert_eq!(acc.state, AccountState::Revoked);
    }

    #[test]
    fn state_disabled_to_revoked() {
        let mut acc = new_account(ProviderFamily::Aws);
        acc.disable("reason".to_owned()).unwrap();
        acc.revoke().unwrap();
        assert_eq!(acc.state, AccountState::Revoked);
    }

    #[test]
    fn invalid_draft_to_active_directly() {
        let mut acc = new_account(ProviderFamily::Claude);
        assert_eq!(acc.activate(), Err(AccountError::MustVerifyBeforeActivate));
    }

    #[test]
    fn invalid_revoked_to_verified() {
        let mut acc = new_account(ProviderFamily::Claude);
        acc.revoke().unwrap();
        assert_eq!(
            acc.verify(sref("sref://s1")),
            Err(AccountError::RevokedIsTerminal)
        );
    }

    #[test]
    fn invalid_revoked_to_active() {
        let mut acc = new_account(ProviderFamily::Claude);
        acc.revoke().unwrap();
        assert_eq!(acc.activate(), Err(AccountError::RevokedIsTerminal));
    }

    #[test]
    fn invalid_revoked_to_degraded() {
        let mut acc = new_account(ProviderFamily::Gemini);
        acc.revoke().unwrap();
        assert_eq!(
            acc.degrade("reason".to_owned()),
            Err(AccountError::RevokedIsTerminal)
        );
    }

    #[test]
    fn invalid_revoked_to_disabled() {
        let mut acc = new_account(ProviderFamily::Aws);
        acc.revoke().unwrap();
        assert_eq!(
            acc.disable("reason".to_owned()),
            Err(AccountError::RevokedIsTerminal)
        );
    }

    #[test]
    fn invalid_revoke_again() {
        let mut acc = new_account(ProviderFamily::Claude);
        acc.revoke().unwrap();
        assert_eq!(acc.revoke(), Err(AccountError::RevokedIsTerminal));
    }

    #[test]
    fn invalid_disabled_cannot_activate() {
        let mut acc = new_account(ProviderFamily::Claude);
        acc.verify(sref("sref://s1")).unwrap();
        acc.activate().unwrap();
        acc.disable("reason".to_owned()).unwrap();
        assert!(acc.activate().is_err());
    }

    #[test]
    fn invalid_active_cannot_verify_again() {
        let mut acc = new_account(ProviderFamily::Claude);
        acc.verify(sref("sref://s1")).unwrap();
        acc.activate().unwrap();
        assert!(acc.verify(sref("sref://s2")).is_err());
    }

    #[test]
    fn silent_switch_prevented_same_provider_subscription() {
        let existing = ProviderAccount {
            id: aid("acct-1"),
            provider_family: ProviderFamily::Claude,
            state: AccountState::Active,
            subscription_id: Some("sub-1".to_owned()),
        };
        let candidate = ProviderAccount {
            id: aid("acct-2"),
            provider_family: ProviderFamily::Claude,
            state: AccountState::Active,
            subscription_id: Some("sub-1".to_owned()),
        };
        assert_eq!(
            check_silent_switch(&[&existing], &candidate),
            Err(AccountError::SilentSwitchPrevented)
        );
    }

    #[test]
    fn silent_switch_allowed_different_subscription() {
        let existing = ProviderAccount {
            id: aid("acct-1"),
            provider_family: ProviderFamily::Claude,
            state: AccountState::Active,
            subscription_id: Some("sub-1".to_owned()),
        };
        let candidate = ProviderAccount {
            id: aid("acct-2"),
            provider_family: ProviderFamily::Claude,
            state: AccountState::Active,
            subscription_id: Some("sub-2".to_owned()),
        };
        assert!(check_silent_switch(&[&existing], &candidate).is_ok());
    }

    #[test]
    fn auth_session_valid() {
        let s = AuthSession::new(
            sid("sess-1"),
            aid("acct-1"),
            ProviderFamily::Claude,
            1000,
            2000,
            CapabilityGrant("read".to_owned()),
            PrivacyBoundary("tenant-1".to_owned()),
        );
        assert!(s.is_ok());
    }

    #[test]
    fn auth_session_rejects_expired_at_lte_started() {
        let s = AuthSession::new(
            sid("sess-1"),
            aid("acct-1"),
            ProviderFamily::Gemini,
            2000,
            1000,
            CapabilityGrant("read".to_owned()),
            PrivacyBoundary("tenant-1".to_owned()),
        );
        assert!(s.is_err());
    }

    #[test]
    fn auth_session_rejects_empty_privacy_boundary() {
        let s = AuthSession::new(
            sid("sess-1"),
            aid("acct-1"),
            ProviderFamily::Claude,
            1000,
            2000,
            CapabilityGrant("read".to_owned()),
            PrivacyBoundary(String::new()),
        );
        assert!(s.is_err());
    }

    #[test]
    fn usage_window_five_hour_valid() {
        assert!(UsageWindow::new(UsageWindowKind::FiveHour, 0, 18000, 80, 20).is_ok());
    }
    #[test]
    fn usage_window_one_week_valid() {
        assert!(UsageWindow::new(UsageWindowKind::OneWeek, 0, 604800, 90, 10).is_ok());
    }
    #[test]
    fn usage_window_project_valid() {
        assert!(UsageWindow::new(UsageWindowKind::Project, 0, 86400, 100, 0).is_ok());
    }
    #[test]
    fn usage_window_rejects_ends_before_starts() {
        assert!(UsageWindow::new(UsageWindowKind::FiveHour, 100, 50, 50, 50).is_err());
    }
    #[test]
    fn usage_window_rejects_limit_pct_over_100() {
        assert!(UsageWindow::new(UsageWindowKind::FiveHour, 0, 100, 101, 0).is_err());
    }
    #[test]
    fn usage_window_rejects_reserve_pct_over_100() {
        assert!(UsageWindow::new(UsageWindowKind::FiveHour, 0, 100, 50, 101).is_err());
    }

    #[test]
    fn account_health_creation() {
        let h = AccountHealth {
            account_id: aid("acct-1"),
            is_healthy: true,
            reason: None,
            last_check_at_epoch_secs: 1000,
        };
        assert!(h.is_healthy);
    }

    #[test]
    fn quota_creation() {
        let q = Quota {
            limit_tokens: 1_000_000,
            current_tokens: 250_000,
            reserve_remaining_pct: 75,
        };
        assert_eq!(q.reserve_remaining_pct, 75);
    }

    #[test]
    fn route_explanation_creation() {
        let r = RouteExplanation {
            chosen_provider: ProviderFamily::Claude,
            chosen_account_id: aid("acct-1"),
            chosen_model: "claude-sonnet-4-6".to_owned(),
            reason: "cost".to_owned(),
        };
        assert_eq!(r.chosen_provider, ProviderFamily::Claude);
    }

    #[test]
    fn token_cost_creation() {
        let tc = TokenCost {
            estimated_micros: 42,
            actual_micros: Some(38),
            model: "claude-sonnet-4-6".to_owned(),
            tokens_in: 1000,
            tokens_out: 500,
            cache_hit_pct: 40,
        };
        assert_eq!(tc.tokens_in, 1000);
    }

    #[test]
    fn reason_captured_in_degraded() {
        let mut acc = new_account(ProviderFamily::Gemini);
        acc.verify(sref("sref://s1")).unwrap();
        acc.activate().unwrap();
        acc.degrade("disk full".to_owned()).unwrap();
        assert_eq!(
            acc.state,
            AccountState::Degraded {
                reason: "disk full".to_owned()
            }
        );
    }

    #[test]
    fn reason_captured_in_disabled() {
        let mut acc = new_account(ProviderFamily::Aws);
        acc.disable("compliance hold".to_owned()).unwrap();
        assert_eq!(
            acc.state,
            AccountState::Disabled {
                reason: "compliance hold".to_owned()
            }
        );
    }
}
