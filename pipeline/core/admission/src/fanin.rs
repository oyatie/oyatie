//! Presubmit fan-in contract. GHA bash must match these predicates.

/// GitHub Actions `needs.*.result` values we accept as a hard pass.
pub fn required_success(result: &str) -> bool {
    result == "success"
}

/// Postgres jobs: must run when `live`; may skip only when the gate said not live.
pub fn postgres_ok(result: &str, live: bool) -> bool {
    if live {
        result == "success"
    } else {
        result == "skipped" || result == "success"
    }
}

/// Merge-blocking fan-in. `pg_gate` must itself be success; skipped postgres is
/// allowed only when `live` is false.
pub fn fan_in_ok(
    lint: &str,
    test: &str,
    deny: &str,
    pg_gate: &str,
    pg_adapters: &str,
    pg_facades: &str,
    live: bool,
) -> bool {
    required_success(lint)
        && required_success(test)
        && required_success(deny)
        && required_success(pg_gate)
        && postgres_ok(pg_adapters, live)
        && postgres_ok(pg_facades, live)
}

#[cfg(test)]
mod tests {
    use super::fan_in_ok;

    #[test]
    fn live_all_success() {
        assert!(fan_in_ok(
            "success", "success", "success", "success", "success", "success", true
        ));
    }

    #[test]
    fn pg_gate_failure_with_postgres_skipped_is_red() {
        assert!(!fan_in_ok(
            "success", "success", "success", "failure", "skipped", "skipped", true
        ));
    }

    #[test]
    fn not_live_postgres_skipped_is_green() {
        assert!(fan_in_ok(
            "success", "success", "success", "success", "skipped", "skipped", false
        ));
    }

    #[test]
    fn lint_skipped_is_red() {
        assert!(!fan_in_ok(
            "skipped", "success", "success", "success", "success", "success", true
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
            "success",
            true
        ));
    }

    #[test]
    fn live_true_adapters_skipped_is_red() {
        assert!(!fan_in_ok(
            "success", "success", "success", "success", "skipped", "success", true
        ));
    }
}
