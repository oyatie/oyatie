//! Peer identity extraction from a terminated mTLS connection.
//!
//! Real `apid` terminates mTLS and, for every accepted connection, reads the
//! client certificate's subject: the Common Name is the caller's name and the
//! Organizational Units carry its RBAC roles as `os:<role>` strings. Talos then
//! maps those OUs into a [`RoleSet`] (dropping unrecognized OUs) which the
//! authorizer checks per method.
//!
//! Because this crate does not do real X.509, the certificate is modeled as a
//! [`PeerCertificate`] value object — the same fields apid actually consumes —
//! and [`PeerIdentity::from_certificate`] performs the CN/OU → identity mapping.
//! An *unauthenticated* connection (plaintext or no client cert) yields
//! [`PeerIdentity::anonymous`], which holds no roles and is denied by the
//! authorizer when identity is required.

use os_kernel::role::RoleSet;

/// The subset of a verified client X.509 certificate apid consumes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerCertificate {
    /// The certificate Common Name (the caller's identity name).
    pub common_name: String,
    /// The Organizational Units, each an `os:<role>` (or arbitrary) string.
    pub organizations: Vec<String>,
}

impl PeerCertificate {
    /// Build a certificate from a CN and its OU strings.
    pub fn new(
        common_name: impl Into<String>,
        organizations: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        PeerCertificate {
            common_name: common_name.into(),
            organizations: organizations.into_iter().map(Into::into).collect(),
        }
    }
}

/// The authenticated identity behind a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerIdentity {
    /// The caller name (certificate CN), or empty when anonymous.
    name: String,
    /// The roles extracted from the certificate OUs.
    roles: RoleSet,
    /// Whether the connection presented a verified client certificate.
    authenticated: bool,
}

impl PeerIdentity {
    /// The identity of a connection with no verified client certificate.
    pub fn anonymous() -> Self {
        PeerIdentity {
            name: String::new(),
            roles: RoleSet::new(),
            authenticated: false,
        }
    }

    /// Derive an identity from a verified client certificate, mapping the OUs
    /// into a [`RoleSet`]. Unknown OUs are dropped, matching Talos behavior.
    pub fn from_certificate(cert: &PeerCertificate) -> Self {
        let roles = RoleSet::parse_ous(cert.organizations.iter().map(String::as_str));
        PeerIdentity {
            name: cert.common_name.clone(),
            roles,
            authenticated: true,
        }
    }

    /// The caller name (certificate CN).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The roles this identity carries.
    pub fn roles(&self) -> &RoleSet {
        &self.roles
    }

    /// Whether the connection was mutually authenticated (had a client cert).
    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }

    /// Apply role impersonation: a caller that may impersonate (per
    /// [`RoleSet::can_impersonate`]) can present an `impersonate-roles` set,
    /// which replaces the effective roles. Returns the impersonated identity, or
    /// the original if the caller is not allowed to impersonate.
    ///
    /// Mirrors apid honoring the impersonation OUs from a trusted intermediary
    /// (e.g. the dashboard) while refusing it from an ordinary client.
    pub fn impersonate(&self, requested: &RoleSet) -> Result<PeerIdentity, crate::error::ApiError> {
        if !self.roles.can_impersonate() {
            return Err(crate::error::ApiError::permission_denied(format!(
                "identity '{}' may not impersonate roles",
                self.name
            )));
        }
        Ok(PeerIdentity {
            name: self.name.clone(),
            roles: requested.clone(),
            authenticated: self.authenticated,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use os_kernel::role::Role;

    #[test]
    fn anonymous_has_no_roles() {
        let id = PeerIdentity::anonymous();
        assert!(!id.is_authenticated());
        assert!(id.roles().is_empty());
        assert_eq!(id.name(), "");
    }

    #[test]
    fn certificate_maps_ous_to_roles() {
        let cert = PeerCertificate::new("admin@cluster", ["os:admin", "os:reader"]);
        let id = PeerIdentity::from_certificate(&cert);
        assert!(id.is_authenticated());
        assert_eq!(id.name(), "admin@cluster");
        assert!(id.roles().can_write());
        assert!(id.roles().contains(Role::Admin));
        assert!(id.roles().contains(Role::Reader));
    }

    #[test]
    fn unknown_ous_are_dropped() {
        let cert = PeerCertificate::new("svc", ["os:reader", "department:eng"]);
        let id = PeerIdentity::from_certificate(&cert);
        assert_eq!(id.roles().len(), 1);
        assert!(id.roles().contains(Role::Reader));
    }

    #[test]
    fn reader_cannot_impersonate() {
        let cert = PeerCertificate::new("ro", ["os:reader"]);
        let id = PeerIdentity::from_certificate(&cert);
        let wanted = RoleSet::from_roles([Role::Admin]);
        assert_eq!(
            id.impersonate(&wanted).unwrap_err().grpc_code(),
            "PermissionDenied"
        );
    }

    #[test]
    fn impersonator_can_downgrade_to_requested_roles() {
        let cert = PeerCertificate::new("dash", ["os:impersonator"]);
        let id = PeerIdentity::from_certificate(&cert);
        let wanted = RoleSet::from_roles([Role::Reader]);
        let acting = id.impersonate(&wanted).unwrap();
        assert!(acting.roles().can_read());
        assert!(!acting.roles().can_write());
        assert_eq!(acting.name(), "dash");
    }

    #[test]
    fn admin_may_impersonate() {
        let cert = PeerCertificate::new("a", ["os:admin"]);
        let id = PeerIdentity::from_certificate(&cert);
        let acting = id
            .impersonate(&RoleSet::from_roles([Role::Reader]))
            .unwrap();
        assert!(!acting.roles().can_write());
    }
}
