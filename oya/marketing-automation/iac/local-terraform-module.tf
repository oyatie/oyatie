module "marketing_automation_local_ops" {
  source = "../../infra/modules/microservice-local-ops"

  service_name = "marketing-automation"
  audience_type = "ENTERPRISE_MARKETING_OPERATOR"
  domain_object = "CampaignJourney"
  event_topic = "marketing-automation.local-ops.v1"
  primary_slo = "send-latency"
  policy_directory = "microservices/marketing-automation/policies"
  dashboards = [
    "microservices/marketing-automation/dashboards/local-slo-burn.json",
    "microservices/marketing-automation/dashboards/local-domain-throughput.json"
  ]
}
