use compute_k8s_api::{CloudComputeK8sAcceptanceApiError as Error, CloudComputeK8sOperationLookup};
use compute_k8s_lifecycle_repository_postgres::PgK8sLifecycleRepository;
use sqlx::PgPool;

use super::fixtures::*;
use crate::support::{cluster_id, tenant_count};

pub(super) async fn assert_concurrency(repository: &PgK8sLifecycleRepository, app: &PgPool) {
    let (left, right) = tokio::join!(
        accept(repository, pending_request("race-left", "race")),
        accept(repository, pending_request("race-right", "race")),
    );
    let winner = left.unwrap();
    assert_eq!(winner, right.unwrap());
    assert!(matches!(
        winner.operation.receipt.request_id.as_str(),
        "race-left" | "race-right"
    ));
    assert_eq!(tenant_count(app, "ten_alpha", "SELECT count(*)::bigint FROM compute_k8s_lifecycle.operations WHERE idempotency_key = 'race'").await, 1);

    let mut base = pending_request("ordered", "canonical");
    base.body.node_pools[0]
        .security_groups
        .push("sg_other".into());
    let mut second = base.body.node_pools[0].clone();
    second.id = "np_b".into();
    base.body.node_pools.push(second);
    let first = accept(repository, base.clone()).await.unwrap();
    let mut reordered = base.clone();
    reordered.boundary.request_id = "reordered".into();
    reordered.body.node_pools.reverse();
    for pool in &mut reordered.body.node_pools {
        pool.security_groups.reverse();
    }
    assert_eq!(accept(repository, reordered).await.unwrap(), first);

    for field in [
        "target",
        "region",
        "version",
        "private",
        "residency",
        "pool_id",
        "az",
        "cell",
        "subnet",
        "groups",
        "class",
        "vcpu",
        "memory",
        "ssd",
        "min",
        "max",
        "autoscaling",
        "pool_count",
    ] {
        let mut changed = base.clone();
        let pool = &mut changed.body.node_pools[0];
        match field {
            "target" => changed.body.resource_id = cluster_id("ten_alpha", "other"),
            "region" => {
                changed.body.region = "region-home-2".into();
                changed.body.resource_id = changed
                    .body
                    .resource_id
                    .replace("region-home", "region-home-2");
                for pool in &mut changed.body.node_pools {
                    pool.az = pool.az.replace("region-home", "region-home-2");
                    pool.cell_id = pool.cell_id.replace("region-home", "region-home-2");
                    pool.subnet_id = pool.subnet_id.replace("region-home", "region-home-2");
                }
            }
            "version" => changed.body.control_plane_version = "v1.30.3-oyatie.1".into(),
            "private" => changed.body.control_plane_private = false,
            "residency" => changed.body.residency = "global".into(),
            "pool_id" => pool.id = "np_other".into(),
            "az" => {
                pool.az = "region-home-z".into();
                pool.cell_id = "cell-region-home-z-001".into();
            }
            "cell" => pool.cell_id = "cell-region-home-a-002".into(),
            "subnet" => pool.subnet_id = "oyatie:cloud:region-home:ten_alpha:subnet:other".into(),
            "groups" => pool.security_groups[0] = "sg_changed".into(),
            "class" => pool.flavor.class = "compute_optimized".into(),
            "vcpu" => pool.flavor.vcpu = 8,
            "memory" => pool.flavor.memory_gb = 32,
            "ssd" => pool.flavor.local_ssd_gb = 200,
            "min" => pool.min_nodes = 2,
            "max" => pool.max_nodes = 6,
            "autoscaling" => pool.autoscaling_enabled = false,
            "pool_count" => {
                changed.body.node_pools.pop();
            }
            _ => unreachable!(),
        }
        changed.path_cluster_id = changed.body.resource_id.clone();
        assert_eq!(
            accept(repository, changed).await.unwrap_err(),
            Error::IdempotencyKeyReused,
            "{field}"
        );
    }
    let mut gpu = pending_request("gpu", "gpu");
    gpu.body.node_pools[0].flavor.class = "gpu".into();
    gpu.body.node_pools[0].flavor.gpu_count = 1;
    accept(repository, gpu.clone()).await.unwrap();
    gpu.body.node_pools[0].flavor.gpu_count = 2;
    assert_eq!(
        accept(repository, gpu).await.unwrap_err(),
        Error::IdempotencyKeyReused
    );

    let mut ha = pending_request("ha", "flavor");
    ha.body.flavor = "high_availability".into();
    for suffix in ["b", "c"] {
        let mut pool = ha.body.node_pools[0].clone();
        pool.id = format!("np_{suffix}");
        pool.az = format!("region-home-{suffix}");
        pool.cell_id = format!("cell-region-home-{suffix}-001");
        ha.body.node_pools.push(pool);
    }
    accept(repository, ha.clone()).await.unwrap();
    ha.body.flavor = "standard".into();
    assert_eq!(
        accept(repository, ha).await.unwrap_err(),
        Error::IdempotencyKeyReused
    );
    let mut invalid = base;
    invalid.body.data_class = "INTERNAL_ONLY".into();
    assert!(matches!(
        accept(repository, invalid).await,
        Err(Error::Boundary(_))
    ));
    assert_isolation(repository).await;
    assert_eq!(
        tenant_count(
            app,
            "ten_alpha",
            "SELECT count(*)::bigint FROM compute_k8s_lifecycle.clusters"
        )
        .await,
        0
    );
}

