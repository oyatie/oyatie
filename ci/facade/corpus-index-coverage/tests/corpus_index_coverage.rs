// cloud-ci-corpus-index-coverage live-corpus gate.
//
// Walk failures are errors, never omitted observations: exact counts cannot compensate for a file
// that disappeared from the census because its directory, metadata, content, or BUCK file failed.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use ci_corpus_index_coverage::{
    CODE_COVERAGE_REGRESSION, CODE_VACUOUS_SCAN, CorpusInput, ExtractionDeclaration,
    FaceObservation, OyaCorpusPolicy, PackageObservation, Policy, derive_faces, evaluate,
    evaluate_face_coverage, extraction_declaration,
};

const POLICY_PATH: &str = "ci/facade/corpus-index-coverage/corpus-index-coverage-policy.json";
const MAX_YAML_SOURCE_BYTES: u64 = 1_048_576;
const NESTED_REPAIR_PACKAGES: [&str; 6] = [
    "oya/oya-authn-device-firmware",
    "oya/oya-billing",
    "oya/oya-cost",
    "oya/oya-flags",
    "oya/oya-identity",
    "oya/oya-meter",
];

struct LiveObservation {
    packages: Vec<PackageObservation>,
    unpackaged: usize,
    oya_inputs: Vec<CorpusInput>,
    oya_faces: Vec<FaceObservation>,
}

struct YamlCandidate {
    path: PathBuf,
    resolved: PathBuf,
    source_bytes: u64,
}

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join(POLICY_PATH).is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root (the dir holding {POLICY_PATH})");
}

fn load_policy(root: &Path) -> (Policy, OyaCorpusPolicy) {
    let raw = std::fs::read_to_string(root.join(POLICY_PATH)).expect("read policy");
    let doc: serde_json::Value = serde_json::from_str(&raw).expect("policy parses");
    let field = |key: &str| -> usize {
        doc[key]
            .as_u64()
            .unwrap_or_else(|| panic!("policy field {key} missing")) as usize
    };
    let policy = Policy {
        baseline_uncovered_packages: field("baseline_uncovered_packages"),
        baseline_unpackaged_yaml_files: field("baseline_unpackaged_yaml_files"),
        min_expected_yaml_packages: field("min_expected_yaml_packages"),
        min_expected_yaml_files: field("min_expected_yaml_files"),
        min_expected_unpackaged_yaml_files: field("min_expected_unpackaged_yaml_files"),
    };
    let oya = serde_json::from_value(doc["oya_corpus"].clone()).expect("oya_corpus policy parses");
    (policy, oya)
}

fn skip_dir(name: &str) -> bool {
    name.starts_with('.') || matches!(name, "buck-out" | "target" | "node_modules")
}

fn is_yaml(name: &str) -> bool {
    name.ends_with(".yaml") || name.ends_with(".yml")
}

fn walk(root: &Path) -> Result<(Vec<PathBuf>, Vec<YamlCandidate>), String> {
    let canonical_root = root.canonicalize().map_err(|error| {
        format!(
            "repo root canonicalization {} failed: {error}",
            root.display()
        )
    })?;
    let mut packages = Vec::new();
    let mut yamls = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir)
            .map_err(|error| format!("read_dir {} failed: {error}", dir.display()))?;
        for entry in entries {
            let entry = entry
                .map_err(|error| format!("directory entry in {} failed: {error}", dir.display()))?;
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| format!("non-UTF-8 path under {}", dir.display()))?;
            let kind = entry
                .file_type()
                .map_err(|error| format!("file_type {} failed: {error}", path.display()))?;
            if kind.is_dir() {
                if !skip_dir(name) {
                    stack.push(path);
                }
            } else if name == "BUCK" {
                packages.push(dir.clone());
            } else if is_yaml(name) {
                let resolved = path.canonicalize().map_err(|error| {
                    format!("YAML canonicalization {} failed: {error}", path.display())
                })?;
                if !resolved.starts_with(&canonical_root) {
                    return Err(format!(
                        "YAML {} resolves outside repository root to {}",
                        path.display(),
                        resolved.display()
                    ));
                }
                let metadata = std::fs::metadata(&resolved)
                    .map_err(|error| format!("YAML metadata {} failed: {error}", path.display()))?;
                if !metadata.is_file() {
                    return Err(format!(
                        "YAML {} resolves to non-regular file {}",
                        path.display(),
                        resolved.display()
                    ));
                }
                if metadata.len() > MAX_YAML_SOURCE_BYTES {
                    return Err(format!(
                        "YAML {} is {} bytes, above the {}-byte limit",
                        path.display(),
                        metadata.len(),
                        MAX_YAML_SOURCE_BYTES
                    ));
                }
                let file = File::open(&resolved)
                    .map_err(|error| format!("YAML open {} failed: {error}", path.display()))?;
                let mut bytes = Vec::with_capacity(
                    usize::try_from(metadata.len()).map_err(|error| error.to_string())?,
                );
                file.take(MAX_YAML_SOURCE_BYTES + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|error| {
                        format!("YAML bounded read {} failed: {error}", path.display())
                    })?;
                let bytes_read = u64::try_from(bytes.len()).map_err(|error| error.to_string())?;
                if bytes_read > MAX_YAML_SOURCE_BYTES || bytes_read != metadata.len() {
                    return Err(format!(
                        "YAML {} changed size during bounded observation: metadata={}, read={bytes_read}",
                        path.display(),
                        metadata.len()
                    ));
                }
                yamls.push(YamlCandidate {
                    path,
                    resolved,
                    source_bytes: bytes_read,
                });
            }
        }
    }
    Ok((packages, yamls))
}

