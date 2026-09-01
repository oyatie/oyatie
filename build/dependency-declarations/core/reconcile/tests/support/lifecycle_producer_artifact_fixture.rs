use super::{advisory::*, dependency_graph::*, lifecycle_support::*};
use dependency_declarations_reconcile::*;

pub(super) struct ArtifactFixture {
    pub(super) envelope: FactEnvelopeV1,
    released: ReleaseLedgerV1,
    pub(super) preview: ReleaseLedgerV1,
    pub(super) advisories: AdvisoryLedgerV1,
    dependencies: DependencyGraphV1,
    pub(super) toolchains: ToolchainMatrixV1,
    channels: ToolchainChannelSnapshotV1,
}

impl ArtifactFixture {
    pub(super) fn new() -> Self {
        let toolchains = toolchain_matrix();
        let envelope = artifact_envelope(
            toolchains.identity_sha256(),
            "repository-revision",
            "lifecycle-artifact-producer",
            100,
            300,
        );
        let graph_envelope = artifact_envelope(
            toolchains.identity_sha256(),
            "repository-revision",
            "dependency-graph-producer",
            100,
            300,
        );
        let dependencies = DependencyGraphV1::try_new(
            graph_envelope,
            vec![node(
                "h2-package",
                DependencyGraphNodeKindV1::CargoPackage,
                None,
            )],
            Vec::new(),
            continue_dependency_graph_construction,
        )
        .unwrap();
        let channels = channel_snapshot(&toolchains);
        Self {
            envelope,
            released: released_ledger(),
            preview: preview_ledger(true),
            advisories: advisory_ledger(true),
            dependencies,
            toolchains,
            channels,
        }
    }

    pub(super) fn try_build(self) -> Result<LifecycleProducerArtifactV1, LifecycleFailureV1> {
        LifecycleProducerArtifactV1::try_new(
            self.envelope,
            self.released,
            self.preview,
            self.advisories,
            self.dependencies,
            self.toolchains,
            self.channels,
        )
    }
}

pub(super) fn artifact_envelope(
    toolchain_sha256: DigestV1,
    repository_revision: &str,
    producer: &str,
    observed_at: u64,
    fresh_until: u64,
) -> FactEnvelopeV1 {
    let scope = FactTemporalScopeV1::try_new(
        "oyatie/oyatie",
        digest(repository_revision),
        digest("repository-snapshot"),
        digest("cargo-and-buck-configurations"),
        toolchain_sha256,
        digest(producer),
        digest(&format!("{producer}-schema")),
    )
    .unwrap();
    let temporal = FactTemporalIdentityV1::try_new(
        scope,
        LifecycleTimestampV1::from_unix_seconds(observed_at),
        LifecycleTimestampV1::from_unix_seconds(fresh_until),
    )
    .unwrap();
    FactEnvelopeV1::new(
        FactEvidenceClassesV1::try_new(vec![
            FactEvidenceClassV1::Declared,
            FactEvidenceClassV1::Proven,
        ])
        .unwrap(),
        FactCertaintyV1::Exact,
        FactCoverageV1::CompleteForScope {
            scope_sha256: digest("complete-lifecycle-scope"),
            exclusions_sha256: digest("explicit-exclusions"),
        },
        temporal,
        digest(&format!("{producer}-qualification")),
        digest(&format!("{producer}-derivation")),
    )
}

pub(super) fn released_ledger() -> ReleaseLedgerV1 {
    let source = source(
        LifecycleComponentV1::Rust,
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "rust-1.98.0",
    );
    let item = release_item(&source, "rust-1.98.0/item", ReleaseItemKindV1::Compiler);
    ReleaseLedgerV1::try_new(
        vec![qualified_batch(source, std::slice::from_ref(&item))],
        vec![item.clone()],
        vec![disposition(&item, ReleaseDecisionV1::Adopt)],
        continue_release_ledger,
    )
    .unwrap()
}

pub(super) fn preview_ledger(qualified: bool) -> ReleaseLedgerV1 {
    let source = source(
        LifecycleComponentV1::Rust,
        LifecycleChannelV1::Nightly,
        SourceMaturityV1::Provisional,
        "nightly-0dfb098f3",
    );
    let item = release_item(&source, "nightly/item", ReleaseItemKindV1::Compiler);
    let extraction = extraction(
        &source,
        if qualified {
            ReleaseExtractionQualificationV1::Qualified {
                qualification_receipt_sha256: digest("preview-extraction-qualified"),
            }
        } else {
            ReleaseExtractionQualificationV1::Candidate {
                observation_sha256: digest("preview-extraction-candidate"),
            }
        },
    );
    let batch = ReleaseSourceBatchV1::try_from_items(
        source,
        extraction,
        std::slice::from_ref(&item),
        digest("preview-extraction-observation"),
        continue_release_source_batch,
    )
    .unwrap();
    ReleaseLedgerV1::try_new(
        vec![batch],
        vec![item.clone()],
        vec![disposition(&item, ReleaseDecisionV1::Benchmark)],
        continue_release_ledger,
    )
    .unwrap()
}

