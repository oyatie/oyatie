use super::*;

pub(super) fn read_file(path: &str) -> Result<String, StartError> {
    std::fs::read_to_string(path).map_err(|source| StartError::Io {
        path: path.to_string(),
        source,
    })
}

pub(super) fn read_bytes(path: &str) -> Result<Vec<u8>, StartError> {
    std::fs::read(path).map_err(|source| StartError::Io {
        path: path.to_string(),
        source,
    })
}

/// Compose the optional OIDC issuer state (enabled when a signing key is
/// configured). The file signer is the transitional custody adapter behind
/// the kernel's `JwsSigner` port — the G02 KMS adapter replaces it there.
pub(super) fn build_issuer_state(config: &Config) -> Result<Option<Arc<IssuerState>>, StartError> {
    let Some(path) = &config.signing_key_path else {
        return Ok(None);
    };
    let signer = Es256FileSigner::from_pkcs8_der(&config.signing_kid, &read_bytes(path)?)
        .map_err(StartError::Issuer)?;
    let mut key = signer.signing_key().map_err(StartError::Issuer)?;
    key.activate(default_now_epoch_seconds())
        .map_err(StartError::Issuer)?;
    let issuer_url = IssuerUrl::new(&config.issuer).map_err(StartError::Issuer)?;
    Ok(Some(Arc::new(IssuerState::new(
        issuer_url,
        vec![key],
        Arc::new(signer),
        default_now_epoch_seconds,
    ))))
}

/// Wall-clock epoch seconds, saturating (ADR-0083 panic-free).
pub(super) fn default_now_epoch_seconds() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

/// Compose the application state from configuration (fail-fast).
///
/// # Errors
/// Returns [`StartError`] when a configured file cannot be read or parsed.
pub fn build_state(config: &Config) -> Result<Arc<AppState>, StartError> {
    let jwks = jwks_from_json(&read_file(&config.jwks_path)?).map_err(StartError::Jwks)?;
    let authorizer =
        CedarWorkloadAuthorizer::from_cedar_policies(&read_file(&config.cedar_policy_path)?)
            .map_err(StartError::Cedar)?;
    let (repository, denylist) = match &config.principals_path {
        Some(path) => seed_from_json(&read_file(path)?).map_err(StartError::Seed)?,
        None => (
            InMemoryWorkloadPrincipalRepository::new(),
            InMemoryRevocationDenylist::new(),
        ),
    };
    // AUTH-005 fail-closed mutating control plane (ADR-0581): the lifecycle
    // routes are served ONLY with a verified-caller port + a fail-closed PDP
    // port. Both are REQUIRED here; `Config::from_env` already refuses to load
    // without the bearer credential, so a serving binary always has the seam.
    let caller_verifier = Arc::new(BearerCallerVerifier::new(
        config.lifecycle_bearer.clone(),
        config.lifecycle_caller_tenant.clone(),
        config.lifecycle_caller_id.clone(),
    ));
    let lifecycle_authorizer = Arc::new(TenantScopedLifecycleAuthorizer::new());
    // AUTH-005 read decision surfaces (`/authorize`, `/authorize-with-token`,
    // `/authorize:batch`, `/tokens/validate`): the SAME verified-caller port (the
    // bearer above — NO new config) plus a fail-closed same-tenant decision port,
    // so a forged body / cross-tenant token cannot obtain an arbitrary decision.
    let decision_authorizer = Arc::new(TenantScopedDecisionAuthorizer::new());
    Ok(Arc::new(WorkloadAuthzState::new(
        repository,
        denylist,
        authorizer,
        jwks,
        ValidationConfig::new(&config.issuer, &config.audience),
        TracingAuditSink::new(),
        caller_verifier,
        lifecycle_authorizer,
        decision_authorizer,
    )))
}
