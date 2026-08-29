//! Provider routing, failed-invocation settlement, and compensation.

use crate::*;

use crate::Foundation;

impl Foundation {
    pub(crate) fn resolve_foundation_local_provider_route(
        tenant: &Tenant,
        capability: &Capability,
        budget_snapshot: &BudgetSnapshot,
        request: &CapabilityInvocationRequest,
        privacy_data_classes: &[PrivacyDataClass],
    ) -> Result<ProviderRoute, AdapterError> {
        let provider_id = ProviderId::new(FOUNDATION_LOCAL_PROVIDER_ID.into())?;
        let provider_profile = ProviderProfile::new_with_privacy_data_classes(
            provider_id.clone(),
            ProviderMode::Api,
            ProviderAuth::Api {
                secret_ref: SecretRef::new(
                    request.tenant_id.clone(),
                    request.capability_id.clone(),
                    FOUNDATION_LOCAL_SECRET_REF_NAME.into(),
                )
                .map_err(|_| AdapterError::MissingProviderCapability)?,
                billing_account: request.tenant_id.clone(),
            },
            privacy_data_classes.to_vec(),
            vec![tenant.home_region.value.clone()],
            request.projected_cost_micros,
            FOUNDATION_LOCAL_PROVIDER_P95_LATENCY_MS,
        )?;
        let profiles = [provider_profile];
        let subscription_bindings = SubscriptionBindingRegistry::default();
        let provider_preference = capability
            .provider_preference()
            .iter()
            .cloned()
            .map(ProviderId::new)
            .collect::<Result<Vec<_>, _>>()?;
        resolve_route(ProviderRouteRequest {
            capability,
            policy: InvocationPolicy::new_with_privacy_data_classes(
                Classified::new(request.tenant_id.clone(), DataClass::InternalOnly),
                privacy_data_classes.to_vec(),
                Classified::new(tenant.home_region.value.clone(), DataClass::InternalOnly),
                CostCeiling::from_budget_snapshot(budget_snapshot),
                10_000,
            ),
            preference: ProviderRoutePreference::ordered(provider_preference)?,
            profiles: &profiles,
            subscription_bindings: &subscription_bindings,
        })
    }

    // ADR-0083 amendment 2026-05-15: `settle_failed_invocation` returns
    // `Result<FoundationError, FoundationError>` so the 3 internal
    // `append_classifications` sites can propagate `AuditChainError` via `?`.
    // `Ok(primary_error)` carries the original failure for the outer caller to
    // return as `Err(primary_error)`; `Err(audit_chain_error)` supersedes the
    // primary error when the audit chain itself fails — ADR-0083 Tier 1
    // forbids silently dropping `AuditChainError`.
    pub(crate) fn settle_failed_invocation(
        &mut self,
        request: &CapabilityInvocationRequest,
        reservation_id: Option<&str>,
        run_id: Option<&str>,
        disposition: RunDisposition,
        primary_error: FoundationError,
    ) -> Result<FoundationError, FoundationError> {
        let budget_release = if let Some(reservation_id) = reservation_id {
            if self.cost_budgets.release(reservation_id).is_ok() {
                self.audit_chain.append_classifications(
                    request.tenant_id.clone(),
                    "foundry.cost-budget.release",
                    Plane::Control,
                    request.purpose,
                    vec![DataClass::InternalOnly],
                    "ALLOW",
                )?;
                InvocationSettlementStatus::Completed
            } else {
                self.audit_chain.append_classifications(
                    request.tenant_id.clone(),
                    "foundry.cost-budget.release",
                    Plane::Control,
                    request.purpose,
                    vec![DataClass::InternalOnly],
                    "DENY",
                )?;
                InvocationSettlementStatus::Failed
            }
        } else {
            InvocationSettlementStatus::NotApplicable
        };
        let run_completion = if let Some(run_id) = run_id {
            if self
                .foundry_runs
                .complete(
                    run_id,
                    disposition,
                    request.started_at_epoch_seconds.saturating_add(1),
                )
                .is_err()
            {
                self.audit_chain.append_classifications(
                    request.tenant_id.clone(),
                    "foundry.run.complete",
                    Plane::Data,
                    request.purpose,
                    audit_classifications(),
                    "DENY",
                )?;
                InvocationSettlementStatus::Failed
            } else {
                InvocationSettlementStatus::Completed
            }
        } else {
            InvocationSettlementStatus::NotApplicable
        };
        self.record_invocation_compensation(
            request,
            reservation_id,
            run_id,
            disposition,
            &primary_error,
            budget_release,
            run_completion,
        )?;
        Ok(primary_error)
    }

