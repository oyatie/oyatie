module "incident_management_local_ops" {
  source = "../../infra/modules/microservice-local-ops"

  service_name = "incident-management"
  audience_type = "ONCALL_RESPONDER"
  domain_object = "IncidentCommand"
  event_topic = "incident-management.local-ops.v1"
  primary_slo = "page-to-acknowledge"
  policy_directory = "microservices/incident-management/policies"
  dashboards = [
    "microservices/incident-management/dashboards/local-slo-burn.json",
    "microservices/incident-management/dashboards/local-domain-throughput.json"
  ]
}
