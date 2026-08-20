// governance-check-active-artifact-contract LIVE-TREE gate.
//
// Authority: ADR-0709, the live general apex, which restates the ADR-0069 v3.0.0 active
// machine-readable artifact contract (ADR-0069 itself is Superseded and stamped
// HISTORICAL / NON-AUTHORITY; it is provenance, not law). See `_authority_note` in the policy JSON.
//
// The unit suite inside src/lib.rs proves the kernel correct on hand-written fixtures. It says
// nothing about this repository, and until this file existed nothing did: the crate's only Cargo
// consumer was marketplace/facade/dev-cli, which no workflow invokes, so the doctrine had never
// produced a verdict about the registry it governs — while two rows had been pointing
// `artifact_path` at directories that `git ls-files` never yields.
//
// The kernel is pure and takes pre-parsed rows plus a HEAD-tracked path set; this is the CALLER
// that does the JSON parsing, the file I/O and the `git ls-files` invocation the kernel doc comment
// says the runtime owes it. Walk failures are ERRORS, never omitted observations: a row dropped
// from the census because it failed to parse would quietly shrink the frozen sets, and a shrink
// reads as repair.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use check_active_artifact_contract::{
    ArtifactProfile, ArtifactRow, CapabilityDeclaration, CapabilityKind, CapabilityStatus,
    ValidationReport, validate,
};

const POLICY_PATH: &str =
    "governance/check/active-artifact-contract/active-artifact-contract-policy.json";
const REGISTRY_PATH: &str = "registry/artifact-capabilities-registry.json";

struct Policy {
    min_tracked_files: usize,
    min_registry_rows: usize,
    min_rows_with_tracked_path: usize,
    frozen_untracked_artifact_paths: BTreeSet<String>,
    frozen_duplicate_artifact_ids: BTreeSet<String>,
    frozen_unknown_artifact_profiles: BTreeSet<String>,
    frozen_rows_declaring_capability_status: BTreeSet<String>,
}

