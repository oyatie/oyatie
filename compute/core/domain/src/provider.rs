use crate::{COMPUTE_SCHEMA_VERSION, CloudComputeError, IdempotencyKey, Instance};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ComputeProviderKind {
    AwsEc2,
    OciCompute,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComputeProviderVmCreateRequest {
    pub request_id: String,              // data_class: INTERNAL_ONLY
    pub provider_instance_ref: String,   // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub actor: String,                   // data_class: INTERNAL_ONLY
    pub idempotency_key: String,         // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub instance: Instance,              // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComputeProviderVmReceipt {
    pub provider_kind: ComputeProviderKind, // data_class: PUBLIC
    pub provider_request_id: String,        // data_class: INTERNAL_ONLY
    pub provider_instance_ref: String,      // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String,      // data_class: INTERNAL_ONLY
    pub instance_resource_id: String,       // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub region: String,                     // data_class: PUBLIC
    pub az: String,                         // data_class: PUBLIC
    pub cell_id: String,                    // data_class: PUBLIC
    pub schema_version: u32,                // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComputeProviderVmError {
    InvalidRequest,
    ProviderRejected {
        provider: ComputeProviderKind, // data_class: PUBLIC
        reason: String,                // data_class: INTERNAL_ONLY
    },
}

pub trait ComputeProviderVmPort {
    fn provider_kind(&self) -> ComputeProviderKind;
    fn create_vm(
        &self,
        input: ComputeProviderVmCreateRequest,
    ) -> Result<ComputeProviderVmReceipt, ComputeProviderVmError>;
}
impl ComputeProviderKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::AwsEc2 => "aws_ec2",
            Self::OciCompute => "oci_compute",
        }
    }
}

impl ComputeProviderVmCreateRequest {
    pub fn validate(&self) -> Result<(), ComputeProviderVmError> {
        if self.request_id.trim().is_empty()
            || self.provider_instance_ref.trim().is_empty()
            || self.tenant_id.trim().is_empty()
            || self.actor.trim().is_empty()
            || IdempotencyKey::new(self.idempotency_key.clone()).is_err()
            || self.tenant_id != self.instance.tenant_id.value
        {
            return Err(ComputeProviderVmError::InvalidRequest);
        }
        Ok(())
    }
}

impl ComputeProviderVmReceipt {
    pub fn from_request(
        provider_kind: ComputeProviderKind,
        request: ComputeProviderVmCreateRequest,
        provider_request_id: impl Into<String>,
        provider_evidence_ref: impl Into<String>,
    ) -> Result<Self, ComputeProviderVmError> {
        request.validate()?;
        let provider_request_id = provider_request_id.into();
        let provider_evidence_ref = provider_evidence_ref.into();
        if provider_request_id.trim().is_empty() || provider_evidence_ref.trim().is_empty() {
            return Err(ComputeProviderVmError::InvalidRequest);
        }
        Ok(Self {
            provider_kind,
            provider_request_id,
            provider_instance_ref: request.provider_instance_ref,
            provider_evidence_ref,
            instance_resource_id: request.instance.resource_id.value.value,
            tenant_id: request.tenant_id,
            region: request.instance.region.value.value,
            az: request.instance.az.value.value,
            cell_id: request.instance.cell_id.value.value,
            schema_version: COMPUTE_SCHEMA_VERSION,
        })
    }
}
