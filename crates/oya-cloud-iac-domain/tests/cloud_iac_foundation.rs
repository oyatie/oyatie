// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_cloud_iac_domain::{
    CellDefinition, CellIsolationTier, CellTopologyPlan, GitOpsController, GitOpsEvidence,
    GitOpsEvidenceInput, GitOpsHealthStatus, GitOpsSyncStatus, LocalModuleReleaseStatus,
    LocalOpenTofuModuleCatalog, LocalOpenTofuModuleCatalogEntry, ModuleRegistry, OpenTofuModuleRef,
    OpenTofuModuleRelease,
};

fn tenant_namespace_release() -> OpenTofuModuleRelease {
    OpenTofuModuleRelease::new(
        "oyatie",
        "tenant-namespace",
        "opentofu",
        "1.0.0",
        "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/cloud-iac/tofu/modules/k8s-namespace-bootstrap?ref=v1.0.0",
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "evidence://cloud-iac/modules/tenant-namespace/1.0.0",
    )
    .expect("valid metadata-only module release")
}

fn gitops_input(commit_sha: &str, evidence_ref: &str) -> GitOpsEvidenceInput {
    GitOpsEvidenceInput {
        controller: GitOpsController::ArgoCd,
        tenant_id: "ten_alpha".to_string(),
        cell_id: "cell-kr-seoul-1-a-001".to_string(),
        application_name: "cloud-iac-foundation".to_string(),
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
        format!("microservices/cloud-iac/tofu/modules/{name}"),
        format!("microservices/cloud-iac/tofu/modules/{name}/main.tofu"),
        LocalModuleReleaseStatus::LocalFoundationSkeleton,
        false,
        false,
        false,
        format!("evidence://cloud-iac/modules/{name}/0.1.0/local-foundation"),
    )
    .expect("valid local skeleton catalog entry")
}

#[test]
fn cloud_iac_module_registry_is_metadata_only_and_exactly_pinned() {
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
        "evidence://cloud-iac/modules/tenant-namespace/1.0.0"
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
            "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/cloud-iac/tofu/modules/k8s-namespace-bootstrap?ref=v1",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "evidence://cloud-iac/modules/tenant-namespace/v1",
        )
        .unwrap_err(),
        oya_cloud_iac_domain::CloudIacError::InvalidSemanticVersion
    );

    assert_eq!(
        OpenTofuModuleRelease::new(
            "oyatie",
            "tenant-namespace",
            "opentofu",
            "1.0.0",
            "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/cloud-iac/tofu/modules/k8s-namespace-bootstrap",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "evidence://cloud-iac/modules/tenant-namespace/1.0.0",
        )
        .unwrap_err(),
        oya_cloud_iac_domain::CloudIacError::MissingSourceVersionPin
    );

    assert_eq!(
        OpenTofuModuleRelease::new(
            "oyatie",
            "tenant-namespace",
            "opentofu",
            "1.0.0",
            "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/cloud-iac/tofu/modules/k8s-namespace-bootstrap?ref=v1.0.0-malicious",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "evidence://cloud-iac/modules/tenant-namespace/1.0.0",
        )
        .unwrap_err(),
        oya_cloud_iac_domain::CloudIacError::MissingSourceVersionPin
    );

    assert_eq!(
        OpenTofuModuleRelease::new(
            "oyatie",
            "tenant-namespace",
            "opentofu",
            "1.0.0",
            "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/cloud-iac/tofu/modules/k8s-namespace-bootstrap?ref=v1.0.0&ref=v1.0.0",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "evidence://cloud-iac/modules/tenant-namespace/1.0.0",
        )
        .unwrap_err(),
        oya_cloud_iac_domain::CloudIacError::MissingSourceVersionPin
    );

    assert_eq!(
        OpenTofuModuleRelease::new(
            "oyatie",
            "tenant-namespace",
            "opentofu",
            "01.0.0",
            "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/cloud-iac/tofu/modules/k8s-namespace-bootstrap?ref=v01.0.0",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "evidence://cloud-iac/modules/tenant-namespace/01.0.0",
        )
        .unwrap_err(),
        oya_cloud_iac_domain::CloudIacError::InvalidSemanticVersion
    );

    assert_eq!(
        OpenTofuModuleRelease::new(
            "oyatie",
            "tenant-namespace",
            "opentofu",
            "1.0.0",
            "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/cloud-iac/tofu/modules/k8s-namespace-bootstrap?ref=v1.0.0",
            "sha256:not-a-real-digest",
            "evidence://cloud-iac/modules/tenant-namespace/1.0.0",
        )
        .unwrap_err(),
        oya_cloud_iac_domain::CloudIacError::InvalidDigest
    );

    assert_eq!(
        OpenTofuModuleRelease::new(
            "oyatie",
            "tenant-namespace",
            "opentofu",
            "1.0.0",
            "git::https://git.oyatie.internal/oyatie/oyatie.git//microservices/cloud-iac/tofu/modules/k8s-namespace-bootstrap?ref=v1.0.0",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "token=secret-in-evidence-ref",
        )
        .unwrap_err(),
        oya_cloud_iac_domain::CloudIacError::EvidenceRefLooksSecretLike
    );
}

