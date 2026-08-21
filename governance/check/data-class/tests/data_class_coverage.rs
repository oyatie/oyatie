// governance-check-data-class LIVE-TREE gate (MFL-0008 data-class fitness).
//
// The kernel is pure; this is the CALLER that walks the real repository and hands it observations
// as DATA. Until this file existed the doctrine had never produced a verdict about this codebase:
// the kernel's only consumer was marketplace/facade/dev-cli, which no workflow invokes, so every
// case it had ever run was one of the four hand-written fixtures in `src/lib.rs`. Those fixtures
// are still there and are still the proof that the kernel is correct; this file is the separate
// proof that the kernel has ever been pointed at the tree.
//
// WHAT THE SUBJECT IS. The doctrine governs KERNEL-tier crates: a workspace member whose
// `registry/catalog/<package>.yaml` declares `role: kernel`. That is not a scope invented here to
// make a number small — it is the rule the producer in
// marketplace/facade/dev-cli/src/data_class_gates.rs already applies, and this caller replays it
// so the live verdict is the verdict that producer would have given had anything ever run it.
//
// Walk failures are ERRORS, never omitted observations: a source file dropped from the census
// because it failed to read would quietly shrink the frozen map, and a shrink reads as repair.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use check_data_class::{
    DataClassFitnessError, FieldIdentity, KernelField, LegacyUnannotatedField,
    validate_data_class_fitness,
};
use workspace_members_kernel::resolve_member_dirs;

const POLICY_PATH: &str = "governance/check/data-class/data-class-policy.json";
const LEDGER_PATH: &str = "registry/data-class/legacy-unannotated-fields.tsv";
const CATALOG_DIR: &str = "registry/catalog";

/// How many drifted keys a failure message names before it truncates. Long enough that the
/// ordinary one-or-two-key drift is always fully named, short enough that a corpus-wide collapse
/// does not bury the census line under a thousand paths.
const REPORT_LIMIT: usize = 40;

/// The frozen verdict for a field that carries no data-class annotation.
const ALLOWED: &str = "legacy-allowed";
const UNALLOWED: &str = "unallowed";

struct Policy {
    min_tracked_files: usize,
    min_kernel_members: usize,
    min_kernel_source_files: usize,
    min_fields: usize,
    min_annotated_fields: usize,
    frozen_unannotated_fields: BTreeMap<String, String>,
    frozen_stale_allowances: BTreeSet<String>,
    frozen_unscanned_kernel_catalogs: BTreeSet<String>,
}

struct Observed {
    kernel_members: usize,
    kernel_source_files: usize,
    fields: Vec<KernelField>,
    allowances: Vec<LegacyUnannotatedField>,
    /// Every field carrying no annotation, mapped to whether the legacy ledger allows it.
    ///
    /// A MAP, not two sets, because the interesting movements here are TRANSITIONS. Annotating a
    /// field removes its key; adding a new unannotated field adds one; deleting a ledger row for a
    /// field nobody annotated flips `legacy-allowed` -> `unallowed`; adding a ledger row flips it
    /// the other way. All four are drift against this one structure, and every one of them names
    /// the exact field, which a count never could.
    ///
    /// Keyed `<path>::<Struct>::<field>` — never line-anchored, because a line number moves
    /// whenever anything above the field is edited, and an edit above a line-anchored construct
    /// can leave no legal edit at all.
    unannotated: BTreeMap<String, String>,
    /// Ledger rows that match no unannotated field: the allowance is spent or was never real.
    stale_allowances: BTreeSet<String>,
    /// A `role: kernel` catalog that resolves to a workspace member the scan did not cover — a
    /// kernel crate escaping the doctrine entirely.
    unscanned_kernel_catalogs: BTreeSet<String>,
    tracked_files: usize,
}

