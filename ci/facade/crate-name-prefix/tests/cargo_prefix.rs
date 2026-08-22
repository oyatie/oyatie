// ADR-0017 cloud-ci-cargo-prefix: scoped self-test over TODAY's real corpus. Buck and CI run the
// declared producer over their materialized SCM face. A direct Cargo test has no materialization
// pre-step, so the exact Cargo-only adapter binding may use the shared Rust SCM emitter plus the
// canonical workspace-member resolver in isolated RAII temporary storage. Existing declared
// inputs always win; unsafe or malformed inputs stay RED. Advisory-scoped de-branded candidates
// remain visible coverage but do not create born-blocking baseline debt. The count is MEASURED +
// reported, not hardcoded. ADR-0083 Tier-3: integration tests assert via unwrap/expect.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use oya_ci_config_kernel::{CONFIG_SEARCH_ORDER, OyaCiConfig};
use serde_json::{Value, json};

use ci_crate_name_prefix::{Verdict, evaluate, evaluate_keyed};

const SCM_FACTS_SCHEMA: &str = "oya-ci/scm-facts/v2";
const SCM_FACTS_RELATIVE_PATH: &str =
    "ci/facade/artifact-inventory-registry/scm-facts.generated.json";
const CARGO_TEST_PRODUCER_ENV: &str = "OYA_CI_PRODUCER_BIN";
const CARGO_TEST_SCM_FACTS_EMITTER_ENV: &str = "OYA_CI_CARGO_TEST_SCM_FACTS_EMITTER_BIN";
const CARGO_TEST_PRODUCER_BINDING: &str =
    "cargo-test-binary:oya-cloud-ci-cargo-test-producer-adapter";
const CARGO_TEST_SCM_FACTS_EMITTER_BINDING: &str =
    "cargo-test-binary:oya-cloud-ci-scm-facts-emitter-app";

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

fn producer_binary(root: &Path, value: Option<&OsStr>) -> Result<PathBuf, String> {
    let Some(bin) = value else {
        return Err(
            "FAIL-CLOSED: missing OYA_CI_PRODUCER_BIN; Cargo fallback is forbidden".to_owned(),
        );
    };
    ci_path_resolver_adapters::resolve_cargo_test_binary(root, bin)
}

fn materialized_scm_facts(root: &Path) -> PathBuf {
    root.join(SCM_FACTS_RELATIVE_PATH)
}

#[test]
fn producer_binary_env_is_required_for_gate() {
    let root = Path::new("/repo");
    let producer = producer_binary(root, None).expect_err("missing producer env must fail closed");
    assert!(producer.contains("OYA_CI_PRODUCER_BIN"));
}

struct TemporaryScmFacts {
    directory: tempfile::TempDir,
    stable: PathBuf,
    volatile: PathBuf,
}

impl TemporaryScmFacts {
    fn materialize(root: &Path) -> Result<Self, String> {
        let directory = tempfile::Builder::new()
            .prefix("oya-ci-crate-prefix-scm-")
            .tempdir()
            .map_err(|error| format!("create temporary SCM facts directory: {error}"))?;
        let stable = directory.path().join("scm-facts.generated.json");
        let volatile = directory.path().join("scm-volatile-facts.generated.json");
        ci_scm_facts_snapshot::emit_candidate_scm_facts_out_of_graph(root, &stable, &volatile)?;
        require_regular_non_symlink(&stable, "temporary stable SCM facts")?;
        require_regular_non_symlink(&volatile, "temporary volatile SCM facts")?;
        Ok(Self {
            directory,
            stable,
            volatile,
        })
    }
}

enum ResolvedScmFacts {
    Declared(PathBuf),
    CargoTemporary(TemporaryScmFacts),
}

impl ResolvedScmFacts {
    fn path(&self) -> &Path {
        match self {
            Self::Declared(path) => path,
            Self::CargoTemporary(temporary) => &temporary.stable,
        }
    }

