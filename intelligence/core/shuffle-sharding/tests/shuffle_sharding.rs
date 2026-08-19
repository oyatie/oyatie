use intelligence_shuffle_sharding::{
    CellCandidate, ShuffleShardError, ShuffleShardRequest, select_shuffle_shard,
};

fn cell(cell_id: &str, pack: &str, region: &str) -> CellCandidate {
    CellCandidate {
        cell_id: cell_id.to_string(),
        pack: pack.to_string(),
        region: region.to_string(),
        accepts_new_tenants: true,
    }
}

fn base_request(tenant_id: &str) -> ShuffleShardRequest {
    ShuffleShardRequest {
        tenant_id: tenant_id.to_string(),
        shard_width: 3,
        placement_salt: "cell-assignment-v1".to_string(),
        required_pack: Some("pack-kr".to_string()),
        required_region: None,
        candidates: vec![
            cell("kr-cell-001", "pack-kr", "ap-northeast-2"),
            cell("kr-cell-002", "pack-kr", "ap-northeast-2"),
            cell("kr-cell-003", "pack-kr", "ap-northeast-2"),
            cell("kr-cell-004", "pack-kr", "ap-northeast-2"),
            cell("eu-cell-001", "pack-eu", "eu-central-1"),
        ],
    }
}

#[test]
fn selection_is_deterministic_for_same_tenant_salt_and_candidates() {
    let first = select_shuffle_shard(base_request("ten_acme")).unwrap();
    let second = select_shuffle_shard(base_request("ten_acme")).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.tenant_id, "ten_acme");
    assert_eq!(first.placement_salt, "cell-assignment-v1");
    assert_eq!(first.cell_ids.len(), 3);
}

#[test]
fn selection_changes_for_different_tenants() {
    let first = select_shuffle_shard(base_request("ten_acme")).unwrap();
    let second = select_shuffle_shard(base_request("ten_globex")).unwrap();

    assert_ne!(first.cell_ids, second.cell_ids);
}

#[test]
fn selection_filters_by_required_pack_and_region() {
    let mut request = base_request("ten_acme");
    request.required_region = Some("ap-northeast-2".to_string());

    let shard = select_shuffle_shard(request).unwrap();

    assert!(shard.cell_ids.iter().all(|id| id.starts_with("kr-cell-")));
}

#[test]
fn inactive_cells_are_not_selected() {
    let mut request = base_request("ten_acme");
    request.candidates[0].accepts_new_tenants = false;

    let shard = select_shuffle_shard(request).unwrap();

    assert!(!shard.cell_ids.contains(&"kr-cell-001".to_string()));
}

#[test]
fn rejects_duplicate_cell_ids() {
    let mut request = base_request("ten_acme");
    request
        .candidates
        .push(cell("kr-cell-001", "pack-kr", "ap-northeast-2"));

    let err = select_shuffle_shard(request).unwrap_err();

    assert_eq!(
        err,
        ShuffleShardError::DuplicateCellId("kr-cell-001".to_string())
    );
}

#[test]
fn rejects_insufficient_eligible_cells() {
    let mut request = base_request("ten_acme");
    request.shard_width = 5;

    let err = select_shuffle_shard(request).unwrap_err();

    assert_eq!(
        err,
        ShuffleShardError::NotEnoughEligibleCells {
            required: 5,
            available: 4
        }
    );
}
