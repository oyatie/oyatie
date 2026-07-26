//! Certificate freshness tracking and rotation decisions.
//!
//! Talos's secrets controllers regenerate leaf certificates well before they
//! expire and also when their inputs (SANs, CA) change. This module models the
//! renewal policy and the small state machine each managed certificate moves
//! through, independent of which CA issued it. It corresponds to the renewal
//! logic shared by the `KubernetesCerts`, `EtcdCerts`, `APICert` and
//! `TrustdCerts` controllers.

use crate::bundle::{Certificate, Validity};
use os_kernel::error::{Error, Result};

/// The renewal policy: renew when the remaining lifetime drops below a fraction
/// of the total lifetime. Talos renews leaf certs at roughly 50% of life.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenewalPolicy {
    /// Numerator of the renew-at-fraction threshold.
    pub num: u64,
    /// Denominator of the renew-at-fraction threshold.
    pub den: u64,
}

impl Default for RenewalPolicy {
    fn default() -> Self {
        // Renew once less than half the lifetime remains.
        RenewalPolicy { num: 1, den: 2 }
    }
}

impl RenewalPolicy {
    /// Construct a policy, validating the fraction is in `(0, 1]`.
    pub fn new(num: u64, den: u64) -> Result<Self> {
        if den == 0 || num == 0 || num > den {
            return Err(Error::invalid("renewal fraction must be in (0, 1]"));
        }
        Ok(RenewalPolicy { num, den })
    }

    /// Whether a validity window needs renewal at `now` under this policy.
    pub fn needs_renewal(&self, validity: &Validity, now: u64) -> bool {
        let total = validity.total();
        let threshold = total.saturating_mul(self.num) / self.den;
        validity.remaining(now) <= threshold
    }
}

/// The lifecycle state of a managed certificate, as seen by a controller's
/// reconcile loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertState {
    /// No certificate exists yet; one must be generated.
    Missing,
    /// Present and well within its validity window.
    Valid,
    /// Present but inside the renewal threshold; should be rotated soon.
    Renewing,
    /// Present but its inputs (SANs/CA) changed; must be regenerated now.
    Stale,
    /// Present but already expired; must be regenerated now.
    Expired,
}

impl CertState {
    /// Whether a controller must (re)issue a certificate in this state.
    pub fn needs_issue(self) -> bool {
        matches!(
            self,
            CertState::Missing | CertState::Stale | CertState::Expired
        )
    }

    /// Whether the controller should proactively rotate (issue) here. This is
    /// true for `Renewing` in addition to the hard cases.
    pub fn should_rotate(self) -> bool {
        self.needs_issue() || matches!(self, CertState::Renewing)
    }

    /// The lowercase status string surfaced on the secret status resource.
    pub fn as_str(self) -> &'static str {
        match self {
            CertState::Missing => "missing",
            CertState::Valid => "valid",
            CertState::Renewing => "renewing",
            CertState::Stale => "stale",
            CertState::Expired => "expired",
        }
    }
}

/// Decide the state of an optional managed certificate.
///
/// * `current` — the certificate currently on disk, if any.
/// * `desired_fingerprint` — fingerprint of the inputs (SAN set + CA serial)
///   the controller wants; if it differs from what produced `current`, the cert
///   is [`CertState::Stale`].
/// * `current_fingerprint` — the fingerprint the current cert was built from.
pub fn evaluate(
    current: Option<&Certificate>,
    desired_fingerprint: u64,
    current_fingerprint: u64,
    policy: &RenewalPolicy,
    now: u64,
) -> CertState {
    let Some(cert) = current else {
        return CertState::Missing;
    };
    if cert.validity.is_expired(now) {
        return CertState::Expired;
    }
    if desired_fingerprint != current_fingerprint {
        return CertState::Stale;
    }
    if policy.needs_renewal(&cert.validity, now) {
        return CertState::Renewing;
    }
    CertState::Valid
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::{CertUsage, Certificate, Subject};

    fn cert(not_before: u64, ttl: u64) -> Certificate {
        Certificate {
            serial: 1,
            subject: Subject::common("x"),
            issuer: Subject::common("ca"),
            validity: Validity::from_duration(not_before, ttl).unwrap(),
            usage: CertUsage::ServerAuth,
            sans: vec![],
            public_key_der: vec![1, 2, 3],
            signature: vec![9],
        }
    }

    #[test]
    fn renewal_policy_validation() {
        assert!(RenewalPolicy::new(1, 2).is_ok());
        assert!(RenewalPolicy::new(0, 2).is_err());
        assert!(RenewalPolicy::new(3, 2).is_err());
        assert!(RenewalPolicy::new(1, 0).is_err());
    }

    #[test]
    fn needs_renewal_at_half_life() {
        let p = RenewalPolicy::default();
        let v = Validity::from_duration(1000, 1000).unwrap();
        assert!(!p.needs_renewal(&v, 1400)); // 600 remaining > 500
        assert!(p.needs_renewal(&v, 1600)); // 400 remaining <= 500
    }

    #[test]
    fn evaluate_missing_and_expired() {
        let p = RenewalPolicy::default();
        assert_eq!(evaluate(None, 0, 0, &p, 0), CertState::Missing);
        let c = cert(1000, 100);
        assert_eq!(evaluate(Some(&c), 5, 5, &p, 2000), CertState::Expired);
    }

    #[test]
    fn evaluate_stale_then_renewing_then_valid() {
        let p = RenewalPolicy::default();
        let c = cert(1000, 1000);
        // Fingerprint mismatch -> stale (even if otherwise fine).
        assert_eq!(evaluate(Some(&c), 7, 8, &p, 1100), CertState::Stale);
        // Same fingerprint, past half-life -> renewing.
        assert_eq!(evaluate(Some(&c), 7, 7, &p, 1600), CertState::Renewing);
        // Same fingerprint, fresh -> valid.
        assert_eq!(evaluate(Some(&c), 7, 7, &p, 1100), CertState::Valid);
    }

    #[test]
    fn cert_state_action_flags() {
        assert!(CertState::Missing.needs_issue());
        assert!(CertState::Expired.needs_issue());
        assert!(CertState::Stale.needs_issue());
        assert!(!CertState::Renewing.needs_issue());
        assert!(CertState::Renewing.should_rotate());
        assert!(!CertState::Valid.should_rotate());
        assert_eq!(CertState::Renewing.as_str(), "renewing");
    }
}
