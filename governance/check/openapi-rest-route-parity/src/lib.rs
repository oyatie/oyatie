//! OpenAPI ↔ REST route parity validator.
//!
//! Closes drift between:
//!   - `_ROUTE: &str = "..."` constants in `crates/oya-ops-*-rest/src/lib.rs`
//!   - `paths:` keys in `contracts/*.openapi.yaml`
//!
//! Without this, the comment "Routes here MUST stay 1:1 with paths in
//! contracts/*.openapi.yaml" stays paper. With it, any rename / addition /
//! removal on either side fails the lane loudly.
//!
//! Pure std-only kernel. The runtime gate parses Rust + YAML and populates
//! these inputs; this crate just compares sets.

use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RouteParityInputs {
    /// Distinct route strings discovered in REST crate sources (every
    /// `pub const *_ROUTE: &str = "...";` value).
    pub rest_routes: BTreeSet<String>,
    /// Distinct path keys discovered under each OpenAPI `paths:` map.
    pub openapi_paths: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Violation {
    /// Route declared in REST source but never appears in any OpenAPI contract.
    MissingFromOpenapi { route: String },
    /// Path declared in OpenAPI contract but no REST handler route constant
    /// names it.
    MissingFromRest { path: String },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ValidationReport {
    pub violations: Vec<Violation>,
    pub rest_route_count: usize,
    pub openapi_path_count: usize,
}

impl ValidationReport {
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

pub fn validate(inputs: &RouteParityInputs) -> ValidationReport {
    let mut violations = Vec::new();
    for route in &inputs.rest_routes {
        if !inputs.openapi_paths.contains(route) {
            violations.push(Violation::MissingFromOpenapi {
                route: route.clone(),
            });
        }
    }
    for path in &inputs.openapi_paths {
        if !inputs.rest_routes.contains(path) {
            violations.push(Violation::MissingFromRest { path: path.clone() });
        }
    }
    ValidationReport {
        violations,
        rest_route_count: inputs.rest_routes.len(),
        openapi_path_count: inputs.openapi_paths.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs(rest: &[&str], openapi: &[&str]) -> RouteParityInputs {
        RouteParityInputs {
            rest_routes: rest.iter().map(|s| s.to_string()).collect(),
            openapi_paths: openapi.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn matching_sets_are_clean() {
        let report = validate(&inputs(
            &["/workspace", "/workspace/api/v1/health"],
            &["/workspace", "/workspace/api/v1/health"],
        ));
        assert!(report.is_clean());
        assert_eq!(report.rest_route_count, 2);
        assert_eq!(report.openapi_path_count, 2);
    }

    #[test]
    fn route_missing_from_openapi_fails() {
        let report = validate(&inputs(&["/workspace", "/orphan"], &["/workspace"]));
        assert_eq!(report.violations.len(), 1);
        assert!(matches!(
            &report.violations[0],
            Violation::MissingFromOpenapi { route } if route == "/orphan"
        ));
    }

    #[test]
    fn path_missing_from_rest_fails() {
        let report = validate(&inputs(&["/workspace"], &["/workspace", "/orphan"]));
        assert_eq!(report.violations.len(), 1);
        assert!(matches!(
            &report.violations[0],
            Violation::MissingFromRest { path } if path == "/orphan"
        ));
    }

    #[test]
    fn both_directions_independently_flagged() {
        let report = validate(&inputs(&["/only-rest"], &["/only-openapi"]));
        assert_eq!(report.violations.len(), 2);
        let has_missing_from_openapi = report
            .violations
            .iter()
            .any(|v| matches!(v, Violation::MissingFromOpenapi { route } if route == "/only-rest"));
        let has_missing_from_rest = report
            .violations
            .iter()
            .any(|v| matches!(v, Violation::MissingFromRest { path } if path == "/only-openapi"));
        assert!(has_missing_from_openapi);
        assert!(has_missing_from_rest);
    }

    #[test]
    fn empty_inputs_clean() {
        let report = validate(&RouteParityInputs::default());
        assert!(report.is_clean());
        assert_eq!(report.rest_route_count, 0);
        assert_eq!(report.openapi_path_count, 0);
    }

    #[test]
    fn duplicate_rest_routes_dedup_via_btreeset() {
        // BTreeSet deduplicates; validation should see only the unique route.
        let mut rest = BTreeSet::new();
        rest.insert("/workspace".to_string());
        rest.insert("/workspace".to_string()); // duplicate insert
        let inputs = RouteParityInputs {
            rest_routes: rest,
            openapi_paths: ["/workspace".to_string()].into_iter().collect(),
        };
        let report = validate(&inputs);
        assert!(report.is_clean());
        assert_eq!(report.rest_route_count, 1);
    }

    #[test]
    fn report_counters_reflect_input_sizes() {
        let report = validate(&inputs(&["/a", "/b", "/c"], &["/a", "/b", "/c", "/d"]));
        assert_eq!(report.rest_route_count, 3);
        assert_eq!(report.openapi_path_count, 4);
        assert_eq!(report.violations.len(), 1); // /d missing from rest
    }
}