fn key_of(identity: &FieldIdentity) -> String {
    format!(
        "{}::{}::{}",
        identity.path, identity.struct_name, identity.field_name
    )
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
    let set = |key: &str| -> BTreeSet<String> {
        doc[key]
            .as_array()
            .unwrap_or_else(|| panic!("policy field {key} missing or not an array"))
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .unwrap_or_else(|| panic!("{key} holds a non-string"))
                    .to_owned()
            })
            .collect()
    };
    let frozen_unannotated_fields = doc["frozen_unannotated_fields"]
        .as_object()
        .expect("policy field frozen_unannotated_fields missing or not an object")
        .iter()
        .map(|(key, value)| {
            let verdict = value
                .as_str()
                .unwrap_or_else(|| panic!("frozen_unannotated_fields[{key}] is not a string"));
            assert!(
                verdict == ALLOWED || verdict == UNALLOWED,
                "frozen_unannotated_fields[{key}] is '{verdict}', which is neither {ALLOWED} nor \
                 {UNALLOWED}"
            );
            (key.clone(), verdict.to_owned())
        })
        .collect();
    Policy {
        min_tracked_files: number("min_tracked_files"),
        min_kernel_members: number("min_kernel_members"),
        min_kernel_source_files: number("min_kernel_source_files"),
        min_fields: number("min_fields"),
        min_annotated_fields: number("min_annotated_fields"),
        frozen_unannotated_fields,
        frozen_stale_allowances: set("frozen_stale_allowances"),
        frozen_unscanned_kernel_catalogs: set("frozen_unscanned_kernel_catalogs"),
    }
}

/// The tracked file list, from git — the same corpus boundary every other live gate here uses.
///
/// The dev-cli producer walks the filesystem instead. This caller deliberately does not: with the
/// map pinned by equality, an ignored or untracked `*.rs` sitting in a kernel `src/` would make a
/// developer's run disagree with CI's, and a red gate CI cannot reproduce is worse than no gate.
fn tracked_files(root: &Path) -> Result<Vec<String>, String> {
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

/// The `role:` a catalog record declares. Same top-level-key scan the producer uses.
fn catalog_role(text: &str) -> Option<String> {
    for line in text.lines() {
        let stripped = line.trim();
        if stripped.is_empty() || stripped.starts_with('#') {
            continue;
        }
        let Some((key, value)) = stripped.split_once(':') else {
            continue;
        };
        if key.trim() == "role" {
            return Some(value.trim().to_owned());
        }
    }
    None
}

/// `traceability.source_crate`, when the record carries one.
fn catalog_source_crate(text: &str) -> Option<String> {
    let mut in_traceability = false;
    for line in text.lines() {
        let stripped = line.trim();
        if stripped.is_empty() || stripped.starts_with('#') {
            continue;
        }
        if stripped == "traceability:" {
            in_traceability = true;
            continue;
        }
        if in_traceability {
            let indented = line.starts_with(' ') || line.starts_with('\t');
            if !indented && stripped.contains(':') {
                break;
            }
            let Some((key, value)) = stripped.split_once(':') else {
                continue;
            };
            if key.trim() == "source_crate" {
                return Some(value.trim().to_owned());
            }
        }
    }
    None
}

fn package_name(manifest: &Path) -> Result<Option<String>, String> {
    let text = std::fs::read_to_string(manifest)
        .map_err(|e| format!("manifest unreadable {}: {e}", manifest.display()))?;
    let doc: toml::Value = toml::from_str(&text)
        .map_err(|e| format!("manifest unparseable {}: {e}", manifest.display()))?;
    Ok(doc
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned))
}

/// Parse the `pub struct` fields of one source file, exactly as the dev-cli producer does.
///
/// This is a RE-STATEMENT of `marketplace/facade/dev-cli/src/data_class_gates.rs::parse_kernel_fields`,
/// not a fresh rule: that function is `pub(crate)` inside a facade binary crate, so it cannot be
/// called from here, and a live gate that invented its own extraction would freeze a finding set
/// the producer never produces. The agreement between the two is not left to inspection —
/// `the_kernel_agrees_with_this_callers_partition` hands the result back to the kernel and
/// requires the kernel to accept it, so any drift in this parser shows up as a kernel error rather
/// than as a quietly wrong baseline.
fn parse_kernel_fields(path: &str, contents: &str) -> Vec<KernelField> {
    let mut fields = Vec::new();
    let mut current_struct: Option<String> = None;
    let mut brace_depth = 0_i32;
    let mut previous_line_has_data_class_annotation = false;

    for line in contents.lines() {
        let trimmed = line.trim();
        let Some(active_struct) = current_struct.clone() else {
            if let Some(struct_name) = pub_struct_name(trimmed) {
                current_struct = Some(struct_name);
                brace_depth = brace_delta(line);
                previous_line_has_data_class_annotation = false;
                if brace_depth <= 0 {
                    current_struct = None;
                }
            }
            continue;
        };

        if let Some(field_name) = pub_field_name(trimmed) {
            let has_data_class_annotation = previous_line_has_data_class_annotation
                || trimmed.contains("data_class:")
                || trimmed.contains("Classified<")
                || trimmed.contains("DataClass")
                || field_name == "data_class"
                || field_name == "data_classes_touched";
            fields.push(KernelField {
                identity: FieldIdentity {
                    path: path.to_owned(),
                    struct_name: active_struct,
                    field_name,
                },
                has_data_class_annotation,
            });
            previous_line_has_data_class_annotation = false;
        } else if trimmed.starts_with("//") {
            previous_line_has_data_class_annotation = trimmed.contains("data_class:");
        } else if trimmed.starts_with("#[") || trimmed.is_empty() {
        } else {
            previous_line_has_data_class_annotation = false;
        }

        brace_depth += brace_delta(line);
        if brace_depth <= 0 {
            current_struct = None;
            previous_line_has_data_class_annotation = false;
        }
    }

    fields
}

