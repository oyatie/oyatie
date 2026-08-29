//! Identity, token issuance, and data-use grants.

use crate::*;

use crate::Foundation;

impl Foundation {
    pub fn upsert_identity(
        &mut self,
        registration: IdentityRegistration,
    ) -> Result<User, FoundationError> {
        let tenant = self.require_tenant(&registration.tenant_id)?;
        let region_pack = tenant
            .regulatory_packs
            .value
            .iter()
            .find(|pack| pack.starts_with("pack-"))
            .cloned()
            .unwrap_or_else(|| {
                format!(
                    "pack-{}",
                    tenant
                        .residency_class
                        .value
                        .label()
                        .unwrap_or("global")
                        .replace('_', "-")
                )
            });
        let idp_binding = IdpBinding::new(
            region_pack,
            "idp_foundation_local".to_string(),
            registration.primary_identifier.clone(),
            0,
        )
        .map_err(map_identity_error)?;
        let user = User::new(
            registration.tenant_id.clone(),
            registration.user_id,
            registration.primary_identifier,
            registration.display_name,
            registration.roles,
            idp_binding,
        )
        .map_err(map_identity_error)?;
        self.users.insert(
            (
                registration.tenant_id.clone(),
                user.user_id().as_str().to_string(),
            ),
            user.clone(),
        );
        self.audit_chain.append_classifications(
            registration.tenant_id,
            "identity.user.upsert",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::PiiIdentifying, DataClass::InternalOnly],
            "ALLOW",
        )?;
        Ok(user)
    }

    pub fn issue_token(&mut self, request: TokenRequest) -> Result<Token, FoundationError> {
        self.require_user(&request.tenant_id, &request.user_id)?;
        match issue_token(
            request.tenant_id.clone(),
            request.user_id,
            request.purpose,
            request.ttl_seconds,
            request.issued_at_epoch_seconds,
        ) {
            Ok(token) => {
                self.audit_chain.append_classifications(
                    token.tenant_id.clone(),
                    "identity.token.issue",
                    Plane::Control,
                    request.purpose,
                    vec![DataClass::PiiIdentifying],
                    "ALLOW",
                )?;
                Ok(token)
            }
            Err(IdentityError::TokenTtlTooLong) => {
                self.audit_chain.append_classifications(
                    request.tenant_id,
                    "identity.token.issue",
                    Plane::Control,
                    request.purpose,
                    vec![DataClass::PiiIdentifying],
                    "DENY",
                )?;
                Err(FoundationError::TokenTtlTooLong)
            }
            Err(_) => Err(FoundationError::InvalidInput),
        }
    }

    pub fn grant_data_use(
        &mut self,
        tenant_id: &str,
        purpose: Purpose,
        data_class: PrivacyDataClass,
    ) -> Result<(), FoundationError> {
        self.grant_privacy_data_use(tenant_id, purpose, data_class)
    }

    /// Compatibility entry point for raw-label callers at import/API seams.
    ///
    /// The canonical grant path takes `PrivacyDataClass`; this path preserves
    /// older raw `DataClass` ingestion while failing closed for operational
    /// markers and subject markers.
    pub fn try_grant_legacy_data_use(
        &mut self,
        tenant_id: &str,
        purpose: Purpose,
        data_class: DataClass,
    ) -> Result<(), FoundationError> {
        let data_class =
            PrivacyDataClass::try_from(data_class).map_err(|_| FoundationError::InvalidInput)?;
        self.grant_privacy_data_use(tenant_id, purpose, data_class)
    }

    pub fn grant_privacy_data_use(
        &mut self,
        tenant_id: &str,
        purpose: Purpose,
        data_class: PrivacyDataClass,
    ) -> Result<(), FoundationError> {
        self.require_tenant(tenant_id)?;
        let current = self.consent_scopes.remove(tenant_id).unwrap_or_default();
        self.consent_scopes.insert(
            tenant_id.to_string(),
            current.allow_privacy_data_class(purpose, data_class),
        );
        let audit_data_class = data_class.data_class();
        self.audit_chain.append_classifications(
            tenant_id,
            "privacy.data-use.grant",
            Plane::Control,
            purpose,
            vec![audit_data_class],
            "ALLOW",
        )?;
        Ok(())
    }
}
