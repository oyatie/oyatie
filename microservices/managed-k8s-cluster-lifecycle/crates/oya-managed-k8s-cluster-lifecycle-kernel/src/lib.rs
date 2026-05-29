#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use core::fmt;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum DesiredTier {
    #[default]
    Hosted,
    Dedicated,
}

impl DesiredTier {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Hosted => "hosted",
            Self::Dedicated => "dedicated",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "hosted" | "hosted_kamaji" => Some(Self::Hosted),
            "dedicated" | "dedicated_talos_spoke" => Some(Self::Dedicated),
            _ => None,
        }
    }
}

impl fmt::Display for DesiredTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClusterResourceRequest {
    pub nodes: u32,   // data_class: TENANT_SCOPED
    pub vcpu: u32,    // data_class: TENANT_SCOPED
    pub ram_gib: u32, // data_class: TENANT_SCOPED
}

impl ClusterResourceRequest {
    #[must_use]
    pub const fn new(nodes: u32, vcpu: u32, ram_gib: u32) -> Self {
        Self {
            nodes,
            vcpu,
            ram_gib,
        }
    }

    #[must_use]
    pub const fn default_small() -> Self {
        Self {
            nodes: 3,
            vcpu: 8,
            ram_gib: 32,
        }
    }

    fn validate(&self) -> Result<(), LifecycleValidationError> {
        if self.nodes == 0 {
            return Err(LifecycleValidationError::ZeroResource("nodes"));
        }
        if self.vcpu == 0 {
            return Err(LifecycleValidationError::ZeroResource("vcpu"));
        }
        if self.ram_gib == 0 {
            return Err(LifecycleValidationError::ZeroResource("ram_gib"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LifecycleRequest {
    pub tenant_id: String,                 // data_class: TENANT_SCOPED
    pub cluster_name: String,              // data_class: TENANT_SCOPED
    pub desired_tier: DesiredTier,         // data_class: TENANT_SCOPED
    pub resources: ClusterResourceRequest, // data_class: TENANT_SCOPED
}

impl LifecycleRequest {
    pub fn new(
        tenant_id: impl Into<String>,
        cluster_name: impl Into<String>,
        desired_tier: DesiredTier,
        resources: ClusterResourceRequest,
    ) -> Result<Self, LifecycleValidationError> {
        let request = Self {
            tenant_id: tenant_id.into(),
            cluster_name: cluster_name.into(),
            desired_tier,
            resources,
        };
        request.validate()?;
        Ok(request)
    }

    pub fn validate(&self) -> Result<(), LifecycleValidationError> {
        if self.tenant_id.trim().is_empty() {
            return Err(LifecycleValidationError::EmptyTenantId);
        }
        if self.cluster_name.trim().is_empty() {
            return Err(LifecycleValidationError::EmptyClusterName);
        }
        self.resources.validate()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LifecycleValidationError {
    EmptyTenantId,
    EmptyClusterName,
    ZeroResource(&'static str),
    ZeroTargetNodeCount,
    TargetNodeCountExceedsFloor,
}

impl fmt::Display for LifecycleValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTenantId => f.write_str("tenant_id must not be empty"),
            Self::EmptyClusterName => f.write_str("cluster_name must not be empty"),
            Self::ZeroResource(field) => write!(f, "resource field {field} must be > 0"),
            Self::ZeroTargetNodeCount => f.write_str("target_node_count must be > 0"),
            Self::TargetNodeCountExceedsFloor => write!(
                f,
                "target_node_count must not exceed the ceiling of {NODE_COUNT_CEILING}"
            ),
        }
    }
}

impl std::error::Error for LifecycleValidationError {}

// ---------------------------------------------------------------------------
// node-pool op surface
// ---------------------------------------------------------------------------

/// Hard ceiling on node count for any pool operation.
pub const NODE_COUNT_CEILING: u32 = 500;

/// Minimum node count for a Hosted-tier pool.
pub const HOSTED_NODE_FLOOR: u32 = 1;

/// Minimum node count for a Dedicated-tier pool.
pub const DEDICATED_NODE_FLOOR: u32 = 3;

/// Action requested for a node-pool operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodePoolAction {
    ScaleUp,
    ScaleDown,
    Cordon,
    Drain,
}

/// A tenant's request to operate on a node pool in their cluster.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodePoolOpRequest {
    pub tenant_id: String,      // data_class: TENANT_SCOPED
    pub cluster_name: String,   // data_class: TENANT_SCOPED
    pub target_node_count: u32, // data_class: TENANT_SCOPED
    pub action: NodePoolAction, // data_class: TENANT_SCOPED
}

impl NodePoolOpRequest {
    pub fn new(
        tenant_id: impl Into<String>,
        cluster_name: impl Into<String>,
        target_node_count: u32,
        action: NodePoolAction,
    ) -> Result<Self, LifecycleValidationError> {
        let request = Self {
            tenant_id: tenant_id.into(),
            cluster_name: cluster_name.into(),
            target_node_count,
            action,
        };
        request.validate()?;
        Ok(request)
    }

    pub fn validate(&self) -> Result<(), LifecycleValidationError> {
        if self.tenant_id.trim().is_empty() {
            return Err(LifecycleValidationError::EmptyTenantId);
        }
        if self.cluster_name.trim().is_empty() {
            return Err(LifecycleValidationError::EmptyClusterName);
        }
        if self.target_node_count == 0 {
            return Err(LifecycleValidationError::ZeroTargetNodeCount);
        }
        if self.target_node_count > NODE_COUNT_CEILING {
            return Err(LifecycleValidationError::TargetNodeCountExceedsFloor);
        }
        Ok(())
    }
}

/// Outcome of a drain admission evaluation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DrainAdmission {
    Allow,
    Deny { reason: String },
}

/// Pure, deterministic drain admission check.
///
/// Denies if:
/// - `drain_target` is zero (would empty the pool)
/// - `drain_target` >= `current_nodes` (no nodes would remain)
/// - remaining nodes after drain would fall below the tier floor
///
/// Allows otherwise.
#[must_use]
pub fn evaluate_drain_admission(
    current_nodes: u32,
    drain_target: u32,
    desired_tier: DesiredTier,
) -> DrainAdmission {
    if drain_target == 0 {
        return DrainAdmission::Deny {
            reason: "drain_target must be > 0; draining to zero is not permitted".into(),
        };
    }
    if drain_target >= current_nodes {
        return DrainAdmission::Deny {
            reason: format!(
                "drain_target ({drain_target}) must be < current_nodes ({current_nodes}); \
                 at least one node must remain"
            ),
        };
    }
    let remaining = current_nodes - drain_target;
    let floor = match desired_tier {
        DesiredTier::Hosted => HOSTED_NODE_FLOOR,
        DesiredTier::Dedicated => DEDICATED_NODE_FLOOR,
    };
    if remaining < floor {
        return DrainAdmission::Deny {
            reason: format!(
                "drain would leave {remaining} node(s), below the {tier} floor of {floor}",
                tier = desired_tier.as_str()
            ),
        };
    }
    DrainAdmission::Allow
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_validates_identity_and_resources() {
        assert!(
            LifecycleRequest::new(
                "ten_zero",
                "dogfood-a",
                DesiredTier::Hosted,
                ClusterResourceRequest::default_small()
            )
            .is_ok()
        );
        assert!(matches!(
            LifecycleRequest::new(
                "",
                "dogfood-a",
                DesiredTier::Hosted,
                ClusterResourceRequest::default_small()
            ),
            Err(LifecycleValidationError::EmptyTenantId)
        ));
        assert!(matches!(
            LifecycleRequest::new(
                "ten_zero",
                " ",
                DesiredTier::Hosted,
                ClusterResourceRequest::default_small()
            ),
            Err(LifecycleValidationError::EmptyClusterName)
        ));
        assert!(matches!(
            LifecycleRequest::new(
                "ten_zero",
                "dogfood-a",
                DesiredTier::Hosted,
                ClusterResourceRequest::new(0, 8, 32)
            ),
            Err(LifecycleValidationError::ZeroResource("nodes"))
        ));
    }

    #[test]
    fn tier_parse_is_fail_closed() {
        assert_eq!(DesiredTier::parse("hosted"), Some(DesiredTier::Hosted));
        assert_eq!(
            DesiredTier::parse("dedicated_talos_spoke"),
            Some(DesiredTier::Dedicated)
        );
        assert_eq!(DesiredTier::parse("unknown"), None);
    }

    // -----------------------------------------------------------------------
    // [np-1] NodePoolOpRequest + validate() — RED tests
    // -----------------------------------------------------------------------

    #[test]
    fn nodepool_op_request_validates_happy_path() {
        // All four actions must succeed with valid identity and count.
        for action in [
            NodePoolAction::ScaleUp,
            NodePoolAction::ScaleDown,
            NodePoolAction::Cordon,
            NodePoolAction::Drain,
        ] {
            let result = NodePoolOpRequest::new("ten_alpha", "dogfood-a", 3, action);
            assert!(
                result.is_ok(),
                "expected Ok for action {action:?}, got {result:?}"
            );
            let req = result.unwrap();
            assert_eq!(req.tenant_id, "ten_alpha");
            assert_eq!(req.cluster_name, "dogfood-a");
            assert_eq!(req.target_node_count, 3);
            assert_eq!(req.action, action);
        }
    }

    #[test]
    fn nodepool_op_request_rejects_empty_tenant_id() {
        assert!(matches!(
            NodePoolOpRequest::new("", "dogfood-a", 3, NodePoolAction::ScaleUp),
            Err(LifecycleValidationError::EmptyTenantId)
        ));
        // whitespace-only
        assert!(matches!(
            NodePoolOpRequest::new("  ", "dogfood-a", 3, NodePoolAction::Drain),
            Err(LifecycleValidationError::EmptyTenantId)
        ));
    }

    #[test]
    fn nodepool_op_request_rejects_empty_cluster_name() {
        assert!(matches!(
            NodePoolOpRequest::new("ten_alpha", "", 3, NodePoolAction::ScaleUp),
            Err(LifecycleValidationError::EmptyClusterName)
        ));
        assert!(matches!(
            NodePoolOpRequest::new("ten_alpha", "\t", 3, NodePoolAction::Cordon),
            Err(LifecycleValidationError::EmptyClusterName)
        ));
    }

    #[test]
    fn nodepool_op_request_rejects_zero_target() {
        assert!(matches!(
            NodePoolOpRequest::new("ten_alpha", "dogfood-a", 0, NodePoolAction::ScaleDown),
            Err(LifecycleValidationError::ZeroTargetNodeCount)
        ));
    }

    #[test]
    fn nodepool_op_request_rejects_over_ceiling() {
        assert!(matches!(
            NodePoolOpRequest::new(
                "ten_alpha",
                "dogfood-a",
                NODE_COUNT_CEILING + 1,
                NodePoolAction::ScaleUp
            ),
            Err(LifecycleValidationError::TargetNodeCountExceedsFloor)
        ));
        // exactly at ceiling must be accepted
        assert!(
            NodePoolOpRequest::new(
                "ten_alpha",
                "dogfood-a",
                NODE_COUNT_CEILING,
                NodePoolAction::ScaleUp
            )
            .is_ok()
        );
    }

    // -----------------------------------------------------------------------
    // [np-1] NodePoolAction serde round-trip — RED tests
    // -----------------------------------------------------------------------

    #[test]
    fn nodepool_action_serde_roundtrip() {
        let cases = [
            (NodePoolAction::ScaleUp, "\"scale_up\""),
            (NodePoolAction::ScaleDown, "\"scale_down\""),
            (NodePoolAction::Cordon, "\"cordon\""),
            (NodePoolAction::Drain, "\"drain\""),
        ];
        for (action, expected_json) in cases {
            let serialized = serde_json::to_string(&action)
                .unwrap_or_else(|e| panic!("serialize {action:?} failed: {e}"));
            assert_eq!(serialized, expected_json, "JSON for {action:?} mismatch");
            let roundtripped: NodePoolAction = serde_json::from_str(&serialized)
                .unwrap_or_else(|e| panic!("deserialize {action:?} failed: {e}"));
            assert_eq!(roundtripped, action, "round-trip mismatch for {action:?}");
        }
    }

    // -----------------------------------------------------------------------
    // [np-2] evaluate_drain_admission — RED tests
    // -----------------------------------------------------------------------

    #[test]
    fn drain_admission_denies_drain_target_zero() {
        // drain_target == 0 is always denied
        let result = evaluate_drain_admission(5, 0, DesiredTier::Hosted);
        assert!(
            matches!(result, DrainAdmission::Deny { .. }),
            "expected Deny, got {result:?}"
        );
    }

    #[test]
    fn drain_admission_denies_when_drain_target_equals_current_nodes() {
        // drain_target >= current_nodes → Deny
        let result = evaluate_drain_admission(5, 5, DesiredTier::Dedicated);
        assert!(matches!(result, DrainAdmission::Deny { .. }));
    }

    #[test]
    fn drain_admission_denies_when_drain_target_exceeds_current_nodes() {
        let result = evaluate_drain_admission(3, 4, DesiredTier::Hosted);
        assert!(matches!(result, DrainAdmission::Deny { .. }));
    }

    #[test]
    fn drain_admission_denies_below_dedicated_floor() {
        // Dedicated: floor = 3. current=4, drain=2 → remaining=2 < 3 → Deny
        let result = evaluate_drain_admission(4, 2, DesiredTier::Dedicated);
        assert!(matches!(result, DrainAdmission::Deny { .. }));
    }

    #[test]
    fn drain_admission_denies_below_hosted_floor() {
        // Hosted: floor = 1. current=2, drain=2 → drain_target==current → Deny (zero path)
        let result = evaluate_drain_admission(2, 2, DesiredTier::Hosted);
        assert!(matches!(result, DrainAdmission::Deny { .. }));
    }

    #[test]
    fn drain_admission_allows_safe_hosted_drain() {
        // Hosted: floor = 1. current=5, drain=2 → remaining=3 >= 1 → Allow
        let result = evaluate_drain_admission(5, 2, DesiredTier::Hosted);
        assert!(
            matches!(result, DrainAdmission::Allow),
            "expected Allow, got {result:?}"
        );
    }

    #[test]
    fn drain_admission_allows_safe_dedicated_drain() {
        // Dedicated: floor = 3. current=6, drain=2 → remaining=4 >= 3 → Allow
        let result = evaluate_drain_admission(6, 2, DesiredTier::Dedicated);
        assert!(
            matches!(result, DrainAdmission::Allow),
            "expected Allow, got {result:?}"
        );
    }

    #[test]
    fn drain_admission_deterministic() {
        // Same inputs called twice must return the same variant.
        let a = evaluate_drain_admission(6, 2, DesiredTier::Dedicated);
        let b = evaluate_drain_admission(6, 2, DesiredTier::Dedicated);
        assert_eq!(a, b);
    }

    #[test]
    fn drain_admission_deny_reason_is_non_empty() {
        // Deny variants must carry a meaningful reason string.
        match evaluate_drain_admission(4, 2, DesiredTier::Dedicated) {
            DrainAdmission::Deny { reason } => {
                assert!(!reason.is_empty(), "Deny reason must not be empty");
            }
            DrainAdmission::Allow => panic!("expected Deny, got Allow"),
        }
    }
}