struct Observed {
    report: ValidationReport,
    tracked_files: usize,
    rows: usize,
    /// `<artifact_id>::<artifact_path>` for every R01 violation, carrying the path exactly as the
    /// registry declares it (leading slash and all) rather than as the caller normalized it.
    untracked_artifact_paths: BTreeSet<String>,
    /// `<artifact_id>` for every R02 violation.
    duplicate_artifact_ids: BTreeSet<String>,
    /// `<artifact_id>::<artifact_profile>` for rows naming a profile outside the closed enum.
    unknown_artifact_profiles: BTreeSet<String>,
    /// `<artifact_id>::<capability>` for any override that declares a real status. See the
    /// tripwire test below for why this exists.
    rows_declaring_capability_status: BTreeSet<String>,
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

fn load_policy(root: &Path) -> Policy {
    let raw = std::fs::read_to_string(root.join(POLICY_PATH)).expect("read policy");
    let doc: serde_json::Value = serde_json::from_str(&raw).expect("policy parses");
    let number = |key: &str| -> usize {
        usize::try_from(
            doc[key]
                .as_u64()
                .unwrap_or_else(|| panic!("policy field {key} missing or not a number")),
        )
        .expect("policy number fits usize")
    };
    let string_set = |key: &str| -> BTreeSet<String> {
        doc[key]
            .as_array()
            .unwrap_or_else(|| panic!("policy field {key} missing or not an array"))
            .iter()
            .map(|entry| {
                entry
                    .as_str()
                    .unwrap_or_else(|| panic!("policy field {key} holds a non-string"))
                    .to_owned()
            })
            .collect()
    };
    Policy {
        min_tracked_files: number("min_tracked_files"),
        min_registry_rows: number("min_registry_rows"),
        min_rows_with_tracked_path: number("min_rows_with_tracked_path"),
        frozen_untracked_artifact_paths: string_set("frozen_untracked_artifact_paths"),
        frozen_duplicate_artifact_ids: string_set("frozen_duplicate_artifact_ids"),
        frozen_unknown_artifact_profiles: string_set("frozen_unknown_artifact_profiles"),
        frozen_rows_declaring_capability_status: string_set(
            "frozen_rows_declaring_capability_status",
        ),
    }
}

/// The HEAD-tracked path set, from git — exactly the input the kernel's doc comment says the
/// runtime owes it, and the same corpus boundary every other live gate here uses.
///
/// Walking the working tree instead would answer "does this file exist on disk", which is the
/// question that let the two frozen directory rows look fine for months: both directories DO exist,
/// and neither is a tracked path.
fn tracked_files(root: &Path) -> Result<BTreeSet<String>, String> {
    let out = Command::new("git")
        .current_dir(root)
        .args(["ls-files", "-z"])
        .output()
        .map_err(|e| format!("git ls-files failed to start: {e}"))?;
    if !out.status.success() {
        return Err(format!("git ls-files exited with {}", out.status));
    }
    let text =
        String::from_utf8(out.stdout).map_err(|e| format!("git ls-files output not UTF-8: {e}"))?;
    Ok(text
        .split('\0')
        .filter(|entry| !entry.is_empty())
        .map(str::to_owned)
        .collect())
}

/// The registry writes repo-absolute paths (`/specs/...`) on most rows and repo-relative ones
/// (`intelligence/core/...`) on others; `git ls-files` emits repo-relative. Normalizing here rather
/// than in the frozen keys keeps the baseline showing what the registry SAYS.
fn normalize(artifact_path: &str) -> &str {
    artifact_path.strip_prefix('/').unwrap_or(artifact_path)
}

/// The neutral capability declaration.
///
/// NOT a defaulting choice dressed up as data. The registry declares no per-capability `status`
/// anywhere — see `_scope_is_R01_R02_and_that_limit_is_itself_machine_checked` in the policy — so
/// R03..R07 have no input, and any status this caller invented would produce findings that describe
/// the caller. `NotApplicable` with an explicit rationale is the one shape that satisfies R03
/// (all nine present) and R07 (rationale non-empty) without asserting anything the registry did not
/// say, leaving R01 and R02 as the only rules that can fire. `capability_filler_is_neutral` below
/// asserts that property against the live corpus rather than trusting this comment.
fn neutral_capabilities() -> BTreeMap<CapabilityKind, CapabilityDeclaration> {
    CapabilityKind::ALL
        .into_iter()
        .map(|kind| {
            (
                kind,
                CapabilityDeclaration {
                    status: CapabilityStatus::NotApplicable,
                    evidence_ref: None,
                    prerequisite_for_operational: Vec::new(),
                    not_applicable_rationale: Some(
                        "registry/artifact-capabilities-registry.json declares no per-capability \
                         status; R04-R07 have no input in this corpus"
                            .to_owned(),
                    ),
                },
            )
        })
        .collect()
}

struct RegistryRow {
    row: ArtifactRow,
    profile: String,
    statuses: Vec<String>,
}

fn parse_registry(root: &Path) -> Result<Vec<RegistryRow>, String> {
    let raw = std::fs::read_to_string(root.join(REGISTRY_PATH))
        .map_err(|e| format!("read {REGISTRY_PATH} failed: {e}"))?;
    let doc: serde_json::Value =
        serde_json::from_str(&raw).map_err(|e| format!("{REGISTRY_PATH} does not parse: {e}"))?;

    let mut raw_rows: Vec<&serde_json::Value> = doc["rows"]
        .as_array()
        .ok_or_else(|| format!("{REGISTRY_PATH} has no `rows` array"))?
        .iter()
        .collect();
    // `_self_row` is the registry's own row in the same shape. Including it is the point of a
    // self-describing control plane: the row that describes the audit surface is itself audited.
    raw_rows.push(&doc["_self_row"]);

    let mut out = Vec::with_capacity(raw_rows.len());
    for (index, raw_row) in raw_rows.iter().enumerate() {
        let field = |name: &str| -> Result<String, String> {
            raw_row[name]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("row {index} has no string `{name}`"))
        };
        let artifact_id = field("artifact_id")?;
        let mut statuses = Vec::new();
        if let Some(overrides) = raw_row["capability_overrides"].as_object() {
            for (capability, declaration) in overrides {
                if declaration.get("status").is_some() {
                    statuses.push(format!("{artifact_id}::{capability}"));
                }
            }
        }
        out.push(RegistryRow {
            profile: field("artifact_profile")?,
            statuses,
            row: ArtifactRow {
                artifact_path: field("artifact_path")?,
                artifact_format: field("artifact_format")?,
                contract_version: field("contract_version")?,
                artifact_id,
                capabilities: neutral_capabilities(),
            },
        });
    }
    Ok(out)
}

