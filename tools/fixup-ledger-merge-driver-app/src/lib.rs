//! Pure three-way merge for `registry/fixuptasks.jsonl`.
//!
//! The registry is an append-mostly JSONL ledger: line 1 is a schema header carrying **no `id`**,
//! and every subsequent line is one task row keyed by `id`. Two lanes that each file a row touch
//! adjacent bytes at the end of the file, so git's text merge conflicts on essentially every pair
//! of concurrent PRs (GH #1412) even though the change is semantically disjoint.
//!
//! `merge=union` is the obvious shortcut and it is WRONG here for the same reason it is wrong for
//! the friction ledger: it keeps BOTH sides of a row that two lanes edited, yielding two rows with
//! one `id`. Consumers key on `id`, so that is silent corruption rather than a visible conflict.
//!
//! This kernel does a real three-way merge per `id` and is deliberately conservative: it resolves
//! only what is unambiguous, and returns [`MergeErrorKind::Conflict`] otherwise so git falls back
//! to a normal conflict. Zero I/O — the caller owns files.
//!
//! ## The header is carried by POSITION, never by id lookup
//!
//! This is the whole reason the kernel exists in this shape. A resolver that indexes rows by `id`
//! and skips falsy ids drops the header silently, because the header's id is absent. That exact
//! bug (`if i and i not in seen`, where `None` is falsy) deleted the schema header from four
//! branches before anyone noticed. Here the header is bound to line 1 structurally and asserted
//! present on the way out, so the failure mode is a hard error rather than a quiet deletion.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;

use serde_json::Value;

/// Why a merge did not produce a result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeErrorKind {
    /// The sides disagree in a way this kernel will not guess at. Git should conflict normally.
    Conflict,
    /// A side is not the shape this kernel models. Never guess at unmodelled input.
    Parse,
    /// The merged result failed its own post-conditions. Refuse to emit it.
    Validate,
}

#[derive(Debug, Clone)]
pub struct MergeError {
    kind: MergeErrorKind,
    message: String,
}

impl MergeError {
    pub fn new(kind: MergeErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> MergeErrorKind {
        self.kind
    }
}

impl fmt::Display for MergeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for MergeError {}

/// The outcome of a merge.
///
/// `conflicted` does NOT mean "nothing was produced". `content` is always a complete file that
/// contains every row from every side; conflicting regions are wrapped in diff3 markers for a
/// human to resolve. This matters more than it looks: git does NOT re-run its own text merge when
/// a driver exits nonzero — it takes whatever the driver left in `%A` as the conflicted working
/// tree. A driver that exits 1 without writing therefore leaves `ours` standing alone, with no
/// markers and the other side's rows simply absent. The file looks clean and complete, so a
/// reflexive `git add` loses rows silently. That is the very failure class this crate exists to
/// prevent, so the conflict path must write, not abstain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Merged {
    /// The full file to place in `%A`, markers included when `conflicted`.
    pub content: String,
    /// True when at least one region needs a human. The caller should exit nonzero.
    pub conflicted: bool,
}

const OURS_MARKER: &str = "<<<<<<< ours";
const BASE_MARKER: &str = "||||||| base";
const SPLIT_MARKER: &str = "=======";
const THEIRS_MARKER: &str = ">>>>>>> theirs";

/// One parsed ledger side: the schema header line, then rows in file order keyed by `id`.
struct Ledger<'a> {
    header: &'a str,
    /// `id` -> exact source line. Insertion order is file order.
    rows: Vec<(String, &'a str)>,
}

impl<'a> Ledger<'a> {
    fn by_id(&self) -> BTreeMap<&str, &'a str> {
        self.rows
            .iter()
            .map(|(id, line)| (id.as_str(), *line))
            .collect()
    }
}

