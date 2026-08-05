#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_cloud_ci_load_balancer_inventory_app::{Verdict, evaluate, evaluate_keyed};
use serde_json::json;

#[test]
fn green_when_api_gateway_http_and_mail_protocol_edge_are_classified() {
    let input = json!({"rows": [
        {
            "row_type": "load_balancer",
            "resource_id": "oya/api-gateway/iac/k8s-deployment.yaml#Service/api-gateway-envoy",
            "path": "oya/api-gateway/iac/k8s-deployment.yaml",
            "owner": "api-gateway",
            "tenant_facing": true,
            "classification": "api_gateway_http_grpc_load_balancer",
            "ports": [443]
        },
        {
            "row_type": "load_balancer",
            "resource_id": "oya/api-gateway/iac/k8s/helm/templates/mail-protocol-routes.yaml#mail-protocol-edge",
            "path": "oya/api-gateway/iac/k8s/helm/templates/mail-protocol-routes.yaml",
            "owner": "api-gateway/edge-platform",
            "tenant_facing": true,
            "classification": "authorized_non_http_protocol_edge",
            "ports": [25, 465, 587, 143, 993, 4190],
            "authority_refs": ["docs/decisions/ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation.md#non-http-protocol-edge-extension"],
            "controls": [
                "mtls_spiffe",
                "cilium_network_policy_allowlist",
                "otel_audit_correlation",
                "l4_ddos_connection_rate_limit",
                "per_tenant_ip_rate_limit",
                "starttls_mta_sts_tls_rpt",
                "sasl_oidc_binding",
                "open_relay_refusal",
                "dmarc_spf_dkim_arc_rspamd",
                "rollback_and_protocol_smoke_evidence"
            ]
        },
        {
            "row_type": "mail_workload_service",
            "resource_id": "oya/mail/iac/helm/templates/service.yaml#mail-workloads",
            "path": "oya/mail/iac/helm/templates/service.yaml",
            "owner": "mail",
            "workload_service_type": "ClusterIP",
            "direct_public_ingress": false
        }
    ]});

    assert_eq!(evaluate(&input).verdict, Verdict::Green);
    assert!(evaluate_keyed(&input).is_empty());
}

#[test]
fn direct_mail_load_balancer_and_public_ingress_are_red() {
    let input = json!({"rows": [
        {
            "row_type": "load_balancer",
            "resource_id": "oya/mail/iac/helm/templates/service.yaml#inbound-smtp-mx",
            "path": "oya/mail/iac/helm/templates/service.yaml",
            "owner": "mail",
            "tenant_facing": true,
            "classification": "authorized_non_http_protocol_edge",
            "ports": [25, 465],
            "authority_refs": ["docs/decisions/ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation.md#non-http-protocol-edge-extension"],
            "controls": []
        },
        {
            "row_type": "mail_workload_service",
            "resource_id": "oya/mail/iac/helm/templates/service.yaml#mail-workloads",
            "path": "oya/mail/iac/helm/templates/service.yaml",
            "owner": "mail",
            "workload_service_type": "LoadBalancer",
            "direct_public_ingress": true
        }
    ]});

    let codes = evaluate(&input).violations;
    for code in [
        "direct_mail_workload_load_balancer",
        "mail_workload_not_clusterip",
        "mail_public_ingress_bypass",
        "non_http_edge_missing_control",
    ] {
        assert!(codes.contains(code), "missing {code}: {codes:?}");
    }
    assert_eq!(evaluate(&input).verdict, Verdict::Red);
}

#[test]
fn mail_owned_load_balancer_is_direct_violation_even_when_unclassified() {
    let input = json!({"rows": [{
        "row_type": "load_balancer",
        "resource_id": "oya/mail/iac/helm/templates/service.yaml#mail-workload-direct-loadbalancer",
        "path": "oya/mail/iac/helm/templates/service.yaml",
        "owner": "mail",
        "tenant_facing": true,
        "classification": "unclassified_tenant_facing_load_balancer",
        "ports": [25, 465, 587, 143, 993, 4190]
    }]});

    let codes = evaluate(&input).violations;
    assert!(codes.contains("direct_mail_workload_load_balancer"));
    assert!(codes.contains("unclassified_tenant_facing_load_balancer"));
}

#[test]
fn unclassified_tenant_facing_load_balancer_fails_closed() {
    let input = json!({"rows": [{
        "row_type": "load_balancer",
        "resource_id": "oya/unknown/iac/service.yaml#public-lb",
        "path": "oya/unknown/iac/service.yaml",
        "owner": "unknown-workload",
        "tenant_facing": true,
        "classification": "",
        "ports": [8443]
    }]});

    assert!(
        evaluate(&input)
            .violations
            .contains("unclassified_tenant_facing_load_balancer")
    );
}

#[test]
fn non_http_edge_without_authority_or_allowed_ports_is_red() {
    let input = json!({"rows": [{
        "row_type": "load_balancer",
        "resource_id": "oya/api-gateway/iac/k8s/helm/templates/mail-protocol-routes.yaml#bad-port",
        "owner": "api-gateway/edge-platform",
        "tenant_facing": true,
        "classification": "authorized_non_http_protocol_edge",
        "ports": [110],
        "authority_refs": [],
        "controls": [
            "mtls_spiffe",
            "cilium_network_policy_allowlist",
            "otel_audit_correlation",
            "l4_ddos_connection_rate_limit",
            "per_tenant_ip_rate_limit",
            "starttls_mta_sts_tls_rpt",
            "sasl_oidc_binding",
            "open_relay_refusal",
            "dmarc_spf_dkim_arc_rspamd",
            "rollback_and_protocol_smoke_evidence"
        ]
    }]});

    let codes = evaluate(&input).violations;
    assert!(codes.contains("non_http_edge_missing_authority"));
    assert!(codes.contains("non_http_edge_port_not_authorized"));
}

#[test]
fn non_http_edge_rejects_placeholder_or_wildcard_ports() {
    let input = json!({"rows": [{
        "row_type": "load_balancer",
        "resource_id": "oya/api-gateway/iac/k8s/helm/templates/mail-protocol-routes.yaml#placeholder-port",
        "owner": "api-gateway/edge-platform",
        "tenant_facing": true,
        "classification": "authorized_non_http_protocol_edge",
        "ports": [0],
        "authority_refs": ["docs/decisions/ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation.md#non-http-protocol-edge-extension"],
        "controls": [
            "mtls_spiffe",
            "cilium_network_policy_allowlist",
            "otel_audit_correlation",
            "l4_ddos_connection_rate_limit",
            "per_tenant_ip_rate_limit",
            "starttls_mta_sts_tls_rpt",
            "sasl_oidc_binding",
            "open_relay_refusal",
            "dmarc_spf_dkim_arc_rspamd",
            "rollback_and_protocol_smoke_evidence"
        ]
    }]});

    assert!(
        evaluate(&input)
            .violations
            .contains("non_http_edge_port_not_authorized")
    );
}
