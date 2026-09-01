//! Who is calling, established from the credential alone.
//!
//! The tenant, the principal and the roles all come from the presented
//! token — never from a header, a path segment or the request body. That
//! is the difference between a surface that authorizes and one that asks
//! callers who they would like to be: a caller holding tenant A's token
//! cannot address tenant B by saying so, because nothing in the request
//! can move the tenant.
//!
//! Token comparison is constant-time over the whole roster. A short-circuit
//! compare leaks a prefix oracle, and checking only until the first
//! mismatch would let a caller learn a token one byte at a time.

use crate::authz::Caller;

/// One operator the process will recognize. Configuration, not policy: the
/// seed decides what an operator may DO; this only decides who they are.
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

/// Extract the bearer token from an `Authorization` header value.
pub fn bearer_token(header: Option<&str>) -> Option<&str> {
    header?
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|token| !token.is_empty())
}

/// Resolve a presented token to its caller, in constant time with respect
/// to the roster: every credential is compared, and the comparison itself
/// does not stop at the first differing byte.
pub fn authenticate(operators: &[OperatorCredential], presented: &str) -> Option<Caller> {
    let mut found: Option<&OperatorCredential> = None;
    for operator in operators {
        if constant_time_eq(operator.token.as_bytes(), presented.as_bytes()) {
            found = Some(operator);
        }
    }
    found.map(OperatorCredential::caller)
}

/// Byte equality that always inspects the full length of the longer input.
/// Length inequality is folded into the accumulator rather than returning
/// early, so neither the length nor any prefix is observable through
/// timing.
fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    let mut difference = (left.len() ^ right.len()) as u8;
    let width = left.len().max(right.len());
    for index in 0..width {
        let l = left.get(index).copied().unwrap_or(0);
        let r = right.get(index).copied().unwrap_or(0);
        difference |= l ^ r;
    }
    difference == 0
}
