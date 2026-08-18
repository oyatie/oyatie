//! Wire declarations → neutral [`Declaration`] nodes, with the vocabulary checks.

use std::collections::{BTreeMap, BTreeSet};

use port_engine_api::Declaration;

use crate::error::SnapshotError;
use crate::vocabulary::{
    KNOWN_ATTR_KEYS, KNOWN_DECLARATION_KINDS, KNOWN_FLAGS, KNOWN_MEMBER_KINDS,
};
use crate::wire::DeclarationEntry;

pub(crate) fn convert_declarations(
    unit_id: &str,
    entries: &[DeclarationEntry],
) -> Result<Vec<Declaration>, SnapshotError> {
    convert_level(unit_id, entries, KNOWN_DECLARATION_KINDS)
}

fn convert_level(
    unit_id: &str,
    entries: &[DeclarationEntry],
    allowed_kinds: &[&str],
) -> Result<Vec<Declaration>, SnapshotError> {
    let mut named = BTreeSet::new();
    let mut out = Vec::with_capacity(entries.len());

    for entry in entries {
        if !allowed_kinds.contains(&entry.kind.as_str()) {
            return Err(SnapshotError::UnknownDeclarationKind {
                unit_id: unit_id.to_owned(),
                actual: entry.kind.clone(),
            });
        }
        if entry.name.contains('\0') || entry.type_ref.contains('\0') {
            return Err(SnapshotError::Schema {
                field: "packages.declarations",
            });
        }
        if !entry.name.is_empty() && entry.name != "_" && !named.insert(entry.name.clone()) {
            return Err(SnapshotError::DuplicateDeclaration {
                unit_id: unit_id.to_owned(),
                name: entry.name.clone(),
            });
        }

        let mut flags = BTreeSet::new();
        for flag in &entry.flags {
            if !KNOWN_FLAGS.contains(&flag.as_str()) {
                return Err(SnapshotError::UnknownFlag {
                    unit_id: unit_id.to_owned(),
                    actual: flag.clone(),
                });
            }
            flags.insert(flag.clone());
        }

        let mut attrs = BTreeMap::new();
        for (key, value) in &entry.attrs {
            if !KNOWN_ATTR_KEYS.contains(&key.as_str()) {
                return Err(SnapshotError::UnknownAttr {
                    unit_id: unit_id.to_owned(),
                    actual: key.clone(),
                });
            }
            attrs.insert(key.clone(), value.clone());
        }

        out.push(Declaration {
            kind: entry.kind.clone(),
            name: entry.name.clone(),
            type_ref: entry.type_ref.clone(),
            flags,
            attrs,
            children: convert_level(unit_id, &entry.children, KNOWN_MEMBER_KINDS)?,
        });
    }

    Ok(out)
}
