//! Presubmit/postsubmit fan-in. GHA bash must match these predicates.

/// GitHub Actions `needs.*.result` values we accept as a hard pass.
pub fn required_success(result: &str) -> bool {
    result == "success"
}

/// Reusable Postgres workflow: must run when either cell requires qualification.
pub fn postgres_ok(result: &str, required: bool) -> bool {
    conditional_job_ok(result, required)
}

pub fn reindeer_qualification_ok(result: &str, required: bool) -> bool {
    conditional_job_ok(result, required)
}

pub fn gate_value(value: &str) -> Option<bool> {
    match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// Reusable-workflow verdict. At least one cell must be selected, and every
/// selected cell must succeed; an unselected cell may be skipped or succeed.
pub fn live_postgres_cells_ok(
    backbone_result: &str,
    compute_result: &str,
    run_backbone: bool,
    run_compute: bool,
) -> bool {
    (run_backbone || run_compute)
        && conditional_job_ok(backbone_result, run_backbone)
        && conditional_job_ok(compute_result, run_compute)
}

fn conditional_job_ok(result: &str, required: bool) -> bool {
    if required {
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
    pub layout: &'a str,
    pub occupancy: &'a str,
    pub lint: &'a str,
    pub clippy: &'a str,
    pub test: &'a str,
    pub deny: &'a str,
    pub change_gate: &'a str,
    pub pg_live: &'a str,
    pub reindeer_qualification: &'a str,
    pub backbone_postgres_gate: &'a str,
    pub compute_lifecycle_postgres_gate: &'a str,
    pub reindeer_gate: &'a str,
    pub pull_request: bool,
}

pub fn fan_in_ok(g: FanIn<'_>) -> bool {
    let Some(backbone_postgres_required) = gate_value(g.backbone_postgres_gate) else {
        return false;
    };
    let Some(compute_lifecycle_postgres_required) = gate_value(g.compute_lifecycle_postgres_gate)
    else {
        return false;
    };
    let Some(reindeer_required) = gate_value(g.reindeer_gate) else {
        return false;
    };
    required_success(g.layout)
        && occupancy_ok(g.occupancy, g.pull_request)
        && required_success(g.lint)
        && required_success(g.clippy)
        && required_success(g.test)
        && required_success(g.deny)
        && required_success(g.change_gate)
        && postgres_ok(
            g.pg_live,
            backbone_postgres_required || compute_lifecycle_postgres_required,
        )
        && reindeer_qualification_ok(g.reindeer_qualification, reindeer_required)
}

/// Trunk honesty. Both jobs must run; skip is red.
pub fn postsubmit_ok(test: &str, pg_live: &str) -> bool {
    required_success(test) && required_success(pg_live)
}

#[cfg(test)]
mod tests {
    use super::{FanIn, fan_in_ok, live_postgres_cells_ok, postsubmit_ok};

    fn green() -> FanIn<'static> {
        FanIn {
            layout: "success",
            occupancy: "success",
            lint: "success",
            clippy: "success",
            test: "success",
            deny: "success",
            change_gate: "success",
            pg_live: "success",
            reindeer_qualification: "success",
            backbone_postgres_gate: "true",
            compute_lifecycle_postgres_gate: "true",
            reindeer_gate: "true",
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
    fn layout_failure_is_red() {
        let mut g = green();
        g.layout = "failure";
        assert!(!fan_in_ok(g));
    }

    #[test]
    fn clippy_skipped_is_red() {
        let mut g = green();
        g.clippy = "skipped";
        assert!(!fan_in_ok(g));
    }

    #[test]
    fn occupancy_skipped_on_merge_group_is_green() {
        let mut g = green();
        g.occupancy = "skipped";
        g.backbone_postgres_gate = "false";
        g.compute_lifecycle_postgres_gate = "false";
        g.pull_request = false;
        assert!(fan_in_ok(g));
    }

    #[test]
    fn change_gate_failure_with_postgres_skipped_is_red() {
        let mut g = green();
        g.change_gate = "failure";
        g.pg_live = "skipped";
        assert!(!fan_in_ok(g));
    }

    #[test]
    fn no_postgres_cell_required_may_skip() {
        let mut g = green();
        g.pg_live = "skipped";
        g.backbone_postgres_gate = "false";
        g.compute_lifecycle_postgres_gate = "false";
        assert!(fan_in_ok(g));
    }

    #[test]
    fn either_required_postgres_cell_cannot_skip() {
        for (backbone, compute) in [("true", "false"), ("false", "true"), ("true", "true")] {
            let mut g = green();
            g.pg_live = "skipped";
            g.backbone_postgres_gate = backbone;
            g.compute_lifecycle_postgres_gate = compute;
            assert!(!fan_in_ok(g), "backbone={backbone} compute={compute}");
        }
    }

    #[test]
    fn unknown_gate_output_is_red() {
        for invalid in ["", "TRUE", "0", "unknown"] {
            for field in 0..3 {
                let mut g = green();
                match field {
                    0 => g.backbone_postgres_gate = invalid,
                    1 => g.compute_lifecycle_postgres_gate = invalid,
                    _ => g.reindeer_gate = invalid,
                }
                assert!(!fan_in_ok(g), "field={field} value={invalid:?}");
            }
        }
    }

    #[test]
    fn live_cell_verdict_is_fail_closed_and_cell_aware() {
        assert!(live_postgres_cells_ok("success", "skipped", true, false));
        assert!(live_postgres_cells_ok("skipped", "success", false, true));
        assert!(live_postgres_cells_ok("success", "success", true, true));
        assert!(!live_postgres_cells_ok("skipped", "skipped", false, false));
        assert!(!live_postgres_cells_ok("failure", "skipped", true, false));
        assert!(!live_postgres_cells_ok("skipped", "failure", false, true));
        assert!(!live_postgres_cells_ok("success", "skipped", true, true));
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
    fn required_reindeer_qualification_cannot_skip() {
        let mut g = green();
        g.reindeer_qualification = "skipped";
        assert!(!fan_in_ok(g));
    }

    #[test]
    fn ungated_reindeer_qualification_may_skip() {
        let mut g = green();
        g.reindeer_gate = "false";
        g.reindeer_qualification = "skipped";
        assert!(fan_in_ok(g));
    }

    #[test]
    fn change_gate_failure_is_red_when_qualification_skips() {
        let mut g = green();
        g.change_gate = "failure";
        g.reindeer_gate = "false";
        g.reindeer_qualification = "skipped";
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