/// Parse one side. Fails closed: a row without a string `id`, a duplicate `id`, an unparseable
/// line, or a missing/id-bearing first line is refused rather than guessed at.
fn parse<'a>(text: &'a str, side: &str) -> Result<Ledger<'a>, MergeError> {
    let mut lines = text.split('\n').filter(|line| !line.trim().is_empty());

    let header = lines.next().ok_or_else(|| {
        MergeError::new(MergeErrorKind::Parse, format!("{side}: ledger is empty"))
    })?;
    let header_value: Value = serde_json::from_str(header).map_err(|err| {
        MergeError::new(
            MergeErrorKind::Parse,
            format!("{side}: schema header is not JSON: {err}"),
        )
    })?;
    if header_value.get("id").is_some() {
        return Err(MergeError::new(
            MergeErrorKind::Parse,
            format!("{side}: line 1 carries an `id`; expected the id-less schema header"),
        ));
    }

    let mut rows = Vec::new();
    let mut seen: BTreeMap<String, ()> = BTreeMap::new();
    for (index, line) in lines.enumerate() {
        let value: Value = serde_json::from_str(line).map_err(|err| {
            MergeError::new(
                MergeErrorKind::Parse,
                format!("{side}: row {} is not JSON: {err}", index + 2),
            )
        })?;
        let id = value
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                MergeError::new(
                    MergeErrorKind::Parse,
                    format!("{side}: row {} has no string `id`", index + 2),
                )
            })?
            .to_owned();
        if seen.insert(id.clone(), ()).is_some() {
            return Err(MergeError::new(
                MergeErrorKind::Parse,
                format!("{side}: duplicate id `{id}`"),
            ));
        }
        rows.push((id, line));
    }

    Ok(Ledger { header, rows })
}

/// Three-way merge `base`, `ours`, `theirs` into one canonical ledger.
///
/// Per `id`, resolved only where unambiguous:
///
/// | base | ours | theirs | result |
/// |------|------|--------|--------|
/// | any  | =base| edited | theirs |
/// | any  | edited | =base | ours |
/// | any  | edited | edited (same bytes) | that row |
/// | any  | edited | edited (differently) | **conflict** |
/// | absent | added | absent | ours |
/// | absent | absent | added | theirs |
/// | absent | added | added (same bytes) | that row |
/// | absent | added | added (differently) | **conflict** |
/// | present | deleted | present | **preserved** |
///
/// Deletion never wins. The registry's own `_meta` declares it append-only, and a row silently
/// vanishing in a merge is exactly the failure this kernel exists to stop — the same trade-off
/// `evidence/audit-chain.jsonl` already makes. A legitimate redaction is a single linearised
/// commit on `dev`, not a merge outcome.
///
/// Row order is `ours` file order, then `theirs`-only rows appended in their own order. Output is
/// byte-preserving: source lines are copied verbatim, never re-serialised, so a row's formatting
/// and any non-ASCII content survive untouched.
pub fn merge_ledgers(base: &str, ours: &str, theirs: &str) -> Result<Merged, MergeError> {
    let base = parse(base, "base")?;
    let ours = parse(ours, "ours")?;
    let theirs = parse(theirs, "theirs")?;

    let mut conflicted = false;
    let mut body: Vec<String> = Vec::with_capacity(ours.rows.len() + theirs.rows.len());

    // The header gets the same three-way treatment as a row. Comparing only ours-vs-theirs would
    // call a ONE-sided header edit a conflict, and a `_meta` schema bump is not hypothetical.
    let header = match resolve_three_way(Some(base.header), Some(ours.header), Some(theirs.header))
    {
        Resolution::Row(line) => line.to_owned(),
        Resolution::Absent => unreachable!("all three sides parsed a header"),
        Resolution::Conflict {
            ours: o,
            base: b,
            theirs: t,
        } => {
            conflicted = true;
            conflict_block(o, b, t)
        }
    };

    let base_rows = base.by_id();
    let ours_rows = ours.by_id();
    let theirs_rows = theirs.by_id();
    let mut emitted: BTreeMap<&str, ()> = BTreeMap::new();

    let mut take = |id: &str, resolution: Resolution<'_>, body: &mut Vec<String>| match resolution {
        Resolution::Row(line) => {
            body.push(line.to_owned());
            let _ = id;
        }
        Resolution::Absent => {}
        Resolution::Conflict {
            ours: o,
            base: b,
            theirs: t,
        } => {
            conflicted = true;
            body.push(conflict_block(o, b, t));
        }
    };

    for (id, ours_line) in ours.rows.iter().map(|(id, line)| (id.as_str(), *line)) {
        let resolution = resolve_three_way(
            base_rows.get(id).copied(),
            Some(ours_line),
            theirs_rows.get(id).copied(),
        );
        take(id, resolution, &mut body);
        emitted.insert(id, ());
    }

    for (id, theirs_line) in theirs.rows.iter().map(|(id, line)| (id.as_str(), *line)) {
        if emitted.contains_key(id) {
            continue;
        }
        let resolution = resolve_three_way(
            base_rows.get(id).copied(),
            ours_rows.get(id).copied(),
            Some(theirs_line),
        );
        take(id, resolution, &mut body);
        emitted.insert(id, ());
    }

    // Rows deleted by BOTH sides. Neither loop above reaches them, and deletion never wins, so
    // they are carried. This is the only case this pass handles — a one-sided delete is already
    // resolved above by the `(base, Some, None)` arm.
    for (id, base_line) in base.rows.iter().map(|(id, line)| (id.as_str(), *line)) {
        if emitted.contains_key(id) {
            continue;
        }
        let resolution = resolve_three_way(Some(base_line), None, None);
        take(id, resolution, &mut body);
        emitted.insert(id, ());
    }

    let mut merged = String::new();
    merged.push_str(&header);
    for line in &body {
        merged.push('\n');
        merged.push_str(line);
    }
    merged.push('\n');

    if !conflicted {
        validate(&merged, &base, &ours, &theirs)?;
    }
    Ok(Merged {
        content: merged,
        conflicted,
    })
}

