use std::sync::{Arc, Mutex};

use foundry_ontology_app::PolicyEnforcementPoint;
use policy_pdp_kernel::{EntitySlice, PdpError, PdpOutcome, PolicyDecisionPoint};
use shared_platform_contracts_kernel::pdp::{AuthorizationRequest, Decision, PolicyVersion};

pub struct EndpointPdp {
    pub denied: Mutex<Option<&'static str>>,
    pub unavailable: Mutex<Option<&'static str>>,
    pub resources: Mutex<Vec<String>>, // data_class: INTERNAL_ONLY
    inner: Arc<dyn PolicyDecisionPoint>,
}

impl EndpointPdp {
    pub fn new(version: &str) -> Arc<Self> {
        Arc::new(Self {
            denied: Mutex::new(None),
            unavailable: Mutex::new(None),
            resources: Mutex::new(Vec::new()),
            inner: PolicyEnforcementPoint::cedar(version).unwrap(),
        })
    }
}

impl PolicyDecisionPoint for EndpointPdp {
    fn authorize(
        &self,
        request: &AuthorizationRequest,
        entities: &EntitySlice,
    ) -> Result<PdpOutcome, PdpError> {
        assert_eq!(request.principal.entity_type, "Principal");
        assert_eq!(request.principal.entity_id, "prn_alice");
        assert_eq!(request.resource.entity_type, "OntologyObject");
        assert_eq!(request.action, "foundry.ontology.use");
        self.resources
            .lock()
            .unwrap()
            .push(request.resource.entity_id.clone());
        if *self.unavailable.lock().unwrap() == Some(request.resource.entity_id.as_str()) {
            return Err(PdpError::Evaluation {
                detail: "injected endpoint policy failure".into(),
            });
        }
        let mut result = self.inner.authorize(request, entities)?;
        if *self.denied.lock().unwrap() == Some(request.resource.entity_id.as_str()) {
            result.response.decision = Decision::Deny;
            result.audit.decision = Decision::Deny;
        }
        Ok(result)
    }

    fn loaded_policy_version(&self) -> PolicyVersion {
        self.inner.loaded_policy_version()
    }
}
