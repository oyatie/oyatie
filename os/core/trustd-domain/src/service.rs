//! The `SecurityService` gRPC API surface.
//!
//! Talos `trustd` exposes a `SecurityService` with a single RPC, `Certificate`,
//! that takes a CSR + the cluster join token and returns a signed certificate
//! plus the CA chain. This module models that request/response handler, the
//! authentication gate (join token), and node certificate rotation/renewal.

use crate::ca::{CertificateAuthority, CertificateSigningRequest};
use crate::certificate::{Certificate, IssuedIdentity};
use crate::crl::{RevocationList, RevocationReason};
use crate::error::{Result, TrustError};
use crate::signer::SigningBackend;
use crate::token::JoinToken;
use crate::x509::KeyPair;
use os_kernel::Role;
use os_kernel::role::RoleSet;

/// The `Certificate` RPC request: a CSR plus the caller's presented join token.
#[derive(Debug, Clone)]
pub struct CertificateRequest {
    /// The presented cluster join token (raw string from gRPC metadata).
    pub join_token: String,
    /// The certificate signing request.
    pub csr: CertificateSigningRequest,
}

/// The `Certificate` RPC response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificateResponse {
    /// The issued identity (cert + key + CA chain).
    pub identity: IssuedIdentity,
}

/// trustd's `SecurityService`: an authenticated front-end over a
/// [`CertificateAuthority`].
pub struct SecurityService<S: SigningBackend> {
    token: JoinToken,
    ca: CertificateAuthority<S>,
    crl: RevocationList,
    /// Number of certificates successfully issued (audit metric).
    issued: u64,
}

impl<S: SigningBackend> SecurityService<S> {
    /// Construct the service from a configured join token and CA.
    pub fn new(token: JoinToken, ca: CertificateAuthority<S>) -> Self {
        SecurityService {
            token,
            ca,
            crl: RevocationList::new(),
            issued: 0,
        }
    }

    /// Total certificates issued by this service so far.
    pub fn issued_count(&self) -> u64 {
        self.issued
    }

    /// Immutable view of the service's revocation list.
    pub fn revocations(&self) -> &RevocationList {
        &self.crl
    }

    /// Revoke an issued certificate by serial so it is refused by [`authorize`].
    /// Returns the new CRL number.
    ///
    /// [`authorize`]: SecurityService::authorize
    pub fn revoke(&mut self, serial: u64, reason: RevocationReason, now: u64) -> u64 {
        self.crl.revoke(serial, reason, now)
    }

    /// Handle a `Certificate` RPC: authenticate via the join token, run the CA
    /// issuance flow, and return the signed identity. `requester_key` stands in
    /// for the private key the node holds locally (the CA only sees the public
    /// half via the CSR).
    pub fn handle_certificate(
        &mut self,
        req: &CertificateRequest,
        requester_key: &KeyPair,
        now: u64,
    ) -> Result<CertificateResponse> {
        // Authentication gate: only callers holding the cluster token may
        // request a certificate.
        self.token.verify_presented(&req.join_token)?;
        let identity = self.ca.issue_identity(&req.csr, requester_key, now)?;
        self.issued = self.issued.saturating_add(1);
        Ok(CertificateResponse { identity })
    }

    /// Verify a presented client certificate and return the RBAC roles it
    /// carries. Mirrors apid's authorization check: the cert must chain to this
    /// CA and be currently valid, after which its OUs determine permissions.
    pub fn authorize(&self, cert: &Certificate, now: u64) -> Result<RoleSet> {
        self.crl.ensure_valid(cert)?;
        self.ca.verify(cert, now)?;
        Ok(cert.roles())
    }

    /// Enforce that a presented certificate grants at least the capability
    /// implied by `required`. Authorization is by *implication*, matching Talos
    /// RBAC: an `admin`/`os` identity satisfies a `reader` requirement, a
    /// `reader` identity does not satisfy an `admin` requirement. Returns the
    /// cert's role set on success.
    pub fn require_role(&self, cert: &Certificate, required: Role, now: u64) -> Result<RoleSet> {
        let roles = self.authorize(cert, now)?;
        let granted = match required {
            // read-only capability is satisfied by any read-capable role
            Role::Reader => roles.can_read(),
            // write capability (admin/os) requirement
            Role::Admin | Role::Os => roles.can_write(),
            // impersonation is an exact, privileged capability
            Role::Impersonator => {
                roles.contains(Role::Impersonator)
                    || roles.contains(Role::Admin)
                    || roles.contains(Role::Os)
            }
            // etcd snapshot/backup capability (os:etcd-backup or admin/os).
            Role::EtcdBackup => roles.can_etcd_backup(),
            // operator management APIs: satisfied by operator (read-capable) or
            // any write-capable identity (admin/os).
            Role::Operator => roles.contains(Role::Operator) || roles.can_write(),
            // image verification: an exact capability, also implied by admin/os.
            Role::ImageVerifier => roles.contains(Role::ImageVerifier) || roles.can_write(),
            // META mutation: an exact capability, also implied by admin/os.
            Role::MetaWriter => roles.contains(Role::MetaWriter) || roles.can_write(),
        };
        if !granted {
            return Err(TrustError::permission_denied(format!(
                "certificate lacks required role '{}'",
                required.as_str()
            )));
        }
        Ok(roles)
    }

