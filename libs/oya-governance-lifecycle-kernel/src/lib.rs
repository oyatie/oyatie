//! Foundry lifecycle-automation framework kernel.
//!
//! # Naming justification
//!
//! - Crate `oya-governance-lifecycle-kernel` —
//!   v4 BNF `oya-<product:foundry>-<facet:fitness>-<topic:lifecycle>-<layer:kernel>`;
//!   13-layer-enum suffix `kernel` (innermost ring: I/O-free port + pure check
//!   functions per ADR-0056 "port-in-kernel"). Per-lifecycle dev-CLI wrappers
//!   live under `tools/oya-governance-<name>-lifecycle-app/`.
//!
//! # Intent (per ADR-0109)
//!
//! One generic kernel that consumes a [`LifecycleConfig`] (declared per
//! lifecycle in `specs/lifecycle-configs/<name>.json`) plus
//! a slice of discovered [`LifecycledArtifact`]s, and returns a
//! [`Vec<Violation>`] for any artifact whose state-machine posture is
//! drifted (unknown stage, overdue deadline, missing supersession edge,
//! illegal transition history, milestone-overdue).
//!
//! Per-lifecycle dev-CLIs do the I/O ring: walk repo paths, parse
//! front-matter, build `LifecycledArtifact` records, call [`evaluate`].
//!
//! # Algorithm (kernel — I/O-free)
//!
//! For each artifact:
//! 1. If `current_stage` is `None`, emit `StageNotDeclared`.
//! 2. If `current_stage` is `Some` but absent from `config.stages`, emit
//!    `UnknownStage`.
//! 3. If `deadline_at` is set, `now > deadline_at`, and the stage is not
//!    terminal, emit `OverdueTransition`.
//! 4. If the current stage is terminal AND
//!    `stage.requires_supersession_edge` is true AND
//!    `supersession_target` is `None`, emit `MissingSupersession`.
//! 5. If `milestone_anchor` is set, the milestone is reached, and the
//!    stage has not advanced past the milestone-gated transition, emit
//!    `MilestoneOverdue`.
//! 6. For every `(from, to)` in `history`, if no matching transition
//!    exists in `config.transitions`, emit `IllegalTransition`.

#![forbid(unsafe_code)]

use std::collections::BTreeSet;

/// Minimal Y-M-D date triple (kernel stays zero-dep). Compatible with
/// `chrono::NaiveDate` at the value-object boundary: dev-CLIs may parse
/// `chrono::NaiveDate` and pass `.year()/.month()/.day()`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct NaiveDate {
    // data_class: INTERNAL_ONLY
    pub year: i32, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub month: u8, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub day: u8, // data_class: INTERNAL_ONLY
}

impl NaiveDate {
    pub const fn ymd(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    pub fn checked_ymd(year: i32, month: u8, day: u8) -> Option<Self> {
        if month == 0 || month > 12 {
            return None;
        }
        let max_day = days_in_month(year, month);
        if day == 0 || day > max_day {
            return None;
        }
        Some(Self { year, month, day })
    }
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// One declared stage in a lifecycle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stage {
    // data_class: INTERNAL_ONLY
    pub id: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub terminal: bool, // data_class: INTERNAL_ONLY
    /// When true, an artifact in this stage MUST carry a non-empty
    /// `supersession_target` (e.g. `superseded_by:` for ADRs).
    // data_class: INTERNAL_ONLY
    pub requires_supersession_edge: bool, // data_class: INTERNAL_ONLY
    /// Optional milestone gate: if set, the artifact MUST have advanced
    /// to (or past) this stage by the time the milestone is reached.
    // data_class: INTERNAL_ONLY
    pub gated_by_milestone: Option<String>, // data_class: INTERNAL_ONLY
}

/// Allowed `from → to` transition in the state machine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transition {
    // data_class: INTERNAL_ONLY
    pub from: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub to: String, // data_class: INTERNAL_ONLY
}

/// Source-spec abstraction. Concrete dev-CLIs translate these into
/// directory walks + front-matter parsing; the kernel only sees the
/// resulting `LifecycledArtifact` records.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceSpec {
    // data_class: INTERNAL_ONLY
    pub kind: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub glob: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub stage_field: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub supersession_field: Option<String>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub deadline_field: Option<String>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub milestone_field: Option<String>, // data_class: INTERNAL_ONLY
    /// Optional in-scope predicate. When present, a discovered file is
    /// only treated as a `LifecycledArtifact` if it matches the filter.
    /// Files outside the filter are silently skipped (NOT reported as
    /// `StageNotDeclared`). This narrows a lane's scope from "every file
    /// matching `glob`" to "every file matching `glob` AND declaring
    /// `kind: <name>` in front-matter OR carrying one of
    /// `filename_contains_any` substrings in its path".
    ///
    /// Canonical sub-rule per ADR-0109: lifecycles whose population is a
    /// PROPER SUBSET of their glob (e.g. migration-status applies to
    /// migration/cutover plans only, not every plan under
    /// `.omc/plans/**`) MUST declare that subset via `filter`. This is a
    /// canonical extension, not an exception (per
    /// `feedback_no_exceptions_canonical.md`).
    // data_class: INTERNAL_ONLY
    pub filter: Option<SourceFilter>, // data_class: INTERNAL_ONLY
}

/// In-scope predicate for a `SourceSpec`. The predicate is OR-composed
/// across its fields: a file is in scope iff ANY one of the declared
/// conditions matches. Empty `filename_contains_any` AND missing
/// `kind_field_value` means "no filter" (caller should set
/// `SourceSpec.filter = None` in that case).
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct SourceFilter {
    /// When `Some((field, value))`, the file is in scope if its
    /// front-matter declares `<field>: <value>` (case-insensitive value
    /// match). Typical use: `("kind", "migration")`.
    // data_class: INTERNAL_ONLY
    pub kind_field_value: Option<(String, String)>, // data_class: INTERNAL_ONLY
    /// When non-empty, the file is in scope if its path or filename
    /// contains any of these substrings (case-insensitive). Typical
    /// use: `["migration", "cutover", "rename", "rewrite"]`.
    // data_class: INTERNAL_ONLY
    pub filename_contains_any: Vec<String>, // data_class: INTERNAL_ONLY
}

/// Defaults applied when fields are absent in artifact metadata.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Defaults {
    // data_class: INTERNAL_ONLY
    pub wave: Option<String>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub case_insensitive_stage_match: bool, // data_class: INTERNAL_ONLY
}

/// Full lifecycle configuration. One per `specs/lifecycle-configs/<name>.json`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleConfig {
    // data_class: INTERNAL_ONLY
    pub name: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub version: u32, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub stages: Vec<Stage>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub transitions: Vec<Transition>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub sources: Vec<SourceSpec>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub defaults: Defaults, // data_class: INTERNAL_ONLY
}

/// A discovered artifact in the workspace — produced by dev-CLI I/O ring.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycledArtifact {
    // data_class: INTERNAL_ONLY
    pub location: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub kind: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub current_stage: Option<String>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub observed_at: NaiveDate, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub deadline_at: Option<NaiveDate>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub history: Vec<Transition>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub supersession_target: Option<String>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub milestone_anchor: Option<String>, // data_class: INTERNAL_ONLY
}

