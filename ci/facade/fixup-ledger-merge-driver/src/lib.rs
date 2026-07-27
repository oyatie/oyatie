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
pub fn merge_ledgers(base: &str, ours: &str, theirs: &str) -> Result<String, MergeError> {
    let base = parse(base, "base")?;
    let ours = parse(ours, "ours")?;
    let theirs = parse(theirs, "theirs")?;

    if ours.header != theirs.header {
        return Err(MergeError::new(
            MergeErrorKind::Conflict,
            "the two sides changed the schema header differently; resolve it by hand",
        ));
    }

    let base_rows = base.by_id();
    let ours_rows = ours.by_id();
    let theirs_rows = theirs.by_id();

    let mut out: Vec<&str> = Vec::with_capacity(ours.rows.len() + theirs.rows.len());
    let mut emitted: BTreeMap<&str, ()> = BTreeMap::new();

    for (id, ours_line) in ours.rows.iter().map(|(id, line)| (id.as_str(), *line)) {
        let resolved = resolve(
            id,
            base_rows.get(id).copied(),
            Some(ours_line),
            theirs_rows.get(id).copied(),
        )?;
        if let Some(line) = resolved {
            out.push(line);
            emitted.insert(id, ());
        }
    }

    for (id, theirs_line) in theirs.rows.iter().map(|(id, line)| (id.as_str(), *line)) {
        if emitted.contains_key(id) {
            continue;
        }
        let resolved = resolve(
            id,
            base_rows.get(id).copied(),
            ours_rows.get(id).copied(),
            Some(theirs_line),
        )?;
        if let Some(line) = resolved {
            out.push(line);
            emitted.insert(id, ());
        }
    }

    // Deletion never wins: a base row dropped by one side and untouched by the other is carried.
    for (id, base_line) in base.rows.iter().map(|(id, line)| (id.as_str(), *line)) {
        if emitted.contains_key(id) {
            continue;
        }
        out.push(base_line);
        emitted.insert(id, ());
    }

    let mut merged = String::with_capacity(ours.header.len() + 1);
    merged.push_str(ours.header);
    for line in &out {
        merged.push('\n');
        merged.push_str(line);
    }
    merged.push('\n');

    validate(&merged, &base, &ours, &theirs)?;
    Ok(merged)
}

/// Resolve one `id` across the three sides. `None` means "emit nothing here".
fn resolve<'a>(
    id: &str,
    base: Option<&'a str>,
    ours: Option<&'a str>,
    theirs: Option<&'a str>,
) -> Result<Option<&'a str>, MergeError> {
    match (base, ours, theirs) {
        (_, None, None) => Ok(None),
        (_, Some(line), None) | (_, None, Some(line)) => Ok(Some(line)),
        (_, Some(ours_line), Some(theirs_line)) if ours_line == theirs_line => Ok(Some(ours_line)),
        (Some(base_line), Some(ours_line), Some(theirs_line)) => {
            if ours_line == base_line {
                Ok(Some(theirs_line))
            } else if theirs_line == base_line {
                Ok(Some(ours_line))
            } else {
                Err(MergeError::new(
                    MergeErrorKind::Conflict,
                    format!("row `{id}` was edited differently on both sides"),
                ))
            }
        }
        (None, Some(_), Some(_)) => Err(MergeError::new(
            MergeErrorKind::Conflict,
            format!("row `{id}` was added with different content on both sides"),
        )),
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