fn pub_struct_name(trimmed: &str) -> Option<String> {
    let rest = trimmed.strip_prefix("pub struct ")?;
    let name = rest
        .split(|character: char| character == '<' || character == '{' || character.is_whitespace())
        .next()?
        .trim();
    (!name.is_empty()).then(|| name.to_owned())
}

fn pub_field_name(trimmed: &str) -> Option<String> {
    let mut rest = trimmed.strip_prefix("pub")?.trim_start();
    if rest.starts_with('(') {
        let (_, after_visibility) = rest.split_once(')')?;
        rest = after_visibility.trim_start();
    }
    let (field_name, _) = rest.split_once(':')?;
    let field_name = field_name.trim();
    if field_name.is_empty() || field_name.contains(char::is_whitespace) {
        None
    } else {
        Some(field_name.trim_start_matches("r#").to_owned())
    }
}

fn brace_delta(line: &str) -> i32 {
    i32::try_from(line.chars().filter(|c| *c == '{').count()).unwrap_or(i32::MAX)
        - i32::try_from(line.chars().filter(|c| *c == '}').count()).unwrap_or(i32::MAX)
}

fn read_ledger(root: &Path) -> Result<Vec<LegacyUnannotatedField>, String> {
    let text = std::fs::read_to_string(root.join(LEDGER_PATH))
        .map_err(|e| format!("legacy data-class allowance ledger unreadable: {e}"))?;
    let mut allowances = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.splitn(4, '\t').collect();
        if parts.len() != 4 {
            return Err(format!(
                "{LEDGER_PATH}:{}: a legacy allowance must have four tab-separated fields",
                index + 1
            ));
        }
        allowances.push(LegacyUnannotatedField {
            identity: FieldIdentity {
                path: parts[0].to_owned(),
                struct_name: parts[1].to_owned(),
                field_name: parts[2].to_owned(),
            },
            rationale: parts[3].to_owned(),
        });
    }
    Ok(allowances)
}

