use dependency_declarations_reconcile::*;

use crate::support::digest;

#[test]
fn semantic_inputs_must_match_the_repository_read_set() {
    let manifest = file(InputFileRoleV1::Manifest, "Cargo.toml", b"[workspace]\n");
    let lock = file(InputFileRoleV1::Lock, "Cargo.lock", b"version = 4\n");
    let config = file(InputFileRoleV1::Config, "reindeer.toml", b"[buck]\n");
    let fixup = entry(
        "third-party/fixups/demo/fixups.toml",
        TreeFileModeV1::Regular,
        b"run = true\n",
    );
    let repository_entries = vec![
        entry_for_file(&manifest, TreeFileModeV1::Regular),
        entry_for_file(&lock, TreeFileModeV1::Regular),
        entry_for_file(&config, TreeFileModeV1::Regular),
        fixup.clone(),
    ];
    let fixups = tree(TreeRoleV1::Fixups, "fixups.manifest", vec![fixup.clone()]);
    let cargo_home = tree(
        TreeRoleV1::CargoHomeRead,
        "cargo-home.manifest",
        vec![entry(
            "registry/src/demo/src/lib.rs",
            TreeFileModeV1::Regular,
            b"pub fn demo() {}\n",
        )],
    );
    let valid = GenerationInputsV1::try_new(
        manifest.clone(),
        lock.clone(),
        config.clone(),
        tree(
            TreeRoleV1::RepositoryRead,
            "repository.manifest",
            repository_entries.clone(),
        ),
        fixups.clone(),
        cargo_home.clone(),
    );
    assert!(valid.is_ok());

    let missing_manifest = repository_entries
        .iter()
        .filter(|candidate| candidate.path().as_str() != "Cargo.toml")
        .cloned()
        .collect();
    assert_invalid(GenerationInputsV1::try_new(
        manifest.clone(),
        lock.clone(),
        config.clone(),
        tree(
            TreeRoleV1::RepositoryRead,
            "repository.manifest",
            missing_manifest,
        ),
        fixups.clone(),
        cargo_home.clone(),
    ));

    let mut executable_manifest = repository_entries.clone();
    executable_manifest[0] = entry_for_file(&manifest, TreeFileModeV1::Executable);
    assert_invalid(GenerationInputsV1::try_new(
        manifest.clone(),
        lock.clone(),
        config.clone(),
        tree(
            TreeRoleV1::RepositoryRead,
            "repository.manifest",
            executable_manifest,
        ),
        fixups.clone(),
        cargo_home.clone(),
    ));

    let unknown_fixup = tree(
        TreeRoleV1::Fixups,
        "fixups.manifest",
        vec![entry(
            "third-party/fixups/unknown/fixups.toml",
            TreeFileModeV1::Regular,
            b"run = false\n",
        )],
    );
    assert_invalid(GenerationInputsV1::try_new(
        manifest,
        lock,
        config,
        tree(
            TreeRoleV1::RepositoryRead,
            "repository.manifest",
            repository_entries,
        ),
        unknown_fixup,
        cargo_home,
    ));
}

#[test]
fn typed_tree_entries_are_sorted_and_mode_bound() {
    let regular = tree(
        TreeRoleV1::RepositoryRead,
        "regular.manifest",
        vec![
            entry("z/tool", TreeFileModeV1::Regular, b"tool"),
            entry("a/source.rs", TreeFileModeV1::Regular, b"source"),
        ],
    );
    let executable = tree(
        TreeRoleV1::RepositoryRead,
        "regular.manifest",
        vec![
            entry("a/source.rs", TreeFileModeV1::Regular, b"source"),
            entry("z/tool", TreeFileModeV1::Executable, b"tool"),
        ],
    );

    assert_eq!(regular.entries()[0].path().as_str(), "a/source.rs");
    assert_eq!(regular.entries()[1].path().as_str(), "z/tool");
    assert_eq!(regular.entries()[1].mode(), TreeFileModeV1::Regular);
    assert_eq!(executable.entries()[1].mode(), TreeFileModeV1::Executable);
    assert_ne!(regular.root_sha256(), executable.root_sha256());
}

fn file(role: InputFileRoleV1, path: &str, bytes: &[u8]) -> InputFileV1 {
    InputFileV1::try_new(
        role,
        CanonicalPathV1::try_new(path).unwrap(),
        bytes.to_vec(),
    )
    .unwrap()
}

fn entry(path: &str, mode: TreeFileModeV1, bytes: &[u8]) -> TreeEntryV1 {
    TreeEntryV1::new(
        CanonicalPathV1::try_new(path).unwrap(),
        mode,
        u64::try_from(bytes.len()).unwrap(),
        digest(bytes),
    )
}

fn entry_for_file(file: &InputFileV1, mode: TreeFileModeV1) -> TreeEntryV1 {
    TreeEntryV1::new(
        file.path().clone(),
        mode,
        file.length_bytes(),
        file.sha256(),
    )
}

fn tree(role: TreeRoleV1, manifest: &str, entries: Vec<TreeEntryV1>) -> InputTreeV1 {
    InputTreeV1::try_from_entries(role, CanonicalPathV1::try_new(manifest).unwrap(), entries)
        .unwrap()
}

fn assert_invalid(result: Result<GenerationInputsV1, FailureV1>) {
    assert_eq!(result.unwrap_err().class(), FailureClassV1::InvalidRequest);
}
