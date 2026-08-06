// Reference implementation — Rust client creating a task with dependencies.
//
// This pattern demonstrates the canonical "create-task-then-link-dependencies"
// flow with proper error handling for cycle-prevention rejection. Copy into
// `examples/create-task-with-deps.rs` in any consumer crate.
//
// Doctrine references:
//   - ADR-TASKS-0002  Dependency-graph cycle prevention (Kahn's algorithm)
//   - ADR-0263       Audit-chain canonical event registry (trace_id propagation)
//   - microservices/tasks/PRD.md §Functional Requirements F-T-08, F-T-12

use oya_tasks_client::{TasksClient, TaskCreate, DependencyCreate, DependencyType, TasksError};
use chrono::NaiveDate;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = TasksClient::builder()
        .endpoint(std::env::var("TASKS_ENDPOINT")?)
        .workspace_id(std::env::var("WORKSPACE_ID")?)
        .auth_token(std::env::var("TASKS_AUTH_TOKEN")?)
        .trace_id(Uuid::new_v4().to_string())  // propagated to audit-chain per ADR-0263
        .build()?;

    let project_id = std::env::var("PROJECT_ID")?;

    // Step 1: create the dependency-target task (the one others will block on).
    let setup_task = client.tasks().create(TaskCreate {
        project_id: project_id.clone(),
        title: "Set up test environment".to_string(),
        description: Some("Provision drill cluster, seed test tenants, warm caches".to_string()),
        status: "Todo".to_string(),
        priority: "P1".to_string(),
        assignee_user_id: Some(std::env::var("ASSIGNEE_USER_ID")?),
        due_date: Some(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
        labels: vec!["infrastructure".to_string(), "blocking".to_string()],
        custom_fields: serde_json::json!({}),
    }).await?;

    println!("Created setup task: {}", setup_task.id);

    // Step 2: create the dependent task (the one that will be blocked-by setup_task).
    let work_task = client.tasks().create(TaskCreate {
        project_id: project_id.clone(),
        title: "Run end-to-end smoke tests".to_string(),
        description: Some("Execute the smoke suite against the warmed environment".to_string()),
        status: "Todo".to_string(),
        priority: "P1".to_string(),
        assignee_user_id: Some(std::env::var("ASSIGNEE_USER_ID")?),
        due_date: Some(NaiveDate::from_ymd_opt(2026, 6, 2).unwrap()),
        labels: vec!["testing".to_string()],
        custom_fields: serde_json::json!({}),
    }).await?;

    println!("Created work task: {}", work_task.id);

    // Step 3: link the dependency. `blocks` semantics: setup_task blocks work_task.
    // Equivalent: work_task is blocked-by setup_task. Either form is acceptable; the API
    // accepts the inverse via the `inverse_of` field if you prefer to write blocked-by.
    let dep_result = client.dependencies().create(DependencyCreate {
        source_task_id: setup_task.id.clone(),
        target_task_id: work_task.id.clone(),
        dependency_type: DependencyType::Blocks,
    }).await;

    match dep_result {
        Ok(dep) => println!("Created dependency: {}", dep.id),
        Err(TasksError::CycleDetected { cycle_path, .. }) => {
            // Per ADR-TASKS-0002, cycles are rejected at write-time. The error includes
            // the cycle path so the caller can present a helpful message.
            eprintln!("ERROR: dependency would create cycle: {}",
                cycle_path.iter().map(|n| n.task_title.as_str()).collect::<Vec<_>>().join(" -> ")
            );
            return Err("cycle rejected".into());
        }
        Err(TasksError::TaskNotFound { task_id }) => {
            eprintln!("ERROR: referenced task {} does not exist", task_id);
            return Err("task not found".into());
        }
        Err(other) => return Err(Box::new(other)),
    }

    // Step 4: verify the dependency is visible in the read path.
    let work_task_fresh = client.tasks().get(&work_task.id).await?;
    let blockers = work_task_fresh.blockers();
    assert_eq!(blockers.len(), 1, "expected exactly 1 blocker");
    assert_eq!(blockers[0].id, setup_task.id, "blocker should be setup_task");

    println!("Verified: work task has 1 blocker (setup task)");

    // Step 5 (optional): try to mark work_task as "In Progress" while setup is incomplete.
    // The block-warning lane will emit a warning event but allow the operation; if the
    // workspace has `block_warning_mode = "hard-block"` configured this will return an
    // error instead.
    let advance_result = client.tasks().update_status(&work_task.id, "In Progress").await;
    match advance_result {
        Ok(_) => println!("Status updated to InProgress (block-warning mode: warn)"),
        Err(TasksError::BlockedByIncompleteBlockers { blockers, .. }) => {
            eprintln!("Cannot advance: {} incomplete blockers (workspace block_warning_mode = hard-block)",
                blockers.len()
            );
        }
        Err(other) => return Err(Box::new(other)),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cycle_rejection_includes_path() {
        // This test runs against an in-memory `TasksClient` mock.
        let client = TasksClient::mock();
        let p = "test-project";

        let t1 = client.tasks().create_quick(p, "T1").await.unwrap();
        let t2 = client.tasks().create_quick(p, "T2").await.unwrap();
        let t3 = client.tasks().create_quick(p, "T3").await.unwrap();

        // Form a chain: t1 -> t2 -> t3
        client.dependencies().create_quick(&t1.id, &t2.id, DependencyType::Blocks).await.unwrap();
        client.dependencies().create_quick(&t2.id, &t3.id, DependencyType::Blocks).await.unwrap();

        // Try to close the cycle: t3 -> t1
        let err = client.dependencies().create_quick(&t3.id, &t1.id, DependencyType::Blocks).await
            .err().expect("expected cycle rejection");

        match err {
            TasksError::CycleDetected { cycle_path, .. } => {
                assert_eq!(cycle_path.len(), 3, "cycle should report all 3 nodes");
            }
            other => panic!("expected CycleDetected, got {:?}", other),
        }
    }
}
