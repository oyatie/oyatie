use compute_k8s_api::{
    CloudComputeK8sClusterCreateRequest, CloudComputeK8sClusterRecord,
    CloudComputeK8sCreateCommand, CloudComputeK8sCreateReceipt, CloudComputeK8sDeleteCommand,
    CloudComputeK8sDeleteReceipt, CloudComputeK8sLifecycleRepository,
    CloudComputeK8sLifecycleRepositoryError, CloudComputeK8sRepositoryFuture,
};
use shared_postgres_command_kernel::SET_LOCAL_TENANT_SQL;
use sqlx::Row;

use crate::{
    PgK8sLifecycleRepository, SCHEMA_VERSION,
    error::{integrity, unavailable},
    operation::{
        INSERT_CLUSTER_SQL, SELECT_CLUSTER_FOR_UPDATE_SQL, UPDATE_CLUSTER_SQL, complete_operation,
        decode, encode, is_unique_violation, replay_create, replay_delete, reserve_operation,
        select_operation_for_update, validate_create_command, validate_delete_command,
    },
};

impl CloudComputeK8sLifecycleRepository for PgK8sLifecycleRepository {
    fn commit_create<'a>(
        &'a self,
        command: CloudComputeK8sCreateCommand,
    ) -> CloudComputeK8sRepositoryFuture<
        'a,
        Result<CloudComputeK8sCreateReceipt, CloudComputeK8sLifecycleRepositoryError>,
    > {
        Box::pin(async move {
            validate_create_command(&command)?;
            let desired_spec_json = encode(&command.desired_spec)?;
            let cluster_json = encode(&command.cluster)?;
            let mut tx = self.pool.begin().await.map_err(unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&command.operation_key.tenant_id)
                .execute(&mut *tx)
                .await
                .map_err(unavailable)?;

            let inserted = reserve_operation(
                &mut tx,
                &command.operation_key,
                &command.cluster.resource_id,
                &command.fingerprint,
            )
            .await?;
            if !inserted {
                let row = select_operation_for_update(&mut tx, &command.operation_key).await?;
                let receipt = replay_create(&row, &command)?;
                tx.commit().await.map_err(unavailable)?;
                return Ok(receipt);
            }

            let insert = sqlx::query(INSERT_CLUSTER_SQL)
                .bind(&command.operation_key.tenant_id)
                .bind(&command.cluster.resource_id)
                .bind(&desired_spec_json)
                .bind(&cluster_json)
                .bind(&command.cluster.state)
                .bind(&command.cluster.desired_state)
                .bind(SCHEMA_VERSION)
                .execute(&mut *tx)
                .await;
            if let Err(error) = insert {
                if is_unique_violation(&error) {
                    return Err(CloudComputeK8sLifecycleRepositoryError::ClusterAlreadyExists);
                }
                return Err(CloudComputeK8sLifecycleRepositoryError::Unavailable);
            }

            let receipt = CloudComputeK8sCreateReceipt {
                cluster: command.cluster,
                request_id: command.request_id,
            };
            let receipt_json = encode(&receipt)?;
            complete_operation(&mut tx, &command.operation_key, "create", &receipt_json).await?;
            tx.commit().await.map_err(unavailable)?;
            Ok(receipt)
        })
    }

    fn commit_deletion<'a>(
        &'a self,
        command: CloudComputeK8sDeleteCommand,
    ) -> CloudComputeK8sRepositoryFuture<
        'a,
        Result<CloudComputeK8sDeleteReceipt, CloudComputeK8sLifecycleRepositoryError>,
    > {
        Box::pin(async move {
            validate_delete_command(&command)?;
            let mut tx = self.pool.begin().await.map_err(unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&command.operation_key.tenant_id)
                .execute(&mut *tx)
                .await
                .map_err(unavailable)?;

            let inserted = reserve_operation(
                &mut tx,
                &command.operation_key,
                &command.resource_id.value,
                &command.resource_id.value,
            )
            .await?;
            if !inserted {
                let row = select_operation_for_update(&mut tx, &command.operation_key).await?;
                let receipt = replay_delete(&row, &command)?;
                tx.commit().await.map_err(unavailable)?;
                return Ok(receipt);
            }

            let row = sqlx::query(SELECT_CLUSTER_FOR_UPDATE_SQL)
                .bind(&command.operation_key.tenant_id)
                .bind(&command.resource_id.value)
                .fetch_optional(&mut *tx)
                .await
                .map_err(unavailable)?
                .ok_or(CloudComputeK8sLifecycleRepositoryError::ClusterNotFound)?;
            let desired_spec_json: serde_json::Value =
                row.try_get("desired_spec_json").map_err(integrity)?;
            let cluster_json: serde_json::Value = row.try_get("cluster_json").map_err(integrity)?;
            let observed_state: String = row.try_get("observed_state").map_err(integrity)?;
            let desired_state: String = row.try_get("desired_state").map_err(integrity)?;
            let schema_version: i32 = row.try_get("schema_version").map_err(integrity)?;
            let desired_spec: CloudComputeK8sClusterCreateRequest = decode(desired_spec_json)?;
            let mut cluster: CloudComputeK8sClusterRecord = decode(cluster_json)?;
            if schema_version != SCHEMA_VERSION
                || desired_spec.resource_id != command.resource_id.value
                || desired_spec.tenant_id != command.operation_key.tenant_id
                || cluster.resource_id != command.resource_id.value
                || cluster.tenant_id != command.operation_key.tenant_id
                || cluster.state != observed_state
                || cluster.desired_state != desired_state
            {
                return Err(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation);
            }

            cluster.desired_state = "deleted".to_string();
            let updated_cluster_json = encode(&cluster)?;
            let update = sqlx::query(UPDATE_CLUSTER_SQL)
                .bind(&command.operation_key.tenant_id)
                .bind(&command.resource_id.value)
                .bind(&updated_cluster_json)
                .bind(&cluster.state)
                .bind(&cluster.desired_state)
                .bind(SCHEMA_VERSION)
                .execute(&mut *tx)
                .await
                .map_err(unavailable)?;
            if update.rows_affected() != 1 {
                return Err(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation);
            }

            let receipt = CloudComputeK8sDeleteReceipt {
                cluster,
                request_id: command.request_id,
            };
            let receipt_json = encode(&receipt)?;
            complete_operation(&mut tx, &command.operation_key, "delete", &receipt_json).await?;
            tx.commit().await.map_err(unavailable)?;
            Ok(receipt)
        })
    }
}
