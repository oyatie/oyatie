//! Recursive filesystem walking with filters.
//!
//! Mirrors `pkg/archiver/walker.go`: a [`Walker`] descends a tree rooted at a
//! path, applying include/exclude glob-ish filters and a maximum depth, and
//! emits a deterministic, depth-first ordered list of [`WalkEntry`] records.
//!
//! Because this crate is fully offline, the filesystem itself is modeled by the
//! in-memory [`FileTree`] (the OS boundary). The walking *logic* — ordering,
//! filtering, symlink-vs-regular classification, special-file skipping — is the
//! real, testable part and matches Talos behavior.

use std::collections::BTreeMap;

use os_kernel::Error;

/// The classification of a node in the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    /// A regular file with byte contents.
    Regular,
    /// A directory.
    Directory,
    /// A symbolic link (its target is stored separately).
    Symlink,
    /// A character device / fifo / socket — "special" files. Talos's archiver
    /// records the type but never streams content for these.
    Special,
}

impl FileKind {
    /// Whether this kind carries file content bytes.
    pub fn has_content(self) -> bool {
        matches!(self, FileKind::Regular)
    }
}

/// A single node stored in the in-memory [`FileTree`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// Node classification.
    pub kind: FileKind,
    /// File mode bits (e.g. `0o644`).
    pub mode: u32,
    /// Owner uid.
    pub uid: u32,
    /// Owner gid.
    pub gid: u32,
    /// Modification time, seconds since epoch.
    pub mtime: u64,
    /// Regular-file contents (empty for non-regular).
    pub data: Vec<u8>,
    /// Symlink target (empty unless [`FileKind::Symlink`]).
    pub link_target: String,
}

impl Node {
    fn dir(mode: u32) -> Self {
        Node {
            kind: FileKind::Directory,
            mode,
            uid: 0,
            gid: 0,
            mtime: 0,
            data: Vec::new(),
            link_target: String::new(),
        }
    }
}

/// In-memory filesystem standing in for the OS boundary.
///
/// Paths are absolute, slash-separated, normalized (no trailing slash except
/// root `/`). This is the trait-equivalent "syscall provider" for walking: real
/// Talos hits `os.Lstat`/`ioutil.ReadDir`, the tests hit this map.
#[derive(Debug, Clone, Default)]
pub struct FileTree {
    nodes: BTreeMap<String, Node>,
}

impl FileTree {
    /// An empty tree containing only the implicit root directory.
    pub fn new() -> Self {
        let mut nodes = BTreeMap::new();
        nodes.insert("/".to_string(), Node::dir(0o755));
        FileTree { nodes }
    }

    /// Normalize a path: collapse `//`, strip trailing slash (keep root).
    pub fn normalize(path: &str) -> String {
        if path.is_empty() || path == "/" {
            return "/".to_string();
        }
        let mut parts: Vec<&str> = Vec::new();
        for seg in path.split('/') {
            if seg.is_empty() || seg == "." {
                continue;
            }
            parts.push(seg);
        }
        let joined = parts.join("/");
        format!("/{joined}")
    }

    fn ensure_parents(&mut self, path: &str) {
        let norm = Self::normalize(path);
        let mut cur = String::new();
        for seg in norm.trim_start_matches('/').split('/') {
            if seg.is_empty() {
                continue;
            }
            cur.push('/');
            cur.push_str(seg);
            // Stop before the leaf itself.
            if cur == norm {
                break;
            }
            self.nodes
                .entry(cur.clone())
                .or_insert_with(|| Node::dir(0o755));
        }
    }

    /// Insert a directory node, creating any missing parents.
    pub fn add_dir(&mut self, path: &str, mode: u32) {
        let norm = Self::normalize(path);
        self.ensure_parents(&norm);
        self.nodes.insert(norm, Node::dir(mode));
    }

