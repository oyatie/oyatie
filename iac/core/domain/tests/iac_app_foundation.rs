// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use iac_domain::{
    CellDefinition, CellIsolationTier, CellTopologyPlan, GitOpsController, GitOpsDriftReport,
    GitOpsDriftVerdict, GitOpsEvidence, GitOpsEvidenceInput, GitOpsHealthStatus, GitOpsSyncStatus,
    LocalModuleReleaseStatus, LocalOpenTofuModuleCatalog, LocalOpenTofuModuleCatalogEntry,
    ModuleRegistry, OpenTofuModuleRef, OpenTofuModuleRelease, reconcile_gitops_drift,
};

fn tenant_namespace_release() -> OpenTofuModuleRelease {
    OpenTofuModuleRelease::new(
        "oyatie",
        "tenant-namespace",
        "opentofu",
        "1.0.0",
        "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/iac-app/tofu/modules/k8s-namespace-bootstrap?ref=v1.0.0",
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "evidence://iac-app/modules/tenant-namespace/1.0.0",
    )
    .expect("valid metadata-only module release")
}

fn module_release(name: &str, version: &str, digest_hex: char) -> OpenTofuModuleRelease {
    OpenTofuModuleRelease::new(
        "oyatie",
        name,
        "opentofu",
        version,
        format!(
            "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/iac-app/tofu/modules/{name}?ref=v{version}"
        ),
        format!("sha256:{}", digest_hex.to_string().repeat(64)),
        format!("evidence://iac-app/modules/{name}/{version}/local-foundation"),
    )
    .expect("valid local module release")
}

fn gitops_input(commit_sha: &str, evidence_ref: &str) -> GitOpsEvidenceInput {
    GitOpsEvidenceInput {
        controller: GitOpsController::ArgoCd,
        tenant_id: "ten_alpha".to_string(),
        cell_id: "cell-kr-seoul-1-a-001".to_string(),
        application_name: "iac-app-foundation".to_string(),
        repository_url: "https://git.oyatie.internal/oyatie/oyatie.git".to_string(),
        commit_sha: commit_sha.to_string(),
        sync_status: GitOpsSyncStatus::Synced,
        health_status: GitOpsHealthStatus::Healthy,
        evidence_ref: evidence_ref.to_string(),
    }
}

fn local_catalog_entry(name: &str) -> LocalOpenTofuModuleCatalogEntry {
    LocalOpenTofuModuleCatalogEntry::new(
        "oyatie",
        name,
        "opentofu",
        "0.1.0",
        format!("microservices/iac-app/tofu/modules/{name}"),
        format!("microservices/iac-app/tofu/modules/{name}/main.tofu"),
        LocalModuleReleaseStatus::LocalFoundationSkeleton,
        false,
        false,
        false,
        format!("evidence://iac-app/modules/{name}/0.1.0/local-foundation"),
    )
    .expect("valid local skeleton catalog entry")
}