    // ADR-0083 amendment 2026-05-15: `record_invocation_compensation` returns
    // `Result<(), FoundationError>` so the 6 internal `append_classifications`
    // sites can propagate `AuditChainError` via `?`. Caller
    // (`settle_failed_invocation`) re-propagates so the outermost invocation
    // path surfaces audit-chain failure rather than silently swallowing it.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_invocation_compensation(
        &mut self,
        request: &CapabilityInvocationRequest,
        reservation_id: Option<&str>,
        run_id: Option<&str>,
        disposition: RunDisposition,
        primary_error: &FoundationError,
        budget_release: InvocationSettlementStatus,
        run_completion: InvocationSettlementStatus,
    ) -> Result<(), FoundationError> {
        let compensation_audit_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                "foundry.invocation.compensate",
                Plane::Audit,
                request.purpose,
                audit_classifications(),
                "ALLOW",
            )?
            .hash
            .clone();
        let Some(run_id) = run_id else {
            return Ok(());
        };
        let Some(capability) = self.capabilities.get(&request.capability_id).cloned() else {
            self.audit_chain.append_classifications(
                request.tenant_id.clone(),
                "foundry.invocation.compensate",
                Plane::Audit,
                request.purpose,
                audit_classifications(),
                "DENY",
            )?;
            return Ok(());
        };
        let mut evidence_fields = BTreeMap::from([
            (
                "audit_event_hash".to_string(),
                compensation_audit_hash.clone(),
            ),
            (
                "budget_release".to_string(),
                budget_release.as_release_str().to_string(),
            ),
            ("decision".to_string(), "FAIL".to_string()),
            ("disposition".to_string(), format!("{disposition:?}")),
            (
                "evidence_topic".to_string(),
                capability.evidence_topic.value.clone(),
            ),
            ("primary_error".to_string(), format!("{primary_error:?}")),
            ("reason".to_string(), "invocation_compensation".to_string()),
            (
                "run_completion".to_string(),
                run_completion.as_completion_str().to_string(),
            ),
            ("run_id".to_string(), run_id.to_string()),
        ]);
        if let Some(reservation_id) = reservation_id {
            evidence_fields.insert(
                "cost_reservation_id".to_string(),
                reservation_id.to_string(),
            );
        }
        let evidence = match self.foundry_evidence.append(
            request.tenant_id.clone(),
            run_id.to_string(),
            None,
            request.capability_id.clone(),
            EvidenceKind::CapabilityInvocation,
            evidence_fields,
            capability.touched_privacy_data_classes().to_vec(),
            request.started_at_epoch_seconds.saturating_add(1),
        ) {
            Ok(evidence) => evidence,
            Err(_) => {
                self.audit_chain.append_classifications(
                    request.tenant_id.clone(),
                    "foundry.invocation.compensate",
                    Plane::Audit,
                    request.purpose,
                    audit_classifications(),
                    "DENY",
                )?;
                return Ok(());
            }
        };
        if self
            .outbox
            .publish(
                request.tenant_id.clone(),
                capability.evidence_topic.value,
                evidence.evidence_id.value.clone(),
                format!("foundry-evidence:{}", evidence.evidence_id.value),
            )
            .is_err()
        {
            self.audit_chain.append_classifications(
                request.tenant_id.clone(),
                "foundry.invocation.compensate.outbox",
                Plane::Audit,
                request.purpose,
                audit_classifications(),
                "DENY",
            )?;
            return Ok(());
        }
        self.audit_chain.append_classifications(
            request.tenant_id.clone(),
            "foundry.evidence.topic.emit",
            Plane::Audit,
            request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        self.audit_chain.append_classifications(
            request.tenant_id.clone(),
            "foundry.evidence.emit",
            Plane::Audit,
            request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod compensation_tests;
#[cfg(test)]
mod run_failure_tests;
#[cfg(test)]
mod test_support;
