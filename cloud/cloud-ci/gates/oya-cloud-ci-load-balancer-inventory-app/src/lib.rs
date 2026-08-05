//! # cloud-ci-load-balancer-inventory
//!
//! Pure cloud-ci readiness gate packet for tenant-facing `Service.type=LoadBalancer`
//! inventory. The producer owns repository manifest scanning and feeds rows shaped as
//! Kubernetes public-listener facts; this crate only classifies DATA rows against the
//! accepted ADR-0157/ADR-0182 taxonomy. It does not apply Kubernetes manifests, mutate
//! clusters, provision load balancers, or claim production readiness.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use serde_json::Value;

pub const GATE_ID: &str = "cloud-ci-load-balancer-inventory";

pub const VIOLATION_CODES: [&str; 8] = [
    "load_balancer_inventory_missing",
    "unclassified_tenant_facing_load_balancer",
    "direct_mail_workload_load_balancer",
    "mail_workload_not_clusterip",
    "non_http_edge_missing_authority",
    "non_http_edge_port_not_authorized",
    "non_http_edge_missing_control",
    "mail_public_ingress_bypass",
];

const REQUIRED_NON_HTTP_CONTROLS: [&str; 10] = [
    "mtls_spiffe",
    "cilium_network_policy_allowlist",
    "otel_audit_correlation",
    "l4_ddos_connection_rate_limit",
    "per_tenant_ip_rate_limit",
    "starttls_mta_sts_tls_rpt",
    "sasl_oidc_binding",
    "open_relay_refusal",
    "dmarc_spf_dkim_arc_rspamd",
    "rollback_and_protocol_smoke_evidence",
];

const AUTHORIZED_MAIL_PORTS: [u64; 6] = [25, 143, 465, 587, 993, 4190];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
    pub detail: String,
}

impl Finding {
    fn new(code: &str, key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.into(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub violations: BTreeSet<String>,
}

impl Report {
    fn from_findings(findings: &BTreeSet<Finding>) -> Self {
        let violations = findings
            .iter()
            .map(|finding| finding.code.clone())
            .collect::<BTreeSet<_>>();
        Self {
            verdict: if violations.is_empty() {
                Verdict::Green
            } else {
                Verdict::Red
            },
            violations,
        }
    }
}

fn string_field<'a>(row: &'a Value, field: &str) -> Option<&'a str> {
    row.get(field).and_then(Value::as_str).map(str::trim)
}

fn bool_field(row: &Value, field: &str) -> bool {
    row.get(field).and_then(Value::as_bool).unwrap_or(false)
}

fn row_key(row: &Value, index: usize) -> String {
    string_field(row, "resource_id")
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .or_else(|| {
            string_field(row, "path")
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| format!("<load-balancer-row-{index}>"))
}

fn string_array_contains(row: &Value, field: &str, expected: &str) -> bool {
    row.get(field)
        .and_then(Value::as_array)
        .is_some_and(|items| items.iter().any(|item| item.as_str() == Some(expected)))
}

fn port_values(row: &Value) -> Vec<u64> {
    row.get("ports")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_u64).collect())
        .unwrap_or_default()
}

fn ports_authorized_for_mail(row: &Value) -> bool {
    let ports = port_values(row);
    !ports.is_empty()
        && ports
            .iter()
            .all(|port| AUTHORIZED_MAIL_PORTS.iter().any(|allowed| allowed == port))
}

fn has_all_required_non_http_controls(row: &Value) -> Vec<&'static str> {
    REQUIRED_NON_HTTP_CONTROLS
        .iter()
        .copied()
        .filter(|control| !string_array_contains(row, "controls", control))
        .collect()
}

fn evaluate_load_balancer_row(index: usize, row: &Value, findings: &mut BTreeSet<Finding>) {
    let key = row_key(row, index);
    if !bool_field(row, "tenant_facing") {
        return;
    }

    let mail_owned_tenant_facing_load_balancer = string_field(row, "owner") == Some("mail");
    if mail_owned_tenant_facing_load_balancer {
        findings.insert(Finding::new(
            "direct_mail_workload_load_balancer",
            key.clone(),
            "mail workloads may not own tenant-facing LoadBalancer resources",
        ));
    }

    match string_field(row, "classification") {
        Some("api_gateway_http_grpc_load_balancer") => {
            if string_field(row, "owner") != Some("api-gateway") {
                findings.insert(Finding::new(
                    "unclassified_tenant_facing_load_balancer",
                    key,
                    "HTTP/gRPC LoadBalancer classification is only valid for api-gateway-owned edge resources",
                ));
            }
        }
        Some("authorized_non_http_protocol_edge") => {
            if !string_array_contains(
                row,
                "authority_refs",
                "docs/decisions/ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation.md#non-http-protocol-edge-extension",
            ) {
                findings.insert(Finding::new(
                    "non_http_edge_missing_authority",
                    key.clone(),
                    "authorized non-HTTP protocol edge rows must cite ADR-0182 non-HTTP edge authority",
                ));
            }
            if !ports_authorized_for_mail(row) {
                findings.insert(Finding::new(
                    "non_http_edge_port_not_authorized",
                    key.clone(),
                    "authorized mail protocol edge rows must use only TCP ports 25/465/587/143/993/4190",
                ));
            }
            let missing = has_all_required_non_http_controls(row);
            if !missing.is_empty() {
                findings.insert(Finding::new(
                    "non_http_edge_missing_control",
                    key,
                    format!(
                        "authorized non-HTTP protocol edge missing controls: {}",
                        missing.join(", ")
                    ),
                ));
            }
        }
        _ => {
            findings.insert(Finding::new(
                "unclassified_tenant_facing_load_balancer",
                key,
                "tenant-facing LoadBalancer must be classified as api_gateway_http_grpc_load_balancer or authorized_non_http_protocol_edge",
            ));
        }
    }
}

fn evaluate_mail_workload_row(index: usize, row: &Value, findings: &mut BTreeSet<Finding>) {
    let key = row_key(row, index);
    if string_field(row, "owner") != Some("mail") {
        return;
    }
    if string_field(row, "workload_service_type") != Some("ClusterIP") {
        findings.insert(Finding::new(
            "mail_workload_not_clusterip",
            key.clone(),
            "mail workload Services behind the edge protocol listener must remain ClusterIP",
        ));
    }
    if bool_field(row, "direct_public_ingress") {
        findings.insert(Finding::new(
            "mail_public_ingress_bypass",
            key,
            "mail workload ingress must be allowlisted from api-gateway/edge, not 0.0.0.0/0",
        ));
    }
}

fn evaluate_row(index: usize, row: &Value, findings: &mut BTreeSet<Finding>) {
    match string_field(row, "row_type") {
        Some("mail_workload_service") => evaluate_mail_workload_row(index, row, findings),
        Some("load_balancer") => evaluate_load_balancer_row(index, row, findings),
        _ => {
            let key = row_key(row, index);
            findings.insert(Finding::new(
                "load_balancer_inventory_missing",
                key,
                "producer row missing recognized row_type",
            ));
        }
    }
}

pub fn evaluate_keyed(input: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let rows = input
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if rows.is_empty() {
        findings.insert(Finding::new(
            "load_balancer_inventory_missing",
            "<load-balancer-inventory>",
            "producer emitted no LoadBalancer inventory rows",
        ));
        return findings;
    }
    for (index, row) in rows.iter().enumerate() {
        evaluate_row(index, row, &mut findings);
    }
    findings
}

pub fn evaluate(input: &Value) -> Report {
    Report::from_findings(&evaluate_keyed(input))
}
