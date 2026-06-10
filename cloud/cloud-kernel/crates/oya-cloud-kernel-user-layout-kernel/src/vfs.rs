// Pure, arch-neutral in-RAM VFS: an index-slab inode/dentry tree, a
// multi-component absolute-path walker, and a mount table.
//
// This file is the body of the `user_layout::vfs` module (it is `include!`d
// from `lib.rs`, alongside `layout.rs`/`signal.rs`/`timekeep.rs`, which supply
// the crate-level `#![no_std]`). Like those it carries **no inner attributes**
// (`#![...]`) or `//!` module docs, because it is `include!`d both into this
// crate and into the out-of-workspace host harness's module body, where inner
// attributes are not permitted.
//
// It is the **single source of truth** for the parts of the M2 tmpfs/VFS that
// are a *pure function* of their inputs and therefore identical on every arch:
// the tree shape (`/`, `/dev/console`, `/dev/null`), the path tokenizer (`/`,
// `.`, `..`, trailing slash, empty/double-slash components), the `mkdir -p`
// helper, and the mount table. It depends on **nothing** outside `core` +
// `alloc` (the kernel registers a `#[global_allocator]`, so `Vec`/`String` are
// available) and contains **zero `unsafe`**, so the `check-tcb.sh` ratchet
// stays green and the tree/walker is exhaustively host-tested (see `mod
// vfs_tests` at the bottom, run through `crates/arch-aarch64/tests-host/`).
//
// Keeping this logic pure keeps the `unsafe` arch Frames thin: they only do the
// things that *must* be unsafe (copying the path bytes out of user memory, the
// one-`unsafe`-block `with_vfs` accessor), delegating all the tree walking and
// mount bookkeeping here where it can be tested.
//
// ## Why an index-based slab (not `Rc`/`RefCell`)
// `Rc`/`RefCell` parent<->child cycles are the classic tree representation but
// (a) `Rc` is not `Send`, (b) cyclic `Rc` leaks, and (c) `RefCell` runtime
// borrow panics are a footgun in a kernel. An index-based slab — a `Vec<Node>`
// addressed by a `u32` `NodeId`, parent/children stored as indices — is
// strictly safe Rust, allocation-light, and cycle-free, matching the precedent
// of `user_layout`'s existing pure PODs (`signal.rs`, `timekeep.rs`).

// In the bare-metal `user_layout` crate this `extern crate alloc;` links the
// allocator the kernel registers; in the out-of-workspace host test harness it
// links `std`'s precompiled `alloc` (the same crate `std` re-exports), bringing
// `alloc::{Vec,String}` into scope in BOTH contexts without an inner attribute.
// The harness must build with `-Zbuild-std=` (build-std off) so there is a
// single `alloc` — see `tests-host/.cargo/config.toml` + its run command.
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

/// Index into [`Vfs::nodes`]. `u32` keeps nodes small; `0` is always the root
/// `"/"` (see [`ROOT`]).
pub type NodeId = u32;

/// The root node id. `Vfs::new()` always installs `"/"` at slot 0, so a walk of
/// an absolute path starts here.
pub const ROOT: NodeId = 0;

// ---------------------------------------------------------------------------
// Slice-3 baked file contents (the synthetic files talos-init reads at boot).
//
// These are the **minimal** artifacts that empirically carry the real,
// unmodified talos-init through its `LoadConfig` phase, derived by tracing
// `talos-machined/src/boot.rs` + `talos-init/src/{main,config,platform_config}.rs`:
//
//  * `talos-machined/src/boot.rs:1126-1133` (`loadAndValidateConfig`) reads the
//    config, aborts only if `raw.trim().is_empty()`, then `apply_config(&raw)`.
//  * `talos-init/src/main.rs:2337-2354` (`LibcRuntime::load_config`) reads
//    `/proc/cmdline` (`unwrap_or_default`, so absence is fine) + the machine
//    config at `MACHINE_CONFIG_PATH = "/machine-config.yaml"`, then calls
//    `resolve_config(cmdline, store, fallback)`.
//  * `talos-init/src/platform_config.rs:210-252` (`resolve_config`): the `metal`
//    platform with no `talos.config=` has an **empty** `config_sources()`
//    (`talos-platform/src/metal.rs:137-148`), so it takes the `fallback`
//    branch = the `/machine-config.yaml` contents below.
//  * `talos-init/src/main.rs:2370-2386` (`apply_config`) ⇒
//    `talos-init/src/config.rs:95-105` (`try_early_config`) ⇒
//    `machine_config_dhcp_operators` ⇒ `talos-machine-config` `load_from_bytes`,
//    which *parses & validates* the document: it requires a `version:`/`apiVersion:`
//    header (`encoder.rs:53-63`) and a valid `machine.type` (`load.rs:83-111`).
//
// Hence the config must be a schema-valid v1alpha1 document (not merely
// non-empty). The content below is verified to pass `try_early_config` +
// `resolve_config` against the real crates.

