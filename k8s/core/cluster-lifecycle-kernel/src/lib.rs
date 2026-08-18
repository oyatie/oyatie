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

// ---------------------------------------------------------------------------
// cluster-level provisioning state machine (ADR-0376)
// ---------------------------------------------------------------------------

/// The lifecycle phase a tenant cluster moves through from creation request
/// to deletion. Legal transitions are enforced by
/// [`ClusterLifecycleState::can_transition_to`] /
/// [`ClusterLifecycleState::transition`].
///
/// ## Transition graph
/// ```text
/// Requested   -> Provisioning | Failed
/// Provisioning-> Ready        | Failed
/// Ready       -> Updating     | Draining | Failed
/// Updating    -> Ready        | Failed
/// Draining    -> Deleted      | Failed
/// Deleted     -> (terminal)
/// Failed      -> (terminal)
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterLifecycleState {
    /// Cluster creation accepted; no infrastructure allocated yet.
    Requested,
    /// Infrastructure is being allocated and configured.
    Provisioning,
    /// Cluster is healthy and serving tenant workloads.
    Ready,
    /// In-place upgrade or config change in flight (sub-phase of serving).
    Updating,
    /// Cluster is being drained ahead of deletion.
    Draining,
    /// Terminal-success: cluster has been torn down.
    Deleted,
    /// Fault-terminal: unrecoverable failure; reachable from any non-terminal state.
    Failed,
}

impl ClusterLifecycleState {
    /// The initial state for a freshly-accepted cluster creation request.
    #[must_use]
    pub const fn initial() -> Self {
        Self::Requested
    }

    /// True for `Deleted` and `Failed` (no outgoing transitions).
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Deleted | Self::Failed)
    }

    /// True when the cluster is serving tenant workloads: `Ready` or `Updating`.
    #[must_use]
    pub const fn is_serving(&self) -> bool {
        matches!(self, Self::Ready | Self::Updating)
    }

    /// Stable wire/log slug (snake_case).
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Provisioning => "provisioning",
            Self::Ready => "ready",
            Self::Updating => "updating",
            Self::Draining => "draining",
            Self::Deleted => "deleted",
            Self::Failed => "failed",
        }
    }

    /// Parse from wire slug. Returns `None` for unknown values (fail-closed; no panic).
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "requested" => Some(Self::Requested),
            "provisioning" => Some(Self::Provisioning),
            "ready" => Some(Self::Ready),
            "updating" => Some(Self::Updating),
            "draining" => Some(Self::Draining),
            "deleted" => Some(Self::Deleted),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }

    /// Pure predicate: is `next` a legal successor of `self`?
    ///
    /// `Failed` is reachable from any non-terminal state. Terminal states
    /// (`Deleted`, `Failed`) have no outgoing transitions.
    #[must_use]
    pub const fn can_transition_to(&self, next: Self) -> bool {
        // Any non-terminal state may fault to Failed.
        if matches!(next, Self::Failed) {
            return !self.is_terminal();
        }
        matches!(
            (self, next),
            (Self::Requested, Self::Provisioning)
                | (Self::Provisioning, Self::Ready)
                | (Self::Ready, Self::Updating)
                | (Self::Ready, Self::Draining)
                | (Self::Updating, Self::Ready)
                | (Self::Draining, Self::Deleted)
        )
    }

    /// Attempt the transition, returning the new state or a typed error.
    ///
    /// Never panics; callers fail closed on [`Err(IllegalClusterTransition)`].
    ///
    /// # Errors
    /// Returns [`IllegalClusterTransition`] when `next` is not a legal
    /// successor of `self` (including any outgoing move from a terminal state).
    pub fn transition(self, next: Self) -> Result<Self, IllegalClusterTransition> {
        if self.can_transition_to(next) {
            Ok(next)
        } else {
            Err(IllegalClusterTransition {
                from: self,
                to: next,
            })
        }
    }
}

impl fmt::Display for ClusterLifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An attempted cluster lifecycle transition that the state machine forbids.
/// Carries the offending `from`/`to` pair for fail-closed diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IllegalClusterTransition {
    /// The state the cluster was in.
    pub from: ClusterLifecycleState, // data_class: INTERNAL_ONLY
    /// The illegal target state.
    pub to: ClusterLifecycleState, // data_class: INTERNAL_ONLY
}

impl fmt::Display for IllegalClusterTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "illegal cluster lifecycle transition: {} -> {}",
            self.from.as_str(),
            self.to.as_str()
        )
    }
}

