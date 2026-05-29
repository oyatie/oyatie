module "whiteboard_local_ops" {
  source = "../../infra/modules/microservice-local-ops"

  service_name = "whiteboard"
  audience_type = "WORKSPACE_COLLABORATOR"
  domain_object = "WhiteboardSession"
  event_topic = "whiteboard.local-ops.v1"
  primary_slo = "cursor-latency"
  policy_directory = "microservices/whiteboard/policies"
  dashboards = [
    "microservices/whiteboard/dashboards/local-slo-burn.json",
    "microservices/whiteboard/dashboards/local-domain-throughput.json"
  ]
}
