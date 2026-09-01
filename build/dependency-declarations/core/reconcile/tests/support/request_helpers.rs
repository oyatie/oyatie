use super::*;

pub(super) fn tree(role: TreeRoleV1, manifest: &str, entry: &str) -> InputTreeV1 {
    let entry = tree_entry(entry, entry.as_bytes());
    InputTreeV1::try_from_entries(
        role,
        CanonicalPathV1::try_new(manifest).unwrap(),
        vec![entry],
    )
    .unwrap()
}

pub(super) fn tree_entry(path: &str, bytes: &[u8]) -> TreeEntryV1 {
    TreeEntryV1::new(
        CanonicalPathV1::try_new(path).unwrap(),
        TreeFileModeV1::Regular,
        u64::try_from(bytes.len()).unwrap(),
        digest(bytes),
    )
}

pub(super) fn entry_for_file(file: &InputFileV1) -> TreeEntryV1 {
    TreeEntryV1::new(
        file.path().clone(),
        TreeFileModeV1::Regular,
        file.length_bytes(),
        file.sha256(),
    )
}

pub(super) fn platform(name: &str, triple: &str, execution: bool) -> PlatformIdentityV1 {
    PlatformIdentityV1::try_new(
        name,
        triple,
        format!("//platform:{name}-select"),
        format!("//platform:{name}"),
        execution,
    )
    .unwrap()
}

pub(super) fn artifact(name: &str, version: &str) -> ArtifactIdentityV1 {
    ArtifactIdentityV1::try_new(
        name,
        version,
        format!("{name}-revision"),
        digest(format!("{name}-source").as_bytes()),
        digest(format!("{name}-artifact").as_bytes()),
    )
    .unwrap()
}

pub(super) fn buck_consumer_profile(variation: BuckConsumerVariation) -> BuckConsumerProfileV1 {
    let version = |field| {
        if variation == field { "changed" } else { "v1" }
    };
    let bytes = |field, baseline: &'static [u8]| {
        if variation == field {
            b"changed".as_slice()
        } else {
            baseline
        }
    };
    BuckConsumerProfileV1::try_new(
        artifact("buck2", version(BuckConsumerVariation::Buck2)),
        artifact("buck2-prelude", version(BuckConsumerVariation::Prelude)),
        digest(bytes(BuckConsumerVariation::Rules, b"owned rules")),
        digest(bytes(BuckConsumerVariation::Toolchain, b"buck toolchain")),
        digest(bytes(BuckConsumerVariation::CellConfig, b"cell config")),
        digest(bytes(BuckConsumerVariation::BuckConfig, b"buck config")),
        digest(bytes(
            BuckConsumerVariation::QualificationPlan,
            b"configured query and representative consumption plan",
        )),
    )
    .unwrap()
}

pub(super) fn tool(name: &str, version: &str) -> ToolIdentityV1 {
    ToolIdentityV1::try_new(
        name,
        version,
        format!("{name}-commit"),
        "aarch64-apple-darwin",
        digest(format!("{name}-binary").as_bytes()),
    )
    .unwrap()
}
