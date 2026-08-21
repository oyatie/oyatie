//! Shell token brokerage (ADR-0536 D-4; story G007).
//!
//! The shell is the SOLE token broker: product surfaces never hold raw
//! tokens. This module encodes that as structure, on the locked G05 OIDC
//! port (`oya-shared-oidc-client-kernel`):
//!
//! - [`ShellTokenBroker::establish`] verifies a bearer through the
//!   [`OidcClient`] port and discards it — only verified claims survive, in a
//!   [`BrokeredSession`] whose claims are private to this module;
//! - a product module receives a [`ModuleGrant`]: the scoped subset of the
//!   session bound to ONE capability-registry entry. The type has no token
//!   field and no claims field, so leaking either through a module is
//!   unrepresentable;
//! - grants are deny-by-default: a grant is minted only when the session's
//!   ACR meets the capability's floor.
//!
//! The broker is generic over the [`OidcClient`] port so deployment-specific
//! OIDC adapters can swap without changing this module or its callers.

use shared_oidc_client_kernel::{AcrLevel, OidcClaims, OidcClient, OidcError, VerifyConfig};
use shared_platform_contracts_kernel::shell_bff::CapabilityRegistryEntry;

/// A verified session held by the shell. The claims never leave this module:
/// product surfaces interact only through [`ModuleGrant`]s minted from it.
pub struct BrokeredSession {
    claims: OidcClaims,
}

impl BrokeredSession {
    pub fn subject(&self) -> &str {
        &self.claims.sub
    }

    pub fn tenant_id(&self) -> &str {
        &self.claims.tenant_id
    }
}

/// The scoped view a product module receives: capability-bound identity
/// facts only. No bearer, no claims object, no headers — by construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleGrant {
    pub capability_id: String,
    pub module_id: String,
    pub subject: String,
    pub tenant_id: String,
    pub acr: AcrLevel,
    pub expires_at_unix: i64,
}

/// Why a grant was refused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GrantRefusal {
    /// The session's ACR does not meet the capability's required floor.
    AcrBelowFloor {
        required: AcrLevel,
        actual: AcrLevel,
    },
}

/// ACR floor per navigation sensitivity. Admin-audit-grade capabilities
/// (PDP action prefix `audit.`) demand step-up auth (Sensitive); everything else takes
/// the routine floor.
fn acr_floor_for(capability: &CapabilityRegistryEntry) -> AcrLevel {
    if capability.required_action.starts_with("audit.") {
        AcrLevel::Sensitive
    } else {
        AcrLevel::Routine
    }
}

/// The shell's broker over the locked OIDC port.
pub struct ShellTokenBroker<C: OidcClient> {
    client: C,
    config: VerifyConfig,
}

impl<C: OidcClient> ShellTokenBroker<C> {
    pub fn new(client: C, config: VerifyConfig) -> Self {
        Self { client, config }
    }

    /// Verify a bearer through the port and retain ONLY the claims. The raw
    /// token is dropped at the end of this function; nothing downstream can
    /// reach it.
    pub fn establish(&self, bearer: &str) -> Result<BrokeredSession, OidcError> {
        let claims = self.client.verify(bearer, &self.config)?;
        Ok(BrokeredSession { claims })
    }

    /// Mint the scoped grant a product module receives for one registered
    /// capability. Deny-by-default: no floor-meeting ACR, no grant.
    pub fn grant_for_capability(
        &self,
        session: &BrokeredSession,
        capability: &CapabilityRegistryEntry,
    ) -> Result<ModuleGrant, GrantRefusal> {
        let floor = acr_floor_for(capability);
        if !session.claims.acr.meets(floor) {
            return Err(GrantRefusal::AcrBelowFloor {
                required: floor,
                actual: session.claims.acr,
            });
        }
        Ok(ModuleGrant {
            capability_id: capability.capability_id.clone(),
            module_id: capability.module_id.clone(),
            subject: session.claims.sub.clone(),
            tenant_id: session.claims.tenant_id.clone(),
            acr: session.claims.acr,
            expires_at_unix: session.claims.exp,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use shared_oidc_client_kernel::Audience;
    use shared_platform_contracts_kernel::shell_bff::NavigationSurface;

    use super::*;

    const TEST_BEARER: &str = "test-raw-bearer-token-must-never-leak";

    struct FixedClaimsClient {
        acr: AcrLevel,
    }

    impl OidcClient for FixedClaimsClient {
        fn verify(&self, bearer: &str, _cfg: &VerifyConfig) -> Result<OidcClaims, OidcError> {
            if bearer != TEST_BEARER {
                return Err(OidcError::SignatureInvalid);
            }
            Ok(OidcClaims {
                iss: "https://idp.oyatie.test".to_owned(),
                aud: Audience::Single("oya-console".to_owned()),
                sub: "principal-7".to_owned(),
                iat: 1_000,
                exp: 4_000,
                nbf: None,
                jti: None,
                tenant_id: "acme".to_owned(),
                acr: self.acr,
                acr_event_at: None,
                purpose: None,
                data_class: None,
                additional: BTreeMap::new(),
            })
        }
    }

    fn broker(acr: AcrLevel) -> ShellTokenBroker<FixedClaimsClient> {
        ShellTokenBroker::new(
            FixedClaimsClient { acr },
            VerifyConfig {
                expected_issuer: "https://idp.oyatie.test".to_owned(),
                expected_audience: "oya-console".to_owned(),
                clock_tolerance: Default::default(),
                now_unix_seconds: 2_000,
            },
        )
    }

    fn capability(action: &str) -> CapabilityRegistryEntry {
        CapabilityRegistryEntry {
            capability_id: "audit-chain".to_owned(),
            display_name: "Audit Chain".to_owned(),
            module_id: "audit".to_owned(),
            required_action: action.to_owned(),
            navigation_surface: NavigationSurface::PrimaryNav,
        }
    }

    #[test]
    fn module_grant_carries_no_token_material() {
        let broker = broker(AcrLevel::Sensitive);
        let session = broker.establish(TEST_BEARER).expect("verified session");
        let grant = broker
            .grant_for_capability(&session, &capability("audit.inspect"))
            .expect("grant");

        // The grant is the ONLY thing a product surface receives; assert the
        // bearer is absent from its entire debug projection.
        let projected = format!("{grant:?}");
        assert!(!projected.contains(TEST_BEARER), "{projected}");
        assert_eq!(grant.subject, "principal-7");
        assert_eq!(grant.tenant_id, "acme");
        assert_eq!(grant.expires_at_unix, 4_000);
    }

    #[test]
    fn invalid_bearer_yields_no_session() {
        let broker = broker(AcrLevel::Routine);
        // BrokeredSession deliberately implements no Debug (claims privacy),
        // so match on the result instead of unwrap_err.
        assert!(matches!(
            broker.establish("forged"),
            Err(OidcError::SignatureInvalid)
        ));
    }

    #[test]
    fn audit_capabilities_require_step_up_acr() {
        let broker = broker(AcrLevel::Routine);
        let session = broker.establish(TEST_BEARER).expect("verified session");
        let refused = broker
            .grant_for_capability(&session, &capability("audit.inspect"))
            .unwrap_err();
        assert_eq!(
            refused,
            GrantRefusal::AcrBelowFloor {
                required: AcrLevel::Sensitive,
                actual: AcrLevel::Routine,
            }
        );

        // Standard capabilities mint at the routine floor.
        broker
            .grant_for_capability(&session, &capability("tenancy.administer"))
            .expect("standard-floor grant");
    }
}