    /// Renew a node certificate that is approaching expiry. Talos rotates a node
    /// cert when less than half its lifetime remains. Returns `Ok(None)` when
    /// renewal is not yet needed, or the freshly issued identity otherwise.
    pub fn renew_if_needed(
        &mut self,
        current: &Certificate,
        requester_key: &KeyPair,
        now: u64,
    ) -> Result<Option<IssuedIdentity>> {
        // Revoked certificates must never mint a fresh serial.
        self.crl.ensure_valid(current)?;
        // The cert must currently chain to this CA before we'll rotate it.
        self.ca.verify(current, now).or_else(|e| {
            // An already-expired cert still gets rotated, but other failures
            // (wrong issuer, bad signature) are fatal.
            if matches!(e, TrustError::Expired(_)) {
                Ok(())
            } else {
                Err(e)
            }
        })?;
        // Renewal is bound to the key that already holds `current`.
        if !requester_key.matches_public(&current.public_key_der) {
            return Err(TrustError::csr_rejected(
                "renewal requester key does not match the presented certificate",
            ));
        }

        let needs = current.validity.is_expired(now) || current.validity.needs_renewal(now, 1, 2);
        if !needs {
            return Ok(None);
        }

        let ttl = current
            .validity
            .not_after
            .saturating_sub(current.validity.not_before);
        let csr = CertificateSigningRequest {
            subject: current.subject.clone(),
            usage: current.usage,
            sans: current.sans.clone(),
            public_key_der: current.public_key_der.clone(),
            ttl_secs: ttl.max(1),
        };
        let identity = self.ca.issue_identity(&csr, requester_key, now)?;
        Ok(Some(identity))
    }

