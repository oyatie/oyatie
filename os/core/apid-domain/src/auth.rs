//! Per-method RBAC authorization, mirroring apid's gRPC auth interceptor.
//!
//! Talos `apid` terminates mTLS and extracts the client certificate's
//! organizational units into a [`RoleSet`]. Every gRPC call is then checked
//! against a policy: read-only methods require read access, mutating methods
//! require write access, and a handful of methods are restricted to the internal
//! `os` role. This module models that interceptor as a small, overridable
//! policy table.

use crate::error::ApiError;
use crate::request::Method;
use std::collections::BTreeMap;
use os_kernel::role::{Role, RoleSet};

/// The access level a method requires from the caller's role set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    /// Any authenticated identity with read access may call.
    Read,
    /// Requires write (admin/os) access.
    Write,
    /// Requires the internal `os` role specifically (machined-only calls).
    OsOnly,
}

impl Access {
    /// Whether `roles` satisfies this access requirement.
    pub fn satisfied_by(self, roles: &RoleSet) -> bool {
        match self {
            Access::Read => roles.can_read(),
            Access::Write => roles.can_write(),
            Access::OsOnly => roles.contains(Role::Os),
        }
    }
}

/// The RBAC policy that gates every apid call.
///
/// The default policy derives the requirement from the method's mutating flag
/// (read for non-mutating, write for mutating). Specific methods can be pinned
/// to a stricter level via the override table — e.g. `Reset` and `Upgrade` are
/// modeled as `OsOnly` to reflect their destructive nature.
#[derive(Debug, Clone)]
pub struct Authorizer {
    overrides: BTreeMap<&'static str, Access>,
    /// When true, an empty role set is denied outright (mirrors mTLS being
    /// required). When false, anonymous read is allowed (used only in tests).
    require_identity: bool,
}

impl Default for Authorizer {
    fn default() -> Self {
        let mut overrides = BTreeMap::new();
        // Destructive lifecycle operations are restricted to the internal role.
        overrides.insert("Reset", Access::OsOnly);
        overrides.insert("Upgrade", Access::OsOnly);
        Authorizer {
            overrides,
            require_identity: true,
        }
    }
}

impl Authorizer {
    /// An authorizer with no overrides and identity required.
    pub fn new() -> Self {
        Authorizer {
            overrides: BTreeMap::new(),
            require_identity: true,
        }
    }

    /// Pin a method (by short name) to a specific access level.
    pub fn set_override(&mut self, method: &'static str, access: Access) {
        self.overrides.insert(method, access);
    }

    /// Whether an authenticated identity is required for every call.
    pub fn set_require_identity(&mut self, required: bool) {
        self.require_identity = required;
    }

    /// The access level required by `method` under this policy.
    pub fn required_access(&self, method: Method) -> Access {
        if let Some(&access) = self.overrides.get(method.short_name()) {
            return access;
        }
        if method.is_mutating() {
            Access::Write
        } else {
            Access::Read
        }
    }

    /// Authorize a call to `method` by an identity holding `roles`.
    ///
    /// Returns [`ApiError::PermissionDenied`] if the role set is empty (and
    /// identity is required) or does not satisfy the method's access level.
    pub fn authorize(&self, method: Method, roles: &RoleSet) -> Result<(), ApiError> {
        if self.require_identity && roles.is_empty() {
            return Err(ApiError::permission_denied(format!(
                "{} requires an authenticated identity",
                method.short_name()
            )));
        }
        let access = self.required_access(method);
        if access.satisfied_by(roles) {
            Ok(())
        } else {
            Err(ApiError::permission_denied(format!(
                "{} requires {:?} access, caller roles: [{}]",
                method.short_name(),
                access,
                roles.to_string_list()
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine_service::MachineMethod;

    fn roles(rs: &[Role]) -> RoleSet {
        RoleSet::from_roles(rs.iter().copied())
    }

    #[test]
    fn reader_can_read_not_write() {
        let auth = Authorizer::default();
        let reader = roles(&[Role::Reader]);
        assert!(
            auth.authorize(Method::Machine(MachineMethod::Version), &reader)
                .is_ok()
        );
        let err = auth
            .authorize(Method::Machine(MachineMethod::Reboot), &reader)
            .unwrap_err();
        assert_eq!(err.grpc_code(), "PermissionDenied");
    }

    #[test]
    fn admin_can_write_but_not_os_only() {
        let auth = Authorizer::default();
        let admin = roles(&[Role::Admin]);
        assert!(
            auth.authorize(Method::Machine(MachineMethod::Reboot), &admin)
                .is_ok()
        );
        // Reset is OsOnly by default.
        assert!(
            auth.authorize(Method::Machine(MachineMethod::Reset), &admin)
                .is_err()
        );
    }

    #[test]
    fn os_role_can_do_everything() {
        let auth = Authorizer::default();
        let os = roles(&[Role::Os]);
        assert!(
            auth.authorize(Method::Machine(MachineMethod::Reset), &os)
                .is_ok()
        );
        assert!(
            auth.authorize(Method::Machine(MachineMethod::Upgrade), &os)
                .is_ok()
        );
    }

    #[test]
    fn empty_identity_denied() {
        let auth = Authorizer::default();
        let none = RoleSet::new();
        assert!(
            auth.authorize(Method::Machine(MachineMethod::Version), &none)
                .is_err()
        );
    }

    #[test]
    fn override_changes_requirement() {
        let mut auth = Authorizer::new();
        auth.set_override("Version", Access::Write);
        let reader = roles(&[Role::Reader]);
        assert!(
            auth.authorize(Method::Machine(MachineMethod::Version), &reader)
                .is_err()
        );
        assert_eq!(
            auth.required_access(Method::Machine(MachineMethod::Version)),
            Access::Write
        );
    }
}