    fn is_temporary(&self) -> bool {
        matches!(self, Self::CargoTemporary(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeclaredInputState {
    Present,
    Absent,
}

fn require_regular_non_symlink(path: &Path, label: &str) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("inspect {label} {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "{label} must be a regular non-symlink file: {}",
            path.display()
        ));
    }
    Ok(())
}

fn inspect_declared_scm_facts(root: &Path, declared: &Path) -> Result<DeclaredInputState, String> {
    let relative = declared.strip_prefix(root).map_err(|error| {
        format!(
            "declared SCM facts {} must remain beneath repo root {}: {error}",
            declared.display(),
            root.display()
        )
    })?;
    let components = relative.components().collect::<Vec<_>>();
    if components.is_empty() {
        return Err("declared SCM facts path is empty".to_owned());
    }

    let mut current = root.to_path_buf();
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(component) = component else {
            return Err(format!(
                "declared SCM facts path must be normalized and repo-relative: {}",
                relative.display()
            ));
        };
        current.push(component);
        let is_last = index + 1 == components.len();
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!(
                    "declared SCM facts path contains symlink component: {}",
                    current.display()
                ));
            }
            Ok(metadata) if is_last && !metadata.is_file() => {
                return Err(format!(
                    "declared SCM facts must be a regular file: {}",
                    current.display()
                ));
            }
            Ok(metadata) if !is_last && !metadata.is_dir() => {
                return Err(format!(
                    "declared SCM facts parent is not a directory: {}",
                    current.display()
                ));
            }
            Ok(_) if is_last => return Ok(DeclaredInputState::Present),
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound && is_last => {
                return Ok(DeclaredInputState::Absent);
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Err(format!(
                    "declared SCM facts parent is absent: {}",
                    current.display()
                ));
            }
            Err(error) => {
                return Err(format!(
                    "inspect declared SCM facts path {}: {error}",
                    current.display()
                ));
            }
        }
    }
    Err("declared SCM facts path has no components".to_owned())
}

fn exact_binding(value: Option<&OsStr>, expected: &str) -> bool {
    value == Some(OsStr::new(expected))
}

fn resolve_scm_facts(
    root: &Path,
    declared: &Path,
    producer_binding: Option<&OsStr>,
    emitter_binding: Option<&OsStr>,
) -> Result<ResolvedScmFacts, String> {
    if inspect_declared_scm_facts(root, declared)? == DeclaredInputState::Present {
        return Ok(ResolvedScmFacts::Declared(declared.to_path_buf()));
    }
    if declared != materialized_scm_facts(root) {
        return Err(format!(
            "refusing Cargo fallback for non-canonical absent SCM facts path {}",
            declared.display()
        ));
    }
    if !exact_binding(producer_binding, CARGO_TEST_PRODUCER_BINDING) {
        return Err(format!(
            "declared SCM facts are absent and {CARGO_TEST_PRODUCER_ENV} is not the code-owned Cargo adapter binding"
        ));
    }
    if !exact_binding(emitter_binding, CARGO_TEST_SCM_FACTS_EMITTER_BINDING) {
        return Err(format!(
            "declared SCM facts are absent and {CARGO_TEST_SCM_FACTS_EMITTER_ENV} is not the code-owned Cargo emitter binding"
        ));
    }
    TemporaryScmFacts::materialize(root).map(ResolvedScmFacts::CargoTemporary)
}

fn load_scm_tracked_paths(path: &Path) -> Result<Vec<String>, String> {
    require_regular_non_symlink(path, "SCM facts")?;
    let text = fs::read_to_string(path)
        .map_err(|error| format!("read SCM facts {}: {error}", path.display()))?;
    let value: Value = serde_json::from_str(&text)
        .map_err(|error| format!("parse SCM facts {}: {error}", path.display()))?;
    if value.get("schema").and_then(Value::as_str) != Some(SCM_FACTS_SCHEMA) {
        return Err(format!(
            "SCM facts {} must declare schema {SCM_FACTS_SCHEMA}",
            path.display()
        ));
    }
    let paths = value
        .get("tracked_paths")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("SCM facts {} missing tracked_paths", path.display()))?;
    paths
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            entry.as_str().map(str::to_owned).ok_or_else(|| {
                format!(
                    "SCM facts {} tracked_paths[{index}] is not a string",
                    path.display()
                )
            })
        })
        .collect()
}

fn load_policy_config(root: &Path) -> Result<OyaCiConfig, String> {
    for name in CONFIG_SEARCH_ORDER {
        let path = root.join(name);
        match fs::read_to_string(&path) {
            Ok(text) => {
                return OyaCiConfig::from_toml_str(&text)
                    .map_err(|error| format!("parse {}: {error}", path.display()));
            }
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(error) => return Err(format!("read {}: {error}", path.display())),
        }
    }
    Ok(OyaCiConfig::bundled_default())
}