fn observe(root: &Path) -> Result<Observed, String> {
    let tracked = tracked_files(root)?;
    let members = resolve_member_dirs(root).map_err(|e| format!("workspace members: {e:?}"))?;

    // Which members are kernel-tier? By the catalog record named for the member's package.
    let mut kernel_members: BTreeSet<String> = BTreeSet::new();
    let mut member_by_package: BTreeMap<String, String> = BTreeMap::new();
    for member in &members {
        let manifest = root.join(member).join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        let Some(name) = package_name(&manifest)? else {
            continue;
        };
        member_by_package.insert(name.clone(), member.clone());
        let catalog = root.join(CATALOG_DIR).join(format!("{name}.yaml"));
        if !catalog.is_file() {
            continue;
        }
        let text = std::fs::read_to_string(&catalog)
            .map_err(|e| format!("catalog unreadable {}: {e}", catalog.display()))?;
        if catalog_role(&text).as_deref() == Some("kernel") {
            kernel_members.insert(member.clone());
        }
    }

    // COMPLETENESS, the producer's own second pass: a `role: kernel` catalog that resolves to a
    // workspace member must be covered by the scan, or a kernel crate escapes the doctrine while
    // every count below still looks healthy.
    let mut unscanned_kernel_catalogs = BTreeSet::new();
    for relative in &tracked {
        let Some(name) = relative
            .strip_prefix(&format!("{CATALOG_DIR}/"))
            .and_then(|rest| rest.strip_suffix(".yaml"))
        else {
            continue;
        };
        let text = std::fs::read_to_string(root.join(relative))
            .map_err(|e| format!("catalog unreadable {relative}: {e}"))?;
        if catalog_role(&text).as_deref() != Some("kernel") {
            continue;
        }
        let resolved = catalog_source_crate(&text)
            .map(|source| {
                source
                    .strip_suffix("/Cargo.toml")
                    .unwrap_or(source.as_str())
                    .to_owned()
            })
            .filter(|member| members.iter().any(|candidate| candidate == member))
            .or_else(|| member_by_package.get(name).cloned());
        if let Some(member) = resolved
            && !kernel_members.contains(&member)
        {
            unscanned_kernel_catalogs.insert(format!("{name}::{member}"));
        }
    }

    // The fields themselves, from the TRACKED sources of every kernel member.
    let mut fields: Vec<KernelField> = Vec::new();
    let mut kernel_source_files = 0usize;
    for relative in &tracked {
        if !relative.ends_with(".rs") {
            continue;
        }
        if !kernel_members
            .iter()
            .any(|member| relative.starts_with(&format!("{member}/src/")))
        {
            continue;
        }
        let path = root.join(relative);
        // Every failure below is an ERROR, never an omitted observation.
        let metadata = std::fs::metadata(&path)
            .map_err(|e| format!("metadata {relative} failed: {e} (tracked but unreadable)"))?;
        if !metadata.is_file() {
            continue; // a tracked symlink to a directory carries no fields
        }
        let contents = std::fs::read_to_string(&path)
            .map_err(|e| format!("kernel source unreadable {relative}: {e}"))?;
        kernel_source_files += 1;
        fields.extend(parse_kernel_fields(relative, &contents));
    }

    let allowances = read_ledger(root)?;

    let unannotated_keys: BTreeSet<String> = fields
        .iter()
        .filter(|field| !field.has_data_class_annotation)
        .map(|field| key_of(&field.identity))
        .collect();
    let allowance_keys: BTreeSet<String> = allowances
        .iter()
        .map(|allowance| key_of(&allowance.identity))
        .collect();

    let unannotated: BTreeMap<String, String> = unannotated_keys
        .iter()
        .map(|key| {
            let verdict = if allowance_keys.contains(key) {
                ALLOWED
            } else {
                UNALLOWED
            };
            (key.clone(), verdict.to_owned())
        })
        .collect();
    let stale_allowances: BTreeSet<String> = allowance_keys
        .difference(&unannotated_keys)
        .cloned()
        .collect();

    Ok(Observed {
        kernel_members: kernel_members.len(),
        kernel_source_files,
        fields,
        allowances,
        unannotated,
        stale_allowances,
        unscanned_kernel_catalogs,
        tracked_files: tracked.len(),
    })
}

/// The live walk, done ONCE for the whole binary: it is a pure function of the tree, so every test
/// re-walking it would re-parse the same ~600 kernel sources.
fn live() -> &'static (Policy, Observed) {
    static LIVE: OnceLock<(Policy, Observed)> = OnceLock::new();
    LIVE.get_or_init(|| {
        let root = repo_root();
        let policy = load_policy(&root);
        let observed = observe(&root).expect("live walk");
        (policy, observed)
    })
}

fn annotated(observed: &Observed) -> usize {
    observed
        .fields
        .iter()
        .filter(|field| field.has_data_class_annotation)
        .count()
}

fn census(observed: &Observed) -> String {
    let allowed = observed
        .unannotated
        .values()
        .filter(|verdict| *verdict == ALLOWED)
        .count();
    format!(
        "census: {} kernel workspace members, {} tracked kernel sources over {} tracked files; {} \
         pub struct fields, {} annotated, {} unannotated ({allowed} allowed by the {}-row ledger, \
         {} unallowed); {} stale ledger rows; {} unscanned kernel catalogs\n",
        observed.kernel_members,
        observed.kernel_source_files,
        observed.tracked_files,
        observed.fields.len(),
        annotated(observed),
        observed.unannotated.len(),
        observed.allowances.len(),
        observed.unannotated.len() - allowed,
        observed.stale_allowances.len(),
        observed.unscanned_kernel_catalogs.len(),
    )
}

