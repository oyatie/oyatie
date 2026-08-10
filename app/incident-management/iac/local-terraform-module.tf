module "incident_management_local_ops" {
  source = "../../infra/modules/microservice-local-ops"

  service_name = "incident-management"
  audience_type = "ONCALL_RESPONDER"
  domain_object = "IncidentCommand"
  event_topic = "incident-management.local-ops.v1"
  primary_slo = "page-to-acknowledge"
  policy_directory = "app/incident-management/policies"
  dashboards = [
    "app/incident-management/dashboards/local-slo-burn.json",
    "app/incident-management/dashboards/local-domain-throughput.json"
  ]
}
