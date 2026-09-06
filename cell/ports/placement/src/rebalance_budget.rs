use crate::{Digest32, MoneyMicrounitsV1, MovementBudgetV1};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RebalanceBudgetLimitsV1 {
    pub ordinary_ceiling: MovementBudgetV1,
    pub forward_completion_ceiling: MovementBudgetV1,
    pub window_start_unix_seconds: u64,
    pub window_end_unix_seconds: u64,
    pub limits_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RebalanceBudgetUsageV1 {
    pub bytes: u64,
    pub effects: u64,
    pub cost: MoneyMicrounitsV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RebalanceBudgetAccountingV1 {
    pub limits_digest: Digest32,
    pub ordinary_encumbered: RebalanceBudgetUsageV1,
    pub ordinary_debited: RebalanceBudgetUsageV1,
    pub forward_completion_encumbered: RebalanceBudgetUsageV1,
    pub forward_completion_debited: RebalanceBudgetUsageV1,
    pub outstanding_movement_count: u64,
    pub accounting_digest: Digest32,
}