    /// The CA certificate chain this service issues against.
    pub fn ca_certificate(&self) -> &Certificate {
        self.ca.certificate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ca::CertificateAuthority;
    use crate::certificate::CertUsage;
    use crate::signer::InMemorySigner;

    fn service() -> (SecurityService<InMemorySigner>, JoinToken) {
        let token = JoinToken::new("clusterid.clustersecret").unwrap();
        let ca = CertificateAuthority::bootstrap(
            "talos-ca",
            KeyPair::from_seed(b"ca-seed"),
            InMemorySigner::from_seed("ca-seed"),
            1000,
            1_000_000,
        )
        .unwrap();
        (SecurityService::new(token.clone(), ca), token)
    }

    fn node_request(token: &str) -> (CertificateRequest, KeyPair) {
        let key = KeyPair::from_seed(b"node-1");
        let csr = CertificateSigningRequest::for_node("node-1", &key, CertUsage::ClientAuth, 3600)
            .requesting_role("os:admin");
        (
            CertificateRequest {
                join_token: token.to_string(),
                csr,
            },
            key,
        )
    }

    #[test]
    fn issues_cert_with_valid_token() {
        let (mut svc, _t) = service();
        let (req, key) = node_request("clusterid.clustersecret");
        let resp = svc.handle_certificate(&req, &key, 2000).unwrap();
        assert_eq!(resp.identity.name(), "node-1");
    }

    #[test]
    fn rejects_bad_token() {
        let (mut svc, _t) = service();
        let (req, key) = node_request("clusterid.WRONG");
        assert_eq!(
            svc.handle_certificate(&req, &key, 2000).unwrap_err().kind(),
            "token_mismatch"
        );
    }

    #[test]
    fn authorize_returns_roles_and_enforces() {
        let (mut svc, _t) = service();
        let (req, key) = node_request("clusterid.clustersecret");
        let resp = svc.handle_certificate(&req, &key, 2000).unwrap();
        let cert = &resp.identity.certificate;
        assert!(svc.require_role(cert, Role::Admin, 2500).is_ok());
        // a reader-only requirement is also satisfied by admin
        assert!(svc.require_role(cert, Role::Reader, 2500).is_ok());
    }

    #[test]
    fn require_role_denies_missing_role() {
        let (mut svc, _t) = service();
        // issue a reader-only cert
        let key = KeyPair::from_seed(b"reader");
        let csr =
            CertificateSigningRequest::for_node("reader-node", &key, CertUsage::ClientAuth, 3600)
                .requesting_role("os:reader");
        let req = CertificateRequest {
            join_token: "clusterid.clustersecret".to_string(),
            csr,
        };
        let resp = svc.handle_certificate(&req, &key, 2000).unwrap();
        let cert = &resp.identity.certificate;
        assert_eq!(
            svc.require_role(cert, Role::Admin, 2500)
                .unwrap_err()
                .kind(),
            "permission_denied"
        );
    }

    #[test]
    fn renewal_only_when_needed() {
        let (mut svc, _t) = service();
        let (req, key) = node_request("clusterid.clustersecret");
        // issue with TTL 3600 at now=2000 -> valid [2000,5600)
        let resp = svc.handle_certificate(&req, &key, 2000).unwrap();
        let cert = resp.identity.certificate.clone();
        assert!(resp.identity.key_pem.der().is_empty());
        // early: more than half life remains -> no renewal
        assert!(svc.renew_if_needed(&cert, &key, 2100).unwrap().is_none());
        // late: less than half remains -> renews
        let renewed = svc.renew_if_needed(&cert, &key, 4000).unwrap();
        assert!(renewed.is_some());
        assert!(renewed.unwrap().certificate.serial > cert.serial);
        let foreign = KeyPair::from_seed(b"other-node");
        assert!(svc.renew_if_needed(&cert, &foreign, 4000).is_err());
        svc.revoke(cert.serial, RevocationReason::KeyCompromise, 4100);
        assert!(svc.renew_if_needed(&cert, &key, 4200).is_err());
    }

    #[test]
    fn renewal_rejects_foreign_cert() {
        let (mut svc, _t) = service();
        // a cert from a different CA must not be rotated by this service
        let mut rogue = CertificateAuthority::bootstrap(
            "rogue",
            KeyPair::from_seed(b"rogue"),
            InMemorySigner::from_seed("rogue"),
            1000,
            1_000_000,
        )
        .unwrap();
        let key = KeyPair::from_seed(b"x");
        let csr = CertificateSigningRequest::for_node("x", &key, CertUsage::ClientAuth, 3600);
        let foreign = rogue.sign_csr(&csr, 2000).unwrap();
        assert!(svc.renew_if_needed(&foreign, &key, 5000).is_err());
    }

    #[test]
    fn revoked_cert_is_refused_by_authorize() {
        let (mut svc, _t) = service();
        let (req, key) = node_request("clusterid.clustersecret");
        let resp = svc.handle_certificate(&req, &key, 2000).unwrap();
        let cert = resp.identity.certificate.clone();
        assert!(svc.authorize(&cert, 2500).is_ok());
        svc.revoke(cert.serial, RevocationReason::KeyCompromise, 2400);
        let err = svc.authorize(&cert, 2500).unwrap_err();
        assert_eq!(err.kind(), "verification_failed");
        // require_role likewise denies a revoked cert
        assert!(svc.require_role(&cert, Role::Reader, 2500).is_err());
    }

    #[test]
    fn issued_count_tracks_issuance() {
        let (mut svc, _t) = service();
        assert_eq!(svc.issued_count(), 0);
        let (req, key) = node_request("clusterid.clustersecret");
        svc.handle_certificate(&req, &key, 2000).unwrap();
        svc.handle_certificate(&req, &key, 2000).unwrap();
        assert_eq!(svc.issued_count(), 2);
        // a rejected request must not increment the counter
        let (bad, key2) = node_request("clusterid.WRONG");
        assert!(svc.handle_certificate(&bad, &key2, 2000).is_err());
        assert_eq!(svc.issued_count(), 2);
    }

    #[test]
    fn etcd_backup_role_enforced() {
        let (mut svc, _t) = service();
        let key = KeyPair::from_seed(b"backup-node");
        let csr =
            CertificateSigningRequest::for_node("backup-node", &key, CertUsage::ClientAuth, 3600)
                .requesting_role("os:etcd-backup");
        let req = CertificateRequest {
            join_token: "clusterid.clustersecret".to_string(),
            csr,
        };
        let resp = svc.handle_certificate(&req, &key, 2000).unwrap();
        let cert = &resp.identity.certificate;
        // an etcd-backup identity satisfies an etcd-backup requirement but not admin
        assert!(svc.require_role(cert, Role::EtcdBackup, 2500).is_ok());
        assert!(svc.require_role(cert, Role::Admin, 2500).is_err());
        // and does not grant plain read (etcd-backup is a narrow capability)
        assert!(svc.require_role(cert, Role::Reader, 2500).is_err());
    }

    #[test]
    fn revocations_accessor_reflects_state() {
        let (mut svc, _t) = service();
        assert!(svc.revocations().is_empty());
        svc.revoke(99, RevocationReason::Superseded, 100);
        assert!(svc.revocations().is_revoked(99));
    }
}