/// Baked `/machine-config.yaml`: the minimal schema-valid v1alpha1 Talos
/// machine config that survives `talos-machine-config::load_from_bytes`
/// (valid `version` header + valid `machine.type`), yields hostname `kuberos`
/// to `try_early_config`, and is non-empty for the `boot.rs:1127` guard.
pub const MACHINE_CONFIG_YAML: &str = "version: v1alpha1\n\
machine:\n\
\x20\x20type: controlplane\n\
\x20\x20network:\n\
\x20\x20\x20\x20hostname: kuberos\n\
cluster:\n\
\x20\x20clusterName: kuberos\n";

/// Baked `/proc/cmdline`: selects the `metal` platform whose `config_sources()`
/// is empty, so `resolve_config` takes the initramfs fallback (the baked
/// `/machine-config.yaml`). No `talos.config=` ⇒ `Metal::new()`.
pub const PROC_CMDLINE: &str =
    "talos.platform=metal console=ttyAMA0 console=ttyS0 init_on_alloc=1 slab_nomerge\n";

/// Baked `/proc/version` (informational; some probes read it).
pub const PROC_VERSION: &str =
    "Linux version 6.6.0-kuberos (kuberos@framekernel) (rustc) #1 SMP\n";

/// Baked `/proc/sys/kernel/hostname` (the kernel-default hostname).
pub const PROC_HOSTNAME: &str = "kuberos\n";

/// The kind of a node in the tree. Console/Null exist so `resolve_path` can map
/// a tree hit back to the existing `process::FileKind` (keeping `/dev/console`
/// + `/dev/null` byte-identical); `File` carries baked bytes for Slice 3.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NodeKind {
    /// A directory: has `children`.
    Dir,
    /// A regular file: `data` holds bytes (Slice 3 fills `/machine-config.yaml`
    /// etc.).
    File,
    /// `/dev/console` -> maps to `process::FileKind::Console`.
    Console,
    /// `/dev/null` -> maps to `process::FileKind::Null`.
    Null,
}

/// One node (inode + dentry fused) in the slab. `parent`/`children` are indices
/// into [`Vfs::nodes`], never pointers — strictly safe, cycle-free.
pub struct Node {
    /// What this node is.
    pub kind: NodeKind,
    /// This node's own (leaf) name. The root's name is `""`.
    pub name: String,
    /// Parent index. `ROOT.parent == ROOT` (a self-loop sentinel, so `"/.."`
    /// resolves back to `"/"`).
    pub parent: NodeId,
    /// Child indices (`Dir` only; empty otherwise).
    pub children: Vec<NodeId>,
    /// File bytes (`File` only, Slice 3; empty otherwise).
    pub data: Vec<u8>,
}

impl Node {
    /// A fresh directory node with no children, parented at `parent`.
    fn dir(name: String, parent: NodeId) -> Node {
        Node {
            kind: NodeKind::Dir,
            name,
            parent,
            children: Vec::new(),
            data: Vec::new(),
        }
    }

    /// A fresh device node (`Console`/`Null`) parented at `parent`.
    fn dev(kind: NodeKind, name: String, parent: NodeId) -> Node {
        Node {
            kind,
            name,
            parent,
            children: Vec::new(),
            data: Vec::new(),
        }
    }

    /// A fresh regular-file node carrying `data`, parented at `parent`
    /// (Slice 3: baked `/machine-config.yaml`, `/proc/cmdline`, …).
    fn file(name: String, parent: NodeId, data: Vec<u8>) -> Node {
        Node {
            kind: NodeKind::File,
            name,
            parent,
            children: Vec::new(),
            data,
        }
    }
}

/// A recorded mount. M2 records `(source, target, fstype, flags)` and ensures a
/// `Dir` node exists at `target`; it implements **no** backing filesystem (see
/// the honesty section in the spec §9). `flags` is the `MS_*` bitmask the arch
/// glue passes through (informational in M2 — not enforced).
pub struct MountRecord {
    /// e.g. `"proc"`, `"sysfs"`, `"devtmpfs"`, `"tmpfs"`, `"devpts"`.
    pub source: String,
    /// e.g. `"/proc"`, `"/sys"`, `"/dev"`, `"/run"`, `"/dev/pts"`, `"/dev/shm"`.
    pub target: String,
    /// `== source` for these pseudo-fs.
    pub fstype: String,
    /// `MS_*` bitmask (recorded, not enforced in M2).
    pub flags: u64,
}

/// Walk failure, mapped by the arch glue to an errno.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WalkErr {
    /// A path component did not exist -> `-ENOENT` (-2).
    NotFound,
    /// A non-final component was not a directory -> `-ENOTDIR` (-20).
    NotDir,
}