#[test]
fn cloud_iac_registry_rejects_duplicate_module_versions() {
    let release = tenant_namespace_release();
    let mut registry = ModuleRegistry::default();

    registry
        .publish(release.clone())
        .expect("first publish succeeds");
    assert_eq!(
        registry.publish(release.clone()).unwrap_err(),
        oya_cloud_iac_domain::CloudIacError::DuplicateModuleVersion
    );

    let resolved = registry
        .resolve("oyatie", "tenant-namespace", "opentofu", "1.0.0")
        .expect("published module resolves");
    assert_eq!(resolved, &release);
}

#[test]
fn cloud_iac_cell_topology_requires_isolated_cells_and_module_refs() {
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
        "evidence://cloud-iac/cell-topology/kr-seoul-1/001",
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
        oya_cloud_iac_domain::CloudIacError::MissingModuleRefs
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
        oya_cloud_iac_domain::CloudIacError::DefaultCrossCellTrafficForbidden
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
        oya_cloud_iac_domain::CloudIacError::InvalidTenantId
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
        oya_cloud_iac_domain::CloudIacError::RegionEmpty
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
        oya_cloud_iac_domain::CloudIacError::CellIdEmpty
    );

    assert_eq!(
        plan.clone().add_cell(plan.cells()[0].clone()).unwrap_err(),
        oya_cloud_iac_domain::CloudIacError::DuplicateCellId
    );
}

#[test]
fn cloud_iac_gitops_evidence_records_versioned_reconciliation_without_secrets() {
    let evidence = GitOpsEvidence::new(gitops_input(
        "abcdef1234567890abcdef1234567890abcdef12",
        "evidence://cloud-iac/gitops/cloud-iac-foundation/abcdef12",
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
            "evidence://cloud-iac/gitops/cloud-iac-foundation/dev",
        ))
        .unwrap_err(),
        oya_cloud_iac_domain::CloudIacError::InvalidCommitSha
    );

    assert_eq!(
        GitOpsEvidence::new(gitops_input(
            "abcdef1234567890abcdef1234567890abcdef12",
            "kubeconfig: raw cluster admin data",
        ))
        .unwrap_err(),
        oya_cloud_iac_domain::CloudIacError::EvidenceRefLooksSecretLike
    );
}

#[test]
fn cloud_iac_local_module_catalog_prevents_skeleton_false_greens() {
    let entries = vec![
        local_catalog_entry("cloud-account"),
        local_catalog_entry("dns"),
        local_catalog_entry("k8s-namespace-bootstrap"),
        local_catalog_entry("kms"),
        local_catalog_entry("secrets-bootstrap"),
        local_catalog_entry("vpc"),
    ];

    let catalog = LocalOpenTofuModuleCatalog::new(
        "cloud-iac-opentofu-modules-local-foundation",
        "microservices/cloud-iac/tofu/modules",
        entries,
    )
    .expect("current local catalog is coherent");

    assert_eq!(
        catalog.catalog_id(),
        "cloud-iac-opentofu-modules-local-foundation"
    );
    assert_eq!(
        catalog.source_path_root(),
        "microservices/cloud-iac/tofu/modules"
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
            "cloud-iac-opentofu-modules-local-foundation",
            "microservices/cloud-iac/tofu/modules",
            vec![local_catalog_entry("vpc"), local_catalog_entry("vpc")],
        )
        .unwrap_err(),
        oya_cloud_iac_domain::CloudIacError::DuplicateModuleVersion
    );

    assert_eq!(
        LocalOpenTofuModuleCatalog::new(
            "cloud-iac-opentofu-modules-local-foundation",
            "microservices/cloud-iac/tofu/modules",
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
                    "evidence://cloud-iac/modules/vpc/0.1.0/local-foundation",
                )
                .expect("entry shape is valid before catalog root check")
            ],
        )
        .unwrap_err(),
        oya_cloud_iac_domain::CloudIacError::CatalogPathOutsideRoot
    );

    assert_eq!(
        LocalOpenTofuModuleCatalogEntry::new(
            "oyatie",
            "vpc",
            "opentofu",
            "0.1.0",
            "microservices/cloud-iac/tofu/modules/vpc",
            "microservices/cloud-iac/tofu/modules/vpc/not-main.tofu",
            LocalModuleReleaseStatus::LocalFoundationSkeleton,
            false,
            false,
            false,
            "evidence://cloud-iac/modules/vpc/0.1.0/local-foundation",
        )
        .unwrap_err(),
        oya_cloud_iac_domain::CloudIacError::CatalogMainFileInvalid
    );

    assert_eq!(
        LocalOpenTofuModuleCatalogEntry::new(
            "oyatie",
            "vpc",
            "opentofu",
            "0.1.0",
            "microservices/cloud-iac/tofu/modules/vpc",
            "microservices/cloud-iac/tofu/modules/vpc/main.tofu",
            LocalModuleReleaseStatus::LocalFoundationSkeleton,
            true,
            false,
            false,
            "evidence://cloud-iac/modules/vpc/0.1.0/local-foundation",
        )
        .unwrap_err(),
        oya_cloud_iac_domain::CloudIacError::CatalogSkeletonOverclaim
    );
}
