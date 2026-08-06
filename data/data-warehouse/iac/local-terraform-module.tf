module "data_warehouse_local_ops" {
  source = "../../infra/modules/microservice-local-ops"

  service_name = "data-warehouse"
  audience_type = "DATA_WAREHOUSE_OPERATOR"
  domain_object = "WarehouseDataset"
  event_topic = "data-warehouse.local-ops.v1"
  primary_slo = "freshness"
  policy_directory = "microservices/data-warehouse/policies"
  dashboards = [
    "microservices/data-warehouse/dashboards/local-slo-burn.json",
    "microservices/data-warehouse/dashboards/local-domain-throughput.json"
  ]
}
