use dependency_declarations_reconcile::*;

pub fn digest(value: &str) -> DigestV1 {
    DigestV1::of(value.as_bytes())
}

pub fn source(
    component: LifecycleComponentV1,
    channel: LifecycleChannelV1,
    maturity: SourceMaturityV1,
    revision: &str,
) -> LifecycleSourceV1 {
    LifecycleSourceV1::try_new(
        LifecycleSourceDescriptorV1::try_new(
            "rust-lang",
            component,
            channel,
            revision,
            "release-object",
            LifecycleSourceScopeV1::Global,
            maturity,
        )
        .unwrap(),
        1024,
        digest(&format!("{revision}-object")),
        digest("schema-v1"),
    )
    .unwrap()
}

fn tool(name: &str, version: &str, commit: &str) -> ToolIdentityV1 {
    ToolIdentityV1::try_new(
        name,
        version,
        commit,
        "aarch64-apple-darwin",
        digest(&format!("{name}-{commit}")),
    )
    .unwrap()
}

pub fn profile(
    role: ToolchainRoleV1,
    version: RustVersionV1,
    channel: LifecycleChannelV1,
    maturity: SourceMaturityV1,
    rustc_commit: &str,
    cargo_commit: &str,
) -> ToolchainProfileV1 {
    let qualification = match role {
        ToolchainRoleV1::DeclaredMsrvCompatibility => ToolchainQualificationV1::Compatibility {
            qualification_receipt_sha256: digest(&format!("{rustc_commit}-msrv")),
        },
        ToolchainRoleV1::QualifiedStableExecution => ToolchainQualificationV1::Production {
            qualification_receipt_sha256: digest(&format!("{rustc_commit}-stable")),
        },
        ToolchainRoleV1::BetaShadow | ToolchainRoleV1::NightlyShadow => {
            ToolchainQualificationV1::Shadow {
                observation_receipt_sha256: digest(&format!("{rustc_commit}-shadow")),
            }
        }
    };
    profile_with_qualification(
        role,
        version,
        channel,
        maturity,
        rustc_commit,
        cargo_commit,
        qualification,
    )
    .unwrap()
}

pub fn profile_with_qualification(
    role: ToolchainRoleV1,
    version: RustVersionV1,
    channel: LifecycleChannelV1,
    maturity: SourceMaturityV1,
    rustc_commit: &str,
    cargo_commit: &str,
    qualification: ToolchainQualificationV1,
) -> Result<ToolchainProfileV1, LifecycleFailureV1> {
    let tool_version = format!(
        "{}.{}.{}",
        version.major(),
        version.minor(),
        version.patch()
    );
    let tools = ToolchainToolsV1::try_new(
        tool("rustc", &tool_version, rustc_commit),
        tool("cargo", &tool_version, cargo_commit),
        tool("rustfmt", &tool_version, rustc_commit),
        tool("clippy", &tool_version, rustc_commit),
    )
    .unwrap();
    ToolchainProfileV1::try_new(
        role,
        version,
        source(
            LifecycleComponentV1::RustDistribution,
            channel,
            maturity,
            rustc_commit,
        ),
        tools,
        qualification,
        "LLVM 23.1.0",
        vec![
            ToolchainTargetV1::try_new(
                "aarch64-apple-darwin",
                digest(&format!("{rustc_commit}-std")),
                digest(&format!("{rustc_commit}-components")),
            )
            .unwrap(),
        ],
    )
}

pub fn release_item(
    source: &LifecycleSourceV1,
    key: &str,
    kind: ReleaseItemKindV1,
) -> ReleaseItemV1 {
    ReleaseItemV1::try_new(
        source,
        key,
        format!("upstream/{key}"),
        kind,
        digest(&format!("{key}-content")),
    )
    .unwrap()
}

pub fn extraction(
    source: &LifecycleSourceV1,
    qualification: ReleaseExtractionQualificationV1,
) -> ReleaseExtractionProfileV1 {
    let extractor = ArtifactIdentityV1::try_new(
        "release-note-extractor",
        "1",
        "extractor-commit",
        digest("extractor-source"),
        digest("extractor-binary"),
    )
    .unwrap();
    ReleaseExtractionProfileV1::new(source, extractor, digest("release-grammar"), qualification)
}

pub fn qualified_batch(source: LifecycleSourceV1, items: &[ReleaseItemV1]) -> ReleaseSourceBatchV1 {
    let extraction = extraction(
        &source,
        ReleaseExtractionQualificationV1::Qualified {
            qualification_receipt_sha256: digest("qualified-extraction"),
        },
    );
    ReleaseSourceBatchV1::try_from_items(
        source,
        extraction,
        items,
        digest("extraction-observation"),
    )
    .unwrap()
}

pub fn disposition(item: &ReleaseItemV1, decision: ReleaseDecisionV1) -> ReleaseDispositionV1 {
    ReleaseDispositionV1::try_new(
        item.identity_sha256(),
        "product-owner",
        decision,
        ReleaseDispositionEvidenceV1::new(
            digest("rationale"),
            ReleaseAffectedUnitsV1::try_new(1, 32, digest("affected-units")).unwrap(),
            ReleaseMsrvEffectV1::NoChange {
                evidence_sha256: digest("msrv-evidence"),
            },
            digest("evidence"),
            ReevaluationTriggerV1::OnUpstreamChange,
        ),
    )
    .unwrap()
}