fn read_buck_declaration(dir: &Path, package: &str) -> Result<ExtractionDeclaration, String> {
    let path = dir.join("BUCK");
    let buck = std::fs::read_to_string(&path)
        .map_err(|error| format!("BUCK read {} failed: {error}", path.display()))?;
    extraction_declaration(package, &buck)
        .map_err(|error| format!("BUCK declaration {} failed: {error}", path.display()))
}

fn observe(root: &Path) -> Result<LiveObservation, String> {
    let (packages, yamls) = walk(root)?;
    let package_set: BTreeSet<PathBuf> = packages.into_iter().collect();
    let mut owned: BTreeMap<PathBuf, Vec<CorpusInput>> = BTreeMap::new();
    let mut unpackaged = 0usize;
    let mut oya_inputs = Vec::new();

    for candidate in yamls {
        let relative = candidate
            .path
            .strip_prefix(root)
            .map_err(|error| format!("{} is outside root: {error}", candidate.path.display()))?
            .to_str()
            .ok_or_else(|| format!("non-UTF-8 repo-relative path {}", candidate.path.display()))?
            .replace('\\', "/");
        let input = CorpusInput {
            path: relative.clone(),
            source_bytes: candidate.source_bytes,
        };
        if relative.starts_with("oya/") {
            oya_inputs.push(input.clone());
        }

        let mut cursor = candidate.path.parent();
        let mut owner = None;
        while let Some(dir) = cursor {
            if package_set.contains(dir) {
                owner = Some(dir.to_path_buf());
                break;
            }
            if dir == root {
                break;
            }
            cursor = dir.parent();
        }
        match owner {
            Some(dir) => {
                let canonical_owner = dir.canonicalize().map_err(|error| {
                    format!("package canonicalization {} failed: {error}", dir.display())
                })?;
                if !candidate.resolved.starts_with(&canonical_owner) {
                    return Err(format!(
                        "YAML {} resolves outside owning package {} to {}",
                        candidate.path.display(),
                        dir.display(),
                        candidate.resolved.display()
                    ));
                }
                owned.entry(dir).or_default().push(input);
            }
            None => unpackaged += 1,
        }
    }

    let mut observations = Vec::with_capacity(owned.len());
    let mut oya_faces = Vec::new();
    for (dir, inputs) in owned {
        let package = dir
            .strip_prefix(root)
            .map_err(|error| format!("{} is outside root: {error}", dir.display()))?
            .to_str()
            .ok_or_else(|| format!("non-UTF-8 package path {}", dir.display()))?
            .replace('\\', "/");
        let declaration = read_buck_declaration(&dir, &package)?;
        let faces = derive_faces(&package, &inputs, declaration)
            .map_err(|error| format!("face derivation for {package} failed: {error}"))?;
        if package == "oya" || package.starts_with("oya/") {
            oya_faces.extend(faces);
        }
        observations.push(PackageObservation {
            package,
            yaml_files: inputs.len(),
            indexed: declaration != ExtractionDeclaration::None,
        });
    }
    oya_inputs.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
    observations.sort_by(|left, right| left.package.cmp(&right.package));
    Ok(LiveObservation {
        packages: observations,
        unpackaged,
        oya_inputs,
        oya_faces,
    })
}

fn validate_oya_census(actual: usize, expected: usize) -> Result<(), String> {
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "observed {actual} Oya YAML files, expected exactly {expected}"
        ))
    }
}

