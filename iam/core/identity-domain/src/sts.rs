use super::*;

impl Principal {
    pub fn human(tenant_id: String, user_id: String) -> Result<Self, IdentityError> {
        validate_tenant_id(&tenant_id)?;
        validate_user_id(&user_id)?;
        Ok(Self::Human {
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            user_id: Classified::new(user_id, DataClass::PiiIdentifying),
        })
    }

    pub fn service(
        tenant_id: String,
        service_principal_id: String,
        owning_capability_id: String,
    ) -> Result<Self, IdentityError> {
        validate_tenant_id(&tenant_id)?;
        if !service_principal_id.starts_with("sp_") || service_principal_id.len() <= 3 {
            return Err(IdentityError::InvalidServicePrincipalId);
        }
        if !owning_capability_id.starts_with("cap.") {
            return Err(IdentityError::InvalidCapabilityId);
        }
        Ok(Self::ServicePrincipal {
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            service_principal_id: Classified::new(service_principal_id, DataClass::InternalOnly),
            owning_capability_id: Classified::new(owning_capability_id, DataClass::InternalOnly),
        })
    }

    fn fingerprint_material(&self) -> String {
        match self {
            Self::Human { tenant_id, user_id } => {
                format!("human:{}:{}", tenant_id.value, user_id.value)
            }
            Self::ServicePrincipal {
                tenant_id,
                service_principal_id,
                owning_capability_id,
            } => format!(
                "service:{}:{}:{}",
                tenant_id.value, service_principal_id.value, owning_capability_id.value
            ),
        }
    }
}

impl StsCredential {
    pub fn is_active(&self, now_epoch_seconds: u64) -> bool {
        now_epoch_seconds < self.expires_at_epoch_seconds.value
    }
}

impl fmt::Debug for StsCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StsCredential")
            .field("principal", &self.principal)
            .field("purpose", &self.purpose)
            .field("scopes", &self.scopes)
            .field("issued_at_epoch_seconds", &self.issued_at_epoch_seconds)
            .field("expires_at_epoch_seconds", &self.expires_at_epoch_seconds)
            .field("token_fingerprint", &self.token_fingerprint.value)
            .finish()
    }
}

pub fn issue_credential(request: CredentialRequest) -> Result<StsCredential, IdentityError> {
    if request.kind == CredentialRequestKind::LongLivedApiKey {
        return Err(IdentityError::LongLivedCredentialForbidden);
    }
    if request.scopes.is_empty() || request.scopes.iter().any(|scope| scope.trim().is_empty()) {
        return Err(IdentityError::MissingCredentialScope);
    }
    let token = issue_token(
        principal_tenant_id(&request.principal).to_string(),
        principal_subject_id(&request.principal).to_string(),
        request.purpose,
        request.ttl_seconds,
        request.issued_at_epoch_seconds,
    )?;
    let fingerprint = credential_fingerprint(&request.principal, &request.scopes, &token);
    Ok(StsCredential {
        principal: request.principal,
        purpose: Classified::new(token.purpose, DataClass::InternalOnly),
        scopes: Classified::new(request.scopes, DataClass::InternalOnly),
        issued_at_epoch_seconds: Classified::new(
            token.issued_at_epoch_seconds,
            DataClass::InternalOnly,
        ),
        expires_at_epoch_seconds: Classified::new(
            token.expires_at_epoch_seconds,
            DataClass::InternalOnly,
        ),
        token_fingerprint: Classified::new(fingerprint, DataClass::InternalOnly),
    })
}

fn principal_tenant_id(principal: &Principal) -> &str {
    match principal {
        Principal::Human { tenant_id, .. } | Principal::ServicePrincipal { tenant_id, .. } => {
            &tenant_id.value
        }
    }
}

fn principal_subject_id(principal: &Principal) -> &str {
    match principal {
        Principal::Human { user_id, .. } => &user_id.value,
        Principal::ServicePrincipal {
            service_principal_id,
            ..
        } => &service_principal_id.value,
    }
}

fn credential_fingerprint(principal: &Principal, scopes: &[String], token: &Token) -> String {
    let mut state = 0xcbf29ce484222325_u64;
    fn feed(state: &mut u64, bytes: &[u8]) {
        for byte in bytes {
            *state ^= u64::from(*byte);
            *state = state.wrapping_mul(0x100000001b3);
        }
    }
    feed(&mut state, principal.fingerprint_material().as_bytes());
    feed(&mut state, token.purpose.pascal_label().as_bytes());
    for scope in scopes {
        feed(&mut state, scope.as_bytes());
    }
    feed(
        &mut state,
        token.issued_at_epoch_seconds.to_string().as_bytes(),
    );
    feed(
        &mut state,
        token.expires_at_epoch_seconds.to_string().as_bytes(),
    );
    format!("sts1:{state:016x}")
}
