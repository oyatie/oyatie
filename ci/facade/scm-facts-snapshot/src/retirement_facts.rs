//! Input-fact projection, path snapshots, and equivalence index.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;

use serde_json::{Value, json};

use super::{
    ControlPlaneEntry, ExpectedInput, RetirementObjectSource, TreeEntry, require_regular,
    sha256_digest,
};

pub(crate) fn input_fact(
    source: &impl RetirementObjectSource,
    input: &ExpectedInput,
    protected: &BTreeMap<String, TreeEntry>,
    candidate: &BTreeMap<String, TreeEntry>,
    equivalence_index: &EquivalenceIndex,
) -> Result<Value, String> {
    let protected_snapshot = path_snapshot(source, protected.get(&input.path))?;
    let candidate_snapshot = path_snapshot(source, candidate.get(&input.path))?;
    let candidate_equivalent_paths = equivalence_index
        .candidate
        .get(&input.path)
        .cloned()
        .unwrap_or_default();
    let protected_equivalent_paths = equivalence_index
        .protected
        .get(&input.path)
        .cloned()
        .unwrap_or_default();
    let candidate_new_equivalent_paths = candidate_equivalent_paths
        .iter()
        .filter(|path| !protected_equivalent_paths.contains(path))
        .cloned()
        .collect::<Vec<_>>();
    Ok(json!({
        "path": input.path,
        "mode": input.mode,
        "predecessor_blob_oid": input.predecessor_blob_oid,
        "sha256": input.sha256,
        "byte_count": input.byte_count,
        "predecessor_path_exists": true,
        "predecessor_path_kind": "regular",
        "predecessor_sha256": input.sha256,
        "predecessor_byte_count": input.byte_count,
        "predecessor_mode": input.mode,
        "protected_path_exists": protected_snapshot.exists,
        "protected_path_kind": protected_snapshot.kind,
        "protected_blob_oid": protected_snapshot.blob_oid,
        "protected_sha256": protected_snapshot.sha256,
        "protected_byte_count": protected_snapshot.byte_count,
        "protected_mode": protected_snapshot.mode,
        "candidate_path_exists": candidate_snapshot.exists,
        "candidate_path_kind": candidate_snapshot.kind,
        "candidate_blob_oid": candidate_snapshot.blob_oid,
        "candidate_sha256": candidate_snapshot.sha256,
        "candidate_byte_count": candidate_snapshot.byte_count,
        "candidate_mode": candidate_snapshot.mode,
        "candidate_new_equivalent_paths": candidate_new_equivalent_paths,
        "candidate_equivalent_paths": candidate_equivalent_paths,
    }))
}

#[derive(Debug)]
pub(crate) struct PathSnapshot {
    pub(crate) exists: bool,
    pub(crate) kind: Value,
    pub(crate) blob_oid: Value,
    pub(crate) sha256: Value,
    pub(crate) byte_count: Value,
    pub(crate) mode: Value,
}

pub(crate) fn path_snapshot(
    source: &impl RetirementObjectSource,
    entry: Option<&TreeEntry>,
) -> Result<PathSnapshot, String> {
    let Some(entry) = entry else {
        return Ok(PathSnapshot {
            exists: false,
            kind: Value::Null,
            blob_oid: Value::Null,
            sha256: Value::Null,
            byte_count: Value::Null,
            mode: Value::Null,
        });
    };
    require_regular(entry, "retirement target")?;
    let bytes = source.read_blob(&entry.oid)?;
    Ok(PathSnapshot {
        exists: true,
        kind: json!("regular"),
        blob_oid: json!(entry.oid),
        sha256: json!(sha256_digest(&bytes)),
        byte_count: json!(bytes.len() as u64),
        mode: json!(entry.mode),
    })
}

#[derive(Debug, Default)]
pub(crate) struct EquivalenceIndex {
    pub(crate) protected: BTreeMap<String, Vec<String>>,
    pub(crate) candidate: BTreeMap<String, Vec<String>>,
}

