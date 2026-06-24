//! CRM gRPC adapter.
//!
//! ## AUTH-005 fail-closed seam (ADR-0603)
//!
//! [`GrpcRequest::tenant_id`] is a CALLER-SUPPLIED payload field — it is
//! **non-authoritative** (grants nothing, never selects the resource tenant).
//! Every mutating method MUST go through [`GrpcHandler::handle`], which runs the
//! [`crate::authz`] fail-closed gate FIRST (verified peer SVID/bearer → 401,
//! PDP authorize against the VERIFIED tenant → 403) before any business logic.
//!
//! ## Edge obligation (dead-until-edge)
//!
//! The business logic still returns `contract_stub`. The gRPC server that binds
//! a real listener MUST extract the verified peer credential from the mTLS/SVID
//! transport (NOT the payload), refuse to boot without a
//! [`crate::authz::CrmAuthzProvider`], and run the gate before dispatch.

use crate::authz::{authorize_crm_command, CallerCredential, CrmAction, CrmAuthzProvider};
use crate::domain::Capability;
use crate::error::{Result, ServiceError};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GrpcMethod { pub service: &'static str, pub method: &'static str, pub request: &'static str, pub response: &'static str }
/// gRPC request DTO. NOTE: `tenant_id` is non-authoritative caller-supplied
/// cross-check data (see module docs / ADR-0603).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GrpcRequest { pub tenant_id: String, pub method: String, pub payload_json: serde_json::Value }
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GrpcResponse { pub accepted: bool, pub payload_json: serde_json::Value }

pub struct GrpcHandler;
impl GrpcHandler {
    pub fn methods() -> Vec<GrpcMethod> {
        vec![
            GrpcMethod { service: "oyatie.crm.v1.CrmService", method: "SubmitCommand", request: "SubmitCommandRequest", response: "CommandReceipt" },
            GrpcMethod { service: "oyatie.crm.v1.CrmService", method: "Reconcile", request: "ReconcileRequest", response: "CommandReceipt" },
            GrpcMethod { service: "oyatie.crm.v1.CrmService", method: "ApplyGovernanceHold", request: "ApplyGovernanceHoldRequest", response: "CommandReceipt" },
            GrpcMethod { service: "oyatie.crm.v1.CrmService", method: "ExportEvidence", request: "ExportEvidenceRequest", response: "CommandReceipt" },
        ]
    }

    /// Handle a mutating CRM gRPC call through the fail-closed authz gate.
    ///
    /// `credential` is the verified peer credential from the transport (mTLS
    /// SVID / bearer), NOT the payload. `capability` is server-side method
    /// metadata, NOT the payload. The gate verifies the caller and authorizes
    /// the action against the VERIFIED tenant before any business logic.
    ///
    /// # Errors
    /// `Authorization` on a failed gate; `ContractStub` once authorized.
    pub fn handle(provider: &CrmAuthzProvider, credential: &CallerCredential, capability: Capability, _request: GrpcRequest) -> Result<GrpcResponse> {
        let _principal = authorize_crm_command(provider, credential, CrmAction(capability))
            .map_err(ServiceError::from)?;
        Err(ServiceError::contract_stub("grpc"))
    }
}

pub fn validate_methods(methods: &[GrpcMethod]) -> Result<()> {
    if methods.len() < 4 { return Err(ServiceError::validation("grpc_methods", "scaffold requires command and read gRPC methods")); }
    Ok(())
}