/// Categorical violation kind. Stable string enum so dev-CLIs can group
/// findings without recompiling against a moving rust enum.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ViolationKind {
    StageNotDeclared,
    UnknownStage,
    OverdueTransition,
    MissingSupersession,
    MilestoneOverdue,
    IllegalTransition,
}

impl ViolationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StageNotDeclared => "stage_not_declared",
            Self::UnknownStage => "unknown_stage",
            Self::OverdueTransition => "overdue_transition",
            Self::MissingSupersession => "missing_supersession",
            Self::MilestoneOverdue => "milestone_overdue",
            Self::IllegalTransition => "illegal_transition",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Violation {
    // data_class: INTERNAL_ONLY
    pub kind: ViolationKind, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub location: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub artifact_kind: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub stage: Option<String>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub hint: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleReport {
    // data_class: INTERNAL_ONLY
    pub artifacts_observed: usize, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub stage_counts: Vec<(String, usize)>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub violations: Vec<Violation>, // data_class: INTERNAL_ONLY
}

impl LifecycleReport {
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

fn normalize(stage: &str, case_insensitive: bool) -> String {
    if case_insensitive {
        stage.to_ascii_lowercase()
    } else {
        stage.to_string()
    }
}

fn stage_lookup<'a>(config: &'a LifecycleConfig, id: &str) -> Option<&'a Stage> {
    let want = normalize(id, config.defaults.case_insensitive_stage_match);
    config
        .stages
        .iter()
        .find(|s| normalize(&s.id, config.defaults.case_insensitive_stage_match) == want)
}

fn transition_allowed(config: &LifecycleConfig, from: &str, to: &str) -> bool {
    let ci = config.defaults.case_insensitive_stage_match;
    let want_from = normalize(from, ci);
    let want_to = normalize(to, ci);
    config
        .transitions
        .iter()
        .any(|t| normalize(&t.from, ci) == want_from && normalize(&t.to, ci) == want_to)
}

/// The pure check entry point.
pub fn evaluate(
    config: &LifecycleConfig,
    artifacts: &[LifecycledArtifact],
    now: NaiveDate,
    reached_milestones: &[String],
) -> LifecycleReport {
    let mut violations: Vec<Violation> = Vec::new();
    let mut stage_seen: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    let reached: BTreeSet<&str> = reached_milestones.iter().map(|s| s.as_str()).collect();

    for artifact in artifacts {
        // 1. StageNotDeclared
        let Some(current) = artifact.current_stage.as_ref() else {
            violations.push(Violation {
                kind: ViolationKind::StageNotDeclared,
                location: artifact.location.clone(),
                artifact_kind: artifact.kind.clone(),
                stage: None,
                hint: format!(
                    "artifact has no `{}` lifecycle stage field declared; expected one of {:?}",
                    config.name,
                    config.stages.iter().map(|s| &s.id).collect::<Vec<_>>()
                ),
            });
            continue;
        };

        // Tally stage occurrence (use normalized form for canonical bucket).
        let bucket = normalize(current, config.defaults.case_insensitive_stage_match);
        *stage_seen.entry(bucket).or_insert(0) += 1;

        // 2. UnknownStage
        let Some(stage) = stage_lookup(config, current) else {
            violations.push(Violation {
                kind: ViolationKind::UnknownStage,
                location: artifact.location.clone(),
                artifact_kind: artifact.kind.clone(),
                stage: Some(current.clone()),
                hint: format!(
                    "stage `{}` is not in the `{}` lifecycle config; declared stages: {:?}",
                    current,
                    config.name,
                    config.stages.iter().map(|s| &s.id).collect::<Vec<_>>()
                ),
            });
            continue;
        };

        // 3. OverdueTransition
        if let Some(deadline) = artifact.deadline_at
            && !stage.terminal
            && now > deadline
        {
            violations.push(Violation {
                kind: ViolationKind::OverdueTransition,
                location: artifact.location.clone(),
                artifact_kind: artifact.kind.clone(),
                stage: Some(current.clone()),
                hint: format!(
                    "deadline {:?} passed (now {:?}) but stage `{}` is non-terminal",
                    deadline, now, current
                ),
            });
        }

        // 4. MissingSupersession
        if stage.terminal
            && stage.requires_supersession_edge
            && artifact
                .supersession_target
                .as_ref()
                .map(|s| s.trim().is_empty())
                .unwrap_or(true)
        {
            violations.push(Violation {
                kind: ViolationKind::MissingSupersession,
                location: artifact.location.clone(),
                artifact_kind: artifact.kind.clone(),
                stage: Some(current.clone()),
                hint: format!(
                    "stage `{}` is terminal and requires a supersession edge but `supersession_target` is empty/missing",
                    current
                ),
            });
        }

        // 5. MilestoneOverdue
        if let Some(gate) = &stage.gated_by_milestone
            && reached.contains(gate.as_str())
        {
            // If the stage is gated_by_milestone and the milestone is reached,
            // any non-terminal artifact still on this stage is overdue.
            if !stage.terminal {
                violations.push(Violation {
                    kind: ViolationKind::MilestoneOverdue,
                    location: artifact.location.clone(),
                    artifact_kind: artifact.kind.clone(),
                    stage: Some(current.clone()),
                    hint: format!(
                        "milestone `{}` reached; stage `{}` should have advanced past its gate",
                        gate, current
                    ),
                });
            }
        }

        // 6. IllegalTransition (history audit)
        for t in &artifact.history {
            if !transition_allowed(config, &t.from, &t.to) {
                violations.push(Violation {
                    kind: ViolationKind::IllegalTransition,
                    location: artifact.location.clone(),
                    artifact_kind: artifact.kind.clone(),
                    stage: Some(format!("{} → {}", t.from, t.to)),
                    hint: format!(
                        "transition `{} → {}` is not declared in `{}` config",
                        t.from, t.to, config.name
                    ),
                });
            }
        }
    }

    let stage_counts: Vec<(String, usize)> = stage_seen.into_iter().collect();

    LifecycleReport {
        artifacts_observed: artifacts.len(),
        stage_counts,
        violations,
    }
}

// ===========================================================================
// Optional I/O ring (dev-CLI helpers) — opt-in via `discovery` module.
//
// The framework kernel is I/O-free at its core (`evaluate`, `Stage`,
// `Transition`, `LifecycleConfig`, `LifecycledArtifact`). The thin helpers
// below sit BESIDE the kernel rather than inside it: they are used by the
// per-lifecycle dev-CLIs so each `tools/*-lifecycle-app/` stays under 50
// lines. They depend on `std::fs` only — no third-party crates — and are
// gated behind the `discovery` module so kernel consumers that want pure
// in-memory checks (e.g. WASM, test harnesses) can ignore them entirely.
// ===========================================================================

pub mod discovery {
    //! Minimal JSON config loader + YAML-front-matter scalar reader +
    //! `<dir>/<glob>.<ext>` walker. Zero third-party deps.
    use super::{
        Defaults, LifecycleConfig, LifecycledArtifact, NaiveDate, SourceFilter, SourceSpec, Stage,
        Transition,
    };
    use std::fs;
    use std::path::{Path, PathBuf};