    /// Insert a regular file, creating any missing parents.
    pub fn add_file(&mut self, path: &str, data: &[u8], mode: u32) {
        let norm = Self::normalize(path);
        self.ensure_parents(&norm);
        self.nodes.insert(
            norm,
            Node {
                kind: FileKind::Regular,
                mode,
                uid: 0,
                gid: 0,
                mtime: 0,
                data: data.to_vec(),
                link_target: String::new(),
            },
        );
    }

    /// Insert a symlink whose content is its target path.
    pub fn add_symlink(&mut self, path: &str, target: &str, mode: u32) {
        let norm = Self::normalize(path);
        self.ensure_parents(&norm);
        self.nodes.insert(
            norm,
            Node {
                kind: FileKind::Symlink,
                mode,
                uid: 0,
                gid: 0,
                mtime: 0,
                data: Vec::new(),
                link_target: target.to_string(),
            },
        );
    }

    /// Insert a special (device/fifo/socket) node.
    pub fn add_special(&mut self, path: &str, mode: u32) {
        let norm = Self::normalize(path);
        self.ensure_parents(&norm);
        self.nodes.insert(
            norm,
            Node {
                kind: FileKind::Special,
                mode,
                uid: 0,
                gid: 0,
                mtime: 0,
                data: Vec::new(),
                link_target: String::new(),
            },
        );
    }

    /// Look up a node by absolute path.
    pub fn get(&self, path: &str) -> Option<&Node> {
        self.nodes.get(&Self::normalize(path))
    }

    /// Mutable lookup.
    pub fn get_mut(&mut self, path: &str) -> Option<&mut Node> {
        let norm = Self::normalize(path);
        self.nodes.get_mut(&norm)
    }

    /// Number of nodes stored (including the implicit root).
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether only the implicit root exists.
    pub fn is_empty(&self) -> bool {
        self.nodes.len() <= 1
    }

    /// Immediate children of `dir`, in sorted (deterministic) order.
    fn children(&self, dir: &str) -> Vec<String> {
        let dir = Self::normalize(dir);
        let prefix = if dir == "/" {
            "/".to_string()
        } else {
            format!("{dir}/")
        };
        let mut out = Vec::new();
        for key in self.nodes.keys() {
            if key == &dir {
                continue;
            }
            if let Some(rest) = key.strip_prefix(&prefix)
                && !rest.is_empty() && !rest.contains('/') {
                    out.push(key.clone());
                }
        }
        out.sort();
        out
    }
}

/// A filter rule applied to relative paths during a walk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    /// Match any path whose final component equals this name.
    Name(String),
    /// Match any path that starts with this relative prefix.
    Prefix(String),
    /// Match any path with this file extension (without the dot).
    Extension(String),
}

impl Filter {
    fn matches(&self, rel: &str) -> bool {
        match self {
            Filter::Name(n) => rel.rsplit('/').next() == Some(n.as_str()),
            Filter::Prefix(p) => rel.starts_with(p.as_str()),
            Filter::Extension(ext) => rel
                .rsplit('/')
                .next()
                .and_then(|f| f.rsplit_once('.'))
                .map(|(_, e)| e == ext)
                .unwrap_or(false),
        }
    }
}

/// Options controlling a [`Walker`], mirroring the functional options in
/// `pkg/archiver/walker.go` (`WithMaxRecurseDepth`, `WithSkipRoot`, filters).
#[derive(Debug, Clone, Default)]
pub struct WalkOptions {
    /// Maximum recursion depth; `None` means unbounded. Depth 0 = root only.
    pub max_depth: Option<usize>,
    /// Skip emitting the root entry itself (Talos `WithSkipRoot`).
    pub skip_root: bool,
    /// Follow symlinks instead of recording them as links.
    pub follow_symlinks: bool,
    /// Skip special (device/fifo/socket) files entirely.
    pub skip_special: bool,
    /// If non-empty, only paths matching one of these are emitted.
    pub include: Vec<Filter>,
    /// Paths matching any of these (and their subtrees) are excluded.
    pub exclude: Vec<Filter>,
}

impl WalkOptions {
    /// Builder: set max recursion depth.
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    /// Builder: skip the root entry.
    pub fn skip_root(mut self) -> Self {
        self.skip_root = true;
        self
    }