pub(crate) fn build_equivalence_index(
    source: &impl RetirementObjectSource,
    predecessor_bodies: &BTreeMap<String, Vec<u8>>,
    protected: &BTreeMap<String, TreeEntry>,
    candidate: &BTreeMap<String, TreeEntry>,
) -> Result<EquivalenceIndex, String> {
    let mut paths_by_oid = BTreeMap::<String, (Vec<String>, Vec<String>)>::new();
    for (entries, tree_paths) in [(protected, 0_usize), (candidate, 1_usize)] {
        for entry in entries.values().filter(|entry| entry.kind == "blob") {
            let tree_paths_by_oid = paths_by_oid.entry(entry.oid.clone()).or_default();
            match tree_paths {
                0 => tree_paths_by_oid.0.push(entry.path.clone()),
                _ => tree_paths_by_oid.1.push(entry.path.clone()),
            }
        }
    }
    let mut index = EquivalenceIndex {
        protected: predecessor_bodies
            .keys()
            .cloned()
            .map(|path| (path, Vec::new()))
            .collect(),
        candidate: predecessor_bodies
            .keys()
            .cloned()
            .map(|path| (path, Vec::new()))
            .collect(),
    };
    let blob_oids = paths_by_oid.keys().cloned().collect::<Vec<_>>();
    let targets_by_size = predecessor_bodies
        .iter()
        .map(|(path, bytes)| {
            (
                bytes.len() as u64,
                path.as_str(),
                sha256_digest(bytes),
                bytes.as_slice(),
            )
        })
        .fold(BTreeMap::<u64, Vec<_>>::new(), |mut targets, target| {
            targets.entry(target.0).or_default().push(target);
            targets
        });
    source.visit_blobs(&blob_oids, &mut |blob_oid, size, reader| {
        let Some(targets) = targets_by_size.get(&size) else {
            return Ok(());
        };
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .map_err(|error| format!("retirement read streamed blob {blob_oid}: {error}"))?;
        let hash = sha256_digest(&bytes);
        for (_, input_path, expected_hash, expected_bytes) in targets {
            if hash != *expected_hash || bytes != *expected_bytes {
                continue;
            }
            let (protected_paths, candidate_paths) = paths_by_oid
                .get(blob_oid)
                .ok_or_else(|| format!("retirement streamed unknown blob {blob_oid}"))?;
            index
                .protected
                .get_mut(*input_path)
                .expect("equivalence index has every predecessor input")
                .extend(
                    protected_paths
                        .iter()
                        .filter(|path| path.as_str() != *input_path)
                        .cloned(),
                );
            index
                .candidate
                .get_mut(*input_path)
                .expect("equivalence index has every predecessor input")
                .extend(
                    candidate_paths
                        .iter()
                        .filter(|path| path.as_str() != *input_path)
                        .cloned(),
                );
        }
        Ok(())
    })?;
    for paths in index
        .protected
        .values_mut()
        .chain(index.candidate.values_mut())
    {
        paths.sort();
        paths.dedup();
    }
    Ok(index)
}

pub(crate) fn coverage_scope(
    entry: &ControlPlaneEntry,
    predecessor: &BTreeMap<String, TreeEntry>,
    protected: &BTreeMap<String, TreeEntry>,
    candidate: &BTreeMap<String, TreeEntry>,
) -> Value {
    let selectors = entry
        .selectors
        .iter()
        .flat_map(|selector| {
            selector.expected_inputs.iter().map(|input| {
                let predecessor_exists = predecessor.contains_key(&input.path);
                let protected_exists = protected.contains_key(&input.path);
                let candidate_exists = candidate.contains_key(&input.path);
                let singleton = |present: bool| {
                    present
                        .then(|| input.path.clone())
                        .into_iter()
                        .collect::<Vec<_>>()
                };
                json!({
                    "selector_type": "exact",
                    "selector": input.path,
                    "protected_paths": singleton(protected_exists),
                    "predecessor_paths": singleton(predecessor_exists),
                    "candidate_paths": singleton(candidate_exists),
                    "removed_paths": singleton(protected_exists && !candidate_exists),
                    "surviving_paths": singleton(protected_exists && candidate_exists),
                    "candidate_only_paths": singleton(!protected_exists && candidate_exists),
                    "external_assertion": false,
                })
            })
        })
        .collect::<Vec<_>>();
    let required_retired_paths = entry
        .selectors
        .iter()
        .flat_map(|selector| selector.expected_inputs.iter())
        .map(|input| input.path.clone())
        .collect::<BTreeSet<_>>();
    json!({
        "scope_ref": entry.scope_ref,
        "scope_type": entry.scope_type,
        "selectors": selectors,
        "required_retired_paths": required_retired_paths,
    })
}
