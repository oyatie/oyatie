//! File-existence conditions.
//!
//! Mirrors Talos's `conditions.WaitForFileToExist` and
//! `conditions.WaitForFilesToExist` from `pkg/conditions/file.go`, which the
//! boot sequencer uses to wait for sockets / device nodes / generated config
//! files to appear (e.g. `/var/run/containerd/containerd.sock`,
//! `/etc/kubernetes/...`).
//!
//! The OS filesystem is modeled as the [`FileProbe`] trait so the conditions
//! are testable in-memory.

use crate::condition::{Condition, Poll};
use std::collections::HashSet;

/// Read-only probe of the filesystem: "does this path exist right now?".
///
/// This is the narrow OS boundary the file conditions need. In production it
/// would call `stat(2)`; in tests it is backed by [`InMemoryFiles`].
pub trait FileProbe {
    /// True if `path` currently exists.
    fn exists(&self, path: &str) -> bool;
}

/// In-memory [`FileProbe`] used by tests and by callers that want to simulate
/// files appearing over the course of a boot sequence.
#[derive(Debug, Default, Clone)]
pub struct InMemoryFiles {
    present: HashSet<String>,
}

impl InMemoryFiles {
    /// An empty filesystem.
    pub fn new() -> Self {
        InMemoryFiles {
            present: HashSet::new(),
        }
    }

    /// Mark `path` as existing.
    pub fn create(&mut self, path: impl Into<String>) {
        self.present.insert(path.into());
    }

    /// Remove `path`.
    pub fn remove(&mut self, path: &str) {
        self.present.remove(path);
    }

    /// Number of files currently present.
    pub fn len(&self) -> usize {
        self.present.len()
    }

    /// True when no files are present.
    pub fn is_empty(&self) -> bool {
        self.present.is_empty()
    }
}

impl FileProbe for InMemoryFiles {
    fn exists(&self, path: &str) -> bool {
        self.present.contains(path)
    }
}

/// Wait for a single file to exist.
///
/// Analogue of `conditions.WaitForFileToExist(path)`.
pub struct WaitForFileToExist<'a, P: FileProbe> {
    probe: &'a P,
    path: String,
}

impl<'a, P: FileProbe> WaitForFileToExist<'a, P> {
    /// Construct a condition waiting on `path`.
    pub fn new(probe: &'a P, path: impl Into<String>) -> Self {
        WaitForFileToExist {
            probe,
            path: path.into(),
        }
    }
}

impl<P: FileProbe> Condition for WaitForFileToExist<'_, P> {
    fn poll(&self) -> Poll {
        if self.probe.exists(&self.path) {
            Poll::Ready
        } else {
            Poll::Pending(self.describe())
        }
    }

    fn describe(&self) -> String {
        format!("file {:?} to exist", self.path)
    }
}

/// Wait for *all* of a set of files to exist.
///
/// Analogue of `conditions.WaitForFilesToExist(paths...)`. The status string
/// reports the still-missing files, matching Talos's behaviour of describing
/// exactly what it is blocked on.
pub struct WaitForFilesToExist<'a, P: FileProbe> {
    probe: &'a P,
    paths: Vec<String>,
}

impl<'a, P: FileProbe> WaitForFilesToExist<'a, P> {
    /// Construct a condition waiting on every path in `paths`.
    pub fn new<I, S>(probe: &'a P, paths: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        WaitForFilesToExist {
            probe,
            paths: paths.into_iter().map(Into::into).collect(),
        }
    }

    /// The paths that do not yet exist.
    pub fn missing(&self) -> Vec<&str> {
        self.paths
            .iter()
            .map(String::as_str)
            .filter(|p| !self.probe.exists(p))
            .collect()
    }
}

impl<P: FileProbe> Condition for WaitForFilesToExist<'_, P> {
    fn poll(&self) -> Poll {
        if self.paths.iter().all(|p| self.probe.exists(p)) {
            Poll::Ready
        } else {
            Poll::Pending(self.describe())
        }
    }

    fn describe(&self) -> String {
        let missing = self.missing();
        if missing.is_empty() {
            "all files to exist".to_string()
        } else {
            format!("files {:?} to exist", missing)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::{Poller, SimClock};

    #[test]
    fn single_file_pending_until_created() {
        let mut fs = InMemoryFiles::new();
        let cond = WaitForFileToExist::new(&fs as &InMemoryFiles, "/run/foo.sock");
        assert!(matches!(cond.poll(), Poll::Pending(_)));
        drop(cond);
        fs.create("/run/foo.sock");
        let cond = WaitForFileToExist::new(&fs, "/run/foo.sock");
        assert_eq!(cond.poll(), Poll::Ready);
    }

    #[test]
    fn single_file_describe() {
        let fs = InMemoryFiles::new();
        let cond = WaitForFileToExist::new(&fs, "/etc/kubernetes/kubelet.conf");
        assert_eq!(
            cond.describe(),
            "file \"/etc/kubernetes/kubelet.conf\" to exist"
        );
    }

    #[test]
    fn files_all_present_ready() {
        let mut fs = InMemoryFiles::new();
        fs.create("/a");
        fs.create("/b");
        let cond = WaitForFilesToExist::new(&fs, ["/a", "/b"]);
        assert_eq!(cond.poll(), Poll::Ready);
        assert!(cond.missing().is_empty());
    }

    #[test]
    fn files_reports_missing() {
        let mut fs = InMemoryFiles::new();
        fs.create("/a");
        let cond = WaitForFilesToExist::new(&fs, ["/a", "/b", "/c"]);
        assert!(matches!(cond.poll(), Poll::Pending(_)));
        let mut missing = cond.missing();
        missing.sort();
        assert_eq!(missing, vec!["/b", "/c"]);
    }

    #[test]
    fn waits_then_succeeds_via_poller() {
        // Shared mutable FS that "creates" the file mid-wait is awkward with
        // borrows, so we drive a poller where the file already exists and assert
        // it completes on the first attempt.
        let mut fs = InMemoryFiles::new();
        fs.create("/run/ready");
        let clock = SimClock::new(0);
        let cond = WaitForFileToExist::new(&fs, "/run/ready");
        let report = cond.wait(&clock, Poller::new(3, 5)).unwrap();
        assert_eq!(report.attempts, 1);
    }

    #[test]
    fn in_memory_files_bookkeeping() {
        let mut fs = InMemoryFiles::new();
        assert!(fs.is_empty());
        fs.create("/x");
        fs.create("/y");
        assert_eq!(fs.len(), 2);
        fs.remove("/x");
        assert_eq!(fs.len(), 1);
        assert!(!fs.exists("/x"));
        assert!(fs.exists("/y"));
    }
}