/// The single in-RAM VFS: the inode/dentry tree rooted at `"/"` plus the mount
/// table. One per arch backend (single namespace, M2). Held behind the per-arch
/// `with_vfs` accessor (the only `unsafe`, in the Frame crate); every method
/// here is 0-unsafe and host-tested.
pub struct Vfs {
    /// The slab. `nodes[ROOT]` is `"/"`. Never shrinks (no unlink in M2), so a
    /// `NodeId` is stable for the kernel's lifetime.
    nodes: Vec<Node>,
    /// Recorded mounts (see [`MountRecord`]).
    mounts: Vec<MountRecord>,
}

impl Vfs {
    /// Build the pre-populated tree the old two-string `resolve_path` implied,
    /// so `/dev/console` + `/dev/null` keep resolving bit-for-bit, plus the
    /// Slice-3 baked synthetic files the real talos-init reads during its
    /// `LoadConfig` phase:
    ///
    /// ```text
    /// /                                  (ROOT, Dir)
    /// ├── dev                            (Dir)
    /// │   ├── console                    (Console)
    /// │   └── null                       (Null)
    /// ├── machine-config.yaml            (File, baked v1alpha1 config)
    /// └── proc                           (Dir)
    ///     ├── cmdline                    (File, "talos.platform=metal …")
    ///     ├── version                    (File)
    ///     └── sys/kernel/hostname        (File)
    /// ```
    ///
    /// The config + cmdline are chosen empirically so talos-init's
    /// `resolve_config` (metal platform, no `talos.config=` ⇒ empty
    /// `config_sources()`) takes the initramfs fallback and reads this file,
    /// and `apply_config` ⇒ `try_early_config` ⇒ `load_from_bytes` parses it
    /// without error. See the spec / report for the exact contract.
    pub fn new() -> Vfs {
        let mut v = Vfs {
            nodes: Vec::new(),
            mounts: Vec::new(),
        };
        // ROOT at slot 0; its parent is itself (the `..`-at-root sentinel).
        v.nodes.push(Node::dir(String::new(), ROOT));
        // /dev
        let dev = v.push_child(ROOT, Node::dir(str_of("dev"), ROOT));
        // /dev/console, /dev/null
        let console = Node::dev(NodeKind::Console, str_of("console"), dev);
        v.push_child(dev, console);
        let null = Node::dev(NodeKind::Null, str_of("null"), dev);
        v.push_child(dev, null);

        // ---- Slice 3: baked, readable synthetic files --------------------
        // /machine-config.yaml — the initramfs fallback machine config.
        v.bake_file(b"/machine-config.yaml", MACHINE_CONFIG_YAML.as_bytes());
        // /proc/cmdline — selects the `metal` platform (whose config source
        // set is empty, so resolve_config takes the fallback above).
        v.bake_file(b"/proc/cmdline", PROC_CMDLINE.as_bytes());
        // /proc/version + /proc/sys/kernel/hostname — read by various early
        // probes; harmless, readable, non-empty.
        v.bake_file(b"/proc/version", PROC_VERSION.as_bytes());
        v.bake_file(b"/proc/sys/kernel/hostname", PROC_HOSTNAME.as_bytes());
        v
    }

    /// Pre-populate a baked `File` at an absolute `path` (creating parent
    /// directories like `mkdir -p`). Used only by [`Vfs::new`] for the fixed
    /// synthetic files; ignores the (impossible-here) `NotDir` collision.
    fn bake_file(&mut self, path: &[u8], data: &[u8]) {
        let mut bytes = Vec::with_capacity(data.len());
        bytes.extend_from_slice(data);
        let _ = self.mkfile_p(path, bytes);
    }

    /// Push `node` into the slab as a child of `parent`, returning its new id.
    /// Caller guarantees `parent` is a `Dir`.
    fn push_child(&mut self, parent: NodeId, node: Node) -> NodeId {
        let id = self.nodes.len() as NodeId;
        self.nodes.push(node);
        self.nodes[parent as usize].children.push(id);
        id
    }

    /// The kind of `id`. (Caller passes a valid id from `walk`/`new`.)
    pub fn kind(&self, id: NodeId) -> NodeKind {
        self.nodes[id as usize].kind
    }

    /// Read-only access to a node's baked bytes (Slice 3 file reads).
    pub fn data(&self, id: NodeId) -> &[u8] {
        &self.nodes[id as usize].data
    }

    /// Number of recorded mounts (used by host tests + a future `getdents`).
    pub fn mount_count(&self) -> usize {
        self.mounts.len()
    }