#[test]
fn iac_app_module_registry_is_metadata_only_and_exactly_pinned() {
    let release = tenant_namespace_release();

    assert_eq!(release.namespace(), "oyatie");
    assert_eq!(release.name(), "tenant-namespace");
    assert_eq!(release.system(), "opentofu");
    assert_eq!(release.version(), "1.0.0");
    assert_eq!(
        release.digest(),
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    );
    assert_eq!(
        release.evidence_ref(),
        "evidence://iac-app/modules/tenant-namespace/1.0.0"
    );

    let debug = format!("{release:?}");
    assert!(!debug.contains("-----BEGIN"));
    assert!(!debug.contains("token="));
    assert!(!debug.contains("kubeconfig"));

    assert_eq!(
        OpenTofuModuleRelease::new(
            "oyatie",
            "tenant-namespace",
            "opentofu",
            "v1",
            "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/iac-app/tofu/modules/k8s-namespace-bootstrap?ref=v1",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "evidence://iac-app/modules/tenant-namespace/v1",
        )
        .unwrap_err(),
        iac_domain::CloudIacError::InvalidSemanticVersion
    );

    assert_eq!(
        OpenTofuModuleRelease::new(
            "oyatie",
            "tenant-namespace",
            "opentofu",
            "1.0.0",
            "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/iac-app/tofu/modules/k8s-namespace-bootstrap",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "evidence://iac-app/modules/tenant-namespace/1.0.0",
        )
        .unwrap_err(),
        iac_domain::CloudIacError::MissingSourceVersionPin
    );

    assert_eq!(
        OpenTofuModuleRelease::new(
            "oyatie",
            "tenant-namespace",
            "opentofu",
            "1.0.0",
            "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/iac-app/tofu/modules/k8s-namespace-bootstrap?ref=v1.0.0-malicious",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "evidence://iac-app/modules/tenant-namespace/1.0.0",
        )
        .unwrap_err(),
        iac_domain::CloudIacError::MissingSourceVersionPin
    );

    assert_eq!(
        OpenTofuModuleRelease::new(
            "oyatie",
            "tenant-namespace",
            "opentofu",
            "1.0.0",
            "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/iac-app/tofu/modules/k8s-namespace-bootstrap?ref=v1.0.0&ref=v1.0.0",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "evidence://iac-app/modules/tenant-namespace/1.0.0",
        )
        .unwrap_err(),
        iac_domain::CloudIacError::MissingSourceVersionPin
    );

    assert_eq!(
        OpenTofuModuleRelease::new(
            "oyatie",
            "tenant-namespace",
            "opentofu",
            "01.0.0",
            "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/iac-app/tofu/modules/k8s-namespace-bootstrap?ref=v01.0.0",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "evidence://iac-app/modules/tenant-namespace/01.0.0",
        )
        .unwrap_err(),
        iac_domain::CloudIacError::InvalidSemanticVersion
    );

    assert_eq!(
        OpenTofuModuleRelease::new(
            "oyatie",
            "tenant-namespace",
            "opentofu",
            "1.0.0",
            "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/iac-app/tofu/modules/k8s-namespace-bootstrap?ref=v1.0.0",
            "sha256:not-a-real-digest",
            "evidence://iac-app/modules/tenant-namespace/1.0.0",
        )
        .unwrap_err(),
        iac_domain::CloudIacError::InvalidDigest
    );

    assert_eq!(
        OpenTofuModuleRelease::new(
            "oyatie",
            "tenant-namespace",
            "opentofu",
            "1.0.0",
            "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/iac-app/tofu/modules/k8s-namespace-bootstrap?ref=v1.0.0",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "token=secret-in-evidence-ref",
        )
        .unwrap_err(),
        iac_domain::CloudIacError::EvidenceRefLooksSecretLike
    );
}

#[test]
fn iac_app_module_registry_accepts_local_relative_archive_locations() {
    let release = OpenTofuModuleRelease::new(
        "oyatie",
        "vpc",
        "opentofu",
        "0.1.0",
        "/artifacts/modules/oyatie-vpc-opentofu-0.1.0.zip",
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "evidence://iac-app/modules/vpc/0.1.0/local-foundation",
    )
    .expect("relative archive location is valid for registry download response");

    assert_eq!(
        release.source(),
        "/artifacts/modules/oyatie-vpc-opentofu-0.1.0.zip"
    );

    assert_eq!(
        OpenTofuModuleRelease::new(
            "oyatie",
            "vpc",
            "opentofu",
            "0.1.0",
            "/artifacts/modules/oyatie-vpc-opentofu-latest.zip",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "evidence://iac-app/modules/vpc/0.1.0/local-foundation",
        )
        .unwrap_err(),
        iac_domain::CloudIacError::MissingSourceVersionPin
    );

    assert_eq!(
        OpenTofuModuleRelease::new(
            "oyatie",
            "vpc",
            "opentofu",
            "0.1.0",
            "/artifacts/modules/../secret.zip",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "evidence://iac-app/modules/vpc/0.1.0/local-foundation",
        )
        .unwrap_err(),
        iac_domain::CloudIacError::MissingSourceVersionPin
    );
}

