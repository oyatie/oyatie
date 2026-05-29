module "data_pipeline_local_ops" {
  source = "../../infra/modules/microservice-local-ops"

  service_name = "data-pipeline"
  audience_type = "DATA_PIPELINE_OPERATOR"
  domain_object = "PipelineRun"
  event_topic = "data-pipeline.local-ops.v1"
  primary_slo = "ingest-freshness"
  policy_directory = "microservices/data-pipeline/policies"
  dashboards = [
    "microservices/data-pipeline/dashboards/local-slo-burn.json",
    "microservices/data-pipeline/dashboards/local-domain-throughput.json"
  ]
}
