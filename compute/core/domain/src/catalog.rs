use std::collections::BTreeMap;

use compute_resource::ResourceId;

use crate::{
    CloudComputeError, FunctionDeployment, FunctionDeploymentCreate, FunctionDeploymentState,
    FunctionInvocationReceipt, FunctionInvocationRequest, Instance, InstanceCreate, InvocationId,
    KubernetesCluster, KubernetesClusterCreate, internal, map_resource_error, public,
};

const DEFAULT_FUNCTION_INVOCATION_RETENTION_LIMIT: usize = 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeCatalog {
    instances: BTreeMap<ResourceId, Instance>,
    kubernetes_clusters: BTreeMap<ResourceId, KubernetesCluster>,
    functions: BTreeMap<ResourceId, FunctionDeployment>,
    invocations: BTreeMap<InvocationId, FunctionInvocationReceipt>,
    invocation_retention_limit: usize,
}

impl Default for CloudComputeCatalog {
    fn default() -> Self {
        Self::with_invocation_retention_limit(DEFAULT_FUNCTION_INVOCATION_RETENTION_LIMIT)
    }
}

pub trait ComputeRepo {
    fn create_instance(&mut self, input: InstanceCreate) -> Result<Instance, CloudComputeError>;
    fn create_kubernetes_cluster(
        &mut self,
        input: KubernetesClusterCreate,
    ) -> Result<KubernetesCluster, CloudComputeError>;
    fn register_function(
        &mut self,
        input: FunctionDeploymentCreate,
    ) -> Result<FunctionDeployment, CloudComputeError>;
    fn activate_function(
        &mut self,
        id: &ResourceId,
    ) -> Result<FunctionDeployment, CloudComputeError>;
    fn invoke_function(
        &mut self,
        input: FunctionInvocationRequest,
    ) -> Result<FunctionInvocationReceipt, CloudComputeError>;
}
impl ComputeRepo for CloudComputeCatalog {
    fn create_instance(&mut self, input: InstanceCreate) -> Result<Instance, CloudComputeError> {
        let instance = Instance::new(input)?;
        if self.instances.contains_key(&instance.resource_id.value) {
            return Err(CloudComputeError::DuplicateInstance);
        }
        self.instances
            .insert(instance.resource_id.value.clone(), instance.clone());
        Ok(instance)
    }

    fn create_kubernetes_cluster(
        &mut self,
        input: KubernetesClusterCreate,
    ) -> Result<KubernetesCluster, CloudComputeError> {
        let cluster = KubernetesCluster::new(input)?;
        if self
            .kubernetes_clusters
            .contains_key(&cluster.resource_id.value)
        {
            return Err(CloudComputeError::DuplicateKubernetesCluster);
        }
        self.kubernetes_clusters
            .insert(cluster.resource_id.value.clone(), cluster.clone());
        Ok(cluster)
    }

    fn register_function(
        &mut self,
        input: FunctionDeploymentCreate,
    ) -> Result<FunctionDeployment, CloudComputeError> {
        let function = FunctionDeployment::new(input)?;
        if self.functions.contains_key(&function.resource_id.value) {
            return Err(CloudComputeError::DuplicateFunction);
        }
        self.functions
            .insert(function.resource_id.value.clone(), function.clone());
        Ok(function)
    }

    fn activate_function(
        &mut self,
        id: &ResourceId,
    ) -> Result<FunctionDeployment, CloudComputeError> {
        let function = self
            .functions
            .get_mut(id)
            .ok_or(CloudComputeError::UnknownFunction)?;
        if function.state.value != FunctionDeploymentState::Deploying {
            return Err(CloudComputeError::InvalidFunctionState);
        }
        function.state = public(FunctionDeploymentState::Active);
        Ok(function.clone())
    }

    fn invoke_function(
        &mut self,
        input: FunctionInvocationRequest,
    ) -> Result<FunctionInvocationReceipt, CloudComputeError> {
        let invocation_id = InvocationId::new(input.invocation_id.clone())?;
        if self.invocations.contains_key(&invocation_id) {
            return Err(CloudComputeError::DuplicateInvocation);
        }
        let function_id = ResourceId::new(input.function_id.clone()).map_err(map_resource_error)?;
        let function = self
            .functions
            .get(&function_id)
            .ok_or(CloudComputeError::UnknownFunction)?;
        let receipt = function.invoke(input)?;
        self.remember_invocation(invocation_id, receipt.clone());
        Ok(receipt)
    }
}

impl CloudComputeCatalog {
    pub fn with_invocation_retention_limit(invocation_retention_limit: usize) -> Self {
        Self {
            instances: BTreeMap::new(),
            kubernetes_clusters: BTreeMap::new(),
            functions: BTreeMap::new(),
            invocations: BTreeMap::new(),
            invocation_retention_limit: invocation_retention_limit.max(1),
        }
    }

    fn remember_invocation(
        &mut self,
        invocation_id: InvocationId,
        receipt: FunctionInvocationReceipt,
    ) {
        if self.invocations.len() >= self.invocation_retention_limit
            && let Some(evicted) = self.invocations.keys().next().cloned()
        {
            self.invocations.remove(&evicted);
        }
        self.invocations.insert(invocation_id, receipt);
    }

    pub fn instances(&self) -> impl Iterator<Item = &Instance> {
        self.instances.values()
    }

    pub fn kubernetes_clusters(&self) -> impl Iterator<Item = &KubernetesCluster> {
        self.kubernetes_clusters.values()
    }

    pub fn functions(&self) -> impl Iterator<Item = &FunctionDeployment> {
        self.functions.values()
    }

    pub fn invocations(&self) -> impl Iterator<Item = &FunctionInvocationReceipt> {
        self.invocations.values()
    }
}
