module "financial_planning_local_ops" {
  source = "../../infra/modules/microservice-local-ops"

  service_name = "financial-planning"
  audience_type = "FPNA_CONTROLLER"
  domain_object = "PlanningCycle"
  event_topic = "financial-planning.local-ops.v1"
  primary_slo = "close-cycle-latency"
  policy_directory = "microservices/financial-planning/policies"
  dashboards = [
    "microservices/financial-planning/dashboards/local-slo-burn.json",
    "microservices/financial-planning/dashboards/local-domain-throughput.json"
  ]
}