pub(super) fn mixed_preview_ledger() -> ReleaseLedgerV1 {
    let released_source = source(
        LifecycleComponentV1::Rust,
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "released-source-in-preview",
    );
    let preview_source = source(
        LifecycleComponentV1::Rust,
        LifecycleChannelV1::Nightly,
        SourceMaturityV1::Provisional,
        "provisional-source-in-preview",
    );
    let released_item = release_item(
        &released_source,
        "released-item-in-preview",
        ReleaseItemKindV1::Compiler,
    );
    let preview_item = release_item(
        &preview_source,
        "provisional-item-in-preview",
        ReleaseItemKindV1::Compiler,
    );
    ReleaseLedgerV1::try_new(
        vec![
            qualified_batch(released_source, std::slice::from_ref(&released_item)),
            qualified_batch(preview_source, std::slice::from_ref(&preview_item)),
        ],
        vec![released_item.clone(), preview_item.clone()],
        vec![
            disposition(&released_item, ReleaseDecisionV1::Benchmark),
            disposition(&preview_item, ReleaseDecisionV1::Benchmark),
        ],
        continue_release_ledger,
    )
    .unwrap()
}

pub(super) fn advisory_ledger(complete: bool) -> AdvisoryLedgerV1 {
    let rustsec = identifier(AdvisoryNamespaceV1::RustSec, "RUSTSEC-2026-0258");
    let ghsa = identifier(AdvisoryNamespaceV1::Ghsa, "GHSA-q83h-524g-xf6h");
    let affected = if complete {
        complete_h2("0.4.16")
    } else {
        AdvisoryAffectedSetV1::reference_only(digest("reference-only"))
    };
    let record = active_record(
        record_source(
            LifecycleComponentV1::RustSec,
            AdvisoryAuthorityV1::RustSec,
            "rustsec-revision",
            qualified(),
        ),
        rustsec,
        vec![ghsa],
        affected,
        200,
    );
    AdvisoryLedgerV1::try_normalize(vec![record], continue_advisory_normalization).unwrap()
}

fn toolchain_matrix() -> ToolchainMatrixV1 {
    let v98 = RustVersionV1::try_new(1, 98, 0).unwrap();
    ToolchainMatrixV1::try_new(
        profile(
            ToolchainRoleV1::DeclaredMsrvCompatibility,
            v98,
            LifecycleChannelV1::Stable,
            SourceMaturityV1::Released,
            "88d9e12ae",
            "797e8a9bc",
        ),
        profile(
            ToolchainRoleV1::QualifiedStableExecution,
            v98,
            LifecycleChannelV1::Stable,
            SourceMaturityV1::Released,
            "88d9e12ae",
            "797e8a9bc",
        ),
        profile(
            ToolchainRoleV1::BetaShadow,
            RustVersionV1::try_new(1, 99, 0).unwrap(),
            LifecycleChannelV1::Beta,
            SourceMaturityV1::Provisional,
            "f47d5bb13",
            "eb98b54bc",
        ),
        profile(
            ToolchainRoleV1::NightlyShadow,
            RustVersionV1::try_new(1, 100, 0).unwrap(),
            LifecycleChannelV1::Nightly,
            SourceMaturityV1::Provisional,
            "bff8e12ff",
            "e8cb624d5",
        ),
    )
    .unwrap()
}

fn channel_snapshot(toolchains: &ToolchainMatrixV1) -> ToolchainChannelSnapshotV1 {
    let beta = profile(
        ToolchainRoleV1::BetaShadow,
        RustVersionV1::try_new(1, 99, 0).unwrap(),
        LifecycleChannelV1::Beta,
        SourceMaturityV1::Provisional,
        "cbae9b4ca",
        "cargo-beta-head",
    );
    let nightly = profile(
        ToolchainRoleV1::NightlyShadow,
        RustVersionV1::try_new(1, 100, 0).unwrap(),
        LifecycleChannelV1::Nightly,
        SourceMaturityV1::Provisional,
        "0dfb098f3",
        "cargo-nightly-head",
    );
    ToolchainChannelSnapshotV1::try_new(
        ToolchainChannelHeadV1::new(
            toolchains.stable().material().clone(),
            LifecycleTimestampV1::from_unix_seconds(100),
        ),
        ToolchainChannelHeadV1::new(
            beta.material().clone(),
            LifecycleTimestampV1::from_unix_seconds(180),
        ),
        ToolchainChannelHeadV1::new(
            nightly.material().clone(),
            LifecycleTimestampV1::from_unix_seconds(190),
        ),
        LifecycleTimestampV1::from_unix_seconds(200),
        ToolchainChannelSnapshotEvidenceV1::new(
            digest("channel-provider"),
            digest("channel-schema"),
            digest("channel-source-snapshot"),
            digest("channel-completeness"),
        ),
    )
    .unwrap()
}
