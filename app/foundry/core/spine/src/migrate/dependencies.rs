use super::{PlanError, UpcastTransform};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn check(transforms: &[UpcastTransform]) -> Result<(), PlanError> {
    // Admission has already established unique targets. Walking backwards
    // therefore has at most one successor per property. Finished paths are
    // shared across walks, avoiding recursive stack growth or quadratic work.
    let dependencies: BTreeMap<&str, &str> = transforms
        .iter()
        .filter_map(|transform| match transform {
            UpcastTransform::CopyAs { from, to } if from == to => None,
            UpcastTransform::CopyAs { from, to } | UpcastTransform::ConvertAs { from, to, .. } => {
                Some((to.as_str(), from.as_str()))
            }
            UpcastTransform::DefaultTo { .. } => None,
        })
        .collect();
    let mut finished = BTreeSet::new();
    for start in dependencies.keys().copied() {
        let mut path = BTreeSet::new();
        let mut current = start;
        while !finished.contains(current) {
            if !path.insert(current) {
                return Err(PlanError::CyclicTransforms);
            }
            let Some(next) = dependencies.get(current) else {
                break;
            };
            current = next;
        }
        finished.extend(path);
    }
    Ok(())
}
