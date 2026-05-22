module "contact_center_local_ops" {
  source = "../../infra/modules/microservice-local-ops"

  service_name = "contact-center"
  audience_type = "CONTACT_CENTER_SUPERVISOR"
  domain_object = "OmnichannelInteraction"
  event_topic = "contact-center.local-ops.v1"
  primary_slo = "call-drop-rate"
  policy_directory = "microservices/contact-center/policies"
  dashboards = [
    "microservices/contact-center/dashboards/local-slo-burn.json",
    "microservices/contact-center/dashboards/local-domain-throughput.json"
  ]
}