impl std::error::Error for IllegalClusterTransition {}

/// Validate that a cluster satisfies the tier-specific node-floor requirement
/// before transitioning to [`ClusterLifecycleState::Ready`].
///
/// - `Dedicated` tier: `node_count` must be >= [`DEDICATED_NODE_FLOOR`].
/// - `Hosted` tier: no additional floor constraint (resource-request validation
///   already checked at admission time).
///
/// This is a pure function; callers invoke it before calling
/// `state.transition(ClusterLifecycleState::Ready)` for dedicated-tier clusters.
/// The state machine itself does not call this.
///
/// # Errors
/// - [`LifecycleValidationError::ZeroTargetNodeCount`] when `node_count == 0`.
/// - [`LifecycleValidationError::TargetNodeCountExceedsFloor`] when `node_count < DEDICATED_NODE_FLOOR`
///   and `tier == DesiredTier::Dedicated`.
pub fn validate_dedicated_readiness(
    node_count: u32,
    tier: DesiredTier,
) -> Result<(), LifecycleValidationError> {
    if node_count == 0 {
        return Err(LifecycleValidationError::ZeroTargetNodeCount);
    }
    if matches!(tier, DesiredTier::Dedicated) && node_count < DEDICATED_NODE_FLOOR {
        return Err(LifecycleValidationError::TargetNodeCountExceedsFloor);
    }
    Ok(())
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

    // -----------------------------------------------------------------------
    // [cls-1] initial() returns Requested
    // -----------------------------------------------------------------------

    #[test]
    fn cluster_lifecycle_initial_is_requested() {
        assert_eq!(
            ClusterLifecycleState::initial(),
            ClusterLifecycleState::Requested
        );
    }

    // -----------------------------------------------------------------------
    // [cls-2] is_terminal()
    // -----------------------------------------------------------------------

    #[test]
    fn cluster_lifecycle_is_terminal_only_for_deleted_and_failed() {
        assert!(ClusterLifecycleState::Deleted.is_terminal());
        assert!(ClusterLifecycleState::Failed.is_terminal());
        for non_terminal in [
            ClusterLifecycleState::Requested,
            ClusterLifecycleState::Provisioning,
            ClusterLifecycleState::Ready,
            ClusterLifecycleState::Updating,
            ClusterLifecycleState::Draining,
        ] {
            assert!(
                !non_terminal.is_terminal(),
                "{non_terminal} should not be terminal"
            );
        }
    }

    // -----------------------------------------------------------------------
    // [cls-3] is_serving()
    // -----------------------------------------------------------------------

    #[test]
    fn cluster_lifecycle_is_serving_only_for_ready_and_updating() {
        assert!(ClusterLifecycleState::Ready.is_serving());
        assert!(ClusterLifecycleState::Updating.is_serving());
        for not_serving in [
            ClusterLifecycleState::Requested,
            ClusterLifecycleState::Provisioning,
            ClusterLifecycleState::Draining,
            ClusterLifecycleState::Deleted,
            ClusterLifecycleState::Failed,
        ] {
            assert!(
                !not_serving.is_serving(),
                "{not_serving} should not be serving"
            );
        }
    }

    // -----------------------------------------------------------------------
    // [cls-4 + cls-5] parse() roundtrip and fail-closed
    // -----------------------------------------------------------------------

    #[test]
    fn cluster_lifecycle_parse_roundtrips_all_variants() {
        for state in [
            ClusterLifecycleState::Requested,
            ClusterLifecycleState::Provisioning,
            ClusterLifecycleState::Ready,
            ClusterLifecycleState::Updating,
            ClusterLifecycleState::Draining,
            ClusterLifecycleState::Deleted,
            ClusterLifecycleState::Failed,
        ] {
            assert_eq!(
                ClusterLifecycleState::parse(state.as_str()),
                Some(state),
                "roundtrip failed for {state}"
            );
        }
    }

    #[test]
    fn cluster_lifecycle_parse_unknown_returns_none() {
        assert_eq!(ClusterLifecycleState::parse("paused"), None);
        assert_eq!(ClusterLifecycleState::parse(""), None);
        assert_eq!(ClusterLifecycleState::parse("READY"), None);
    }

    // -----------------------------------------------------------------------
    // [cls-6] happy path: Requested -> Provisioning -> Ready -> Draining -> Deleted
    // -----------------------------------------------------------------------

    #[test]
    fn cluster_lifecycle_happy_path_create_delete() {
        let mut s = ClusterLifecycleState::initial();
        assert_eq!(s, ClusterLifecycleState::Requested);
        for next in [
            ClusterLifecycleState::Provisioning,
            ClusterLifecycleState::Ready,
            ClusterLifecycleState::Draining,
            ClusterLifecycleState::Deleted,
        ] {
            s = s
                .transition(next)
                .unwrap_or_else(|e| panic!("legal transition failed: {e}"));
        }
        assert!(s.is_terminal());
        assert_eq!(s, ClusterLifecycleState::Deleted);
    }

    // -----------------------------------------------------------------------
    // [cls-7] update cycle: Ready -> Updating -> Ready
    // -----------------------------------------------------------------------

    #[test]
    fn cluster_lifecycle_update_cycle() {
        let s = ClusterLifecycleState::Ready;
        let updating = s
            .transition(ClusterLifecycleState::Updating)
            .expect("Ready -> Updating must be legal");
        assert_eq!(updating, ClusterLifecycleState::Updating);
        assert!(updating.is_serving());

        let back_to_ready = updating
            .transition(ClusterLifecycleState::Ready)
            .expect("Updating -> Ready must be legal");
        assert_eq!(back_to_ready, ClusterLifecycleState::Ready);
        assert!(back_to_ready.is_serving());
    }

    // -----------------------------------------------------------------------
    // [cls-8] Failed reachable from every non-terminal state
    // -----------------------------------------------------------------------

    #[test]
    fn cluster_lifecycle_failed_reachable_from_every_non_terminal() {
        for state in [
            ClusterLifecycleState::Requested,
            ClusterLifecycleState::Provisioning,
            ClusterLifecycleState::Ready,
            ClusterLifecycleState::Updating,
            ClusterLifecycleState::Draining,
        ] {
            assert!(
                state.can_transition_to(ClusterLifecycleState::Failed),
                "{state} should be able to transition to Failed"
            );
            let result = state.transition(ClusterLifecycleState::Failed);
            assert!(
                result.is_ok(),
                "{state} -> Failed must succeed, got {result:?}"
            );
            assert_eq!(result.unwrap(), ClusterLifecycleState::Failed);
        }
    }

    // -----------------------------------------------------------------------
    // [cls-9] Terminal states have no outgoing transitions
    // -----------------------------------------------------------------------

    #[test]
    fn cluster_lifecycle_terminal_states_have_no_exit() {
        for terminal in [
            ClusterLifecycleState::Deleted,
            ClusterLifecycleState::Failed,
        ] {
            assert!(terminal.is_terminal());
            for next in [
                ClusterLifecycleState::Requested,
                ClusterLifecycleState::Provisioning,
                ClusterLifecycleState::Ready,
                ClusterLifecycleState::Updating,
                ClusterLifecycleState::Draining,
                ClusterLifecycleState::Deleted,
                ClusterLifecycleState::Failed,
            ] {
                assert!(
                    !terminal.can_transition_to(next),
                    "{terminal} must not transition to {next}"
                );
                assert!(
                    terminal.transition(next).is_err(),
                    "{terminal} -> {next} must return Err"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // [cls-10 + cls-11] IllegalClusterTransition carries from/to
    // -----------------------------------------------------------------------

    #[test]
    fn cluster_lifecycle_illegal_transition_carries_correct_pair() {
        let err = ClusterLifecycleState::Requested
            .transition(ClusterLifecycleState::Ready)
            .expect_err("Requested -> Ready must be illegal");
        assert_eq!(err.from, ClusterLifecycleState::Requested);
        assert_eq!(err.to, ClusterLifecycleState::Ready);
    }

    // -----------------------------------------------------------------------
    // [cls-12] Serde snake_case roundtrip for every variant
    // -----------------------------------------------------------------------

    #[test]
    fn cluster_lifecycle_serde_snake_case_roundtrip() {
        let cases = [
            (ClusterLifecycleState::Requested, "\"requested\""),
            (ClusterLifecycleState::Provisioning, "\"provisioning\""),
            (ClusterLifecycleState::Ready, "\"ready\""),
            (ClusterLifecycleState::Updating, "\"updating\""),
            (ClusterLifecycleState::Draining, "\"draining\""),
            (ClusterLifecycleState::Deleted, "\"deleted\""),
            (ClusterLifecycleState::Failed, "\"failed\""),
        ];
        for (state, expected_json) in cases {
            let serialized = serde_json::to_string(&state)
                .unwrap_or_else(|e| panic!("serialize {state:?} failed: {e}"));
            assert_eq!(serialized, expected_json, "JSON for {state:?} mismatch");
            let roundtripped: ClusterLifecycleState = serde_json::from_str(&serialized)
                .unwrap_or_else(|e| panic!("deserialize {state:?} failed: {e}"));
            assert_eq!(roundtripped, state, "round-trip mismatch for {state:?}");
        }
    }

    // -----------------------------------------------------------------------
    // [cls-13 + cls-14 + cls-15] validate_dedicated_readiness
    // -----------------------------------------------------------------------

    #[test]
    fn validate_dedicated_readiness_allows_dedicated_at_floor() {
        // DEDICATED_NODE_FLOOR = 3; exactly at floor must be accepted
        assert!(
            validate_dedicated_readiness(DEDICATED_NODE_FLOOR, DesiredTier::Dedicated).is_ok(),
            "Dedicated at floor must be Ok"
        );
        // above floor also accepted
        assert!(
            validate_dedicated_readiness(DEDICATED_NODE_FLOOR + 1, DesiredTier::Dedicated).is_ok()
        );
    }

    #[test]
    fn validate_dedicated_readiness_denies_dedicated_below_floor() {
        // 1 node < DEDICATED_NODE_FLOOR (3) → denied
        let err = validate_dedicated_readiness(1, DesiredTier::Dedicated)
            .expect_err("below floor must be rejected");
        assert!(matches!(
            err,
            LifecycleValidationError::TargetNodeCountExceedsFloor
        ));
        // 2 nodes also below floor
        let err2 = validate_dedicated_readiness(2, DesiredTier::Dedicated)
            .expect_err("2 < floor must be rejected");
        assert!(matches!(
            err2,
            LifecycleValidationError::TargetNodeCountExceedsFloor
        ));
    }

    #[test]
    fn validate_dedicated_readiness_always_allows_hosted() {
        // Hosted has no additional floor constraint; any non-zero count is fine
        assert!(validate_dedicated_readiness(1, DesiredTier::Hosted).is_ok());
        assert!(
            validate_dedicated_readiness(DEDICATED_NODE_FLOOR - 1, DesiredTier::Hosted).is_ok()
        );
        assert!(validate_dedicated_readiness(500, DesiredTier::Hosted).is_ok());
    }

    // -----------------------------------------------------------------------
    // [cls-16] skip transitions are denied
    // -----------------------------------------------------------------------

    #[test]
    fn cluster_lifecycle_skip_transition_denied() {
        // Requested -> Ready (skip Provisioning)
        assert!(
            !ClusterLifecycleState::Requested.can_transition_to(ClusterLifecycleState::Ready),
            "Requested -> Ready must be illegal (skips Provisioning)"
        );
        // Provisioning -> Draining (skip Ready)
        assert!(
            !ClusterLifecycleState::Provisioning.can_transition_to(ClusterLifecycleState::Draining),
            "Provisioning -> Draining must be illegal (skips Ready)"
        );
        // Ready -> Deleted (skip Draining)
        assert!(
            !ClusterLifecycleState::Ready.can_transition_to(ClusterLifecycleState::Deleted),
            "Ready -> Deleted must be illegal (skips Draining)"
        );
    }

    // -----------------------------------------------------------------------
    // [cls-17] IllegalClusterTransition Display is non-empty and well-formed
    // -----------------------------------------------------------------------

    #[test]
    fn cluster_lifecycle_illegal_transition_display_well_formed() {
        let err = ClusterLifecycleState::Requested
            .transition(ClusterLifecycleState::Deleted)
            .expect_err("Requested -> Deleted must be illegal");
        let display = err.to_string();
        assert!(!display.is_empty(), "Display must not be empty");
        assert!(
            display.contains("requested"),
            "Display must contain 'requested', got: {display}"
        );
        assert!(
            display.contains("deleted"),
            "Display must contain 'deleted', got: {display}"
        );
    }

    // -----------------------------------------------------------------------
    // [cls-extra] validate_dedicated_readiness rejects zero node_count
    // -----------------------------------------------------------------------

    #[test]
    fn validate_dedicated_readiness_rejects_zero_node_count() {
        for tier in [DesiredTier::Hosted, DesiredTier::Dedicated] {
            let err = validate_dedicated_readiness(0, tier)
                .expect_err("zero node_count must be rejected");
            assert!(
                matches!(err, LifecycleValidationError::ZeroTargetNodeCount),
                "expected ZeroTargetNodeCount for tier {tier:?}, got {err:?}"
            );
        }
    }
}