#[test]
fn live_corpus_is_within_the_frozen_ceiling() {
    let root = repo_root();
    let live = observe(&root).expect("live observation succeeds");
    let (policy, oya_policy) = load_policy(&root);
    let verdict = evaluate(&live.packages, live.unpackaged, &policy);
    assert!(
        !verdict.failed(),
        "corpus index coverage regressed: {:#?}",
        verdict.blocking()
    );
    validate_oya_census(live.oya_inputs.len(), oya_policy.expected_yaml_files).unwrap();
    evaluate_face_coverage(&live.oya_inputs, &live.oya_faces, oya_policy.face_limits()).unwrap();
}

#[test]
fn a_new_uncovered_package_fails_the_ratchet() {
    let root = repo_root();
    let mut live = observe(&root).unwrap();
    let (policy, _) = load_policy(&root);
    live.packages.push(PackageObservation {
        package: "synthetic/new-service".to_owned(),
        yaml_files: 4,
        indexed: false,
    });
    let verdict = evaluate(&live.packages, live.unpackaged, &policy);
    assert!(verdict.failed());
    assert!(
        verdict
            .blocking()
            .iter()
            .any(|finding| finding.code == CODE_COVERAGE_REGRESSION)
    );
}

#[test]
fn a_new_indexed_package_passes_the_ratchet() {
    let root = repo_root();
    let mut live = observe(&root).unwrap();
    let (policy, _) = load_policy(&root);
    live.packages.push(PackageObservation {
        package: "synthetic/new-service".to_owned(),
        yaml_files: 4,
        indexed: true,
    });
    assert!(!evaluate(&live.packages, live.unpackaged, &policy).failed());
}

#[test]
fn the_walk_sees_the_real_corpus() {
    let root = repo_root();
    let live = observe(&root).unwrap();
    let (policy, _) = load_policy(&root);
    let packaged: usize = live
        .packages
        .iter()
        .map(|observation| observation.yaml_files)
        .sum();
    assert!(live.packages.len() >= policy.min_expected_yaml_packages);
    assert!(packaged + live.unpackaged >= policy.min_expected_yaml_files);
    assert!(live.unpackaged >= policy.min_expected_unpackaged_yaml_files);
}

#[test]
fn an_attribution_collapse_fails_the_live_policy() {
    let root = repo_root();
    let live = observe(&root).unwrap();
    let (policy, _) = load_policy(&root);
    assert!(!evaluate(&live.packages, live.unpackaged, &policy).failed());
    let verdict = evaluate(&live.packages, 0, &policy);
    assert!(verdict.failed());
    assert!(
        verdict
            .blocking()
            .iter()
            .any(|finding| finding.code == CODE_VACUOUS_SCAN)
    );
}

#[test]
fn a_vacuous_scan_fails_against_the_live_policy() {
    let root = repo_root();
    let (policy, _) = load_policy(&root);
    let verdict = evaluate(&[], 0, &policy);
    assert!(verdict.failed());
    assert!(
        verdict
            .blocking()
            .iter()
            .any(|finding| finding.code == CODE_VACUOUS_SCAN)
    );
}

#[test]
fn the_frozen_ceilings_equal_todays_counts() {
    let root = repo_root();
    let live = observe(&root).unwrap();
    let (policy, _) = load_policy(&root);
    let verdict = evaluate(&live.packages, live.unpackaged, &policy);
    assert_eq!(
        verdict.coverage.uncovered_packages,
        policy.baseline_uncovered_packages
    );
    assert_eq!(
        verdict.coverage.unpackaged_yaml_files,
        policy.baseline_unpackaged_yaml_files
    );
}

#[test]
fn oya_census_off_by_one_blocks() {
    assert!(validate_oya_census(3_748, 3_749).is_err());
}

#[test]
fn live_oya_union_matches_expected_census() {
    let root = repo_root();
    let live = observe(&root).unwrap();
    let (_, policy) = load_policy(&root);
    validate_oya_census(live.oya_inputs.len(), policy.expected_yaml_files).unwrap();
}

#[test]
fn live_faces_have_zero_missing_duplicate_empty() {
    let root = repo_root();
    let live = observe(&root).unwrap();
    let (_, policy) = load_policy(&root);
    evaluate_face_coverage(&live.oya_inputs, &live.oya_faces, policy.face_limits()).unwrap();
}

#[test]
fn pre_repair_missing_ten_blocks() {
    let root = repo_root();
    let live = observe(&root).unwrap();
    let (_, policy) = load_policy(&root);
    let pre_repair: Vec<_> = live
        .oya_faces
        .iter()
        .filter(|face| !NESTED_REPAIR_PACKAGES.contains(&face.package.as_str()))
        .cloned()
        .collect();
    assert!(evaluate_face_coverage(&live.oya_inputs, &pre_repair, policy.face_limits()).is_err());
}

