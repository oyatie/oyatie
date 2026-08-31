use data_boundary_kernel::DataClass;

use crate::*;

use super::fixtures::*;

#[test]
fn rejects_operational_labels_on_compute_metadata_and_function_payloads() {
    let instance_class_error = Instance::new(InstanceCreate {
        data_class: DataClass::Audit,
        ..instance_create()
    })
    .expect_err("compute metadata is public privacy metadata");
    assert_eq!(instance_class_error, CloudComputeError::InvalidDataClass);

    let function_class_error = FunctionDeployment::new(FunctionDeploymentCreate {
        allowed_data_classes: vec![DataClass::Audit],
        ..function_create()
    })
    .expect_err("function payload allowlists use privacy classes only");
    assert_eq!(function_class_error, CloudComputeError::InvalidDataClass);
}

#[test]
fn catalog_rejects_duplicate_compute_resources() {
    let mut catalog = CloudComputeCatalog::default();
    catalog
        .create_instance(instance_create())
        .expect("first instance");
    let duplicate_instance = catalog
        .create_instance(instance_create())
        .expect_err("duplicate instance id rejected");
    assert_eq!(duplicate_instance, CloudComputeError::DuplicateInstance);

    catalog
        .create_kubernetes_cluster(k8s_create())
        .expect("first cluster");
    let duplicate_cluster = catalog
        .create_kubernetes_cluster(k8s_create())
        .expect_err("duplicate cluster id rejected");
    assert_eq!(
        duplicate_cluster,
        CloudComputeError::DuplicateKubernetesCluster
    );

    catalog
        .register_function(function_create())
        .expect("first function");
    let duplicate_function = catalog
        .register_function(function_create())
        .expect_err("duplicate function id rejected");
    assert_eq!(duplicate_function, CloudComputeError::DuplicateFunction);
}