fn set_drift(label: &str, frozen: &BTreeSet<String>, observed: &BTreeSet<String>) -> String {
    let mut message = String::new();
    for (direction, keys) in [
        ("+", observed.difference(frozen).collect::<Vec<_>>()),
        ("-", frozen.difference(observed).collect::<Vec<_>>()),
    ] {
        if keys.is_empty() {
            continue;
        }
        message.push_str(&format!(
            "{} {label} finding(s) {}\n",
            keys.len(),
            if direction == "+" {
                "are NEW and not in the frozen set"
            } else {
                "are frozen but no longer produced — strike them in THIS change if you fixed them, \
                 or discover that the scan narrowed"
            }
        ));
        for key in keys.iter().take(REPORT_LIMIT) {
            message.push_str(&format!("  {direction} {key}\n"));
        }
        if keys.len() > REPORT_LIMIT {
            message.push_str(&format!("  … {} more\n", keys.len() - REPORT_LIMIT));
        }
    }
    message
}

/// ANTI-VACUITY, asserted before any equality below is read.
///
/// A ratchet pinned by equality cannot distinguish "the corpus was annotated" from "the walk
/// collapsed"; both drive the observed map toward empty. These floors are the machine oracle that
/// separates them. Every floor counts SUBJECTS — members, source files, fields, tracked paths —
/// except `min_annotated_fields`, which counts the COMPLIANT half and therefore only ever rises as
/// the debt is paid. No floor here can red on honest progress.
#[test]
fn the_kernel_corpus_is_intact() {
    let (policy, observed) = live();
    assert!(
        observed.tracked_files >= policy.min_tracked_files,
        "git ls-files returned {} tracked paths, below the floor of {} — the corpus walk is broken \
         and every count below is meaningless\n{}",
        observed.tracked_files,
        policy.min_tracked_files,
        census(observed)
    );
    assert!(
        observed.kernel_members >= policy.min_kernel_members,
        "{} kernel-tier workspace members resolved, below the floor of {}. Kernel crates do not \
         disappear in bulk; a drop here means the member resolver or the catalog role scan stopped \
         matching, and a scan that finds no kernels reports a perfectly annotated tree it never \
         read\n{}",
        observed.kernel_members,
        policy.min_kernel_members,
        census(observed)
    );
    assert!(
        observed.kernel_source_files >= policy.min_kernel_source_files,
        "{} tracked kernel sources read, below the floor of {}\n{}",
        observed.kernel_source_files,
        policy.min_kernel_source_files,
        census(observed)
    );
    assert!(
        observed.fields.len() >= policy.min_fields,
        "{} pub struct fields parsed, below the floor of {} — the field extractor stopped \
         matching\n{}",
        observed.fields.len(),
        policy.min_fields,
        census(observed)
    );
    assert!(
        annotated(observed) >= policy.min_annotated_fields,
        "{} ANNOTATED fields, below the floor of {}. This floor counts the compliant half, so it \
         rises as the debt is paid and can only fall if annotations were deleted or the corpus \
         shrank\n{}",
        annotated(observed),
        policy.min_annotated_fields,
        census(observed)
    );
}