fn observe(root: &Path) -> Result<Observed, String> {
    let tracked = tracked_files(root)?;
    let registry_rows = parse_registry(root)?;

    // The kernel compares `artifact_path` against the tracked set by exact membership, so the
    // leading-slash normalization has to happen on the way IN. The frozen keys below are rebuilt
    // from the row's ORIGINAL declared value, not from these normalized copies.
    let kernel_rows: Vec<ArtifactRow> = registry_rows
        .iter()
        .map(|entry| ArtifactRow {
            artifact_path: normalize(&entry.row.artifact_path).to_owned(),
            ..entry.row.clone()
        })
        .collect();
    let report = validate(&kernel_rows, &tracked);

    let declared_by_id: BTreeMap<&str, &str> = registry_rows
        .iter()
        .map(|entry| {
            (
                entry.row.artifact_id.as_str(),
                entry.row.artifact_path.as_str(),
            )
        })
        .collect();

    let mut untracked_artifact_paths = BTreeSet::new();
    let mut duplicate_artifact_ids = BTreeSet::new();
    for violation in &report.violations {
        match violation.rule_id {
            "R01-artifact-path-not-in-head" => {
                let declared = declared_by_id
                    .get(violation.artifact_id.as_str())
                    .copied()
                    .ok_or_else(|| {
                        format!(
                            "kernel reported R01 for artifact_id `{}`, which is not in the parsed \
                             registry — the caller and the kernel disagree about the corpus",
                            violation.artifact_id
                        )
                    })?;
                untracked_artifact_paths.insert(format!("{}::{declared}", violation.artifact_id));
            }
            "R02-duplicate-artifact-id" => {
                duplicate_artifact_ids.insert(violation.artifact_id.clone());
            }
            other => {
                return Err(format!(
                    "rule {other} fired for `{}`, which cannot happen while the capability filler \
                     is neutral: {}",
                    violation.artifact_id, violation.message
                ));
            }
        }
    }

    let unknown_artifact_profiles = registry_rows
        .iter()
        .filter(|entry| ArtifactProfile::parse(&entry.profile).is_none())
        .map(|entry| format!("{}::{}", entry.row.artifact_id, entry.profile))
        .collect();

    let rows_declaring_capability_status = registry_rows
        .iter()
        .flat_map(|entry| entry.statuses.iter().cloned())
        .collect();

    Ok(Observed {
        tracked_files: tracked.len(),
        rows: registry_rows.len(),
        untracked_artifact_paths,
        duplicate_artifact_ids,
        unknown_artifact_profiles,
        rows_declaring_capability_status,
        report,
    })
}

/// The live walk, done ONCE for the whole binary: it is a pure function of the tree, so every test
/// re-walking it would recompute the same answer over ~14k tracked paths.
fn live() -> &'static (Policy, Observed) {
    static LIVE: OnceLock<(Policy, Observed)> = OnceLock::new();
    LIVE.get_or_init(|| {
        let root = repo_root();
        let policy = load_policy(&root);
        let observed = observe(&root).expect("live walk");
        (policy, observed)
    })
}

fn census(observed: &Observed) -> String {
    let mut out = format!(
        "census: {} registry rows (_self_row included) over {} tracked paths; {} rows resolve to a \
         HEAD-tracked file, {} do not; {} duplicate ids; {} rows name a profile outside the closed \
         enum; {} rows declare a per-capability status\n",
        observed.rows,
        observed.tracked_files,
        observed.rows - observed.untracked_artifact_paths.len(),
        observed.untracked_artifact_paths.len(),
        observed.duplicate_artifact_ids.len(),
        observed.unknown_artifact_profiles.len(),
        observed.rows_declaring_capability_status.len(),
    );
    for key in &observed.untracked_artifact_paths {
        out.push_str(&format!("  untracked  {key}\n"));
    }
    for key in &observed.unknown_artifact_profiles {
        out.push_str(&format!("  profile    {key}\n"));
    }
    out
}