#[test]
fn iac_app_module_registry_accepts_opentofu_s3_and_gcs_archive_source_locations() {
    let s3 = OpenTofuModuleRelease::new(
        "oyatie",
        "vpc",
        "opentofu",
        "0.1.0",
        "s3::https://s3.amazonaws.com/oyatie-iac-app-modules/oyatie/vpc/0.1.0/oyatie-vpc-opentofu-0.1.0.zip",
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "evidence://iac-app/modules/vpc/0.1.0/object-source",
    )
    .expect("valid S3 archive object source");
    assert_eq!(
        s3.source(),
        "s3::https://s3.amazonaws.com/oyatie-iac-app-modules/oyatie/vpc/0.1.0/oyatie-vpc-opentofu-0.1.0.zip"
    );

    let gcs = OpenTofuModuleRelease::new(
        "oyatie",
        "vpc",
        "opentofu",
        "0.1.0",
        "gcs::https://www.googleapis.com/storage/v1/oyatie-iac-app-modules/oyatie/vpc/0.1.0/oyatie-vpc-opentofu-0.1.0.zip",
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "evidence://iac-app/modules/vpc/0.1.0/object-source",
    )
    .expect("valid GCS archive object source");
    assert!(gcs.source().starts_with("gcs::https://"));

    assert_eq!(
        OpenTofuModuleRelease::new(
            "oyatie",
            "vpc",
            "opentofu",
            "0.1.0",
            "s3::https://s3.amazonaws.com/oyatie-iac-app-modules/oyatie/vpc/0.1.0/oyatie-vpc-opentofu-0.2.0.zip",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "evidence://iac-app/modules/vpc/0.1.0/object-source",
        )
        .unwrap_err(),
        iac_domain::CloudIacError::MissingSourceVersionPin
    );
}

#[test]
fn iac_app_registry_rejects_duplicate_module_versions() {
    let release = tenant_namespace_release();
    let mut registry = ModuleRegistry::default();

    registry
        .publish(release.clone())
        .expect("first publish succeeds");
    assert_eq!(
        registry.publish(release.clone()).unwrap_err(),
        iac_domain::CloudIacError::DuplicateModuleVersion
    );

    let resolved = registry
        .resolve("oyatie", "tenant-namespace", "opentofu", "1.0.0")
        .expect("published module resolves");
    assert_eq!(resolved, &release);
}

