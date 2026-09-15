use crate::{ActionSubmitter, SubmitError, SubmitRequest};

/// A fresh adapter must authenticate, authorize, and deduplicate in this order.
/// Tokens are fixture inputs; they are never included in failure diagnostics.
pub async fn check_submission(
    submitter: &dyn ActionSubmitter,
    allowed: &str,
    denied: &str,
    unknown: &str,
    request: SubmitRequest,
) -> Result<(), String> {
    if submitter.submit(unknown, request.clone()).await != Err(SubmitError::Credential) {
        return Err("unknown credential was not refused".into());
    }
    if submitter.submit(denied, request.clone()).await != Err(SubmitError::Authorization) {
        return Err("recognized unauthorized caller was not refused".into());
    }
    let first = submitter
        .submit(allowed, request.clone())
        .await
        .map_err(|_| "authorized submission failed")?;
    if first.outcome != "applied" || first.deduplicated || first.ordinal == 0 {
        return Err("first authorized submission was not applied".into());
    }
    let retry = submitter
        .submit(allowed, request.clone())
        .await
        .map_err(|_| "identical retry failed")?;
    if retry.ordinal != first.ordinal || !retry.deduplicated || retry.outcome != first.outcome {
        return Err("identical retry did not retain the original receipt".into());
    }
    if submitter.submit(denied, request.clone()).await != Err(SubmitError::Authorization) {
        return Err("deduplication bypassed current authorization".into());
    }
    let mut divergent = request;
    divergent.occurred_at_epoch_seconds += 1;
    if submitter.submit(allowed, divergent).await != Err(SubmitError::Conflict) {
        return Err("divergent reuse of an idempotency key was not a conflict".into());
    }
    Ok(())
}

/// A valid credential for another tenant cannot receive the source's data.
pub async fn check_tenant_binding(
    submitter: &dyn ActionSubmitter,
    allowed: &str,
    foreign: &str,
    tenant: &str,
    request: SubmitRequest,
) -> Result<(), String> {
    if submitter
        .submit_in_tenant(foreign, tenant, request.clone())
        .await
        != Err(SubmitError::TenantMismatch)
    {
        return Err("foreign destination credential accepted source tenant data".into());
    }
    if submitter
        .submit_in_tenant(allowed, "", request.clone())
        .await
        != Err(SubmitError::TenantMismatch)
    {
        return Err("empty expected tenant disabled the source boundary".into());
    }
    let receipt = submitter
        .submit_in_tenant(allowed, tenant, request)
        .await
        .map_err(|_| "matching tenant submission failed")?;
    if receipt.ordinal != 1 || receipt.deduplicated {
        return Err("refused tenant mismatch changed destination state".into());
    }
    Ok(())
}