    /// Builder: skip special files.
    pub fn skip_special(mut self) -> Self {
        self.skip_special = true;
        self
    }

    /// Builder: add an include filter.
    pub fn include(mut self, f: Filter) -> Self {
        self.include.push(f);
        self
    }

    /// Builder: add an exclude filter.
    pub fn exclude(mut self, f: Filter) -> Self {
        self.exclude.push(f);
        self
    }
}

/// A single emitted entry from a walk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalkEntry {
    /// Absolute path within the [`FileTree`].
    pub abs_path: String,
    /// Path relative to the walk root (root itself is `"."`).
    pub rel_path: String,
    /// Node classification.
    pub kind: FileKind,
    /// Mode bits.
    pub mode: u32,
    /// Depth below the root (root = 0).
    pub depth: usize,
}

/// The recursive walker.
#[derive(Debug, Clone)]
pub struct Walker {
    opts: WalkOptions,
}

impl Walker {
    /// Construct a walker with the given options.
    pub fn new(opts: WalkOptions) -> Self {
        Walker { opts }
    }

    fn rel_of(root: &str, abs: &str) -> String {
        if abs == root {
            return ".".to_string();
        }
        let root = FileTree::normalize(root);
        let prefix = if root == "/" {
            "/".to_string()
        } else {
            format!("{root}/")
        };
        abs.strip_prefix(&prefix).unwrap_or(abs).to_string()
    }

    fn included(&self, rel: &str) -> bool {
        if self.opts.include.is_empty() {
            return true;
        }
        self.opts.include.iter().any(|f| f.matches(rel))
    }

    fn excluded(&self, rel: &str) -> bool {
        self.opts.exclude.iter().any(|f| f.matches(rel))
    }

    /// Walk `root` depth-first, returning filtered entries in deterministic
    /// order. Returns [`Error::NotFound`] if the root does not exist.
    pub fn walk(&self, tree: &FileTree, root: &str) -> crate::Result<Vec<WalkEntry>> {
        let root = FileTree::normalize(root);
        if tree.get(&root).is_none() {
            return Err(Error::not_found(format!("walk root not found: {root}")));
        }
        let mut out = Vec::new();
        self.walk_inner(tree, &root, &root, 0, &mut out)?;
        Ok(out)
    }