/// THE GATE: a TWO-SIDED, shrink-only ratchet on the MAP of unannotated kernel fields.
///
/// A MAP of `<path>::<Struct>::<field>` -> `legacy-allowed` | `unallowed`, pinned by EQUALITY over
/// the UNION of the frozen and observed key sets. Four distinct movements all land here, and each
/// one names the exact field:
///   * a NEW unannotated field appears as a new key and blocks — this doctrine is fail-closed for
///     new fields, so it is born-blocking;
///   * an ANNOTATED field loses its key and blocks until struck, so the win is recorded rather
///     than silently absorbed;
///   * a ledger row DELETED without annotating its field flips `legacy-allowed` -> `unallowed`;
///   * a ledger row ADDED for an existing unallowed field flips `unallowed` -> `legacy-allowed`,
///     which is how the shrink-only ledger is actually enforced: laundering debt into
///     `registry/data-class/legacy-unannotated-fields.tsv` is still possible, but it can no longer
///     be quiet — it costs a visible two-file diff naming the field.
#[test]
fn unannotated_fields_equal_the_frozen_map() {
    let (policy, observed) = live();

    let keys: BTreeSet<&String> = policy
        .frozen_unannotated_fields
        .keys()
        .chain(observed.unannotated.keys())
        .collect();
    let drift: Vec<String> = keys
        .into_iter()
        .filter_map(|key| {
            let seen = observed
                .unannotated
                .get(key)
                .map_or("absent", String::as_str);
            let want = policy
                .frozen_unannotated_fields
                .get(key)
                .map_or("absent", String::as_str);
            (seen != want).then(|| format!("  {key}: observed {seen}, frozen {want}"))
        })
        .collect();

    let shown: Vec<&String> = drift.iter().take(REPORT_LIMIT).collect();
    assert!(
        drift.is_empty(),
        "data-class drift, per field. `observed unallowed, frozen absent`: a new kernel field \
         carries no `// data_class: <LEVEL>` annotation — annotate it; the ledger is closed to new \
         rows. `observed absent, frozen …`: the field was annotated or removed, so strike its line \
         from `frozen_unannotated_fields` in THIS change and, if it had a ledger row, delete that \
         row too. `observed unallowed, frozen legacy-allowed`: a ledger row was deleted without \
         annotating the field. Re-derive by RUNNING this gate and reading these lines; never by \
         arithmetic on the old values:\n{}{}\n{}",
        shown
            .iter()
            .map(|line| format!("{line}\n"))
            .collect::<String>(),
        if drift.len() > REPORT_LIMIT {
            format!("  … {} more\n", drift.len() - REPORT_LIMIT)
        } else {
            String::new()
        },
        census(observed)
    );
}

/// A ledger row that matches no unannotated field is spent, and pinned by EQUALITY at the empty
/// set — never by a floor, because the honest target of this term is ZERO and a floor on a
/// zero-target term goes red exactly when the doctrine succeeds.
///
/// This set was 274 — every row in the ledger — until 2026-08-19, because the 13 shared kernels it
/// covers MOVED from `crates/` to `libs/` and the ledger kept the old paths. The rows were not
/// obsolete; they had merely stopped naming anything, which is the failure mode a stale-allowance
/// check exists to surface and which nothing had ever run to surface it.
#[test]
fn stale_ledger_rows_equal_the_frozen_set() {
    let (policy, observed) = live();
    let message = set_drift(
        "stale ledger row",
        &policy.frozen_stale_allowances,
        &observed.stale_allowances,
    );
    assert!(message.is_empty(), "{message}{}", census(observed));
}

/// A `role: kernel` catalog whose workspace member the scan never covered means a kernel crate is
/// outside the doctrine while every other count still looks healthy. Pinned by EQUALITY at the
/// empty set for the same reason as above.
#[test]
fn unscanned_kernel_catalogs_equal_the_frozen_set() {
    let (policy, observed) = live();
    let message = set_drift(
        "unscanned kernel catalog",
        &policy.frozen_unscanned_kernel_catalogs,
        &observed.unscanned_kernel_catalogs,
    );
    assert!(message.is_empty(), "{message}{}", census(observed));
}

/// THE KERNEL IS THE ORACLE for this caller's partition.
///
/// Everything above is the caller's own arithmetic over its own parse. That arithmetic could drift
/// from `validate_data_class_fitness` — the very function this crate exists to be — and the frozen
/// map would then be a faithful record of the wrong rule. So: grant an allowance for exactly the
/// fields the caller calls unallowed, drop exactly the rows it calls stale, and hand the whole
/// observation to the kernel. The kernel accepts if and only if the caller's partition IS the
/// kernel's partition, and its report re-derives the three counts independently.
#[test]
fn the_kernel_agrees_with_this_callers_partition() {
    let (_, observed) = live();

    let stale = &observed.stale_allowances;
    let mut granted: Vec<LegacyUnannotatedField> = observed
        .allowances
        .iter()
        .filter(|allowance| !stale.contains(&key_of(&allowance.identity)))
        .cloned()
        .collect();
    for field in &observed.fields {
        if field.has_data_class_annotation {
            continue;
        }
        if observed
            .unannotated
            .get(&key_of(&field.identity))
            .map(String::as_str)
            != Some(UNALLOWED)
        {
            continue;
        }
        granted.push(LegacyUnannotatedField {
            identity: field.identity.clone(),
            rationale: "granted by the live gate solely to ask the kernel whether this caller's \
                        partition is the kernel's partition"
                .to_owned(),
        });
    }

    let report = validate_data_class_fitness(&observed.fields, &granted).unwrap_or_else(|error| {
        panic!(
            "the kernel REJECTED the caller's own partition: {error:?}. The frozen map above is \
             therefore a record of a rule the kernel does not implement — fix the caller, never \
             the baseline.\n{}",
            census(observed)
        )
    });
    assert_eq!(
        report.fields_checked,
        observed.fields.len(),
        "kernel and caller disagree about how many fields exist\n{}",
        census(observed)
    );
    assert_eq!(
        report.annotated_fields,
        annotated(observed),
        "kernel and caller disagree about how many fields are annotated\n{}",
        census(observed)
    );
    assert_eq!(
        report.legacy_unannotated_fields,
        observed.unannotated.len(),
        "kernel and caller disagree about how many fields are unannotated\n{}",
        census(observed)
    );
}

