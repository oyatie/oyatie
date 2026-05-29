//! Port traits for credential storage and operator alerting.
//! Implementations live in adapters (OpenBao in production, in-memory for tests).
// data_class: INTERNAL_ONLY throughout this module.

use std::fmt;

/// Identifier for a subscription seat (opaque string, typically the SecretReference URI).
// data_class: INTERNAL_ONLY
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SeatId(pub String);

impl fmt::Display for SeatId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Opaque encrypted token bytes managed by the credential store.
// data_class: INTERNAL_ONLY
#[derive(Clone)]
pub struct TokenBytes(pub Vec<u8>);

impl fmt::Debug for TokenBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TokenBytes([REDACTED] len={})", self.0.len())
    }
}

/// Alert kind emitted when a seat enters a terminal error state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlertKind {
    /// refresh_token_expired — operator must re-enroll the seat.
    RefreshTokenExpired,
    /// refresh_token_reused — security event; token was already consumed.
    RefreshTokenReused,
    /// refresh_token_invalidated — token revoked server-side.
    RefreshTokenInvalidated,
}

impl fmt::Display for AlertKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RefreshTokenExpired => write!(f, "refresh_token_expired"),
            Self::RefreshTokenReused => write!(f, "refresh_token_reused"),
            Self::RefreshTokenInvalidated => write!(f, "refresh_token_invalidated"),
        }
    }
}

/// Port for persisting OAuth token bytes to an external credential store (OpenBao in production).
/// INVARIANT: `store` must succeed before in-memory state is mutated (persist-before-mutate).
pub trait CredentialStorePort: Send + Sync {
    fn store(&self, seat_id: &SeatId, bytes: TokenBytes) -> Result<(), String>;
    fn load(&self, seat_id: &SeatId) -> Option<TokenBytes>;
    fn delete(&self, seat_id: &SeatId);
}

/// Port for emitting operator alerts on terminal seat errors.
pub trait OperatorAlertPort: Send + Sync {
    fn alert(&self, seat_id: &SeatId, kind: AlertKind);
}