    /// Find a directory child of `parent` whose name equals `name`, or `None`.
    /// Only `Dir` parents have children, so a non-`Dir` parent yields `None`.
    fn child_named(&self, parent: NodeId, name: &[u8]) -> Option<NodeId> {
        let p = &self.nodes[parent as usize];
        for &c in &p.children {
            if self.nodes[c as usize].name.as_bytes() == name {
                return Some(c);
            }
        }
        None
    }

    /// Resolve an absolute path to a [`NodeId`]. Tokenizes on `/`, handling:
    ///  - leading `/`   (absolute; a non-`/` first byte is still walked from ROOT)
    ///  - `""` and `.`  components -> skipped (no-op)
    ///  - `..`          -> the component's parent (`ROOT.parent == ROOT`, so
    ///                     `"/.."` == `"/"`)
    ///  - trailing `/`  -> the empty final component is skipped
    ///  - repeated `//` -> empty components skipped
    ///
    /// Descending into a non-`Dir` mid-path -> `Err(NotDir)`; a missing child ->
    /// `Err(NotFound)`. 100% pure: operates on `&[u8]` already copied out of
    /// user memory by the arch glue.
    pub fn walk(&self, path: &[u8]) -> Result<NodeId, WalkErr> {
        let mut cur = ROOT;
        for comp in path.split(|&b| b == b'/') {
            if comp.is_empty() || comp == b"." {
                // "", "." -> stay put (covers leading '/', trailing '/', '//').
                continue;
            }
            if comp == b".." {
                cur = self.nodes[cur as usize].parent;
                continue;
            }
            // A real component: `cur` must be a directory to descend.
            if self.nodes[cur as usize].kind != NodeKind::Dir {
                return Err(WalkErr::NotDir);
            }
            match self.child_named(cur, comp) {
                Some(child) => cur = child,
                None => return Err(WalkErr::NotFound),
            }
        }
        Ok(cur)
    }

    /// Idempotent `mkdir -p`: resolve `path`, creating any missing intermediate
    /// and final directories. An existing `Dir` at the path -> `Ok(that id)`. A
    /// path component that collides with a non-`Dir` node -> `Err(NotDir)`. Used
    /// by `do_mount` (auto-create the target) now and by `SYS_MKDIRAT` in
    /// Slice 2.
    pub fn mkdir_p(&mut self, path: &[u8]) -> Result<NodeId, WalkErr> {
        let mut cur = ROOT;
        for comp in path.split(|&b| b == b'/') {
            if comp.is_empty() || comp == b"." {
                continue;
            }
            if comp == b".." {
                cur = self.nodes[cur as usize].parent;
                continue;
            }
            // To descend/create under `cur`, `cur` must itself be a directory.
            if self.nodes[cur as usize].kind != NodeKind::Dir {
                return Err(WalkErr::NotDir);
            }
            match self.child_named(cur, comp) {
                Some(child) => {
                    // Exists. If it is a non-Dir and we still have components to
                    // descend, the next loop iteration will reject it (NotDir);
                    // if it is the final component and a non-Dir, that is also a
                    // collision -> reject so we never return a File as a "dir".
                    if self.nodes[child as usize].kind != NodeKind::Dir {
                        return Err(WalkErr::NotDir);
                    }
                    cur = child;
                }
                None => {
                    // Create a fresh directory child.
                    let name = string_from_bytes(comp);
                    cur = self.push_child(cur, Node::dir(name, cur));
                }
            }
        }
        Ok(cur)
    }

    /// Create (or overwrite) a regular `File` at absolute `path`, carrying
    /// `data`, creating any missing intermediate directories (`mkdir -p` of the
    /// parent). The final component becomes a `File` node:
    ///  - if it does not exist → a fresh `File` is created;
    ///  - if it already exists as a `File` → its `data` is replaced;
    ///  - if it already exists as a non-`File` (Dir/Console/Null) → `Err(NotDir)`
    ///    (we never turn a directory or device into a file).
    ///
    /// A component along the parent path that collides with a non-`Dir` node →
    /// `Err(NotDir)`. Returns the file's [`NodeId`]. Pure + host-tested.
    pub fn mkfile_p(&mut self, path: &[u8], data: Vec<u8>) -> Result<NodeId, WalkErr> {
        // Split the path into parent components + final (file) name.
        let mut comps: Vec<&[u8]> = Vec::new();
        for comp in path.split(|&b| b == b'/') {
            if comp.is_empty() || comp == b"." {
                continue;
            }
            comps.push(comp);
        }
        // An empty path (or "/") has no file name to create.
        let (name, parents) = match comps.split_last() {
            Some((last, head)) => (*last, head),
            None => return Err(WalkErr::NotDir),
        };
        // `..` as the final component is not a creatable file name.
        if name == b".." {
            return Err(WalkErr::NotDir);
        }
        // Walk/create the parent directory chain.
        let mut cur = ROOT;
        for &comp in parents {
            if comp == b".." {
                cur = self.nodes[cur as usize].parent;
                continue;
            }
            if self.nodes[cur as usize].kind != NodeKind::Dir {
                return Err(WalkErr::NotDir);
            }
            cur = match self.child_named(cur, comp) {
                Some(child) => {
                    if self.nodes[child as usize].kind != NodeKind::Dir {
                        return Err(WalkErr::NotDir);
                    }
                    child
                }
                None => {
                    let nm = string_from_bytes(comp);
                    self.push_child(cur, Node::dir(nm, cur))
                }
            };
        }
        // The parent must be a directory to host the file.
        if self.nodes[cur as usize].kind != NodeKind::Dir {
            return Err(WalkErr::NotDir);
        }
        // Create or overwrite the final File node.
        match self.child_named(cur, name) {
            Some(existing) => {
                if self.nodes[existing as usize].kind != NodeKind::File {
                    return Err(WalkErr::NotDir);
                }
                self.nodes[existing as usize].data = data;
                Ok(existing)
            }
            None => {
                let nm = string_from_bytes(name);
                Ok(self.push_child(cur, Node::file(nm, cur, data)))
            }
        }
    }

