module "itsm_local_ops" {
  source = "../../infra/modules/microservice-local-ops"

  service_name = "itsm"
  audience_type = "IT_SERVICE_MANAGER"
  domain_object = "ServiceManagementRecord"
  event_topic = "itsm.local-ops.v1"
  primary_slo = "mttr-objective"
  policy_directory = "microservices/itsm/policies"
  dashboards = [
    "microservices/itsm/dashboards/local-slo-burn.json",
    "microservices/itsm/dashboards/local-domain-throughput.json"
  ]
}