/// The LIVE VERDICT, stated rather than implied — and stated so that it cannot punish progress.
///
/// Handed the real ledger, the kernel today REJECTS this tree, because hundreds of kernel fields
/// carry no annotation and no allowance. That is the finding this gate exists to hold still. The
/// assertion is conditional on the observed state, never on a hard-coded non-zero count: if the
/// debt is ever paid in full the kernel returns `Ok` and this test stays green, which is how a
/// liveness assert avoids going red exactly when the burn-down succeeds.
#[test]
fn the_kernel_verdict_on_the_real_ledger_matches_the_frozen_map() {
    let (_, observed) = live();
    let unallowed: BTreeSet<&String> = observed
        .unannotated
        .iter()
        .filter_map(|(key, verdict)| (verdict == UNALLOWED).then_some(key))
        .collect();

    match validate_data_class_fitness(&observed.fields, &observed.allowances) {
        Ok(report) => {
            assert!(
                unallowed.is_empty() && observed.stale_allowances.is_empty(),
                "the kernel accepted the tree, but this caller still reports {} unallowed field(s) \
                 and {} stale ledger row(s) — caller and kernel disagree about the live \
                 verdict\n{}",
                unallowed.len(),
                observed.stale_allowances.len(),
                census(observed)
            );
            println!("kernel verdict: CLEAN — {report:?}\n{}", census(observed));
        }
        Err(DataClassFitnessError::UnknownUnannotatedField { field }) => {
            let key = key_of(&field);
            assert!(
                unallowed.contains(&key),
                "the kernel rejected {key}, which this caller does not list as unallowed — the \
                 frozen map is not the kernel's verdict\n{}",
                census(observed)
            );
            println!(
                "kernel verdict: REJECTED at {key} (first of {} unallowed fields)\n{}",
                unallowed.len(),
                census(observed)
            );
        }
        Err(DataClassFitnessError::StaleLegacyAllowance { field }) => {
            let key = key_of(&field);
            assert!(
                observed.stale_allowances.contains(&key),
                "the kernel rejected stale allowance {key}, which this caller does not list as \
                 stale\n{}",
                census(observed)
            );
            println!(
                "kernel verdict: STALE ALLOWANCE {key}\n{}",
                census(observed)
            );
        }
        Err(other) => panic!(
            "the kernel rejected the live tree for a reason this gate does not model: {other:?}. \
             A duplicate field or an empty identity is a corpus defect, not baseline-able \
             debt.\n{}",
            census(observed)
        ),
    }
}