/// Render one conflicting region as a diff3 block. Every side that exists is present, so no
/// content is lost — resolving it is a human edit, not a recovery.
fn conflict_block(ours: Option<&str>, base: Option<&str>, theirs: Option<&str>) -> String {
    let mut out = String::from(OURS_MARKER);
    if let Some(line) = ours {
        out.push('\n');
        out.push_str(line);
    }
    out.push('\n');
    out.push_str(BASE_MARKER);
    if let Some(line) = base {
        out.push('\n');
        out.push_str(line);
    }
    out.push('\n');
    out.push_str(SPLIT_MARKER);
    if let Some(line) = theirs {
        out.push('\n');
        out.push_str(line);
    }
    out.push('\n');
    out.push_str(THEIRS_MARKER);
    out
}

/// What to do with one region (the header, or one `id`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Resolution<'a> {
    /// Emit this line verbatim.
    Row(&'a str),
    /// Emit nothing.
    Absent,
    /// Needs a human; every present side is carried into the marker block.
    Conflict {
        ours: Option<&'a str>,
        base: Option<&'a str>,
        theirs: Option<&'a str>,
    },
}

/// Three-way resolve for one region.
///
/// | base | ours | theirs | result |
/// |------|------|--------|--------|
/// | any | absent | absent | carried from base (deletion never wins) |
/// | any | present | absent | ours |
/// | any | absent | present | theirs |
/// | any | equal on both sides | | that line |
/// | present | =base | edited | theirs |
/// | present | edited | =base | ours |
/// | present | edited | edited differently | **conflict** |
/// | absent | added | added differently | **conflict** |
fn resolve_three_way<'a>(
    base: Option<&'a str>,
    ours: Option<&'a str>,
    theirs: Option<&'a str>,
) -> Resolution<'a> {
    match (base, ours, theirs) {
        // Deleted on both sides. The registry declares itself append-only, so the base row is
        // carried rather than allowed to vanish in a merge.
        (Some(base_line), None, None) => Resolution::Row(base_line),
        (None, None, None) => Resolution::Absent,
        (_, Some(line), None) | (_, None, Some(line)) => Resolution::Row(line),
        (_, Some(ours_line), Some(theirs_line)) if ours_line == theirs_line => {
            Resolution::Row(ours_line)
        }
        (Some(base_line), Some(ours_line), Some(theirs_line)) => {
            if ours_line == base_line {
                Resolution::Row(theirs_line)
            } else if theirs_line == base_line {
                Resolution::Row(ours_line)
            } else {
                Resolution::Conflict {
                    ours: Some(ours_line),
                    base: Some(base_line),
                    theirs: Some(theirs_line),
                }
            }
        }
        (None, Some(ours_line), Some(theirs_line)) => Resolution::Conflict {
            ours: Some(ours_line),
            base: None,
            theirs: Some(theirs_line),
        },
    }
}

/// Re-parse the result and assert the post-conditions before it is allowed out. A driver that
/// emits a subtly wrong ledger is worse than one that declines, so this is a hard gate: the
/// header must survive, no id may be lost, and no id may be duplicated.
fn validate(
    merged: &str,
    base: &Ledger<'_>,
    ours: &Ledger<'_>,
    theirs: &Ledger<'_>,
) -> Result<(), MergeError> {
    let reparsed = parse(merged, "merged").map_err(|err| {
        MergeError::new(
            MergeErrorKind::Validate,
            format!("merged ledger does not re-parse: {err}"),
        )
    })?;
    let present = reparsed.by_id();

    for side in [base, ours, theirs] {
        for (id, _) in &side.rows {
            if !present.contains_key(id.as_str()) {
                return Err(MergeError::new(
                    MergeErrorKind::Validate,
                    format!("merged ledger dropped row `{id}`"),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