    /// Copy up to `out.len()` bytes of node `id`'s baked `data`, starting at
    /// byte offset `off`, into `out`. Returns the number of bytes copied (0 at
    /// or past EOF). A non-`File` node (Dir/Console/Null) has empty `data`, so
    /// this returns 0 for them — the caller routes those through their existing
    /// fast paths. Pure + host-tested; the per-fd offset lives in the arch
    /// `FileDesc` and is advanced by the caller.
    pub fn read_at(&self, id: NodeId, off: usize, out: &mut [u8]) -> usize {
        let data = &self.nodes[id as usize].data;
        if off >= data.len() {
            return 0;
        }
        let avail = &data[off..];
        let n = core::cmp::min(avail.len(), out.len());
        out[..n].copy_from_slice(&avail[..n]);
        n
    }

    /// Total byte length of node `id`'s baked `data` (0 for non-`File` nodes).
    /// Used by `fstat`-style size reporting; host-tested.
    pub fn len_of(&self, id: NodeId) -> usize {
        self.nodes[id as usize].data.len()
    }

    /// Record a mount of `source` at `target` (type `fstype`, flags `flags`),
    /// auto-creating the target directory (`mkdir -p`) so the tree stays
    /// walkable for nested targets (`/dev/pts`, `/dev/shm`). Returns `Ok(())` on
    /// success (the arch glue maps it to `0`); `Err(NotDir)` only if a target
    /// component collides with a non-`Dir` node (cannot happen for the pseudo
    /// set against the pre-populated tree). Idempotent: re-mounting the same
    /// target simply appends another record (M2 records duplicates; the 6
    /// pseudo-fs targets are distinct so this never triggers in practice).
    pub fn do_mount(
        &mut self,
        source: &[u8],
        target: &[u8],
        fstype: &[u8],
        flags: u64,
    ) -> Result<(), WalkErr> {
        // Ensure the target dir node exists so subsequent walks succeed.
        self.mkdir_p(target)?;
        self.mounts.push(MountRecord {
            source: string_from_bytes(source),
            target: string_from_bytes(target),
            fstype: string_from_bytes(fstype),
            flags,
        });
        Ok(())
    }
}

impl Default for Vfs {
    fn default() -> Vfs {
        Vfs::new()
    }
}

/// Build a `String` from a static `&str` (helper for `Vfs::new`'s fixed names).
fn str_of(s: &str) -> String {
    String::from(s)
}

/// Build a `String` from raw path bytes. The pseudo-fs names are valid UTF-8
/// (ASCII), but a kernel must never panic on arbitrary user bytes, so we decode
/// lossily into a `String` rather than `from_utf8(...).unwrap()`. (Names are
/// compared by *bytes* in `child_named`, so a lossy round-trip of valid ASCII is
/// exact; only non-UTF-8 user input would differ, which the pseudo set never
/// supplies.)
fn string_from_bytes(b: &[u8]) -> String {
    let mut s = String::with_capacity(b.len());
    for &byte in b {
        if byte < 0x80 {
            s.push(byte as char);
        } else {
            s.push('\u{FFFD}');
        }
    }
    s
}

// ===========================================================================
// Host unit tests (std; run via the out-of-workspace tests-host harness).
//   cargo test --manifest-path crates/arch-aarch64/tests-host/Cargo.toml
// They `include!` this exact file, so they exercise the real production tree.
// ===========================================================================
#[cfg(test)]
mod vfs_tests {
    use super::*;