fn set_drift(frozen: &BTreeSet<String>, observed: &BTreeSet<String>) -> Vec<String> {
    let mut drift: Vec<String> = observed
        .difference(frozen)
        .map(|key| format!("  NEW    {key}"))
        .collect();
    drift.extend(
        frozen
            .difference(observed)
            .map(|key| format!("  STALE  {key}")),
    );
    drift
}

/// ANTI-VACUITY, asserted before any equality below is read.
///
/// Sets pinned by equality cannot distinguish "the registry is in order" from "the walk collapsed";
/// both drive the observed sets toward empty, and two of the four frozen sets are empty TODAY, so
/// without these floors a caller that parsed nothing would look identical to a clean registry.
/// Every floor counts SUBJECT rows and tracked paths, never findings. `min_rows_with_tracked_path`
/// deserves the specific note: it is the count of rows that RESOLVE, so repairing an untracked row
/// moves it UP, never down — no floor here can red on honest progress.
#[test]
fn the_registry_corpus_is_intact() {
    let (policy, observed) = live();
    assert!(
        observed.tracked_files >= policy.min_tracked_files,
        "git ls-files returned {} tracked paths, below the floor of {} — the HEAD-tracked set is \
         the entire right-hand side of rule R01, so a collapse here makes every row look untracked \
         or, worse, makes a narrowed walk look like a clean one\n{}",
        observed.tracked_files,
        policy.min_tracked_files,
        census(observed)
    );
    assert!(
        observed.rows >= policy.min_registry_rows,
        "{} registry rows parsed, below the floor of {}. Rows do not disappear in bulk; a drop here \
         is a narrowed parse, and a narrowed parse reports a perfectly-declared control plane it \
         never read\n{}",
        observed.rows,
        policy.min_registry_rows,
        census(observed)
    );
    let resolving = observed.rows - observed.untracked_artifact_paths.len();
    assert!(
        resolving >= policy.min_rows_with_tracked_path,
        "{resolving} rows resolve to a HEAD-tracked file, below the floor of {} — the registry has \
         stopped describing the tree it is the control plane for\n{}",
        policy.min_rows_with_tracked_path,
        census(observed)
    );
}

/// THE GATE: R01, as a TWO-SIDED, equality-pinned SET of `<artifact_id>::<artifact_path>`.
///
/// Keys, not a count. A count would tell a reviewer that the number moved and nothing about which
/// row moved; the key names the row and the exact path it declares, and is reviewable without
/// opening the registry.
///
/// Both entries frozen today are the same defect wearing the same clothes: the row points at a
/// DIRECTORY (registry/check-empirical-evidence/, registry/loop-recovery-patterns/). Both
/// directories exist on disk, which is precisely why an existence check would have called this
/// registry clean; the contract's rule is a HEAD-TRACKED-PATH rule, and `git ls-files` yields files.
#[test]
fn untracked_artifact_paths_equal_the_frozen_set() {
    let (policy, observed) = live();
    let drift = set_drift(
        &policy.frozen_untracked_artifact_paths,
        &observed.untracked_artifact_paths,
    );
    assert!(
        drift.is_empty(),
        "R01 drift — registry rows whose artifact_path is not HEAD-tracked. NEW: a row points at \
         something `git ls-files` does not yield, usually a directory, a path that was renamed, or \
         a file that was deleted without its row. STALE: the row was repaired, so strike its line \
         from `frozen_untracked_artifact_paths` in THIS change; or the walk narrowed and is \
         reporting green over rows it stopped parsing, which the floors above are there to catch. \
         Re-derive by RUNNING this gate and reading these lines; never by arithmetic on the old \
         values:\n{}\n{}",
        drift.join("\n"),
        census(observed)
    );
}