#[test]
fn six_nested_faces_use_nearest_package_ownership() {
    let root = repo_root();
    let live = observe(&root).unwrap();
    let counts: BTreeMap<_, _> = live
        .oya_faces
        .iter()
        .filter(|face| NESTED_REPAIR_PACKAGES.contains(&face.package.as_str()))
        .map(|face| (face.package.as_str(), face.paths.len()))
        .collect();
    assert_eq!(
        counts.values().copied().collect::<Vec<_>>(),
        [1, 2, 2, 1, 2, 2]
    );
    assert_eq!(counts.len(), 6);
}

#[test]
fn unreadable_or_missing_buck_blocks() {
    let missing = std::env::temp_dir().join(format!("corpus-missing-buck-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&missing);
    std::fs::create_dir_all(&missing).unwrap();
    assert!(read_buck_declaration(&missing, "missing").is_err());
    std::fs::remove_dir_all(&missing).unwrap();
}

#[cfg(unix)]
fn symlink_case(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("corpus-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

#[cfg(unix)]
#[test]
fn external_yaml_symlink_blocks() {
    use std::os::unix::fs::symlink;

    let root = symlink_case("external-link");
    let external = root.with_extension("external");
    let _ = std::fs::remove_file(&external);
    std::fs::write(&external, "external: true\n").unwrap();
    symlink(&external, root.join("external.yaml")).unwrap();
    assert!(walk(&root).is_err());
    std::fs::remove_dir_all(&root).unwrap();
    std::fs::remove_file(&external).unwrap();
}

#[cfg(unix)]
#[test]
fn oversized_internal_yaml_symlink_blocks_before_read() {
    use std::os::unix::fs::symlink;

    let root = symlink_case("oversized-link");
    std::fs::write(root.join("large.data"), vec![b'x'; 1_048_577]).unwrap();
    symlink("large.data", root.join("large.yaml")).unwrap();
    assert!(walk(&root).is_err());
    std::fs::remove_dir_all(&root).unwrap();
}

#[cfg(unix)]
#[test]
fn non_regular_internal_yaml_symlink_blocks() {
    use std::os::unix::fs::symlink;

    let root = symlink_case("directory-link");
    std::fs::create_dir(root.join("target-dir")).unwrap();
    symlink("target-dir", root.join("directory.yaml")).unwrap();
    assert!(walk(&root).is_err());
    std::fs::remove_dir_all(&root).unwrap();
}

#[cfg(unix)]
#[test]
fn symlink_escaping_nearest_package_blocks() {
    use std::os::unix::fs::symlink;

    let root = symlink_case("owner-escape");
    let package = root.join("package");
    std::fs::create_dir(&package).unwrap();
    std::fs::write(package.join("BUCK"), "").unwrap();
    std::fs::write(root.join("shared.data"), "shared: true\n").unwrap();
    symlink("../shared.data", package.join("escaped.yaml")).unwrap();
    assert!(observe(&root).is_err());
    std::fs::remove_dir_all(&root).unwrap();
}

#[cfg(unix)]
#[test]
fn current_internal_yaml_symlink_inventory_is_seven_and_safe() {
    let root = repo_root();
    let paths = [
        "oya/connector/contracts/asyncapi-v1.yaml",
        "oya/connector/contracts/openapi-v1.yaml",
        "oya/ops-dashboard-control-center/contracts/asyncapi-v1.yaml",
        "oya/ops-dashboard-control-center/contracts/openapi-v1.yaml",
        "oya/ops-dashboard-control-center/iac/ech-config.yaml",
        "oya/ops-dashboard-control-center/iac/edge-waf.yaml",
        "oya/ops-dashboard-control-center/iac/pqc-cert.yaml",
    ];
    assert!(paths.iter().all(|path| {
        std::fs::symlink_metadata(root.join(path))
            .unwrap()
            .file_type()
            .is_symlink()
    }));
    assert!(walk(&root).is_ok());
}

#[cfg(unix)]
#[test]
fn failed_yaml_metadata_or_read_blocks() {
    use std::os::unix::fs::symlink;

    let root = std::env::temp_dir().join(format!("corpus-broken-yaml-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    symlink(root.join("missing"), root.join("broken.yaml")).unwrap();
    assert!(walk(&root).is_err());
    std::fs::remove_dir_all(&root).unwrap();
}
