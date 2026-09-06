use crate::{Digest32, PlacementIdempotencyKey, PlacementOperationKey, TenantId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlacementIdempotencyRecordV1 {
    pub tenant_id: TenantId,
    pub idempotency_key: PlacementIdempotencyKey,
    pub request_digest: Digest32,
    pub operation: PlacementOperationKey,
    pub immutable_result_digest: Option<Digest32>,
}