    /// The pre-populated tree resolves the two devices and the dirs.
    #[test]
    fn walk_prepopulated_tree() {
        let v = Vfs::new();
        assert_eq!(v.walk(b"/"), Ok(ROOT));
        assert_eq!(v.kind(ROOT), NodeKind::Dir);
        let dev = v.walk(b"/dev").expect("/dev exists");
        assert_eq!(v.kind(dev), NodeKind::Dir);
        let console = v.walk(b"/dev/console").expect("/dev/console exists");
        assert_eq!(v.kind(console), NodeKind::Console);
        let null = v.walk(b"/dev/null").expect("/dev/null exists");
        assert_eq!(v.kind(null), NodeKind::Null);
    }

    /// Missing components -> NotFound; descending through a non-dir -> NotDir.
    #[test]
    fn walk_error_classification() {
        let v = Vfs::new();
        assert_eq!(v.walk(b"/nope"), Err(WalkErr::NotFound));
        assert_eq!(v.walk(b"/dev/nope"), Err(WalkErr::NotFound));
        // /dev/console is a Console (not a Dir): descending through it is NotDir.
        assert_eq!(v.walk(b"/dev/console/x"), Err(WalkErr::NotDir));
        assert_eq!(v.walk(b"/dev/null/anything"), Err(WalkErr::NotDir));
    }

    /// Trailing slash, empty/double-slash components are skipped.
    #[test]
    fn walk_trailing_and_empty_components() {
        let v = Vfs::new();
        // Trailing slash on a dir and on a device both resolve.
        assert_eq!(v.walk(b"/dev/"), v.walk(b"/dev"));
        assert_eq!(v.walk(b"/dev/console/"), v.walk(b"/dev/console"));
        // Double slashes collapse.
        assert_eq!(v.walk(b"//dev//null"), v.walk(b"/dev/null"));
        assert_eq!(v.walk(b"///"), Ok(ROOT));
        // Empty path stays at ROOT (the leading split is "").
        assert_eq!(v.walk(b""), Ok(ROOT));
    }

    /// `.` is a no-op; `..` ascends; `..` at root stays at root.
    #[test]
    fn walk_dot_and_dotdot() {
        let v = Vfs::new();
        assert_eq!(v.walk(b"/./dev/./null"), v.walk(b"/dev/null"));
        assert_eq!(v.walk(b"/dev/../dev/null"), v.walk(b"/dev/null"));
        assert_eq!(v.walk(b"/dev/.."), Ok(ROOT));
        // `..` at root is clamped to root (self-loop sentinel).
        assert_eq!(v.walk(b"/.."), Ok(ROOT));
        assert_eq!(v.walk(b"/../../dev/null"), v.walk(b"/dev/null"));
    }

    /// `mkdir_p` creates missing parents, is idempotent, and is then walkable.
    #[test]
    fn mkdir_p_creates_and_is_idempotent() {
        let mut v = Vfs::new();
        let a = v.mkdir_p(b"/system/state").expect("create /system/state");
        assert_eq!(v.kind(a), NodeKind::Dir);
        // The intermediate /system was created too.
        let sys = v.walk(b"/system").expect("/system now exists");
        assert_eq!(v.kind(sys), NodeKind::Dir);
        // Walk finds the leaf.
        assert_eq!(v.walk(b"/system/state"), Ok(a));
        // Second mkdir_p of the same path is idempotent (same id, no dup child).
        let a2 = v.mkdir_p(b"/system/state").expect("idempotent");
        assert_eq!(a2, a);
        // Creating a sibling does not disturb the first.
        let run = v.mkdir_p(b"/system/run").expect("create /system/run");
        assert_ne!(run, a);
        assert_eq!(v.walk(b"/system/state"), Ok(a));
    }

    /// `mkdir_p` through a non-dir is NotDir (cannot turn a device into a dir).
    #[test]
    fn mkdir_p_through_nondir_is_notdir() {
        let mut v = Vfs::new();
        // /dev/console is a Console; making /dev/console/x must fail NotDir.
        assert_eq!(v.mkdir_p(b"/dev/console/x"), Err(WalkErr::NotDir));
        // mkdir_p of an existing device path (final comp non-Dir) also NotDir.
        assert_eq!(v.mkdir_p(b"/dev/console"), Err(WalkErr::NotDir));
    }