/// R02, equality-pinned and empty. A duplicated `artifact_id` makes one of the two rows unreachable
/// to every consumer that keys on the id, so the debt is born blocking rather than tolerated.
#[test]
fn duplicate_artifact_ids_equal_the_frozen_set() {
    let (policy, observed) = live();
    let drift = set_drift(
        &policy.frozen_duplicate_artifact_ids,
        &observed.duplicate_artifact_ids,
    );
    assert!(
        drift.is_empty(),
        "R02 drift — duplicate artifact_id. NEW: two rows claim the same id, so one of them is \
         invisible to every id-keyed consumer; rename one. STALE: the collision was resolved, so \
         strike the line here in the same change:\n{}\n{}",
        drift.join("\n"),
        census(observed)
    );
}

/// `ArtifactProfile` is a CLOSED enum of seven values, and four rows name something else.
///
/// The kernel's `validate()` never looks at the profile, so this divergence produced no violation
/// and could sit unnoticed indefinitely — while `specs/artifact-profile-defaults.json` supplies
/// those four rows no defaults at all, which is exactly the silent hole the profile system was
/// introduced to close. Frozen by name so each is attributable, two-sided so a fifth cannot arrive
/// unrecorded and a repair cannot go unrecorded either.
#[test]
fn unknown_artifact_profiles_equal_the_frozen_set() {
    let (policy, observed) = live();
    let drift = set_drift(
        &policy.frozen_unknown_artifact_profiles,
        &observed.unknown_artifact_profiles,
    );
    assert!(
        drift.is_empty(),
        "artifact_profile drift against the closed enum {:?}. NEW: a row names a profile the kernel \
         cannot parse, so it inherits no capability defaults — either widen the enum deliberately \
         or move the row onto an existing profile. STALE: the row was repaired, so strike it here \
         in the same change:\n{}\n{}",
        ArtifactProfile::all().map(ArtifactProfile::name),
        drift.join("\n"),
        census(observed)
    );
}

/// THE TRIPWIRE that keeps this gate's scope limit honest.
///
/// This caller binds R01 and R02 only, because R03-R07 are all functions of a per-capability
/// `status` that no row in this registry declares. That is a statement about the corpus, and a
/// statement about the corpus can stop being true. Rather than leave the omission as a comment
/// nobody re-checks, it is asserted: the day a row declares a real status, this reddens, and the
/// fix is to teach the caller to feed real statuses to the kernel so R04-R07 begin running — NOT
/// to add the row to this frozen set.
#[test]
fn no_registry_row_declares_a_capability_status() {
    let (policy, observed) = live();
    let drift = set_drift(
        &policy.frozen_rows_declaring_capability_status,
        &observed.rows_declaring_capability_status,
    );
    assert!(
        drift.is_empty(),
        "a registry row now declares a per-capability `status`, so rules R04-R07 have real input \
         for the first time and this caller is no longer feeding the kernel a neutral filler \
         honestly. Widen `neutral_capabilities()` in this test into a real resolver \
         (artifact_profile defaults from specs/artifact-profile-defaults.json merged with \
         capability_overrides) and freeze the resulting R04-R07 findings. Do NOT silence this by \
         listing the row below:\n{}\n{}",
        drift.join("\n"),
        census(observed)
    );
}

/// The neutral filler is PROVEN neutral against the live corpus, not asserted in a comment.
///
/// If `neutral_capabilities()` ever produced an R03-R07 violation, the two frozen sets above would
/// be measuring this test's defaulting choices rather than the registry. `observe()` already treats
/// any such rule as a hard error; this names the property so a reader sees it tested.
#[test]
fn the_capability_filler_is_neutral() {
    let (_, observed) = live();
    let contaminating: Vec<&str> = observed
        .report
        .violations
        .iter()
        .map(|violation| violation.rule_id)
        .filter(|rule| {
            !matches!(
                *rule,
                "R01-artifact-path-not-in-head" | "R02-duplicate-artifact-id"
            )
        })
        .collect();
    assert!(
        contaminating.is_empty(),
        "the neutral capability filler produced findings of its own ({contaminating:?}); every \
         count in the frozen sets is contaminated by this caller's defaulting choices\n{}",
        census(observed)
    );
}

