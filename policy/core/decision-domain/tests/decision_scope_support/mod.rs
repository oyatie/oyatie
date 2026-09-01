use std::sync::atomic::{AtomicBool, Ordering};

use policy_decision_domain::PolicyDecisionPoint;
use policy_pdp_kernel::{EntitySlice, PdpError, PdpOutcome};
use shared_platform_contracts_kernel::pdp::{AuthorizationRequest, PolicyVersion};

pub struct MustNotDecide(AtomicBool);

impl MustNotDecide {
    pub fn new() -> Self {
        Self(AtomicBool::new(false))
    }

    pub fn was_consulted(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

impl PolicyDecisionPoint for MustNotDecide {
    fn authorize(
        &self,
        _request: &AuthorizationRequest,
        _entities: &EntitySlice,
    ) -> Result<PdpOutcome, PdpError> {
        self.0.store(true, Ordering::SeqCst);
        Err(PdpError::Evaluation {
            detail: "Cedar must not be consulted".to_owned(),
        })
    }

    fn loaded_policy_version(&self) -> PolicyVersion {
        PolicyVersion::new("psv-consistency-1").expect("policy version is valid")
    }
}
