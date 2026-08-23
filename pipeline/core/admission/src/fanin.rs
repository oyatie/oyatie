//! Presubmit/postsubmit fan-in. GHA bash must match these predicates.

/// GitHub Actions `needs.*.result` values we accept as a hard pass.
pub fn required_success(result: &str) -> bool {
    result == "success"
}

/// Postgres job: must run when `live`; may skip only when the gate said not live.
pub fn postgres_ok(result: &str, live: bool) -> bool {
    if live {
        result == "success"
    } else {
        result == "skipped" || result == "success"
    }
}

/// Merge-blocking fan-in. One live-postgres job. Clippy is not a member
/// (it is not a check until it can fail the merge). Dual-emit names are not
/// members (legacy required contexts are not this function).
pub fn fan_in_ok(
    lint: &str,
    test: &str,
    deny: &str,
    pg_gate: &str,
    pg_live: &str,
    live: bool,
) -> bool {
    required_success(lint)
        && required_success(test)
        && required_success(deny)
        && required_success(pg_gate)
        && postgres_ok(pg_live, live)
}

/// Trunk honesty. Both jobs must run; skip is red.
pub fn postsubmit_ok(test: &str, pg_live: &str) -> bool {
    required_success(test) && required_success(pg_live)
}

#[cfg(test)]
mod tests {
    use super::{fan_in_ok, postsubmit_ok};

    #[test]
    fn live_all_success() {
        assert!(fan_in_ok(
            "success", "success", "success", "success", "success", true
        ));
    }

    #[test]
    fn pg_gate_failure_with_postgres_skipped_is_red() {
        assert!(!fan_in_ok(
            "success", "success", "success", "failure", "skipped", true
        ));
    }

    #[test]
    fn not_live_postgres_skipped_is_green() {
        assert!(fan_in_ok(
            "success", "success", "success", "success", "skipped", false
        ));
    }

    #[test]
    fn lint_skipped_is_red() {
        assert!(!fan_in_ok(
            "skipped", "success", "success", "success", "success", true
        ));
    }

    #[test]
    fn test_cancelled_is_red() {
        assert!(!fan_in_ok(
            "success",
            "cancelled",
            "success",
            "success",
            "success",
            true
        ));
    }

    #[test]
    fn live_true_postgres_skipped_is_red() {
        assert!(!fan_in_ok(
            "success", "success", "success", "success", "skipped", true
        ));
    }

    #[test]
    fn postsubmit_requires_both() {
        assert!(postsubmit_ok("success", "success"));
        assert!(!postsubmit_ok("skipped", "success"));
        assert!(!postsubmit_ok("success", "skipped"));
        assert!(!postsubmit_ok("failure", "success"));
    }
}
