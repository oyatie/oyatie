#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use data_boundary_kernel::DataClass;
use hr_employment_domain::{
    HrDomainError, LeaveBalanceAccrualInput, evaluate_leave_balance_accrual,
};

// ---------------------------------------------------------------------------
// [RED] Additional acceptance-criteria tests (hr-3 full coverage)
// These tests were written before the implementation per TDD discipline.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Happy-path
// ---------------------------------------------------------------------------

include!(concat!(env!("OUT_DIR"), "/leave_balance.generated.rs"));