fn is_path_excluded(path: &str, config: &OyaCiConfig) -> bool {
    config
        .repo
        .path_excludes
        .iter()
        .any(|prefix| path.starts_with(prefix) || path.contains(&format!("/{prefix}")))
}

fn load_package_name(path: &Path) -> Result<String, String> {
    require_regular_non_symlink(path, "tracked workspace manifest")?;
    let text = fs::read_to_string(path)
        .map_err(|error| format!("read workspace manifest {}: {error}", path.display()))?;
    let manifest: toml::Value = toml::from_str(&text)
        .map_err(|error| format!("parse workspace manifest {}: {error}", path.display()))?;
    manifest
        .get("package")
        .and_then(toml::Value::as_table)
        .and_then(|package| package.get("name"))
        .and_then(toml::Value::as_str)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("workspace manifest {} missing package.name", path.display()))
}

fn collect_cargo_prefix_face(
    root: &Path,
    tracked_paths: &[String],
    config: &OyaCiConfig,
) -> Result<Value, String> {
    let tracked = tracked_paths
        .iter()
        .filter(|path| !is_path_excluded(path, config))
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let members = oya_workspace_members_kernel::scan_member_dirs(root)
        .map_err(|error| format!("cargo-prefix scan member dirs: {error}"))?;
    let mut by_member = BTreeMap::new();
    for member_path in members.member_dirs {
        let manifest_relative = format!("{member_path}/Cargo.toml");
        if !tracked.contains(manifest_relative.as_str()) {
            continue;
        }
        let package_name = load_package_name(&root.join(&manifest_relative))?;
        by_member.insert(member_path, package_name);
    }

    let rows = by_member
        .into_iter()
        .map(|(member_path, package_name)| {
            let crate_id = member_path.rsplit('/').next().unwrap_or(&member_path);
            let required_prefix = &config.naming.required_prefix;
            let cargo_prefix_scope = if !required_prefix.is_empty()
                && crate_id.starts_with(required_prefix)
                && package_name.starts_with(required_prefix)
            {
                "blocking"
            } else {
                "advisory"
            };
            json!({
                "member_path": member_path,
                "package_name": package_name,
                "cargo_prefix_scope": cargo_prefix_scope,
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({ "rows": rows }))
}

fn run_declared_producer_face(root: &Path, scm_facts: &Path, face: &str) -> Value {
    let producer_binding = std::env::var_os(CARGO_TEST_PRODUCER_ENV);
    let bin = producer_binary(root, producer_binding.as_deref())
        .unwrap_or_else(|error| panic!("{error}"));
    let output = Command::new(bin)
        .arg("--repo-root")
        .arg(root)
        .arg("--scm-facts")
        .arg(scm_facts)
        .arg("--stdout")
        .arg("--face")
        .arg(face)
        .current_dir(root)
        .output()
        .expect("run producer binary");
    assert!(
        output.status.success(),
        "producer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("producer face stdout is valid JSON")
}

fn cargo_prefix_face(root: &Path) -> Value {
    let producer_binding = std::env::var_os(CARGO_TEST_PRODUCER_ENV);
    let emitter_binding = std::env::var_os(CARGO_TEST_SCM_FACTS_EMITTER_ENV);
    let scm_facts = resolve_scm_facts(
        root,
        &materialized_scm_facts(root),
        producer_binding.as_deref(),
        emitter_binding.as_deref(),
    )
    .unwrap_or_else(|error| panic!("FAIL-CLOSED: resolve SCM facts: {error}"));

    if exact_binding(producer_binding.as_deref(), CARGO_TEST_PRODUCER_BINDING) {
        let tracked_paths = load_scm_tracked_paths(scm_facts.path())
            .unwrap_or_else(|error| panic!("FAIL-CLOSED: {error}"));
        let config = load_policy_config(root)
            .unwrap_or_else(|error| panic!("FAIL-CLOSED: load oya-ci policy: {error}"));
        collect_cargo_prefix_face(root, &tracked_paths, &config)
            .unwrap_or_else(|error| panic!("FAIL-CLOSED: collect cargo-prefix face: {error}"))
    } else {
        run_declared_producer_face(root, scm_facts.path(), "cargo-prefix")
    }
}

fn create_declared_parent(root: &Path) -> PathBuf {
    let declared = materialized_scm_facts(root);
    fs::create_dir_all(declared.parent().expect("declared SCM facts parent"))
        .expect("create declared SCM facts parent");
    declared
}

#[test]
fn existing_malformed_declared_input_wins_and_stays_red() {
    let repository = tempfile::tempdir().expect("fixture repository");
    let declared = create_declared_parent(repository.path());
    fs::write(&declared, "not-json\n").expect("write malformed declared face");

    let resolved = resolve_scm_facts(
        repository.path(),
        &declared,
        Some(OsStr::new(CARGO_TEST_PRODUCER_BINDING)),
        Some(OsStr::new(CARGO_TEST_SCM_FACTS_EMITTER_BINDING)),
    )
    .expect("existing declared input must win before Cargo fallback");

    assert!(!resolved.is_temporary());
    assert_eq!(resolved.path(), declared);
    let error = load_scm_tracked_paths(resolved.path())
        .expect_err("malformed declared input must remain RED");
    assert!(
        error.contains("parse SCM facts"),
        "unexpected error: {error}"
    );
}

#[test]
fn nonregular_declared_input_stays_red() {
    let repository = tempfile::tempdir().expect("fixture repository");
    let declared = create_declared_parent(repository.path());
    fs::create_dir(&declared).expect("create declared directory");

    let error = resolve_scm_facts(
        repository.path(),
        &declared,
        Some(OsStr::new(CARGO_TEST_PRODUCER_BINDING)),
        Some(OsStr::new(CARGO_TEST_SCM_FACTS_EMITTER_BINDING)),
    )
    .err()
    .expect("declared directory must remain RED");

    assert!(error.contains("regular file"), "unexpected error: {error}");
}

#[cfg(unix)]
#[test]
fn symlink_declared_input_stays_red() {
    use std::os::unix::fs::symlink;

    let repository = tempfile::tempdir().expect("fixture repository");
    let declared = create_declared_parent(repository.path());
    let target = repository.path().join("real-scm-facts.json");
    fs::write(
        &target,
        r#"{"schema":"oya-ci/scm-facts/v2","tracked_paths":[]}"#,
    )
    .expect("write symlink target");
    symlink(&target, &declared).expect("create declared symlink");

    let error = resolve_scm_facts(
        repository.path(),
        &declared,
        Some(OsStr::new(CARGO_TEST_PRODUCER_BINDING)),
        Some(OsStr::new(CARGO_TEST_SCM_FACTS_EMITTER_BINDING)),
    )
    .err()
    .expect("declared symlink must remain RED");

    assert!(error.contains("symlink"), "unexpected error: {error}");
}

#[test]
fn fallback_requires_canonical_absence_and_both_exact_cargo_bindings() {
    let repository = tempfile::tempdir().expect("fixture repository");
    let declared = create_declared_parent(repository.path());
    let noncanonical = declared.with_file_name("other-scm-facts.json");

    let noncanonical_error = resolve_scm_facts(
        repository.path(),
        &noncanonical,
        Some(OsStr::new(CARGO_TEST_PRODUCER_BINDING)),
        Some(OsStr::new(CARGO_TEST_SCM_FACTS_EMITTER_BINDING)),
    )
    .err()
    .expect("non-canonical absence must remain RED");
    assert!(noncanonical_error.contains("non-canonical"));

    let producer_error = resolve_scm_facts(
        repository.path(),
        &declared,
        None,
        Some(OsStr::new(CARGO_TEST_SCM_FACTS_EMITTER_BINDING)),
    )
    .err()
    .expect("missing Cargo producer capability must remain RED");
    assert!(producer_error.contains(CARGO_TEST_PRODUCER_ENV));

    let emitter_error = resolve_scm_facts(
        repository.path(),
        &declared,
        Some(OsStr::new(CARGO_TEST_PRODUCER_BINDING)),
        Some(OsStr::new("cargo-test-binary:wrong-emitter")),
    )
    .err()
    .expect("wrong Cargo emitter capability must remain RED");
    assert!(emitter_error.contains(CARGO_TEST_SCM_FACTS_EMITTER_ENV));
}

#[test]
fn cargo_materialization_is_deterministic_temporary_and_checkout_clean() {
    let root = repo_root();
    let canonical = materialized_scm_facts(&root);
    let canonical_before = fs::read(&canonical).ok();
    let first = TemporaryScmFacts::materialize(&root).expect("first temporary SCM facts");
    let second = TemporaryScmFacts::materialize(&root).expect("second temporary SCM facts");

    assert_ne!(first.directory.path(), second.directory.path());
    assert_eq!(
        fs::read(&first.stable).expect("read first stable SCM facts"),
        fs::read(&second.stable).expect("read second stable SCM facts")
    );
    assert_eq!(
        fs::read(&first.volatile).expect("read first volatile SCM facts"),
        fs::read(&second.volatile).expect("read second volatile SCM facts")
    );

    let first_directory = first.directory.path().to_path_buf();
    let second_directory = second.directory.path().to_path_buf();
    drop(first);
    drop(second);
    assert!(!first_directory.exists());
    assert!(!second_directory.exists());
    assert_eq!(
        fs::read(&canonical).ok(),
        canonical_before,
        "Cargo fallback must not create or rewrite the declared CI/Buck face"
    );
}

/// Minimal, deliberately SEPARATE `[package] name` presence probe — NOT a call into the
/// producer's own `parse_package_name`. The point of "independent" is that a bug in the shared
/// parser cannot hide behind the census reusing the same buggy code.
fn independent_has_package_name(contents: &str) -> bool {
    let mut in_package = false;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if in_package
            && let Some(rest) = trimmed.strip_prefix("name")
            && let Some(rest) = rest.trim_start().strip_prefix('=')
            && !rest.trim().trim_matches('"').is_empty()
        {
            return true;
        }
    }
    false
}

/// INDEPENDENT dynamic census of the root workspace's member directories, resolved via the
/// canonical `oya_workspace_members_kernel::resolve_member_dirs` with its OWN from-scratch
/// `[package] name` probe.
///
/// Deliberately NOT the producer's `collect_cargo_prefix` path, and deliberately NOT
/// intersected with the producer's tracked-path universe: that intersection is precisely the
/// SILENT-DROP vector this census exists to catch. The producer derives every one of its
/// faces from one `tracked_paths` vector, so a single narrowing there (an over-broad
/// exclusion rule, a lost scan root, a truncated SCM face) shrinks this face with no other
/// signal — and a `rows.len() > 500` floor cannot tell "the corpus shrank" from "the producer
/// stopped seeing most of it". Measured: a 43% producer-side narrowing left this gate GREEN.
///
/// FAILS CLOSED: a resolved member directory whose Cargo.toml is unreadable or carries no
/// `[package] name` is a hard test failure, never a silent skip. Self-adjusts as crates are
/// added and removed — the census moves in lockstep with the face and never needs a bump.
fn independent_member_census(root: &Path) -> BTreeSet<String> {
    let member_dirs = oya_workspace_members_kernel::resolve_member_dirs(root)
        .expect("resolve_member_dirs must resolve the live root workspace Cargo.toml");
    let mut census = BTreeSet::new();
    for dir in member_dirs {
        let manifest_path = root.join(&dir).join("Cargo.toml");
        let contents = fs::read_to_string(&manifest_path).unwrap_or_else(|error| {
            panic!(
                "independent census FAIL-CLOSED: unreadable manifest {} ({error}) — a resolved \
                 workspace member MUST have a readable Cargo.toml",
                manifest_path.display()
            )
        });
        assert!(
            independent_has_package_name(&contents),
            "independent census FAIL-CLOSED: no [package] name in {}",
            manifest_path.display()
        );
        census.insert(dir);
    }
    census
}

/// Assert the face's enumerated member-path set exactly equals the independent census, naming
/// exactly which keys are missing/extra on mismatch. A SET, never a count: a count is
/// unattributable, a set is a reviewable diff of named keys.
fn assert_member_census_matches(face_members: &BTreeSet<String>, census: &BTreeSet<String>) {
    let missing_from_face: Vec<&String> = census.difference(face_members).collect();
    let extra_in_face: Vec<&String> = face_members.difference(census).collect();
    assert!(
        missing_from_face.is_empty() && extra_in_face.is_empty(),
        "face/census SET MISMATCH — missing_from_face={missing_from_face:?} \
         extra_in_face={extra_in_face:?}"
    );
}

#[test]
fn cargo_prefix_verdict_matches_the_live_corpus() {
    let root = repo_root();
    let face = cargo_prefix_face(&root);
    let rows = face["rows"].as_array().expect("cargo-prefix face rows");

    // INDEPENDENT DYNAMIC CENSUS, not a magnitude floor: re-derive the live root-workspace
    // member set from the canonical resolver and assert EXACT set equality against the face.
    // A producer that silently drops even ONE eligible member is caught as a named set
    // difference rather than masked by "still more than 500 rows".
    let face_members: BTreeSet<String> = rows
        .iter()
        .map(|row| {
            row["member_path"]
                .as_str()
                .expect("cargo-prefix row member_path")
                .to_owned()
        })
        .collect();
    assert_eq!(
        face_members.len(),
        rows.len(),
        "cargo-prefix rows must be uniquely keyed by member_path"
    );
    assert_member_census_matches(&face_members, &independent_member_census(&root));

    let advisory_rows = rows
        .iter()
        .filter(|row| row.get("cargo_prefix_scope").and_then(Value::as_str) == Some("advisory"))
        .count();

    let findings = evaluate_keyed(&face);
    let verdict = evaluate(&face).verdict;
    eprintln!(
        "cargo-prefix: member_candidates={} advisory_candidates={} blocking_findings={} verdict={:?}",
        rows.len(),
        advisory_rows,
        findings.len(),
        verdict
    );

    // The verdict follows the blocking-scoped findings only: advisory de-brand candidates are
    // coverage rows, not baseline-block-on-new debt. Assert consistency (no false-green):
    // non-empty blocking findings <=> RED.
    if findings.is_empty() {
        assert_eq!(
            verdict,
            Verdict::Green,
            "no findings must mean GREEN (the gate cleanly passes when every crate conforms)"
        );
    } else {
        assert_eq!(
            verdict,
            Verdict::Red,
            "blocking findings present must mean RED (the gate fires + freezes that scoped debt)"
        );
    }
}

// --- assert_member_census_matches: RED-test the vacuous cases a bare `rows.len() > 500` floor
// is blind to ("still a lot of rows" is not the same fact as "the corpus was enumerated"). ---

#[test]
#[should_panic(expected = "SET MISMATCH")]
fn member_census_mismatch_is_caught_when_face_is_empty_but_census_is_not() {
    let face: BTreeSet<String> = BTreeSet::new();
    let census: BTreeSet<String> = ["audit/core/chain-domain".to_owned()].into_iter().collect();
    assert_member_census_matches(&face, &census);
}

#[test]
#[should_panic(expected = "SET MISMATCH")]
fn member_census_mismatch_is_caught_when_exactly_one_member_is_missing() {
    // The exact silent-drop this gate could not see: a producer-side narrowing removes one
    // eligible member and every magnitude floor still passes.
    let face: BTreeSet<String> = ["audit/core/chain-domain", "audit/core/emission-domain"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    let census: BTreeSet<String> = [
        "audit/core/chain-domain",
        "audit/core/emission-domain",
        "audit/adapters/file",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    assert_member_census_matches(&face, &census);
}

#[test]
#[should_panic(expected = "SET MISMATCH")]
fn member_census_mismatch_is_caught_when_the_face_invents_a_member() {
    let face: BTreeSet<String> = ["audit/core/chain-domain", "audit/core/phantom"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    let census: BTreeSet<String> = ["audit/core/chain-domain".to_owned()].into_iter().collect();
    assert_member_census_matches(&face, &census);
}

#[test]
fn member_census_is_green_when_the_sets_are_equal() {
    let members: BTreeSet<String> = ["audit/core/chain-domain", "audit/adapters/file"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    assert_member_census_matches(&members, &members.clone());
}

#[test]
fn independent_package_name_probe_rejects_a_virtual_manifest() {
    assert!(independent_has_package_name(
        "[package]\nname = \"a-domain\"\n"
    ));
    // A workspace-only manifest carries no [package] name; the census must NOT count it as a
    // package, and the live census fails closed rather than silently skipping such a member.
    assert!(!independent_has_package_name(
        "[workspace]\nmembers = [\"a\"]\n"
    ));
    // `name` under a non-[package] table must never be mistaken for the package name.
    assert!(!independent_has_package_name(
        "[workspace.package]\nname = \"not-a-package\"\n"
    ));
}
