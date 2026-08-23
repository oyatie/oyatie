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

/// Occupancy: required on pull_request; skip is ok on merge_group/dispatch.
pub fn occupancy_ok(result: &str, pull_request: bool) -> bool {
    if pull_request {
        result == "success"
    } else {
        result == "skipped" || result == "success"
    }
}

/// Merge-blocking fan-in inputs. Occupants match `presubmit.yml`.
pub struct FanIn<'a> {
    pub occupancy: &'a str,
    pub lint: &'a str,
    pub test: &'a str,
    pub deny: &'a str,
    pub pg_gate: &'a str,
    pub pg_live: &'a str,
    pub live: bool,
    pub pull_request: bool,
}

pub fn fan_in_ok(g: FanIn<'_>) -> bool {
    occupancy_ok(g.occupancy, g.pull_request)
        && required_success(g.lint)
        && required_success(g.test)
        && required_success(g.deny)
        && required_success(g.pg_gate)
        && postgres_ok(g.pg_live, g.live)
}

/// Trunk honesty. Both jobs must run; skip is red.
pub fn postsubmit_ok(test: &str, pg_live: &str) -> bool {
    required_success(test) && required_success(pg_live)
}

#[cfg(test)]
mod tests {
    use super::{FanIn, fan_in_ok, postsubmit_ok};

    fn green() -> FanIn<'static> {
        FanIn {
            occupancy: "success",
            lint: "success",
            test: "success",
            deny: "success",
            pg_gate: "success",
            pg_live: "success",
            live: true,
            pull_request: true,
        }
    }

    #[test]
    fn live_all_success() {
        assert!(fan_in_ok(green()));
    }

    #[test]
    fn occupancy_skipped_on_pr_is_red() {
        let mut g = green();
        g.occupancy = "skipped";
        assert!(!fan_in_ok(g));
    }

    #[test]
    fn occupancy_skipped_on_merge_group_is_green() {
        let mut g = green();
        g.occupancy = "skipped";
        g.live = false;
        g.pull_request = false;
        assert!(fan_in_ok(g));
    }

    #[test]
    fn pg_gate_failure_with_postgres_skipped_is_red() {
        let mut g = green();
        g.pg_gate = "failure";
        g.pg_live = "skipped";
        assert!(!fan_in_ok(g));
    }

    #[test]
    fn not_live_postgres_skipped_is_green() {
        let mut g = green();
        g.pg_live = "skipped";
        g.live = false;
        assert!(fan_in_ok(g));
    }

    #[test]
    fn lint_skipped_is_red() {
        let mut g = green();
        g.lint = "skipped";
        assert!(!fan_in_ok(g));
    }

    #[test]
    fn test_cancelled_is_red() {
        let mut g = green();
        g.test = "cancelled";
        assert!(!fan_in_ok(g));
    }

    #[test]
    fn live_true_postgres_skipped_is_red() {
        let mut g = green();
        g.pg_live = "skipped";
        assert!(!fan_in_ok(g));
    }

    #[test]
    fn postsubmit_requires_both() {
        assert!(postsubmit_ok("success", "success"));
        assert!(!postsubmit_ok("skipped", "success"));
        assert!(!postsubmit_ok("success", "skipped"));
        assert!(!postsubmit_ok("failure", "success"));
    }
}
