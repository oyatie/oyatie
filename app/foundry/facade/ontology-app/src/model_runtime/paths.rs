use std::path::{Path, PathBuf};

fn resolved(path: &Path) -> Option<PathBuf> {
    path.canonicalize().ok().or_else(|| {
        let absolute = if path.is_absolute() {
            path.to_owned()
        } else {
            std::env::current_dir().ok()?.join(path)
        };
        Some(
            absolute
                .parent()?
                .canonicalize()
                .ok()?
                .join(absolute.file_name()?),
        )
    })
}

pub(super) fn alias(left: &Path, right: &Path) -> bool {
    if left == right
        || resolved(left)
            .zip(resolved(right))
            .is_some_and(|(l, r)| l == r)
    {
        return true;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let (Ok(left), Ok(right)) = (left.metadata(), right.metadata()) {
            return left.dev() == right.dev() && left.ino() == right.ino();
        }
    }
    false
}
