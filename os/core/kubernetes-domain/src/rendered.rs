//! The rendered output of a controller (files written to disk).
//!
//! Mirrors the side of the Talos k8s controllers that writes files to the host
//! (`/etc/kubernetes/...`, static pod manifests, kubelet config). The syscall
//! boundary is modeled as the [`FileSink`] trait with an in-memory
//! implementation ([`InMemoryFileSink`]) so the logic stays testable and
//! dependency-free.

use crate::error::{K8sError, Result};
use std::collections::BTreeMap;

/// POSIX-ish file permission bits, narrowed to what the controllers set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileMode(pub u16);

impl FileMode {
    /// World-readable config (`0644`).
    pub const CONFIG: FileMode = FileMode(0o644);
    /// Private key material (`0600`).
    pub const SECRET: FileMode = FileMode(0o600);
    /// Executable (`0755`).
    pub const EXEC: FileMode = FileMode(0o755);

    /// Whether the mode is group/other readable.
    pub fn is_world_readable(self) -> bool {
        self.0 & 0o044 != 0
    }
}

/// A single rendered file: path, contents, and mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedFile {
    /// Absolute destination path.
    pub path: String,
    /// File contents.
    pub contents: Vec<u8>,
    /// Permission bits.
    pub mode: FileMode,
}

impl RenderedFile {
    /// Build a rendered file, validating the path is absolute and non-empty.
    pub fn new(
        path: impl Into<String>,
        contents: impl Into<Vec<u8>>,
        mode: FileMode,
    ) -> Result<Self> {
        let path = path.into();
        if !path.starts_with('/') {
            return Err(K8sError::Render(format!(
                "rendered file path must be absolute: {path}"
            )));
        }
        Ok(RenderedFile {
            path,
            contents: contents.into(),
            mode,
        })
    }

    /// True if this file holds secret material (mode 0600, not world-readable).
    pub fn is_secret(&self) -> bool {
        !self.mode.is_world_readable()
    }
}

/// The boundary trait controllers write through. The real implementation hits
/// the filesystem; tests use [`InMemoryFileSink`].
pub trait FileSink {
    /// Write a single file, creating or replacing it.
    fn write(&mut self, file: &RenderedFile) -> Result<()>;
}

/// The accumulated output of a controller run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RenderedOutput {
    files: Vec<RenderedFile>,
}

impl RenderedOutput {
    /// An empty output set.
    pub fn new() -> Self {
        RenderedOutput { files: Vec::new() }
    }

    /// Add a file to the output, rejecting a duplicate path.
    pub fn add(&mut self, file: RenderedFile) -> Result<()> {
        if self.files.iter().any(|f| f.path == file.path) {
            return Err(K8sError::Render(format!(
                "duplicate rendered file: {}",
                file.path
            )));
        }
        self.files.push(file);
        Ok(())
    }

    /// The rendered files.
    pub fn files(&self) -> &[RenderedFile] {
        &self.files
    }

    /// Number of files.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Whether the output is empty.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Flush the whole output set through a sink, in insertion order.
    pub fn flush(&self, sink: &mut dyn FileSink) -> Result<()> {
        for f in &self.files {
            sink.write(f)?;
        }
        Ok(())
    }
}

/// An in-memory [`FileSink`] used by tests: records the last write per path.
#[derive(Debug, Default)]
pub struct InMemoryFileSink {
    files: BTreeMap<String, RenderedFile>,
    writes: usize,
}

impl InMemoryFileSink {
    /// A fresh, empty sink.
    pub fn new() -> Self {
        InMemoryFileSink {
            files: BTreeMap::new(),
            writes: 0,
        }
    }

    /// Fetch a written file by path.
    pub fn get(&self, path: &str) -> Option<&RenderedFile> {
        self.files.get(path)
    }

    /// Number of distinct paths written.
    pub fn count(&self) -> usize {
        self.files.len()
    }

    /// Total number of write calls (including overwrites).
    pub fn write_count(&self) -> usize {
        self.writes
    }
}

impl FileSink for InMemoryFileSink {
    fn write(&mut self, file: &RenderedFile) -> Result<()> {
        self.writes += 1;
        self.files.insert(file.path.clone(), file.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_readability() {
        assert!(FileMode::CONFIG.is_world_readable());
        assert!(!FileMode::SECRET.is_world_readable());
    }

    #[test]
    fn rejects_relative_paths() {
        assert!(RenderedFile::new("etc/x", b"x".to_vec(), FileMode::CONFIG).is_err());
        let f = RenderedFile::new("/etc/x", b"x".to_vec(), FileMode::SECRET).unwrap();
        assert!(f.is_secret());
    }

    #[test]
    fn output_rejects_duplicate_paths() {
        let mut out = RenderedOutput::new();
        out.add(RenderedFile::new("/a", b"1".to_vec(), FileMode::CONFIG).unwrap())
            .unwrap();
        let err = out
            .add(RenderedFile::new("/a", b"2".to_vec(), FileMode::CONFIG).unwrap())
            .unwrap_err();
        assert_eq!(err.kind(), "render");
    }

    #[test]
    fn flush_writes_all_to_sink() {
        let mut out = RenderedOutput::new();
        out.add(
            RenderedFile::new(
                "/etc/kubernetes/kubelet.yaml",
                b"a".to_vec(),
                FileMode::CONFIG,
            )
            .unwrap(),
        )
        .unwrap();
        out.add(
            RenderedFile::new(
                "/etc/kubernetes/pki/sa.key",
                b"b".to_vec(),
                FileMode::SECRET,
            )
            .unwrap(),
        )
        .unwrap();
        let mut sink = InMemoryFileSink::new();
        out.flush(&mut sink).unwrap();
        assert_eq!(sink.count(), 2);
        assert_eq!(sink.write_count(), 2);
        assert_eq!(
            sink.get("/etc/kubernetes/pki/sa.key").unwrap().mode,
            FileMode::SECRET
        );
    }

    #[test]
    fn overwrite_counts_as_write_not_path() {
        let mut sink = InMemoryFileSink::new();
        let f = RenderedFile::new("/x", b"1".to_vec(), FileMode::CONFIG).unwrap();
        sink.write(&f).unwrap();
        sink.write(&f).unwrap();
        assert_eq!(sink.count(), 1);
        assert_eq!(sink.write_count(), 2);
    }
}
