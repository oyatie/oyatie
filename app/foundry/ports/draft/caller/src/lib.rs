//! Foundry caller port: the seam through which a presented credential
//! becomes a verified [`Caller`], or nothing.
//!
//! The port owns the caller shape and depends on nothing: which tenant,
//! principal and roles a credential binds is Foundry's contract, and every
//! adapter — the operator roster here, a platform identity facade later —
//! must satisfy the executable laws in [`conformance`] unchanged.
#![forbid(unsafe_code)]

pub mod conformance;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Caller {
    pub tenant_id: String,    // data_class: TENANT_SCOPED
    pub principal_id: String, // data_class: TENANT_SCOPED
    pub roles: Vec<String>,   // data_class: TENANT_SCOPED
}

/// Verify a presented bearer credential; `None` means no verified caller.
/// Verification is total: any input, including an absent credential,
/// yields an answer rather than a panic.
pub trait CallerVerifier: Send + Sync {
    fn verify(&self, presented_bearer: Option<&str>) -> Option<Caller>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperatorCredential {
    pub token: String,        // data_class: SECRET
    pub tenant_id: String,    // data_class: TENANT_SCOPED
    pub principal_id: String, // data_class: TENANT_SCOPED
    /// Roles this credential carries. An empty list is meaningful: the
    /// caller is recognized but reaches no permit, so policy denies it.
    pub roles: Vec<String>, // data_class: TENANT_SCOPED
}

impl OperatorCredential {
    fn caller(&self) -> Caller {
        Caller {
            tenant_id: self.tenant_id.clone(),
            principal_id: self.principal_id.clone(),
            roles: self.roles.clone(),
        }
    }
}

/// The reference adapter: a configured roster of operator credentials. An
/// empty roster verifies nobody; there is no allow-all path.
pub struct RosterVerifier {
    operators: Vec<OperatorCredential>,
}

impl RosterVerifier {
    pub fn new(operators: Vec<OperatorCredential>) -> Self {
        Self { operators }
    }
}

impl CallerVerifier for RosterVerifier {
    /// Constant time with respect to the ROSTER: the loop deliberately has no
    /// `break` and assigns `found` instead of returning, so the work done does
    /// not reveal a matching credential's position. An early return would be a
    /// timing oracle, not an optimization.
    fn verify(&self, presented_bearer: Option<&str>) -> Option<Caller> {
        let presented = presented_bearer?;
        let mut found: Option<&OperatorCredential> = None;
        for operator in &self.operators {
            if constant_time_eq(operator.token.as_bytes(), presented.as_bytes()) {
                found = Some(operator);
            }
        }
        found.map(OperatorCredential::caller)
    }
}

/// Byte equality that always inspects the full length of the longer input.
/// Length inequality is folded into the accumulator rather than returning
/// early, so neither the length nor any prefix is observable through
/// timing.
fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    let mut difference = u8::from(left.len() != right.len());
    let width = left.len().max(right.len());
    for index in 0..width {
        let l = left.get(index).copied().unwrap_or(0);
        let r = right.get(index).copied().unwrap_or(0);
        difference |= l ^ r;
    }
    difference == 0
}