#[test]
fn iac_app_registry_lists_versions_with_path_validation() {
    let mut registry = ModuleRegistry::default();
    registry
        .publish(module_release("vpc", "1.10.0", 'c'))
        .expect("vpc 1.10.0 registers");
    registry
        .publish(module_release("vpc", "1.0.0", 'a'))
        .expect("vpc 1.0.0 registers");
    registry
        .publish(module_release("vpc", "1.2.0", 'b'))
        .expect("vpc 1.2.0 registers");
    registry
        .publish(module_release("dns", "1.0.0", 'd'))
        .expect("dns registers");

    let versions = registry
        .versions("oyatie", "vpc", "opentofu")
        .expect("vpc versions resolve");

    assert_eq!(
        versions
            .iter()
            .map(|release| release.version())
            .collect::<Vec<_>>(),
        vec!["1.0.0", "1.2.0", "1.10.0"]
    );
    assert_eq!(
        registry
            .versions("oyatie", "missing", "opentofu")
            .unwrap_err(),
        iac_domain::CloudIacError::ModuleVersionNotFound
    );
    assert_eq!(
        registry
            .versions("oyatie", "../escape", "opentofu")
            .unwrap_err(),
        iac_domain::CloudIacError::InvalidModuleName
    );
    assert_eq!(
        registry
            .resolve("oyatie", "vpc", "opentofu", "1.2.0")
            .expect("specific vpc version resolves")
            .digest(),
        "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    );
    assert_eq!(
        registry
            .resolve("oyatie", "vpc", "opentofu", "latest")
            .unwrap_err(),
        iac_domain::CloudIacError::InvalidSemanticVersion
    );
}

#[test]
fn iac_app_cell_topology_requires_isolated_cells_and_module_refs() {
    let module_ref: OpenTofuModuleRef = tenant_namespace_release().module_ref();
    let cell = CellDefinition::new(
        "ten_alpha",
        "kr-seoul-1",
        "cell-kr-seoul-1-a-001",
        CellIsolationTier::Foundation,
        vec![module_ref.clone()],
        false,
    )
    .expect("cell with module refs and no default cross-cell traffic is valid");

    assert_eq!(cell.tenant_id(), "ten_alpha");
    assert_eq!(cell.region(), "kr-seoul-1");
    assert_eq!(cell.cell_id(), "cell-kr-seoul-1-a-001");
    assert_eq!(cell.isolation_tier(), CellIsolationTier::Foundation);
    assert!(!cell.default_cross_cell_traffic_allowed());
    assert_eq!(cell.module_refs(), &[module_ref]);

    let plan = CellTopologyPlan::new(
        "topology-kr-seoul-1-001",
        "kr-seoul-1",
        "evidence://iac-app/cell-topology/kr-seoul-1/001",
    )
    .expect("topology envelope is valid")
    .add_cell(cell.clone())
    .expect("first cell is accepted");

    assert_eq!(plan.region(), "kr-seoul-1");
    assert_eq!(plan.cells(), &[cell]);

    assert_eq!(
        CellDefinition::new(
            "ten_alpha",
            "kr-seoul-1",
            "cell-kr-seoul-1-a-002",
            CellIsolationTier::Foundation,
            Vec::new(),
            false,
        )
        .unwrap_err(),
        iac_domain::CloudIacError::MissingModuleRefs
    );

    assert_eq!(
        CellDefinition::new(
            "ten_alpha",
            "kr-seoul-1",
            "cell-kr-seoul-1-a-002",
            CellIsolationTier::Foundation,
            vec![tenant_namespace_release().module_ref()],
            true,
        )
        .unwrap_err(),
        iac_domain::CloudIacError::DefaultCrossCellTrafficForbidden
    );

    assert_eq!(
        CellDefinition::new(
            "ten_alpha/../../root",
            "kr-seoul-1",
            "cell-kr-seoul-1-a-002",
            CellIsolationTier::Foundation,
            vec![tenant_namespace_release().module_ref()],
            false,
        )
        .unwrap_err(),
        iac_domain::CloudIacError::InvalidTenantId
    );

    assert_eq!(
        CellDefinition::new(
            "ten_alpha",
            "KR_SEOUL_1",
            "cell-kr-seoul-1-a-002",
            CellIsolationTier::Foundation,
            vec![tenant_namespace_release().module_ref()],
            false,
        )
        .unwrap_err(),
        iac_domain::CloudIacError::RegionEmpty
    );

    assert_eq!(
        CellDefinition::new(
            "ten_alpha",
            "kr-seoul-1",
            "cell-kr-seoul-1-a-002/escape",
            CellIsolationTier::Foundation,
            vec![tenant_namespace_release().module_ref()],
            false,
        )
        .unwrap_err(),
        iac_domain::CloudIacError::CellIdEmpty
    );

    assert_eq!(
        plan.clone().add_cell(plan.cells()[0].clone()).unwrap_err(),
        iac_domain::CloudIacError::DuplicateCellId
    );
}

#[test]
fn iac_app_gitops_evidence_records_versioned_reconciliation_without_secrets() {
    let evidence = GitOpsEvidence::new(gitops_input(
        "abcdef1234567890abcdef1234567890abcdef12",
        "evidence://iac-app/gitops/iac-app-foundation/abcdef12",
    ))
    .expect("synced healthy reconciliation evidence is valid");

    assert_eq!(evidence.controller(), GitOpsController::ArgoCd);
    assert_eq!(evidence.tenant_id(), "ten_alpha");
    assert_eq!(evidence.cell_id(), "cell-kr-seoul-1-a-001");
    assert_eq!(
        evidence.commit_sha(),
        "abcdef1234567890abcdef1234567890abcdef12"
    );
    assert!(evidence.is_converged());

    let debug = format!("{evidence:?}");
    assert!(!debug.contains("kubeconfig"));
    assert!(!debug.contains("password="));
    assert!(!debug.contains("token="));

    assert_eq!(
        GitOpsEvidence::new(gitops_input(
            "dev",
            "evidence://iac-app/gitops/iac-app-foundation/dev",
        ))
        .unwrap_err(),
        iac_domain::CloudIacError::InvalidCommitSha
    );

    assert_eq!(
        GitOpsEvidence::new(gitops_input(
            "abcdef1234567890abcdef1234567890abcdef12",
            "kubeconfig: raw cluster admin data",
        ))
        .unwrap_err(),
        iac_domain::CloudIacError::EvidenceRefLooksSecretLike
    );
}

#[test]
fn iac_app_local_module_catalog_prevents_skeleton_false_greens() {
    let entries = vec![
        local_catalog_entry("cloud-account"),
        local_catalog_entry("dns"),
        local_catalog_entry("k8s-namespace-bootstrap"),
        local_catalog_entry("kms"),
        local_catalog_entry("secrets-bootstrap"),
        local_catalog_entry("vpc"),
    ];

    let catalog = LocalOpenTofuModuleCatalog::new(
        "iac-app-opentofu-modules-local-foundation",
        "microservices/iac-app/tofu/modules",
        entries,
    )
    .expect("current local catalog is coherent");

    assert_eq!(
        catalog.catalog_id(),
        "iac-app-opentofu-modules-local-foundation"
    );
    assert_eq!(
        catalog.source_path_root(),
        "microservices/iac-app/tofu/modules"
    );
    assert_eq!(catalog.module_count(), 6);
    assert_eq!(catalog.entries()[0].name(), "cloud-account");
    assert_eq!(
        catalog.entries()[0].release_status().as_str(),
        "local-foundation-skeleton"
    );
    assert!(!catalog.entries()[0].provider_resources_implemented());
    assert!(!catalog.entries()[0].outputs_materialized());
    assert!(!catalog.entries()[0].tests_present());

    assert_eq!(
        LocalOpenTofuModuleCatalog::new(
            "iac-app-opentofu-modules-local-foundation",
            "microservices/iac-app/tofu/modules",
            vec![local_catalog_entry("vpc"), local_catalog_entry("vpc")],
        )
        .unwrap_err(),
        iac_domain::CloudIacError::DuplicateModuleVersion
    );

    assert_eq!(
        LocalOpenTofuModuleCatalog::new(
            "iac-app-opentofu-modules-local-foundation",
            "microservices/iac-app/tofu/modules",
            vec![
                LocalOpenTofuModuleCatalogEntry::new(
                    "oyatie",
                    "vpc",
                    "opentofu",
                    "0.1.0",
                    "microservices/other/tofu/modules/vpc",
                    "microservices/other/tofu/modules/vpc/main.tofu",
                    LocalModuleReleaseStatus::LocalFoundationSkeleton,
                    false,
                    false,
                    false,
                    "evidence://iac-app/modules/vpc/0.1.0/local-foundation",
                )
                .expect("entry shape is valid before catalog root check")
            ],
        )
        .unwrap_err(),
        iac_domain::CloudIacError::CatalogPathOutsideRoot
    );

    assert_eq!(
        LocalOpenTofuModuleCatalogEntry::new(
            "oyatie",
            "vpc",
            "opentofu",
            "0.1.0",
            "microservices/iac-app/tofu/modules/vpc",
            "microservices/iac-app/tofu/modules/vpc/not-main.tofu",
            LocalModuleReleaseStatus::LocalFoundationSkeleton,
            false,
            false,
            false,
            "evidence://iac-app/modules/vpc/0.1.0/local-foundation",
        )
        .unwrap_err(),
        iac_domain::CloudIacError::CatalogMainFileInvalid
    );

    assert_eq!(
        LocalOpenTofuModuleCatalogEntry::new(
            "oyatie",
            "vpc",
            "opentofu",
            "0.1.0",
            "microservices/iac-app/tofu/modules/vpc",
            "microservices/iac-app/tofu/modules/vpc/main.tofu",
            LocalModuleReleaseStatus::LocalFoundationSkeleton,
            true,
            false,
            false,
            "evidence://iac-app/modules/vpc/0.1.0/local-foundation",
        )
        .unwrap_err(),
        iac_domain::CloudIacError::CatalogSkeletonOverclaim
    );
}

// ---------------------------------------------------------------------------
// GitOps drift reconciliation tests
// ---------------------------------------------------------------------------

const SHA_DESIRED: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SHA_OBSERVED: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn desired_evidence(commit_sha: &str) -> GitOpsEvidence {
    GitOpsEvidence::new(GitOpsEvidenceInput {
        controller: GitOpsController::ArgoCd,
        tenant_id: "ten_alpha".to_string(),
        cell_id: "cell-kr-seoul-1-a-001".to_string(),
        application_name: "iac-app-foundation".to_string(),
        repository_url: "https://git.oyatie.internal/oyatie/oyatie.git".to_string(),
        commit_sha: commit_sha.to_string(),
        sync_status: GitOpsSyncStatus::Synced,
        health_status: GitOpsHealthStatus::Healthy,
        evidence_ref: "evidence://iac-app/gitops/iac-app-foundation/desired".to_string(),
    })
    .expect("desired evidence is valid")
}

fn observed_evidence(
    commit_sha: &str,
    sync_status: GitOpsSyncStatus,
    health_status: GitOpsHealthStatus,
) -> GitOpsEvidence {
    GitOpsEvidence::new(GitOpsEvidenceInput {
        controller: GitOpsController::ArgoCd,
        tenant_id: "ten_alpha".to_string(),
        cell_id: "cell-kr-seoul-1-a-001".to_string(),
        application_name: "iac-app-foundation".to_string(),
        repository_url: "https://git.oyatie.internal/oyatie/oyatie.git".to_string(),
        commit_sha: commit_sha.to_string(),
        sync_status,
        health_status,
        evidence_ref: "evidence://iac-app/gitops/iac-app-foundation/observed".to_string(),
    })
    .expect("observed evidence is valid")
}

#[test]
fn drift_all_aligned_is_in_sync() {
    let desired = desired_evidence(SHA_DESIRED);
    let observed = observed_evidence(
        SHA_DESIRED,
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Healthy,
    );
    let report = reconcile_gitops_drift(&desired, &observed);
    assert_eq!(report.verdict, GitOpsDriftVerdict::InSync);
    assert_eq!(report.observed_commit_sha, SHA_DESIRED);
    assert_eq!(report.observed_sync_status, GitOpsSyncStatus::Synced);
    assert_eq!(report.observed_health_status, GitOpsHealthStatus::Healthy);
    assert_eq!(report.tenant_id, "ten_alpha");
    assert_eq!(report.cell_id, "cell-kr-seoul-1-a-001");
    assert_eq!(report.application_name, "iac-app-foundation");
    assert_eq!(report.controller, GitOpsController::ArgoCd);
}

#[test]
fn drift_commit_sha_mismatch() {
    let desired = desired_evidence(SHA_DESIRED);
    let observed = observed_evidence(
        SHA_OBSERVED,
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Healthy,
    );
    let report = reconcile_gitops_drift(&desired, &observed);
    assert_eq!(report.verdict, GitOpsDriftVerdict::DriftedCommit);
    assert_eq!(report.observed_commit_sha, SHA_OBSERVED);
}

#[test]
fn drift_sync_status_out_of_sync() {
    let desired = desired_evidence(SHA_DESIRED);
    let observed = observed_evidence(
        SHA_DESIRED,
        GitOpsSyncStatus::OutOfSync,
        GitOpsHealthStatus::Healthy,
    );
    let report = reconcile_gitops_drift(&desired, &observed);
    assert_eq!(report.verdict, GitOpsDriftVerdict::DriftedSyncStatus);
    assert_eq!(report.observed_sync_status, GitOpsSyncStatus::OutOfSync);
}

#[test]
fn drift_sync_status_unknown() {
    let desired = desired_evidence(SHA_DESIRED);
    let observed = observed_evidence(
        SHA_DESIRED,
        GitOpsSyncStatus::Unknown,
        GitOpsHealthStatus::Healthy,
    );
    let report = reconcile_gitops_drift(&desired, &observed);
    assert_eq!(report.verdict, GitOpsDriftVerdict::DriftedSyncStatus);
    assert_eq!(report.observed_sync_status, GitOpsSyncStatus::Unknown);
}

#[test]
fn drift_degraded_health() {
    let desired = desired_evidence(SHA_DESIRED);
    let observed = observed_evidence(
        SHA_DESIRED,
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Degraded,
    );
    let report = reconcile_gitops_drift(&desired, &observed);
    assert_eq!(report.verdict, GitOpsDriftVerdict::DegradedHealth);
    assert_eq!(report.observed_health_status, GitOpsHealthStatus::Degraded);
}

#[test]
fn drift_identity_mismatch_beats_commit_drift() {
    // Even though commit SHAs also differ, IdentityMismatch takes precedence.
    let desired = desired_evidence(SHA_DESIRED);
    // Observed has a different application_name (identity mismatch) AND different SHA.
    let observed = GitOpsEvidence::new(GitOpsEvidenceInput {
        controller: GitOpsController::ArgoCd,
        tenant_id: "ten_alpha".to_string(),
        cell_id: "cell-kr-seoul-1-a-001".to_string(),
        application_name: "different-application".to_string(),
        repository_url: "https://git.oyatie.internal/oyatie/oyatie.git".to_string(),
        commit_sha: SHA_OBSERVED.to_string(),
        sync_status: GitOpsSyncStatus::Synced,
        health_status: GitOpsHealthStatus::Healthy,
        evidence_ref: "evidence://iac-app/gitops/different-application/observed".to_string(),
    })
    .expect("mismatched-identity observed evidence is valid");

    let report = reconcile_gitops_drift(&desired, &observed);
    assert_eq!(report.verdict, GitOpsDriftVerdict::IdentityMismatch);
    // Report carries observed fields regardless
    assert_eq!(report.application_name, "different-application");
    assert_eq!(report.observed_commit_sha, SHA_OBSERVED);
}