/// The gate is DEMONSTRATED CAPABLE OF FAILING against the REAL corpus, not just fixtures.
///
/// A green ratchet proves nothing on its own: a caller that silently parsed zero fields would
/// satisfy every assertion above by reporting a perfectly annotated tree. This injects the exact
/// defect the doctrine exists to catch — a new `pub` field with no data-class annotation — into
/// the TEXT of a real kernel source, and requires both halves to react: the parser must see one
/// more unannotated field, and the kernel must name that field and no other.
///
/// The planted field's name and type carry NO occurrence of `data_class`, `DataClass` or
/// `Classified<`, because the annotation rule is a substring scan: a probe that spelled out what
/// it was testing would annotate itself and the mutation would pass for the wrong reason. The
/// second half of the test plants the SAME field WITH an annotation and requires the finding to
/// disappear, so the rule is proven to be a real discriminator rather than a blanket reject.
#[test]
fn injecting_an_unannotated_kernel_field_reddens_the_kernel() {
    let (_, observed) = live();
    let root = repo_root();

    let subject = observed
        .fields
        .first()
        .expect("no kernel field exists at all; this doctrine has no subject")
        .identity
        .path
        .clone();
    let contents = std::fs::read_to_string(root.join(&subject)).expect("read subject");
    let before = parse_kernel_fields(&subject, &contents);

    let plant = "\npub struct OyaLaneCProbeRecord {\n    pub subject_locator: String,\n}\n";
    let planted = format!("{contents}{plant}");
    let after = parse_kernel_fields(&subject, &planted);
    assert_eq!(
        after.len(),
        before.len() + 1,
        "planting a pub field into the live kernel source {subject} did not change the parsed \
         field count — the parser is not reading this file"
    );
    let planted_identity = FieldIdentity {
        path: subject.clone(),
        struct_name: "OyaLaneCProbeRecord".to_owned(),
        field_name: "subject_locator".to_owned(),
    };
    assert!(
        after
            .iter()
            .any(|field| field.identity == planted_identity && !field.has_data_class_annotation),
        "the field count rose but no unannotated field named {planted_identity:?} was produced — \
         the mutation moved the number for some other reason"
    );

    let error = validate_data_class_fitness(&after, &observed.allowances)
        .expect_err("the kernel accepted a brand-new unannotated kernel field");
    let named_plant = matches!(
        &error,
        DataClassFitnessError::UnknownUnannotatedField { field } if *field == planted_identity
    );
    // The subject file may already carry unallowed fields that sort ahead of the plant, so the
    // kernel's FIRST error is not necessarily the plant. Ask it again with everything else
    // granted, which leaves the plant as the only possible finding.
    let isolating: Vec<LegacyUnannotatedField> = after
        .iter()
        .filter(|field| !field.has_data_class_annotation && field.identity != planted_identity)
        .map(|field| LegacyUnannotatedField {
            identity: field.identity.clone(),
            rationale: "granted so the planted field is the only possible finding".to_owned(),
        })
        .collect();
    let isolated = validate_data_class_fitness(&after, &isolating)
        .expect_err("the kernel accepted the planted field when it was the only candidate");
    assert!(
        named_plant
            || matches!(
                &isolated,
                DataClassFitnessError::UnknownUnannotatedField { field } if *field == planted_identity
            ),
        "the kernel rejected the tree but never named the planted field: {error:?} / {isolated:?}"
    );

    // Same plant, annotated: the finding must vanish, or the rule is a blanket reject that would
    // make the frozen map unrepairable.
    let annotated_plant = "\npub struct OyaLaneCProbeRecord {\n    // data_class: INTERNAL_ONLY\n    pub subject_locator: String,\n}\n";
    let annotated_fields = parse_kernel_fields(&subject, &format!("{contents}{annotated_plant}"));
    assert!(
        annotated_fields
            .iter()
            .any(|field| field.identity == planted_identity && field.has_data_class_annotation),
        "annotating the planted field did not clear the finding, so the doctrine cannot be \
         satisfied by doing what it asks"
    );

    // And the stale-allowance arm, on the same live corpus: an allowance naming nothing.
    let ghost = FieldIdentity {
        path: subject.clone(),
        struct_name: "OyaLaneCProbeRecord".to_owned(),
        field_name: "field_that_this_repository_does_not_contain".to_owned(),
    };
    let ghost_only: Vec<KernelField> = Vec::new();
    assert_eq!(
        validate_data_class_fitness(
            &ghost_only,
            &[LegacyUnannotatedField {
                identity: ghost.clone(),
                rationale: "probe".to_owned(),
            }]
        ),
        Err(DataClassFitnessError::StaleLegacyAllowance { field: ghost }),
        "an allowance naming no live field was not reported stale"
    );

    println!(
        "mutation proof: planting `pub subject_locator` into {subject} raised parsed fields {} -> \
         {} and the kernel named it; annotating the same field cleared it\n{}",
        before.len(),
        after.len(),
        census(observed)
    );
}

/// Evidence, always printed, so a reader can tell a paid-down corpus from a collapsed walk without
/// re-running anything.
#[test]
fn live_census_is_reported() {
    let (_, observed) = live();
    println!("{}", census(observed));
    assert!(!observed.fields.is_empty());
}
