use super::{RuntimeInstallation, installed::FencedRuntime};
use crate::composition::TenantState;
use foundry_model_api::{control::*, *};
use foundry_spine::{ProjectionState, preflight_projection};

fn refused() -> ModelError {
    ModelError::Storage {
        detail: "serving model publication could not be established".into(),
    }
}

fn proof(tenant: &TenantState) -> Result<&FencedRuntime, ModelError> {
    match &tenant.model_installation {
        RuntimeInstallation::Installed(proof) if proof.matches(tenant) => Ok(proof),
        _ => Err(refused()),
    }
}

fn replay(
    tenant: &TenantState,
    registry: &data_ontology_kernel::OntologyEngine,
) -> Result<ProjectionState, ModelError> {
    let id = &tenant.projection.tenant_id;
    let head = tenant.action_log.head(id).map_err(|_| refused())?;
    if tenant.projection.applied_ordinal != head {
        return Err(refused());
    }
    let entries = tenant.action_log.replay(id, 1).map_err(|_| refused())?;
    preflight_projection(id, registry, &*tenant.projection_store, &entries, head)
        .map_err(|_| refused())
}

pub(crate) struct StagedPublication {
    prepared: PreparedPublication,
    receipt: PublicationReceipt,
    projection: ProjectionState,
    installation: RuntimeInstallation,
    response: Vec<u8>,
}

impl StagedPublication {
    pub(crate) fn prepare(
        tenant: &TenantState,
        caller: &ModelCaller,
        id: u64,
        input: &PrepareModelDraft,
    ) -> Result<Self, ModelError> {
        if tenant.publication_unknown.is_some() {
            return Err(refused());
        }
        let proof = proof(tenant)?;
        let model = tenant.model_store.as_ref().ok_or_else(refused)?;
        let prepared = model.store.prepare(
            caller,
            id,
            input.expected_version.get(),
            input.expected_base.get(),
        )?;
        if prepared.base().binding != proof.binding
            || prepared.base().head()?.generation != proof.generation
            || prepared
                .base()
                .registry_with_runtime_baseline(&proof.baseline)?
                != proof.registry
        {
            return Err(refused());
        }
        if replay(tenant, &proof.registry)? != tenant.projection {
            return Err(refused());
        }
        let registry = prepared.registry_with_runtime_baseline(&proof.baseline)?;
        let projection = replay(tenant, &registry)?;
        let receipt = prepared
            .base()
            .publication_receipt(caller, prepared.draft(), &prepared)?;
        let response = ModelPublished {
            receipt: ModelReceiptWire::try_from(&receipt)?,
            lifecycle: super::lifecycle::current(receipt.generation)?,
        }
        .encode()?;
        let installation = proof.replacement(registry, receipt.generation);
        Ok(Self {
            prepared,
            receipt,
            projection,
            installation,
            response,
        })
    }

    pub(crate) fn commit(
        self,
        tenant: &mut TenantState,
        caller: &ModelCaller,
    ) -> Result<Vec<u8>, ModelError> {
        let model = tenant.model_store.as_mut().ok_or_else(refused)?;
        let result = model.store.publish(caller, &self.prepared);
        if result
            .as_ref()
            .is_ok_and(|receipt| receipt == &self.receipt)
        {
            return Ok(self.install(tenant));
        }
        // An error (or contradictory acknowledgement) does not prove rollback.
        let receipt = model.store.receipt(
            caller,
            self.receipt.draft_id,
            self.receipt.draft_version,
            self.receipt.prior_generation,
        );
        let head = model.store.head(caller);
        if let (Ok(receipt), Ok(head)) = (receipt, head) {
            if receipt.as_ref() == Some(&self.receipt)
                && head.generation == self.receipt.generation
                && head.receipt.as_ref() == Some(&self.receipt)
                && head.document == self.prepared.draft().document
            {
                return Ok(self.install(tenant));
            }
            if receipt.is_none()
                && self
                    .prepared
                    .base()
                    .head()
                    .is_ok_and(|prior| prior == &head)
            {
                return Err(result.err().unwrap_or_else(refused));
            }
        }
        tenant.publication_unknown = Some(self.receipt);
        Err(refused())
    }

    fn install(self, tenant: &mut TenantState) -> Vec<u8> {
        tenant.projection = self.projection;
        tenant.model_installation = self.installation;
        tenant.publication_unknown = None;
        self.response
    }
}

pub(crate) struct StagedRecovery {
    projection: ProjectionState,
    installation: RuntimeInstallation,
    pub(crate) response: Option<Vec<u8>>,
}
impl StagedRecovery {
    pub(crate) fn prepare(
        tenant: &TenantState,
        caller: &ModelCaller,
        receipt: Option<&PublicationReceipt>,
    ) -> Result<Self, ModelError> {
        let proof = proof(tenant)?;
        let model = tenant.model_store.as_ref().ok_or_else(refused)?;
        let history = model.store.history(caller)?;
        if history.binding != proof.binding || history.head()?.generation < proof.generation {
            return Err(refused());
        }
        let registry = history.registry_with_runtime_baseline(&proof.baseline)?;
        let generation = history.head()?.generation;
        if let Some(receipt) = receipt {
            if !history
                .publications
                .iter()
                .any(|row| row.receipt.as_ref() == Some(receipt))
            {
                return Err(refused());
            }
        } else if tenant
            .publication_unknown
            .as_ref()
            .is_none_or(|pending| generation != pending.prior_generation)
        {
            return Err(refused());
        }
        let projection = replay(tenant, &registry)?;
        let response = receipt
            .map(|receipt| {
                ModelPublished {
                    receipt: ModelReceiptWire::try_from(receipt)?,
                    lifecycle: super::lifecycle::current(generation)?,
                }
                .encode()
            })
            .transpose()?;
        let installation = proof.replacement(registry, generation);
        Ok(Self {
            projection,
            installation,
            response,
        })
    }
    pub(crate) fn install(self, tenant: &mut TenantState) -> Option<Vec<u8>> {
        tenant.projection = self.projection;
        tenant.model_installation = self.installation;
        tenant.publication_unknown = None;
        self.response
    }
}

#[cfg(test)]
#[path = "publication/tests.rs"]
mod tests;