    /// Apply a `SourceFilter` to a discovered file. The filter is
    /// OR-composed: any matching condition admits the file. With NO
    /// declared conditions (default-constructed filter), nothing
    /// matches — callers should set `SourceSpec.filter = None` in that
    /// case rather than passing an empty filter.
    pub fn path_passes_filter(path: &Path, raw: &str, filter: &SourceFilter) -> bool {
        if let Some((field, want)) = &filter.kind_field_value
            && let Some(got) = frontmatter_scalar(raw, field)
            && got.eq_ignore_ascii_case(want)
        {
            return true;
        }
        if !filter.filename_contains_any.is_empty() {
            let hay = path.to_string_lossy().to_ascii_lowercase();
            for needle in &filter.filename_contains_any {
                let needle = needle.to_ascii_lowercase();
                if hay.contains(&needle) {
                    return true;
                }
            }
        }
        false
    }

    pub fn load_config(path: &Path) -> Result<LifecycleConfig, String> {
        let raw = fs::read_to_string(path)
            .map_err(|e| format!("could not read config {}: {e}", path.display()))?;
        parse_config_json(&raw)
    }

    pub fn discover(
        config: &LifecycleConfig,
        observed_at: NaiveDate,
    ) -> Result<Vec<LifecycledArtifact>, String> {
        let mut out = Vec::new();
        for source in &config.sources {
            let entries = expand_glob(&source.glob)?;
            for path in entries {
                let raw = fs::read_to_string(&path)
                    .map_err(|e| format!("could not read {}: {e}", path.display()))?;
                if let Some(filter) = &source.filter
                    && !path_passes_filter(&path, &raw, filter)
                {
                    continue;
                }
                let stage = frontmatter_scalar(&raw, &source.stage_field);
                let supersession = source
                    .supersession_field
                    .as_deref()
                    .and_then(|f| frontmatter_scalar(&raw, f));
                let deadline = source
                    .deadline_field
                    .as_deref()
                    .and_then(|f| frontmatter_scalar(&raw, f))
                    .and_then(|s| parse_date(&s));
                let milestone = source
                    .milestone_field
                    .as_deref()
                    .and_then(|f| frontmatter_scalar(&raw, f));
                out.push(LifecycledArtifact {
                    location: path.to_string_lossy().into_owned(),
                    kind: config.name.clone(),
                    current_stage: stage,
                    observed_at,
                    deadline_at: deadline,
                    history: vec![],
                    supersession_target: supersession,
                    milestone_anchor: milestone,
                });
            }
        }
        Ok(out)
    }

    pub fn parse_date(s: &str) -> Option<NaiveDate> {
        let parts: Vec<&str> = s.splitn(3, '-').collect();
        if parts.len() != 3 {
            return None;
        }
        let y: i32 = parts[0].parse().ok()?;
        let m: u8 = parts[1].parse().ok()?;
        let d: u8 = parts[2].parse().ok()?;
        NaiveDate::checked_ymd(y, m, d)
    }

    /// Read one top-level scalar out of a document's declaration surface.
    ///
    /// A document that carries a `---` line delimits its declarations with a
    /// front-matter fence and only the fenced region is read. A document with
    /// NO `---` line anywhere is a bare declaration record (a plain YAML file
    /// such as `registry/catalog/*.yaml`) and is read whole — without this the
    /// reader returns `None` for every field of every fence-less file, and a
    /// lane rooted on such a corpus certifies a reader bug as corpus debt.
    ///
    /// The fence-less admission is keyed on the ABSENCE of any `---` line, so
    /// no input that resolves today can change: a file that yields `Some` must
    /// contain a fence, and every fenced file takes the identical code path it
    /// took before. The only behaviour delta is `None` -> `Some` on fence-less
    /// files.
    pub fn frontmatter_scalar(raw: &str, field: &str) -> Option<String> {
        let mut in_fm = !raw.lines().any(|line| line.trim() == "---");
        let mut started = false;
        for line in raw.lines() {
            if line.trim() == "---" {
                if !started {
                    started = true;
                    in_fm = true;
                    continue;
                } else {
                    break;
                }
            }
            if !in_fm {
                continue;
            }
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix(field)
                && let Some(rest) = rest.strip_prefix(':')
            {
                let value = rest.trim().trim_matches('"').trim_matches('\'').trim();
                if value.is_empty() {
                    return None;
                }
                return Some(value.to_string());
            }
        }
        None
    }

    pub fn expand_glob(glob: &str) -> Result<Vec<PathBuf>, String> {
        let glob = glob.trim();
        if let Some((head, _tail)) = glob.split_once("/**/") {
            return recursive(Path::new(head), &glob[head.len() + 4..]);
        }
        if let Some((dir, rest)) = glob.rsplit_once('/') {
            return shallow_glob(Path::new(dir), rest);
        }
        Err(format!("unsupported glob pattern: {glob}"))
    }

    fn shallow_glob(dir: &Path, pattern: &str) -> Result<Vec<PathBuf>, String> {
        let mut out = Vec::new();
        if !dir.exists() {
            return Err(format!("missing source root: {}", dir.display()));
        }
        if !dir.is_dir() {
            return Err(format!("source root is not a directory: {}", dir.display()));
        }
        let entries = fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
            if !entry
                .file_type()
                .map_err(|e| format!("file_type {}: {e}", entry.path().display()))?
                .is_file()
            {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if matches_glob(&name, pattern) {
                out.push(entry.path());
            }
        }
        out.sort();
        Ok(out)
    }

