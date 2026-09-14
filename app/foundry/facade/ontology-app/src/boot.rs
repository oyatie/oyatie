//! Why a process refused to start. Every variant is a refusal, never a
//! degraded serve.

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BootError {
    ActionLogUnopenable {
        detail: String,
    },
    DenialLogUnopenable {
        detail: String,
    },
    /// Both logs name one path. A shared store would let a refusal land in
    /// the log it was refused from.
    LogPathsAliased,
    NoTenantsConfigured,
    SeedRefused {
        tenant_id: String,
        detail: String,
    },
    ReplayFailed {
        tenant_id: String,
        detail: String,
    },
    ProjectionStoreUnopenable {
        detail: String,
    },
    /// The projection store names one of the log paths. A store that is a
    /// log would be caught up from itself.
    StorePathAliased,
    /// The durable projection could not be brought to the log's head; the
    /// detail carries the spine's own reason (built under another log or
    /// registry, unreadable, ahead of the log, or a mirror it refused).
    CatchUpRefused {
        tenant_id: String,
        detail: String,
    },
    PolicyRejected {
        detail: String,
    },
}

impl std::fmt::Display for BootError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ActionLogUnopenable { detail } => {
                write!(formatter, "the action log could not be opened: {detail}")
            }
            Self::DenialLogUnopenable { detail } => {
                write!(formatter, "the denial trail could not be opened: {detail}")
            }
            Self::LogPathsAliased => write!(
                formatter,
                "the action log and the denial trail must be distinct stores"
            ),
            Self::NoTenantsConfigured => {
                write!(
                    formatter,
                    "no tenants configured; the roster is the served set"
                )
            }
            Self::SeedRefused { tenant_id, detail } => {
                write!(
                    formatter,
                    "tenant {tenant_id} could not be seeded: {detail}"
                )
            }
            Self::ReplayFailed { tenant_id, detail } => {
                write!(
                    formatter,
                    "tenant {tenant_id} could not be replayed: {detail}"
                )
            }
            Self::ProjectionStoreUnopenable { detail } => {
                write!(
                    formatter,
                    "the projection store could not be opened: {detail}"
                )
            }
            Self::StorePathAliased => write!(
                formatter,
                "the projection store must be a store distinct from both logs"
            ),
            Self::CatchUpRefused { tenant_id, detail } => {
                write!(
                    formatter,
                    "tenant {tenant_id}'s projection store could not be caught up: {detail}"
                )
            }
            Self::PolicyRejected { detail } => {
                write!(formatter, "the policy seed was rejected: {detail}")
            }
        }
    }
}
