use compute_resource::InstanceFlavor;

use crate::CloudComputeError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ComputeFlavorSpec {
    pub class: InstanceFlavor, // data_class: PUBLIC
    pub vcpu: u32,             // data_class: PUBLIC
    pub memory_gb: u32,        // data_class: PUBLIC
    pub gpu_count: u32,        // data_class: PUBLIC
    pub local_ssd_gb: u32,     // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ComputeQuotaEnvelope {
    pub vcpu_limit: u32,           // data_class: INTERNAL_ONLY
    pub memory_gb_limit: u32,      // data_class: INTERNAL_ONLY
    pub gpu_limit: u32,            // data_class: INTERNAL_ONLY
    pub local_ssd_gb_limit: u32,   // data_class: INTERNAL_ONLY
    pub current_vcpu: u32,         // data_class: INTERNAL_ONLY
    pub current_memory_gb: u32,    // data_class: INTERNAL_ONLY
    pub current_gpu: u32,          // data_class: INTERNAL_ONLY
    pub current_local_ssd_gb: u32, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ComputeUnits {
    vcpu: u32,         // data_class: INTERNAL_ONLY
    memory_gb: u32,    // data_class: INTERNAL_ONLY
    gpu_count: u32,    // data_class: INTERNAL_ONLY
    local_ssd_gb: u32, // data_class: INTERNAL_ONLY
}

impl ComputeFlavorSpec {
    pub fn validate(self) -> Result<Self, CloudComputeError> {
        if self.vcpu == 0
            || self.vcpu > 512
            || self.memory_gb == 0
            || self.memory_gb > 8192
            || self.memory_gb < self.vcpu
            || self.local_ssd_gb > 262_144
        {
            return Err(CloudComputeError::InvalidFlavor);
        }
        if matches!(self.class, InstanceFlavor::Gpu) != (self.gpu_count > 0) {
            return Err(CloudComputeError::InvalidFlavor);
        }
        Ok(self)
    }

    pub(crate) fn units(self) -> ComputeUnits {
        ComputeUnits {
            vcpu: self.vcpu,
            memory_gb: self.memory_gb,
            gpu_count: self.gpu_count,
            local_ssd_gb: self.local_ssd_gb,
        }
    }
}

impl ComputeQuotaEnvelope {
    pub(crate) fn admit(self, requested: ComputeUnits) -> Result<(), CloudComputeError> {
        if self.current_vcpu > self.vcpu_limit
            || self.current_memory_gb > self.memory_gb_limit
            || self.current_gpu > self.gpu_limit
            || self.current_local_ssd_gb > self.local_ssd_gb_limit
        {
            return Err(CloudComputeError::InvalidQuota);
        }
        let next_vcpu = self
            .current_vcpu
            .checked_add(requested.vcpu)
            .ok_or(CloudComputeError::QuotaExceeded)?;
        let next_memory = self
            .current_memory_gb
            .checked_add(requested.memory_gb)
            .ok_or(CloudComputeError::QuotaExceeded)?;
        let next_gpu = self
            .current_gpu
            .checked_add(requested.gpu_count)
            .ok_or(CloudComputeError::QuotaExceeded)?;
        let next_ssd = self
            .current_local_ssd_gb
            .checked_add(requested.local_ssd_gb)
            .ok_or(CloudComputeError::QuotaExceeded)?;
        if next_vcpu > self.vcpu_limit
            || next_memory > self.memory_gb_limit
            || next_gpu > self.gpu_limit
            || next_ssd > self.local_ssd_gb_limit
        {
            return Err(CloudComputeError::QuotaExceeded);
        }
        Ok(())
    }
}

impl ComputeUnits {
    pub(crate) fn checked_add(self, other: Self) -> Result<Self, CloudComputeError> {
        Ok(Self {
            vcpu: self
                .vcpu
                .checked_add(other.vcpu)
                .ok_or(CloudComputeError::QuotaExceeded)?,
            memory_gb: self
                .memory_gb
                .checked_add(other.memory_gb)
                .ok_or(CloudComputeError::QuotaExceeded)?,
            gpu_count: self
                .gpu_count
                .checked_add(other.gpu_count)
                .ok_or(CloudComputeError::QuotaExceeded)?,
            local_ssd_gb: self
                .local_ssd_gb
                .checked_add(other.local_ssd_gb)
                .ok_or(CloudComputeError::QuotaExceeded)?,
        })
    }

    pub(crate) fn checked_mul(self, count: u32) -> Result<Self, CloudComputeError> {
        Ok(Self {
            vcpu: self
                .vcpu
                .checked_mul(count)
                .ok_or(CloudComputeError::QuotaExceeded)?,
            memory_gb: self
                .memory_gb
                .checked_mul(count)
                .ok_or(CloudComputeError::QuotaExceeded)?,
            gpu_count: self
                .gpu_count
                .checked_mul(count)
                .ok_or(CloudComputeError::QuotaExceeded)?,
            local_ssd_gb: self
                .local_ssd_gb
                .checked_mul(count)
                .ok_or(CloudComputeError::QuotaExceeded)?,
        })
    }
}

pub const fn instance_flavor_label(flavor: InstanceFlavor) -> &'static str {
    match flavor {
        InstanceFlavor::GeneralPurpose => "general_purpose",
        InstanceFlavor::ComputeOptimized => "compute_optimized",
        InstanceFlavor::MemoryOptimized => "memory_optimized",
        InstanceFlavor::Gpu => "gpu",
    }
}