/// The gate is DEMONSTRATED CAPABLE OF FAILING against the REAL corpus, not just fixtures.
///
/// A green equality proves nothing on its own — a caller that silently handed the kernel zero rows
/// would satisfy every assertion above by reporting a perfectly declared registry. This takes the
/// REAL parsed rows and the REAL tracked-path set and injects each defect shape, then asserts the
/// violation names THAT injection. The plants are structural: a path that cannot exist and an id
/// copied from a real row. Neither can be satisfied by prose describing it, because the kernel
/// compares set membership and string equality rather than scanning for a substring.
#[test]
fn injecting_each_defect_into_the_real_registry_reddens_the_gate() {
    let root = repo_root();
    let tracked = tracked_files(&root).expect("git ls-files");
    let registry_rows = parse_registry(&root).expect("parse registry");
    assert!(
        !registry_rows.is_empty(),
        "the registry parsed to zero rows; this doctrine has no live subject and must be deleted \
         rather than connected"
    );

    let baseline_rows: Vec<ArtifactRow> = registry_rows
        .iter()
        .map(|entry| ArtifactRow {
            artifact_path: normalize(&entry.row.artifact_path).to_owned(),
            ..entry.row.clone()
        })
        .collect();
    let baseline = validate(&baseline_rows, &tracked);

    // Defect 1 — R01: a row declaring a path that is not HEAD-tracked.
    const PLANTED_PATH: &str = "registry/zzz-planted-by-the-r01-probe/does-not-exist.json";
    assert!(
        !tracked.contains(PLANTED_PATH),
        "the R01 probe path is somehow tracked; pick another"
    );
    let mut with_untracked = baseline_rows.clone();
    with_untracked.push(ArtifactRow {
        artifact_id: "zzz-planted-r01-probe".to_owned(),
        artifact_path: PLANTED_PATH.to_owned(),
        artifact_format: "json".to_owned(),
        contract_version: "v3.0.0".to_owned(),
        capabilities: neutral_capabilities(),
    });
    let untracked_report = validate(&with_untracked, &tracked);
    let r01 = named_violations(&untracked_report, "R01-artifact-path-not-in-head");
    assert_eq!(
        r01.len(),
        named_violations(&baseline, "R01-artifact-path-not-in-head").len() + 1,
        "planting an untracked artifact_path did not raise the R01 count"
    );
    assert!(
        r01.contains(&"zzz-planted-r01-probe"),
        "R01 fired, but not for the planted row — the rise is not attributable to the injection: \
         {r01:?}"
    );

    // Defect 2 — R02: a second row claiming an id a real row already owns.
    let victim = registry_rows[0].row.artifact_id.clone();
    let mut with_duplicate = baseline_rows.clone();
    with_duplicate.push(ArtifactRow {
        artifact_id: victim.clone(),
        artifact_path: normalize(&registry_rows[0].row.artifact_path).to_owned(),
        artifact_format: "json".to_owned(),
        contract_version: "v3.0.0".to_owned(),
        capabilities: neutral_capabilities(),
    });
    let duplicate_report = validate(&with_duplicate, &tracked);
    let r02 = named_violations(&duplicate_report, "R02-duplicate-artifact-id");
    assert_eq!(
        r02,
        vec![victim.as_str()],
        "duplicating the live artifact_id `{victim}` did not produce exactly that R02 finding"
    );

    println!(
        "mutation proof against the live registry ({} rows, {} tracked paths): R01 {} -> {} naming \
         `zzz-planted-r01-probe`; R02 {} -> {} naming `{victim}`",
        registry_rows.len(),
        tracked.len(),
        named_violations(&baseline, "R01-artifact-path-not-in-head").len(),
        r01.len(),
        named_violations(&baseline, "R02-duplicate-artifact-id").len(),
        r02.len(),
    );
}

fn named_violations<'a>(report: &'a ValidationReport, rule_id: &str) -> Vec<&'a str> {
    report
        .violations
        .iter()
        .filter(|violation| violation.rule_id == rule_id)
        .map(|violation| violation.artifact_id.as_str())
        .collect()
}

/// Evidence, always printed, so a reader can tell a well-declared registry from a collapsed walk
/// without re-running anything.
#[test]
fn live_census_is_reported() {
    let (_, observed) = live();
    println!("{}", census(observed));
    assert!(observed.rows > 0);
}