    /// `do_mount` records the entry, auto-creates the target dir, and supports
    /// the nested `/dev/pts` + `/dev/shm` targets the live `pseudo_mounts()`
    /// drives.
    #[test]
    fn do_mount_records_and_creates_target() {
        let mut v = Vfs::new();
        assert_eq!(v.mount_count(), 0);
        // The 6 live pseudo_mounts() entries, in order.
        assert_eq!(v.do_mount(b"proc", b"/proc", b"proc", 0), Ok(()));
        assert_eq!(v.do_mount(b"sysfs", b"/sys", b"sysfs", 0), Ok(()));
        assert_eq!(v.do_mount(b"devtmpfs", b"/dev", b"devtmpfs", 0), Ok(()));
        assert_eq!(v.do_mount(b"tmpfs", b"/run", b"tmpfs", 0), Ok(()));
        assert_eq!(v.do_mount(b"devpts", b"/dev/pts", b"devpts", 0), Ok(()));
        assert_eq!(v.do_mount(b"tmpfs", b"/dev/shm", b"tmpfs", 0), Ok(()));
        assert_eq!(v.mount_count(), 6);
        // The targets are all walkable now (including the nested ones).
        assert_eq!(v.kind(v.walk(b"/proc").unwrap()), NodeKind::Dir);
        assert_eq!(v.kind(v.walk(b"/sys").unwrap()), NodeKind::Dir);
        assert_eq!(v.kind(v.walk(b"/run").unwrap()), NodeKind::Dir);
        assert_eq!(v.kind(v.walk(b"/dev/pts").unwrap()), NodeKind::Dir);
        assert_eq!(v.kind(v.walk(b"/dev/shm").unwrap()), NodeKind::Dir);
        // Mounting devtmpfs on the *existing* /dev did not clobber the devices.
        assert_eq!(v.kind(v.walk(b"/dev/console").unwrap()), NodeKind::Console);
        assert_eq!(v.kind(v.walk(b"/dev/null").unwrap()), NodeKind::Null);
    }

    /// `do_mount` is idempotent on a repeated target (records a 2nd entry,
    /// reuses the dir).
    #[test]
    fn do_mount_idempotent_target() {
        let mut v = Vfs::new();
        assert_eq!(v.do_mount(b"tmpfs", b"/run", b"tmpfs", 0), Ok(()));
        let run1 = v.walk(b"/run").unwrap();
        assert_eq!(v.do_mount(b"tmpfs", b"/run", b"tmpfs", 0), Ok(()));
        let run2 = v.walk(b"/run").unwrap();
        assert_eq!(run1, run2, "same dir reused");
        assert_eq!(v.mount_count(), 2, "both records kept");
    }

    /// A flags bitmask is recorded verbatim (informational in M2).
    #[test]
    fn do_mount_carries_flags() {
        let mut v = Vfs::new();
        // MS_NOSUID|MS_NODEV|MS_NOEXEC = 2|4|8 = 14 (illustrative).
        assert_eq!(v.do_mount(b"proc", b"/proc", b"proc", 14), Ok(()));
        assert_eq!(v.mount_count(), 1);
    }

    /// The pre-populated tree exposes the Slice-3 baked synthetic files as
    /// readable `File` nodes (so talos-init's LoadConfig finds them).
    #[test]
    fn prepopulated_baked_files_are_readable_file_nodes() {
        let v = Vfs::new();
        let cfg = v.walk(b"/machine-config.yaml").expect("config exists");
        assert_eq!(v.kind(cfg), NodeKind::File);
        assert_eq!(v.data(cfg), MACHINE_CONFIG_YAML.as_bytes());
        assert!(!v.data(cfg).is_empty(), "config must be non-empty");

        let cmd = v.walk(b"/proc/cmdline").expect("cmdline exists");
        assert_eq!(v.kind(cmd), NodeKind::File);
        assert_eq!(v.data(cmd), PROC_CMDLINE.as_bytes());

        let ver = v.walk(b"/proc/version").expect("version exists");
        assert_eq!(v.kind(ver), NodeKind::File);

        let host = v
            .walk(b"/proc/sys/kernel/hostname")
            .expect("hostname exists");
        assert_eq!(v.kind(host), NodeKind::File);
        // The intermediate dirs were auto-created by mkfile_p.
        assert_eq!(v.kind(v.walk(b"/proc/sys/kernel").unwrap()), NodeKind::Dir);
        assert_eq!(v.kind(v.walk(b"/proc").unwrap()), NodeKind::Dir);
    }

    /// The baked machine config is exactly the bytes the talos parsers accept:
    /// a valid `version:` header and a `machine.type` line.
    #[test]
    fn baked_machine_config_is_schema_shaped() {
        let s = MACHINE_CONFIG_YAML;
        assert!(s.starts_with("version: v1alpha1\n"));
        assert!(s.contains("\n  type: controlplane\n"));
        assert!(s.contains("\n    hostname: kuberos\n"));
        assert!(!s.trim().is_empty());
    }

    /// `mkfile_p` creates a File with data, creating parents, and is walkable.
    #[test]
    fn mkfile_p_creates_file_with_data_and_parents() {
        let mut v = Vfs::new();
        let id = v
            .mkfile_p(b"/etc/conf/app.yaml", b"hello".to_vec())
            .expect("create file");
        assert_eq!(v.kind(id), NodeKind::File);
        assert_eq!(v.data(id), b"hello");
        // Parents are directories now.
        assert_eq!(v.kind(v.walk(b"/etc").unwrap()), NodeKind::Dir);
        assert_eq!(v.kind(v.walk(b"/etc/conf").unwrap()), NodeKind::Dir);
        // Walkable to the same id.
        assert_eq!(v.walk(b"/etc/conf/app.yaml"), Ok(id));
    }

