use check_authz_tier_discipline::{Tier, scan_cedar, scan_combined, scan_envoy_filter};

#[test]
fn clean_cedar_policy_passes() {
    let body = r#"
permit (principal, action == Action::"Read", resource is Doc)
when { principal.tenant_id == resource.tenant_id };
"#;
    let r = scan_cedar("policy.cedar", body);
    assert!(r.ok());
}

#[test]
fn cedar_policy_referencing_client_ip_flags() {
    let body = r#"
permit (principal, action, resource)
when { context.client_ip != "10.0.0.0/8" };
"#;
    let r = scan_cedar("policy.cedar", body);
    assert_eq!(r.findings.len(), 1);
    assert_eq!(r.findings[0].needle, "client_ip");
    assert_eq!(r.findings[0].source_tier, Tier::Origin);
    assert_eq!(r.findings[0].wrong_concern_tier, Tier::Edge);
}

#[test]
fn cedar_policy_referencing_geo_flags() {
    let body = "permit when { context.geoip.country_code == \"KR\" };";
    let r = scan_cedar("policy.cedar", body);
    // Two needles in one line: geoip + country_code → two findings.
    assert_eq!(r.findings.len(), 2);
}

#[test]
fn cedar_suppression_marker_suppresses() {
    let body = r#"
permit when { context.client_ip != "0.0.0.0" }; // authz-tier-discipline: ok (test fixture)
"#;
    let r = scan_cedar("policy.cedar", body);
    assert!(r.ok());
}

#[test]
fn clean_envoy_filter_passes() {
    let body = r#"
rate_limits:
  - actions:
      - request_headers:
          header_name: x-tenant-id
          descriptor_key: tenant_id_header
"#;
    let r = scan_envoy_filter("envoy.yaml", body);
    assert!(r.ok());
}

#[test]
fn envoy_filter_referencing_acr_flags() {
    let body = r#"
filters:
  - name: custom.acr_gate
    typed_config:
      acr_required: sensitive
"#;
    let r = scan_envoy_filter("envoy.yaml", body);
    assert!(!r.ok());
    let f = &r.findings[0];
    assert_eq!(f.source_tier, Tier::Edge);
    assert_eq!(f.wrong_concern_tier, Tier::Origin);
    assert!(f.needle == "acr_required" || f.needle == "principal.acr" || f.needle == "acr_level");
}

#[test]
fn envoy_filter_referencing_data_class_flags() {
    let body = "match: { data_class: PII_SENSITIVE }";
    let r = scan_envoy_filter("envoy.yaml", body);
    assert_eq!(r.findings.len(), 1);
    assert_eq!(r.findings[0].needle, "data_class");
}

#[test]
fn combined_scan_merges_findings() {
    let cedar = "permit when { context.asn == 12345 };";
    let envoy = "match: { acr_required: critical }";
    let r = scan_combined(&[("p.cedar", cedar)], &[("e.yaml", envoy)]);
    assert!(r.findings.len() >= 2);
    let tiers: std::collections::BTreeSet<Tier> =
        r.findings.iter().map(|f| f.source_tier).collect();
    assert!(tiers.contains(&Tier::Origin));
    assert!(tiers.contains(&Tier::Edge));
}

#[test]
fn line_numbers_are_one_indexed() {
    let body = "// l1\n// l2\npermit when { context.client_ip == \"x\" };";
    let r = scan_cedar("p.cedar", body);
    assert_eq!(r.findings.len(), 1);
    assert_eq!(r.findings[0].line, 3);
}

#[test]
fn finding_carries_remediation() {
    let body = "permit when { context.bot_score > 0.5 };";
    let r = scan_cedar("p.cedar", body);
    assert!(!r.findings.is_empty());
    assert!(r.findings[0].remediation.contains("Envoy edge"));
}

#[test]
fn suppression_marker_only_suppresses_marked_line() {
    let body =
        "context.client_ip == \"x\"; // authz-tier-discipline: ok (intentional)\ncontext.asn == 1;";
    let r = scan_cedar("p.cedar", body);
    // First line suppressed; second still flags.
    assert_eq!(r.findings.len(), 1);
    assert_eq!(r.findings[0].needle, "asn");
}