async fn assert_isolation(repository: &PgK8sLifecycleRepository) {
    let first = accept(repository, pending_request("one-target", "target-one"))
        .await
        .unwrap();
    let second = accept(repository, pending_request("same-target", "target-two"))
        .await
        .unwrap();
    assert_eq!(
        first.operation.receipt.intent,
        second.operation.receipt.intent
    );
    assert_ne!(
        first.operation.receipt.operation_key,
        second.operation.receipt.operation_key
    );
    let mut other_read = operation_read_request("durable-key");
    other_read.principal.principal_id = "sp-other".into();
    assert_eq!(
        read(repository, other_read.clone()).await.unwrap(),
        CloudComputeK8sOperationLookup::NotObserved
    );
    let mut other_create = pending_request("other-principal", "durable-key");
    other_create.principal = other_read.principal.clone();
    let other = accept(repository, other_create).await.unwrap();
    assert_eq!(
        read(repository, other_read).await.unwrap(),
        CloudComputeK8sOperationLookup::Found(other.operation)
    );
    let mut beta = pending_request("beta", "durable-key");
    beta.boundary.tenant_id = "ten_beta".into();
    beta.principal.tenant_id = "ten_beta".into();
    beta.body.tenant_id = "ten_beta".into();
    beta.body.resource_id = cluster_id("ten_beta", "pending");
    beta.path_cluster_id = beta.body.resource_id.clone();
    for pool in &mut beta.body.node_pools {
        pool.subnet_id = pool.subnet_id.replace("ten_alpha", "ten_beta");
    }
    let mut beta_read = operation_read_request("durable-key");
    beta_read.path_cluster_id = beta.path_cluster_id.clone();
    beta_read.boundary = beta.boundary.clone();
    beta_read.principal = beta.principal.clone();
    assert_eq!(
        read(repository, beta_read.clone()).await.unwrap(),
        CloudComputeK8sOperationLookup::NotObserved
    );
    let accepted = accept(repository, beta).await.unwrap();
    assert_eq!(
        read(repository, beta_read).await.unwrap(),
        CloudComputeK8sOperationLookup::Found(accepted.operation)
    );
    let alpha = read(repository, operation_read_request("durable-key"))
        .await
        .unwrap();
    let CloudComputeK8sOperationLookup::Found(alpha) = alpha else {
        panic!("alpha hidden by other scope");
    };
    assert_eq!(alpha.receipt.request_id, "first");
}