    /// `mkfile_p` on an existing File overwrites its data (same id); on a
    /// non-File final component it errors NotDir (never clobber a dir/device).
    #[test]
    fn mkfile_p_overwrites_file_and_rejects_nonfile() {
        let mut v = Vfs::new();
        let id1 = v.mkfile_p(b"/f", b"v1".to_vec()).unwrap();
        let id2 = v.mkfile_p(b"/f", b"v2-longer".to_vec()).unwrap();
        assert_eq!(id1, id2, "overwrite reuses the node");
        assert_eq!(v.data(id2), b"v2-longer");
        // A directory cannot be replaced by a file.
        assert_eq!(v.mkfile_p(b"/dev", b"x".to_vec()), Err(WalkErr::NotDir));
        // A device cannot be replaced by a file.
        assert_eq!(
            v.mkfile_p(b"/dev/console", b"x".to_vec()),
            Err(WalkErr::NotDir)
        );
        // A parent component that is a device → NotDir.
        assert_eq!(
            v.mkfile_p(b"/dev/null/child", b"x".to_vec()),
            Err(WalkErr::NotDir)
        );
    }

    /// `read_at` copies from `data` at an offset, clamps to EOF, and returns 0
    /// past the end — the contract `sys_read` relies on for per-fd offsetting.
    #[test]
    fn read_at_offsets_and_clamps_to_eof() {
        let mut v = Vfs::new();
        let id = v.mkfile_p(b"/d", b"abcdef".to_vec()).unwrap();

        // Full read from 0.
        let mut buf = [0u8; 16];
        assert_eq!(v.read_at(id, 0, &mut buf), 6);
        assert_eq!(&buf[..6], b"abcdef");

        // Partial read mid-file with a short buffer (simulates fd offset walk).
        let mut small = [0u8; 3];
        assert_eq!(v.read_at(id, 0, &mut small), 3);
        assert_eq!(&small, b"abc");
        assert_eq!(v.read_at(id, 3, &mut small), 3);
        assert_eq!(&small, b"def");
        // At EOF → 0.
        assert_eq!(v.read_at(id, 6, &mut small), 0);
        // Past EOF → 0 (no panic).
        assert_eq!(v.read_at(id, 100, &mut small), 0);

        // len_of reports the size.
        assert_eq!(v.len_of(id), 6);
    }

    /// Reading the baked `/machine-config.yaml` via offset chunks reconstructs
    /// the whole file (the streaming pattern `sys_read` uses).
    #[test]
    fn read_at_streams_whole_baked_config() {
        let v = Vfs::new();
        let id = v.walk(b"/machine-config.yaml").unwrap();
        let want = MACHINE_CONFIG_YAML.as_bytes();
        let mut got = alloc::vec::Vec::new();
        let mut off = 0usize;
        let mut chunk = [0u8; 7]; // deliberately not a divisor of the length
        loop {
            let n = v.read_at(id, off, &mut chunk);
            if n == 0 {
                break;
            }
            got.extend_from_slice(&chunk[..n]);
            off += n;
        }
        assert_eq!(got, want);
    }

    /// A non-File node (Dir/Console/Null) reads as empty (0 bytes) so the
    /// arch read path falls back to the Console/Null fast paths unchanged.
    #[test]
    fn read_at_on_nonfile_is_empty() {
        let v = Vfs::new();
        let console = v.walk(b"/dev/console").unwrap();
        let dev = v.walk(b"/dev").unwrap();
        let mut buf = [0u8; 8];
        assert_eq!(v.read_at(console, 0, &mut buf), 0);
        assert_eq!(v.read_at(dev, 0, &mut buf), 0);
        assert_eq!(v.len_of(console), 0);
    }

    /// The slab is stable: ids handed out by `walk`/`mkdir_p` stay valid as the
    /// tree grows (no `NodeId` reuse, no shrink).
    #[test]
    fn node_ids_are_stable_across_growth() {
        let mut v = Vfs::new();
        let console = v.walk(b"/dev/console").unwrap();
        // Grow the tree a lot.
        for i in 0..50u32 {
            let mut p = alloc::vec::Vec::new();
            p.extend_from_slice(b"/grow/");
            // a unique single-component path each time
            p.push(b'a' + (i % 26) as u8);
            p.extend_from_slice(b"/leaf");
            let _ = v.mkdir_p(&p);
        }
        // The original device id still resolves to the same node.
        assert_eq!(v.walk(b"/dev/console"), Ok(console));
        assert_eq!(v.kind(console), NodeKind::Console);
    }
}