    fn walk_inner(
        &self,
        tree: &FileTree,
        root: &str,
        path: &str,
        depth: usize,
        out: &mut Vec<WalkEntry>,
    ) -> crate::Result<()> {
        let node = tree
            .get(path)
            .ok_or_else(|| Error::not_found(format!("path vanished: {path}")))?;
        let rel = Self::rel_of(root, path);
        let is_root = path == root;

        // Apply special-file skipping.
        if node.kind == FileKind::Special && self.opts.skip_special {
            return Ok(());
        }

        // Exclusion prunes the whole subtree.
        if !is_root && self.excluded(&rel) {
            return Ok(());
        }

        let emit = !(is_root && self.opts.skip_root) && (is_root || self.included(&rel));
        if emit {
            out.push(WalkEntry {
                abs_path: path.to_string(),
                rel_path: rel.clone(),
                kind: node.kind,
                mode: node.mode,
                depth,
            });
        }

        // Recurse into directories (and followed symlinks) up to max depth.
        let descend = match node.kind {
            FileKind::Directory => true,
            FileKind::Symlink => self.opts.follow_symlinks,
            _ => false,
        };
        if descend {
            if let Some(max) = self.opts.max_depth
                && depth >= max {
                    return Ok(());
                }
            let dir = if node.kind == FileKind::Symlink {
                FileTree::normalize(&node.link_target)
            } else {
                path.to_string()
            };
            for child in tree.children(&dir) {
                self.walk_inner(tree, root, &child, depth + 1, out)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tree() -> FileTree {
        let mut t = FileTree::new();
        t.add_dir("/var", 0o755);
        t.add_dir("/var/log", 0o755);
        t.add_file("/var/log/messages", b"boot\n", 0o644);
        t.add_file("/var/log/kern.log", b"oops\n", 0o600);
        t.add_dir("/var/lib", 0o700);
        t.add_file("/var/lib/secret.key", b"xxxx", 0o400);
        t.add_symlink("/var/run", "/run", 0o777);
        t.add_special("/var/dev0", 0o660);
        t
    }

    #[test]
    fn normalize_collapses_and_strips() {
        assert_eq!(FileTree::normalize("//var//log/"), "/var/log");
        assert_eq!(FileTree::normalize("/"), "/");
        assert_eq!(FileTree::normalize(""), "/");
        assert_eq!(FileTree::normalize("/a/./b"), "/a/b");
    }

    #[test]
    fn add_file_creates_parents() {
        let mut t = FileTree::new();
        t.add_file("/a/b/c.txt", b"hi", 0o644);
        assert_eq!(t.get("/a").unwrap().kind, FileKind::Directory);
        assert_eq!(t.get("/a/b").unwrap().kind, FileKind::Directory);
        assert_eq!(t.get("/a/b/c.txt").unwrap().data, b"hi");
    }

    #[test]
    fn walk_is_depth_first_and_sorted() {
        let t = sample_tree();
        let w = Walker::new(WalkOptions::default());
        let entries = w.walk(&t, "/var").unwrap();
        let rels: Vec<&str> = entries.iter().map(|e| e.rel_path.as_str()).collect();
        // root first, then lexicographic descent.
        assert_eq!(rels[0], ".");
        assert!(rels.contains(&"log"));
        assert!(rels.contains(&"log/messages"));
        // lib comes before log in sort order; ensure full subtree present.
        assert!(rels.contains(&"lib/secret.key"));
    }

    #[test]
    fn skip_root_omits_root_entry() {
        let t = sample_tree();
        let w = Walker::new(WalkOptions::default().skip_root());
        let entries = w.walk(&t, "/var").unwrap();
        assert!(entries.iter().all(|e| e.rel_path != "."));
    }

    #[test]
    fn max_depth_limits_recursion() {
        let t = sample_tree();
        let w = Walker::new(WalkOptions::default().with_max_depth(1));
        let entries = w.walk(&t, "/var").unwrap();
        // depth 0 (root) and depth 1 children only.
        assert!(entries.iter().all(|e| e.depth <= 1));
        assert!(entries.iter().any(|e| e.rel_path == "log"));
        assert!(entries.iter().all(|e| e.rel_path != "log/messages"));
    }

    #[test]
    fn exclude_prunes_subtree() {
        let t = sample_tree();
        let w = Walker::new(WalkOptions::default().exclude(Filter::Prefix("lib".into())));
        let entries = w.walk(&t, "/var").unwrap();
        assert!(entries.iter().all(|e| !e.rel_path.starts_with("lib")));
        assert!(entries.iter().any(|e| e.rel_path.starts_with("log")));
    }

    #[test]
    fn include_extension_filter() {
        let t = sample_tree();
        let w = Walker::new(WalkOptions::default().include(Filter::Extension("log".into())));
        let entries = w.walk(&t, "/var").unwrap();
        // root always emitted; only *.log files otherwise.
        let non_root: Vec<&WalkEntry> = entries.iter().filter(|e| e.rel_path != ".").collect();
        assert!(non_root.iter().all(|e| e.rel_path.ends_with(".log")));
        assert!(non_root.iter().any(|e| e.rel_path == "log/kern.log"));
    }

    #[test]
    fn skip_special_drops_devices() {
        let t = sample_tree();
        let w = Walker::new(WalkOptions::default().skip_special());
        let entries = w.walk(&t, "/var").unwrap();
        assert!(entries.iter().all(|e| e.kind != FileKind::Special));
    }

    #[test]
    fn walk_missing_root_errors() {
        let t = sample_tree();
        let w = Walker::new(WalkOptions::default());
        let err = w.walk(&t, "/nope").unwrap_err();
        assert_eq!(err.kind(), "not_found");
    }
}
