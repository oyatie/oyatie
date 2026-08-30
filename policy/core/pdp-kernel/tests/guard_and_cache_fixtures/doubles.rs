//! Test PDP implementations: slow, panicking, refusing, and recovering.

#![allow(dead_code)]
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use policy_pdp_kernel::*;
use shared_platform_contracts_kernel::pdp::*;

use super::allow_outcome;

#[derive(Debug)]
pub struct SlowSideEffectPdp {
    pub calls: Arc<AtomicU32>,
    pub active_calls: Arc<AtomicU32>,
    pub max_active_calls: Arc<AtomicU32>,
    pub side_effects: Arc<AtomicU32>,
    pub delay: Duration,
}

impl SlowSideEffectPdp {
    pub fn new(
        delay: Duration,
    ) -> (
        Self,
        Arc<AtomicU32>,
        Arc<AtomicU32>,
        Arc<AtomicU32>,
        Arc<AtomicU32>,
    ) {
        let calls = Arc::new(AtomicU32::new(0));
        let active_calls = Arc::new(AtomicU32::new(0));
        let max_active_calls = Arc::new(AtomicU32::new(0));
        let side_effects = Arc::new(AtomicU32::new(0));
        (
            Self {
                calls: calls.clone(),
                active_calls: active_calls.clone(),
                max_active_calls: max_active_calls.clone(),
                side_effects: side_effects.clone(),
                delay,
            },
            calls,
            active_calls,
            max_active_calls,
            side_effects,
        )
    }
}

impl PolicyDecisionPoint for SlowSideEffectPdp {
    fn authorize(
        &self,
        request: &AuthorizationRequest,
        _entities: &EntitySlice,
    ) -> Result<PdpOutcome, PdpError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let active = self.active_calls.fetch_add(1, Ordering::SeqCst) + 1;
        let mut observed = self.max_active_calls.load(Ordering::SeqCst);
        while active > observed {
            match self.max_active_calls.compare_exchange(
                observed,
                active,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => break,
                Err(current) => observed = current,
            }
        }

        std::thread::sleep(self.delay);
        self.side_effects.fetch_add(1, Ordering::SeqCst);
        self.active_calls.fetch_sub(1, Ordering::SeqCst);

        let version = self.loaded_policy_version();
        let response = AuthorizationResponse {
            decision_id: "dec-runtime-1".to_owned(),
            request_id: request.request_id.clone(),
            decision: Decision::Allow,
            policy_version: version.clone(),
            determining_policy_ids: vec!["permit-admin".to_owned()],
            obligations: vec![],
        };
        let audit = DecisionAuditRecord {
            decision_id: response.decision_id.clone(),
            request_id: response.request_id.clone(),
            tenant_id: request.tenant_id.clone(),
            principal: request.principal.clone(),
            action: request.action.clone(),
            resource: request.resource.clone(),
            decision: response.decision,
            policy_version: version,
            determining_policy_ids: response.determining_policy_ids.clone(),
            cache_hit: false,
        };
        Ok(PdpOutcome {
            response,
            audit,
            cache_hit: false,
        })
    }

    fn loaded_policy_version(&self) -> PolicyVersion {
        PolicyVersion::new("psv-runtime").unwrap()
    }
}

#[derive(Debug)]
pub struct PanicPdp;

impl PolicyDecisionPoint for PanicPdp {
    fn authorize(
        &self,
        _request: &AuthorizationRequest,
        _entities: &EntitySlice,
    ) -> Result<PdpOutcome, PdpError> {
        panic!("pdp runtime bug");
    }

    fn loaded_policy_version(&self) -> PolicyVersion {
        PolicyVersion::new("psv-runtime").unwrap()
    }
}

#[derive(Debug)]
pub struct SlowRefusalPdp {
    pub delay: Duration,
}

impl PolicyDecisionPoint for SlowRefusalPdp {
    fn authorize(
        &self,
        _request: &AuthorizationRequest,
        _entities: &EntitySlice,
    ) -> Result<PdpOutcome, PdpError> {
        std::thread::sleep(self.delay);
        Err(PdpError::UnknownAction {
            action: "resource.retired".to_owned(),
        })
    }

    fn loaded_policy_version(&self) -> PolicyVersion {
        PolicyVersion::new("psv-runtime").unwrap()
    }
}

#[derive(Debug)]
pub struct FastEvaluationPdp;

impl PolicyDecisionPoint for FastEvaluationPdp {
    fn authorize(
        &self,
        _request: &AuthorizationRequest,
        _entities: &EntitySlice,
    ) -> Result<PdpOutcome, PdpError> {
        Err(PdpError::Evaluation {
            detail: "caller-shaped entity slice refusal".to_owned(),
        })
    }

    fn loaded_policy_version(&self) -> PolicyVersion {
        PolicyVersion::new("psv-runtime").unwrap()
    }
}

#[derive(Debug)]
pub struct PanicOnceThenAllowPdp {
    pub calls: Arc<AtomicU32>,
}

impl PolicyDecisionPoint for PanicOnceThenAllowPdp {
    fn authorize(
        &self,
        request: &AuthorizationRequest,
        _entities: &EntitySlice,
    ) -> Result<PdpOutcome, PdpError> {
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        if call == 0 {
            panic!("transient pdp runtime bug");
        }
        Ok(allow_outcome(request, self.loaded_policy_version()))
    }

    fn loaded_policy_version(&self) -> PolicyVersion {
        PolicyVersion::new("psv-runtime").unwrap()
    }
}

#[derive(Debug)]
pub struct SlowFastSlowPdp {
    pub calls: Arc<AtomicU32>,
    pub slow: Duration,
}

impl PolicyDecisionPoint for SlowFastSlowPdp {
    fn authorize(
        &self,
        request: &AuthorizationRequest,
        _entities: &EntitySlice,
    ) -> Result<PdpOutcome, PdpError> {
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        if call == 0 || call == 2 {
            std::thread::sleep(self.slow);
        }
        Ok(allow_outcome(request, self.loaded_policy_version()))
    }

    fn loaded_policy_version(&self) -> PolicyVersion {
        PolicyVersion::new("psv-runtime").unwrap()
    }
}