    fn recursive(root: &Path, pattern: &str) -> Result<Vec<PathBuf>, String> {
        let mut out = Vec::new();
        if !root.exists() {
            return Err(format!("missing source root: {}", root.display()));
        }
        if !root.is_dir() {
            return Err(format!(
                "source root is not a directory: {}",
                root.display()
            ));
        }
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let entries =
                fs::read_dir(&dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
                let path = entry.path();
                let ft = entry
                    .file_type()
                    .map_err(|e| format!("file_type {}: {e}", path.display()))?;
                if ft.is_dir() {
                    stack.push(path);
                } else if ft.is_file() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if matches_glob(&name, pattern) {
                        out.push(path);
                    }
                }
            }
        }
        out.sort();
        Ok(out)
    }

    pub fn matches_glob(name: &str, pattern: &str) -> bool {
        if let Some((prefix, suffix)) = pattern.split_once('*') {
            return name.starts_with(prefix)
                && name.ends_with(suffix)
                && name.len() >= prefix.len() + suffix.len();
        }
        name == pattern
    }

    pub fn parse_config_json(raw: &str) -> Result<LifecycleConfig, String> {
        let v = json_mini::parse(raw)?;
        let name = v.field_str("name")?.to_string();
        let version = v.field_u32("version").unwrap_or(1);
        let stages = v
            .field_arr("stages")?
            .iter()
            .map(|s| {
                Ok::<_, String>(Stage {
                    id: s.field_str("id")?.to_string(),
                    terminal: s.field_bool("terminal").unwrap_or(false),
                    requires_supersession_edge: s
                        .field_bool("requires_supersession_edge")
                        .unwrap_or(false),
                    gated_by_milestone: s.field_str("gated_by_milestone").ok().map(String::from),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let transitions = v
            .field_arr("transitions")?
            .iter()
            .map(|t| {
                Ok::<_, String>(Transition {
                    from: t.field_str("from")?.to_string(),
                    to: t.field_str("to")?.to_string(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let sources = v
            .field_arr("sources")?
            .iter()
            .map(|s| {
                let filter = s.field_obj("filter").and_then(|f| {
                    let kind_field_value = match (
                        f.field_str("kind_field").ok().map(String::from),
                        f.field_str("kind_value").ok().map(String::from),
                    ) {
                        (Some(field), Some(value)) => Some((field, value)),
                        _ => None,
                    };
                    let filename_contains_any = f
                        .field_arr("filename_contains_any")
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|item| item.as_str_value())
                                .map(String::from)
                                .collect::<Vec<String>>()
                        })
                        .unwrap_or_default();
                    if kind_field_value.is_none() && filename_contains_any.is_empty() {
                        None
                    } else {
                        Some(SourceFilter {
                            kind_field_value,
                            filename_contains_any,
                        })
                    }
                });
                Ok::<_, String>(SourceSpec {
                    kind: s.field_str("kind")?.to_string(),
                    glob: s.field_str("glob")?.to_string(),
                    stage_field: s.field_str("stage_field")?.to_string(),
                    supersession_field: s.field_str("supersession_field").ok().map(String::from),
                    deadline_field: s.field_str("deadline_field").ok().map(String::from),
                    milestone_field: s.field_str("milestone_field").ok().map(String::from),
                    filter,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let defaults = v
            .field_obj("defaults")
            .map(|d| Defaults {
                wave: d.field_str("wave").ok().map(String::from),
                case_insensitive_stage_match: d
                    .field_bool("case_insensitive_stage_match")
                    .unwrap_or(false),
            })
            .unwrap_or_default();
        Ok(LifecycleConfig {
            name,
            version,
            stages,
            transitions,
            sources,
            defaults,
        })
    }

    mod json_mini {
        use std::collections::HashMap;
        pub fn parse(input: &str) -> Result<Value, String> {
            let mut p = Parser {
                src: input.as_bytes(),
                pos: 0,
            };
            p.skip_ws();
            let v = p.parse_value()?;
            p.skip_ws();
            if p.pos != p.src.len() {
                return Err(format!("trailing input at byte {}", p.pos));
            }
            Ok(v)
        }
        pub enum Value {
            Null,
            Bool(bool),
            Number(f64),
            Str(String),
            Array(Vec<Value>),
            Object(HashMap<String, Value>),
        }
        impl Value {
            pub fn field_str(&self, name: &str) -> Result<&str, String> {
                match self {
                    Value::Object(m) => match m.get(name) {
                        Some(Value::Str(s)) => Ok(s.as_str()),
                        Some(Value::Null) => Err(format!("field `{name}` is null")),
                        Some(_) => Err(format!("field `{name}` is not a string")),
                        None => Err(format!("field `{name}` missing")),
                    },
                    _ => Err("expected object".into()),
                }
            }
            pub fn field_bool(&self, name: &str) -> Result<bool, String> {
                match self {
                    Value::Object(m) => match m.get(name) {
                        Some(Value::Bool(b)) => Ok(*b),
                        Some(_) => Err(format!("field `{name}` is not a bool")),
                        None => Err(format!("field `{name}` missing")),
                    },
                    _ => Err("expected object".into()),
                }
            }
            pub fn field_u32(&self, name: &str) -> Result<u32, String> {
                match self {
                    Value::Object(m) => match m.get(name) {
                        Some(Value::Number(n)) => Ok(*n as u32),
                        Some(_) => Err(format!("field `{name}` is not a number")),
                        None => Err(format!("field `{name}` missing")),
                    },
                    _ => Err("expected object".into()),
                }
            }
            pub fn field_arr(&self, name: &str) -> Result<&Vec<Value>, String> {
                match self {
                    Value::Object(m) => match m.get(name) {
                        Some(Value::Array(a)) => Ok(a),
                        Some(_) => Err(format!("field `{name}` is not an array")),
                        None => Err(format!("field `{name}` missing")),
                    },
                    _ => Err("expected object".into()),
                }
            }
            pub fn as_str_value(&self) -> Option<&str> {
                match self {
                    Value::Str(s) => Some(s.as_str()),
                    _ => None,
                }
            }
            pub fn field_obj(&self, name: &str) -> Option<&Value> {
                match self {
                    Value::Object(m) => match m.get(name) {
                        Some(v @ Value::Object(_)) => Some(v),
                        _ => None,
                    },
                    _ => None,
                }
            }
        }
        struct Parser<'a> {
            src: &'a [u8],
            pos: usize,
        }
        impl<'a> Parser<'a> {
            fn skip_ws(&mut self) {
                while self.pos < self.src.len() && (self.src[self.pos] as char).is_whitespace() {
                    self.pos += 1;
                }
            }
            fn peek(&self) -> Option<u8> {
                self.src.get(self.pos).copied()
            }
            fn parse_value(&mut self) -> Result<Value, String> {
                self.skip_ws();
                match self.peek() {
                    Some(b'{') => self.parse_object(),
                    Some(b'[') => self.parse_array(),
                    Some(b'"') => Ok(Value::Str(self.parse_string()?)),
                    Some(b't') | Some(b'f') => self.parse_bool(),
                    Some(b'n') => self.parse_null(),
                    Some(c) if c == b'-' || c.is_ascii_digit() => self.parse_number(),
                    Some(c) => Err(format!("unexpected byte {c} at pos {}", self.pos)),
                    None => Err("unexpected EOF".into()),
                }
            }
            fn parse_object(&mut self) -> Result<Value, String> {
                self.pos += 1;
                self.skip_ws();
                let mut map: HashMap<String, Value> = HashMap::new();
                if self.peek() == Some(b'}') {
                    self.pos += 1;
                    return Ok(Value::Object(map));
                }
                loop {
                    self.skip_ws();
                    let key = self.parse_string()?;
                    self.skip_ws();
                    if self.peek() != Some(b':') {
                        return Err("expected ':' in object".into());
                    }
                    self.pos += 1;
                    let val = self.parse_value()?;
                    map.insert(key, val);
                    self.skip_ws();
                    match self.peek() {
                        Some(b',') => self.pos += 1,
                        Some(b'}') => {
                            self.pos += 1;
                            break;
                        }
                        other => return Err(format!("expected ',' or '}}' got {other:?}")),
                    }
                }
                Ok(Value::Object(map))
            }
            fn parse_array(&mut self) -> Result<Value, String> {
                self.pos += 1;
                self.skip_ws();
                let mut items = Vec::new();
                if self.peek() == Some(b']') {
                    self.pos += 1;
                    return Ok(Value::Array(items));
                }
                loop {
                    items.push(self.parse_value()?);
                    self.skip_ws();
                    match self.peek() {
                        Some(b',') => self.pos += 1,
                        Some(b']') => {
                            self.pos += 1;
                            break;
                        }
                        other => return Err(format!("expected ',' or ']' got {other:?}")),
                    }
                }
                Ok(Value::Array(items))
            }
            fn parse_string(&mut self) -> Result<String, String> {
                if self.peek() != Some(b'"') {
                    return Err("expected '\"'".into());
                }
                self.pos += 1;
                let start = self.pos;
                while let Some(c) = self.peek() {
                    if c == b'\\' {
                        self.pos += 2;
                        continue;
                    }
                    if c == b'"' {
                        let s = std::str::from_utf8(&self.src[start..self.pos])
                            .map_err(|e| format!("utf8: {e}"))?
                            .to_string();
                        self.pos += 1;
                        let out = s
                            .replace("\\\\", "\u{0001}")
                            .replace("\\\"", "\"")
                            .replace("\\n", "\n")
                            .replace("\\t", "\t")
                            .replace('\u{0001}', "\\");
                        return Ok(out);
                    }
                    self.pos += 1;
                }
                Err("unterminated string".into())
            }
            fn parse_bool(&mut self) -> Result<Value, String> {
                if self.src[self.pos..].starts_with(b"true") {
                    self.pos += 4;
                    Ok(Value::Bool(true))
                } else if self.src[self.pos..].starts_with(b"false") {
                    self.pos += 5;
                    Ok(Value::Bool(false))
                } else {
                    Err("expected bool".into())
                }
            }
            fn parse_null(&mut self) -> Result<Value, String> {
                if self.src[self.pos..].starts_with(b"null") {
                    self.pos += 4;
                    Ok(Value::Null)
                } else {
                    Err("expected null".into())
                }
            }
            fn parse_number(&mut self) -> Result<Value, String> {
                let start = self.pos;
                if self.peek() == Some(b'-') {
                    self.pos += 1;
                }
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit()
                        || c == b'.'
                        || c == b'e'
                        || c == b'E'
                        || c == b'-'
                        || c == b'+'
                    {
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
                let text = std::str::from_utf8(&self.src[start..self.pos])
                    .map_err(|e| format!("utf8: {e}"))?;
                text.parse::<f64>()
                    .map(Value::Number)
                    .map_err(|e| format!("number: {e}"))
            }
        }
    }
}

pub mod cli {
    //! Reusable dev-CLI runner. Each per-lifecycle binary calls
    //! [`run_default`] with its lane name + default config path.
    //!
    //! # Wave ratchet (per ADR-0109 §"Wave ratchet")
    //!
    //! Per-lane wave is declared in `config.defaults.wave`:
    //!
    //! - `"A"` — WARN baseline. All findings reported on stdout; exit 0.
    //! - `"B"` — delta-BLOCK. Findings on artifacts modified in the trusted
    //!   changed range (`--changed-range BASE..HEAD`) or merge-base pair
    //!   (`--changed-base BASE --changed-head HEAD`) BLOCK; older baseline
    //!   findings still WARN. Missing/unresolvable range data fails closed.
    //! - `"C"` — full-BLOCK. ALL findings BLOCK (exit non-zero on any).
    //!
    //! CLI flags `--block` / `--warn-only` override the config wave for
    //! ad-hoc invocations (CI uses config-driven wave).
    use super::discovery;
    use super::{LifecycleReport, NaiveDate, Violation, ViolationKind, evaluate};
    use std::collections::BTreeSet;
    use std::env;
    use std::path::PathBuf;
    use std::process::{Command, ExitCode, Stdio};

    /// Resolved wave for this run (after CLI override + config default).
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum Wave {
        /// WARN-only: report on stdout, exit 0 unconditionally.
        A,
        /// Delta-BLOCK: BLOCK on findings whose `location` matches a path
        /// in the trusted changed range. Missing or unresolved range inputs
        /// fail closed to full-block behavior.
        B,
        /// Full-BLOCK: BLOCK on every finding.
        C,
    }

    impl Wave {
        /// Parse a single-letter wave label (`A` / `B` / `C`, case-insensitive).
        /// Named `parse_label` rather than `from_str` to avoid colliding with
        /// the `std::str::FromStr::from_str` trait method (clippy
        /// `should_implement_trait`).
        pub fn parse_label(s: &str) -> Option<Self> {
            match s.trim() {
                "A" | "a" => Some(Self::A),
                "B" | "b" => Some(Self::B),
                "C" | "c" => Some(Self::C),
                _ => None,
            }
        }
        pub fn label(self) -> &'static str {
            match self {
                Self::A => "A",
                Self::B => "B",
                Self::C => "C",
            }
        }
    }

    pub fn run_default(lane_name: &'static str, default_config: &'static str) -> ExitCode {
        let args: Vec<String> = env::args().skip(1).collect();
        let opts = match Options::parse(args, default_config) {
            Ok(o) => o,
            Err(msg) => {
                eprintln!("{lane_name} error: {msg}");
                return ExitCode::FAILURE;
            }
        };
        let config = match discovery::load_config(&opts.config) {
            Ok(c) => c,
            Err(msg) => {
                eprintln!("{lane_name} config error: {msg}");
                return ExitCode::FAILURE;
            }
        };
        let wave = resolve_wave(&opts, &config);
        let now = match trusted_today(&opts) {
            Ok(d) => d,
            Err(msg) => {
                eprintln!("{lane_name} time error: {msg}");
                return ExitCode::FAILURE;
            }
        };
        let artifacts = match discovery::discover(&config, now) {
            Ok(a) => a,
            Err(msg) => {
                eprintln!("{lane_name} discovery error: {msg}");
                return ExitCode::FAILURE;
            }
        };
        let report = evaluate(&config, &artifacts, now, &opts.reached_milestones);
        emit(lane_name, &report, wave, &opts)
    }

    /// Resolve the effective wave for this invocation.
    ///
    /// Precedence:
    /// 1. `--wave A|B|C` CLI flag (explicit override).
    /// 2. `--block` → C (legacy ad-hoc block flag).
    /// 3. `--warn-only` → A (legacy ad-hoc warn flag).
    /// 4. `config.defaults.wave` (canonical declaration).
    /// 5. Default: `A` (safe baseline).
    pub fn resolve_wave(opts: &Options, config: &super::LifecycleConfig) -> Wave {
        if let Some(w) = opts.wave_override {
            return w;
        }
        if let Some(legacy) = opts.legacy_wave_override {
            return legacy;
        }
        if let Some(declared) = config.defaults.wave.as_deref()
            && let Some(w) = Wave::parse_label(declared)
        {
            return w;
        }
        Wave::A
    }

    /// Set of file paths changed in the trusted Wave-B range. Returns `None`
    /// when the caller did not provide a range, merge-base resolution fails, or
    /// `git diff --name-only` fails. Wave-B callers treat `None` as fail-closed
    /// full-block behavior rather than silently falling back to HEAD-relative
    /// assumptions.
    fn changed_paths_for_wave_b(opts: &Options) -> Option<BTreeSet<String>> {
        if let Some((base, head)) = &opts.changed_range {
            return diff_name_only(base, head);
        }

        let (Some(base), Some(head)) = (&opts.changed_base, &opts.changed_head) else {
            return None;
        };
        let merge_base = merge_base(base, head)?;
        diff_name_only(&merge_base, head)
    }

    fn merge_base(base: &str, head: &str) -> Option<String> {
        let output = Command::new("git")
            .args(["merge-base", base, head])
            .stderr(Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let text = std::str::from_utf8(&output.stdout).ok()?.trim();
        (!text.is_empty()).then(|| text.to_string())
    }

    fn diff_name_only(base: &str, head: &str) -> Option<BTreeSet<String>> {
        let output = Command::new("git")
            .args(["diff", "--name-only", base, head])
            .stderr(Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let text = std::str::from_utf8(&output.stdout).ok()?;
        let mut out: BTreeSet<String> = BTreeSet::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                out.insert(trimmed.to_string());
            }
        }
        Some(out)
    }

    /// Is this violation's location in the changed-set? Wave-B delta predicate.
    fn finding_is_new(violation: &Violation, changed: &BTreeSet<String>) -> bool {
        // Direct match, plus substring match (locations can carry `#fragment`
        // suffixes such as `path#prose` from sunset-style runners — we treat
        // a finding as "new" if its location starts with a changed path).
        if changed.contains(&violation.location) {
            return true;
        }
        changed.iter().any(|c| violation.location.starts_with(c))
    }

    fn emit(lane_name: &str, report: &LifecycleReport, wave: Wave, opts: &Options) -> ExitCode {
        if report.is_clean() {
            println!(
                "{lane_name} ok (wave {}): artifacts_observed={} stage_counts={:?} violations=0",
                wave.label(),
                report.artifacts_observed,
                report.stage_counts
            );
            return ExitCode::SUCCESS;
        }
        let _ = ViolationKind::StageNotDeclared; // type pin
        let mut breakdown: std::collections::BTreeMap<&'static str, usize> =
            std::collections::BTreeMap::new();
        for v in &report.violations {
            *breakdown.entry(v.kind.as_str()).or_insert(0) += 1;
        }
        match wave {
            Wave::A => {
                println!(
                    "{lane_name} WARN (wave A): artifacts_observed={} stage_counts={:?} violations={} breakdown={:?}",
                    report.artifacts_observed,
                    report.stage_counts,
                    report.violations.len(),
                    breakdown
                );
                for v in &report.violations {
                    println!(
                        "  - [{}] {} stage={:?} hint={}",
                        v.kind.as_str(),
                        v.location,
                        v.stage,
                        v.hint
                    );
                }
                ExitCode::SUCCESS
            }
            Wave::B => {
                let changed = match changed_paths_for_wave_b(opts) {
                    Some(c) => c,
                    None => {
                        // Trusted changed-range input is missing or cannot be
                        // resolved. Fail closed to Wave-C full-block behavior
                        // rather than silently using a caller-forgeable or
                        // HEAD-relative fallback.
                        eprintln!(
                            "{lane_name} FAIL (wave B; changed-range unavailable → fail-closed to wave C full-block): artifacts_observed={} stage_counts={:?} violations={} breakdown={:?}",
                            report.artifacts_observed,
                            report.stage_counts,
                            report.violations.len(),
                            breakdown
                        );
                        for v in &report.violations {
                            eprintln!(
                                "  - [{}] {} stage={:?} hint={}",
                                v.kind.as_str(),
                                v.location,
                                v.stage,
                                v.hint
                            );
                        }
                        return ExitCode::FAILURE;
                    }
                };
                let mut new_findings: Vec<&Violation> = Vec::new();
                let mut baseline: Vec<&Violation> = Vec::new();
                for v in &report.violations {
                    if finding_is_new(v, &changed) {
                        new_findings.push(v);
                    } else {
                        baseline.push(v);
                    }
                }
                if new_findings.is_empty() {
                    println!(
                        "{lane_name} WARN (wave B; baseline-only): artifacts_observed={} stage_counts={:?} violations={} baseline={} new=0 breakdown={:?}",
                        report.artifacts_observed,
                        report.stage_counts,
                        report.violations.len(),
                        baseline.len(),
                        breakdown
                    );
                    for v in &report.violations {
                        println!(
                            "  - [{}] {} stage={:?} hint={}",
                            v.kind.as_str(),
                            v.location,
                            v.stage,
                            v.hint
                        );
                    }
                    ExitCode::SUCCESS
                } else {
                    eprintln!(
                        "{lane_name} FAIL (wave B; delta-block): artifacts_observed={} stage_counts={:?} violations={} baseline={} new={} breakdown={:?}",
                        report.artifacts_observed,
                        report.stage_counts,
                        report.violations.len(),
                        baseline.len(),
                        new_findings.len(),
                        breakdown
                    );
                    eprintln!("  -- NEW (block-on-new) --");
                    for v in &new_findings {
                        eprintln!(
                            "  - [{}] {} stage={:?} hint={}",
                            v.kind.as_str(),
                            v.location,
                            v.stage,
                            v.hint
                        );
                    }
                    eprintln!("  -- BASELINE (warn) --");
                    for v in &baseline {
                        eprintln!(
                            "  - [{}] {} stage={:?} hint={}",
                            v.kind.as_str(),
                            v.location,
                            v.stage,
                            v.hint
                        );
                    }
                    ExitCode::FAILURE
                }
            }
            Wave::C => {
                eprintln!(
                    "{lane_name} FAIL (wave C; full-block): artifacts_observed={} stage_counts={:?} violations={} breakdown={:?}",
                    report.artifacts_observed,
                    report.stage_counts,
                    report.violations.len(),
                    breakdown
                );
                for v in &report.violations {
                    eprintln!(
                        "  - [{}] {} stage={:?} hint={}",
                        v.kind.as_str(),
                        v.location,
                        v.stage,
                        v.hint
                    );
                }
                ExitCode::FAILURE
            }
        }
    }

    #[derive(Debug)]
    pub struct Options {
        // data_class: INTERNAL_ONLY
        pub config: PathBuf, // data_class: INTERNAL_ONLY
        // data_class: INTERNAL_ONLY
        pub reached_milestones: Vec<String>, // data_class: INTERNAL_ONLY
        /// Explicit `--wave A|B|C` flag (highest precedence).
        // data_class: INTERNAL_ONLY
        pub wave_override: Option<Wave>, // data_class: INTERNAL_ONLY
        /// Legacy `--block` / `--warn-only` flags (mapped to C / A).
        // data_class: INTERNAL_ONLY
        pub legacy_wave_override: Option<Wave>, // data_class: INTERNAL_ONLY
        /// Explicit trusted evaluation date. Required so stale build defaults
        /// and caller-forgeable environment variables cannot make overdue
        /// artifacts false-green.
        // data_class: INTERNAL_ONLY
        pub trusted_now: Option<NaiveDate>, // data_class: INTERNAL_ONLY
        /// Explicit changed range for Wave-B delta blocking.
        // data_class: INTERNAL_ONLY
        pub changed_range: Option<(String, String)>, // data_class: INTERNAL_ONLY
        /// Base ref used with `--changed-head` to compute a merge-base range.
        // data_class: INTERNAL_ONLY
        pub changed_base: Option<String>, // data_class: INTERNAL_ONLY
        /// Head ref used with `--changed-base` to compute a merge-base range.
        // data_class: INTERNAL_ONLY
        pub changed_head: Option<String>, // data_class: INTERNAL_ONLY
    }

    impl Options {
        pub fn parse(args: Vec<String>, default_config: &str) -> Result<Self, String> {
            let mut config = PathBuf::from(default_config);
            let mut reached_milestones: Vec<String> = Vec::new();
            let mut wave_override: Option<Wave> = None;
            let mut legacy_wave_override: Option<Wave> = None;
            let mut trusted_now: Option<NaiveDate> = None;
            let mut changed_range: Option<(String, String)> = None;
            let mut changed_base: Option<String> = None;
            let mut changed_head: Option<String> = None;
            let mut i = 0usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--config" => {
                        i += 1;
                        config = PathBuf::from(args.get(i).ok_or("--config needs a path")?.clone());
                    }
                    "--milestone" => {
                        i += 1;
                        reached_milestones
                            .push(args.get(i).ok_or("--milestone needs an id")?.to_string());
                    }
                    "--wave" => {
                        i += 1;
                        let v = args.get(i).ok_or("--wave needs a value (A|B|C)")?;
                        wave_override = Some(
                            Wave::parse_label(v)
                                .ok_or_else(|| format!("--wave expects A|B|C, got `{v}`"))?,
                        );
                    }
                    "--trusted-now" => {
                        i += 1;
                        let v = args.get(i).ok_or("--trusted-now needs YYYY-MM-DD")?;
                        trusted_now = Some(discovery::parse_date(v).ok_or_else(|| {
                            format!("--trusted-now expects YYYY-MM-DD, got `{v}`")
                        })?);
                    }
                    "--changed-range" => {
                        i += 1;
                        let v = args.get(i).ok_or("--changed-range needs BASE..HEAD")?;
                        changed_range = Some(parse_changed_range(v)?);
                    }
                    "--changed-base" => {
                        i += 1;
                        changed_base =
                            Some(args.get(i).ok_or("--changed-base needs a ref")?.to_string());
                    }
                    "--changed-head" => {
                        i += 1;
                        changed_head =
                            Some(args.get(i).ok_or("--changed-head needs a ref")?.to_string());
                    }
                    "--block" => legacy_wave_override = Some(Wave::C),
                    "--warn-only" => legacy_wave_override = Some(Wave::A),
                    "--help" | "-h" => return Err(usage()),
                    other => return Err(format!("unexpected argument '{other}'\n{}", usage())),
                }
                i += 1;
            }
            Ok(Self {
                config,
                reached_milestones,
                wave_override,
                legacy_wave_override,
                trusted_now,
                changed_range,
                changed_base,
                changed_head,
            })
        }
    }

    fn usage() -> String {
        "options: [--config PATH] [--milestone ID]... [--trusted-now YYYY-MM-DD] [--changed-range BASE..HEAD|--changed-base BASE --changed-head HEAD] [--wave A|B|C] [--block|--warn-only]".into()
    }

    fn parse_changed_range(value: &str) -> Result<(String, String), String> {
        let split = value
            .split_once("...")
            .or_else(|| value.split_once(".."))
            .ok_or_else(|| format!("--changed-range expects BASE..HEAD, got `{value}`"))?;
        let base = split.0.trim();
        let head = split.1.trim();
        if base.is_empty() || head.is_empty() {
            return Err(format!(
                "--changed-range expects non-empty BASE and HEAD, got `{value}`"
            ));
        }
        Ok((base.to_string(), head.to_string()))
    }

    fn trusted_today(opts: &Options) -> Result<NaiveDate, String> {
        opts.trusted_now.ok_or_else(|| {
            "missing --trusted-now YYYY-MM-DD; lifecycle evaluation refuses stale default or environment-supplied clocks".into()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg_adr_status() -> LifecycleConfig {
        LifecycleConfig {
            name: "adr-status".into(),
            version: 1,
            stages: vec![
                Stage {
                    id: "proposed".into(),
                    terminal: false,
                    requires_supersession_edge: false,
                    gated_by_milestone: None,
                },
                Stage {
                    id: "accepted".into(),
                    terminal: false,
                    requires_supersession_edge: false,
                    gated_by_milestone: None,
                },
                Stage {
                    id: "superseded".into(),
                    terminal: true,
                    requires_supersession_edge: true,
                    gated_by_milestone: None,
                },
                Stage {
                    id: "archived".into(),
                    terminal: true,
                    requires_supersession_edge: false,
                    gated_by_milestone: None,
                },
            ],
            transitions: vec![
                Transition {
                    from: "proposed".into(),
                    to: "accepted".into(),
                },
                Transition {
                    from: "accepted".into(),
                    to: "superseded".into(),
                },
                Transition {
                    from: "accepted".into(),
                    to: "archived".into(),
                },
                Transition {
                    from: "superseded".into(),
                    to: "archived".into(),
                },
            ],
            sources: vec![],
            defaults: Defaults {
                wave: Some("A".into()),
                case_insensitive_stage_match: true,
            },
        }
    }

    fn artifact(location: &str, stage: Option<&str>) -> LifecycledArtifact {
        LifecycledArtifact {
            location: location.into(),
            kind: "adr-status".into(),
            current_stage: stage.map(String::from),
            observed_at: NaiveDate::ymd(2026, 5, 15),
            deadline_at: None,
            history: vec![],
            supersession_target: None,
            milestone_anchor: None,
        }
    }

    #[test]
    fn flags_stage_not_declared() {
        let cfg = cfg_adr_status();
        let report = evaluate(
            &cfg,
            &[artifact("ADR-0001.md", None)],
            NaiveDate::ymd(2026, 5, 15),
            &[],
        );
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].kind, ViolationKind::StageNotDeclared);
    }

    #[test]
    fn flags_unknown_stage() {
        let cfg = cfg_adr_status();
        let report = evaluate(
            &cfg,
            &[artifact("ADR-0002.md", Some("draft"))],
            NaiveDate::ymd(2026, 5, 15),
            &[],
        );
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].kind, ViolationKind::UnknownStage);
    }

    #[test]
    fn accepts_canonical_stage_case_insensitively() {
        let cfg = cfg_adr_status();
        let report = evaluate(
            &cfg,
            &[
                artifact("ADR-0003.md", Some("Accepted")),
                artifact("ADR-0004.md", Some("ACCEPTED")),
                artifact("ADR-0005.md", Some("accepted")),
            ],
            NaiveDate::ymd(2026, 5, 15),
            &[],
        );
        assert!(report.is_clean(), "{report:?}");
        assert_eq!(report.artifacts_observed, 3);
    }

    #[test]
    fn flags_missing_supersession_on_terminal_stage() {
        let cfg = cfg_adr_status();
        let mut a = artifact("ADR-0006.md", Some("superseded"));
        a.supersession_target = None;
        let report = evaluate(&cfg, &[a], NaiveDate::ymd(2026, 5, 15), &[]);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(
            report.violations[0].kind,
            ViolationKind::MissingSupersession
        );
    }

    #[test]
    fn accepts_supersession_when_target_set() {
        let cfg = cfg_adr_status();
        let mut a = artifact("ADR-0007.md", Some("Superseded"));
        a.supersession_target = Some("ADR-0099".into());
        let report = evaluate(&cfg, &[a], NaiveDate::ymd(2026, 5, 15), &[]);
        assert!(report.is_clean(), "{report:?}");
    }

    #[test]
    fn flags_overdue_transition_against_deadline() {
        let cfg = cfg_adr_status();
        let mut a = artifact("ADR-0008.md", Some("accepted"));
        a.deadline_at = Some(NaiveDate::ymd(2025, 1, 1));
        let report = evaluate(&cfg, &[a], NaiveDate::ymd(2026, 5, 15), &[]);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].kind, ViolationKind::OverdueTransition);
    }

    #[test]
    fn terminal_stage_does_not_emit_overdue() {
        let cfg = cfg_adr_status();
        let mut a = artifact("ADR-0009.md", Some("archived"));
        a.deadline_at = Some(NaiveDate::ymd(2025, 1, 1));
        let report = evaluate(&cfg, &[a], NaiveDate::ymd(2026, 5, 15), &[]);
        assert!(report.is_clean(), "{report:?}");
    }

    #[test]
    fn flags_illegal_transition_in_history() {
        let cfg = cfg_adr_status();
        let mut a = artifact("ADR-0010.md", Some("accepted"));
        a.history = vec![Transition {
            from: "proposed".into(),
            to: "superseded".into(), // skips accepted — not declared
        }];
        let report = evaluate(&cfg, &[a], NaiveDate::ymd(2026, 5, 15), &[]);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].kind, ViolationKind::IllegalTransition);
    }

    #[test]
    fn flags_milestone_overdue_when_gate_reached() {
        let mut cfg = cfg_adr_status();
        // Gate the `proposed` stage on milestone `M01-P07-merge`.
        cfg.stages[0].gated_by_milestone = Some("M01-P07-merge".into());
        let a = artifact("ADR-0011.md", Some("proposed"));
        let report = evaluate(
            &cfg,
            &[a],
            NaiveDate::ymd(2026, 5, 15),
            &["M01-P07-merge".into()],
        );
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].kind, ViolationKind::MilestoneOverdue);
    }

    #[test]
    fn stage_counts_are_tallied() {
        let cfg = cfg_adr_status();
        let report = evaluate(
            &cfg,
            &[
                artifact("a.md", Some("accepted")),
                artifact("b.md", Some("Accepted")),
                artifact("c.md", Some("proposed")),
            ],
            NaiveDate::ymd(2026, 5, 15),
            &[],
        );
        assert!(report.is_clean(), "{report:?}");
        let counts: std::collections::BTreeMap<String, usize> =
            report.stage_counts.into_iter().collect();
        assert_eq!(counts.get("accepted").copied().unwrap_or(0), 2);
        assert_eq!(counts.get("proposed").copied().unwrap_or(0), 1);
    }

    #[test]
    fn cli_options_parse_trusted_date_and_merge_base_inputs() {
        let opts = cli::Options::parse(
            vec![
                "--trusted-now".into(),
                "2026-06-28".into(),
                "--changed-base".into(),
                "origin/dev".into(),
                "--changed-head".into(),
                "HEAD".into(),
                "--wave".into(),
                "B".into(),
            ],
            "specs/lifecycle-configs/adr-status.json",
        )
        .expect("options parse");

        assert_eq!(opts.trusted_now, Some(NaiveDate::ymd(2026, 6, 28)));
        assert_eq!(opts.changed_base.as_deref(), Some("origin/dev"));
        assert_eq!(opts.changed_head.as_deref(), Some("HEAD"));
        assert_eq!(opts.changed_range, None);
        assert_eq!(opts.wave_override, Some(cli::Wave::B));
    }

    #[test]
    fn cli_options_parse_explicit_changed_range() {
        let opts = cli::Options::parse(
            vec![
                "--trusted-now".into(),
                "2026-06-28".into(),
                "--changed-range".into(),
                "abc123..def456".into(),
            ],
            "specs/lifecycle-configs/adr-status.json",
        )
        .expect("options parse");

        assert_eq!(opts.changed_range, Some(("abc123".into(), "def456".into())));
        assert_eq!(opts.changed_base, None);
        assert_eq!(opts.changed_head, None);
    }

    #[test]
    fn cli_options_reject_invalid_trusted_date() {
        let err = cli::Options::parse(
            vec!["--trusted-now".into(), "not-a-date".into()],
            "specs/lifecycle-configs/adr-status.json",
        )
        .expect_err("invalid date rejected");

        assert!(err.contains("--trusted-now expects YYYY-MM-DD"));
    }

    #[test]
    fn cli_options_reject_invalid_calendar_trusted_dates() {
        for value in ["2026-13-40", "2026-00-01", "2025-02-29"] {
            let err = cli::Options::parse(
                vec!["--trusted-now".into(), value.into()],
                "specs/lifecycle-configs/adr-status.json",
            )
            .expect_err("invalid calendar date rejected");

            assert!(err.contains("--trusted-now expects YYYY-MM-DD"));
        }
    }

    #[test]
    fn discovery_rejects_missing_source_roots() {
        let missing = std::env::temp_dir().join(format!(
            "oya-governance-lifecycle-missing-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&missing);
        let shallow = format!("{}/*.md", missing.display());
        let recursive = format!("{}/**/*.md", missing.display());

        assert!(discovery::expand_glob(&shallow).is_err());
        assert!(discovery::expand_glob(&recursive).is_err());
    }
    #[test]
    fn fenceless_document_is_read_whole_without_widening_fenced_documents() {
        // A bare declaration record — the shape of every `registry/catalog/*.yaml`.
        // Before the fence-less admission this returned None for every field.
        let bare = "context: audit\nrole: domain\napi_stability: preview\n";
        assert_eq!(
            discovery::frontmatter_scalar(bare, "api_stability").as_deref(),
            Some("preview"),
            "fence-less record must be read whole"
        );

        // A fenced document must NOT widen: a field that appears only in the
        // BODY stays invisible. This is the assert that fails if the admission
        // is written as "scan the whole document" — i.e. if the `break` at the
        // closing fence is dropped.
        // NOTE: the body scalar deliberately carries no `ADR-<digits>` token.
        // `.rs` is in adr-citation-closure's scan_extensions and its
        // citation_lines census is pinned by EQUALITY, so a realistic-looking
        // ADR id in a test fixture reddens that gate.
        let fenced = "---\nstatus: Accepted\n---\n\nsuperseded_by: replacement-doc\n";
        assert_eq!(
            discovery::frontmatter_scalar(fenced, "status").as_deref(),
            Some("Accepted"),
            "fenced front matter still resolves"
        );
        assert_eq!(
            discovery::frontmatter_scalar(fenced, "superseded_by"),
            None,
            "a body line below the closing fence must stay unread"
        );

        // A line ABOVE the opening fence must also stay unread. THIS is the
        // assert that fails if the admission is written as an unconditional
        // `in_fm = true` rather than being keyed on the absence of a fence:
        // the closing-fence `break` hides that mutation from the body case
        // above, so without this line the over-broad implementation passes.
        let preamble = "doc_status: published\n---\nstatus: Accepted\n---\n";
        assert_eq!(
            discovery::frontmatter_scalar(preamble, "doc_status"),
            None,
            "a line above the opening fence must stay unread"
        );
    }

    #[test]
    fn empty_input_is_clean() {
        let cfg = cfg_adr_status();
        let report = evaluate(&cfg, &[], NaiveDate::ymd(2026, 5, 15), &[]);
        assert!(report.is_clean());
        assert_eq!(report.artifacts_observed, 0);
    }
}
