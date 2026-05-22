module "learning_management_local_ops" {
  source = "../../infra/modules/microservice-local-ops"

  service_name = "learning-management"
  audience_type = "LEARNING_ADMIN"
  domain_object = "LearningCohort"
  event_topic = "learning-management.local-ops.v1"
  primary_slo = "content-delivery-latency"
  policy_directory = "microservices/learning-management/policies"
  dashboards = [
    "microservices/learning-management/dashboards/local-slo-burn.json",
    "microservices/learning-management/dashboards/local-domain-throughput.json"
  ]
}
