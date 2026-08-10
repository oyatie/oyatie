module "healthcare_integration_local_ops" {
  source = "../../infra/modules/microservice-local-ops"

  service_name = "healthcare-integration"
  audience_type = "HEALTHCARE_INTEGRATION_ADMIN"
  domain_object = "ClinicalExchange"
  event_topic = "healthcare-integration.local-ops.v1"
  primary_slo = "phi-delivery-latency"
  policy_directory = "app/healthcare-integration/policies"
  dashboards = [
    "app/healthcare-integration/dashboards/local-slo-burn.json",
    "app/healthcare-integration/dashboards/local-domain-throughput.json"
  ]
}
