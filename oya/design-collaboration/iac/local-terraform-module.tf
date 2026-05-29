module "design_collaboration_local_ops" {
  source = "../../infra/modules/microservice-local-ops"

  service_name = "design-collaboration"
  audience_type = "DESIGN_WORKSPACE_ADMIN"
  domain_object = "DesignFile"
  event_topic = "design-collaboration.local-ops.v1"
  primary_slo = "file-load-time"
  policy_directory = "microservices/design-collaboration/policies"
  dashboards = [
    "microservices/design-collaboration/dashboards/local-slo-burn.json",
    "microservices/design-collaboration/dashboards/local-domain-throughput.json"
  ]
}
