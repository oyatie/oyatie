//! The siderolabs *overlay* installer interface and board-specific install
//! hooks.
//!
//! Modern Talos has moved per-board logic out of `machined` into pluggable
//! "overlays" that expose a small contract: report the board's kernel args and
//! partition options, and run an `Install` step that flashes firmware / writes
//! the DTB. This module models that contract as a trait plus a built-in
//! implementation driven by the [`Board`](crate::board::Board) registry, and an
//! in-memory boot-partition target so the whole flow is testable offline.

use crate::board::{Board, BoardId, PartitionQuirks};
use crate::uboot::{RawDisk, flash_image};
use os_kernel::error::{Error, Result};

/// Options passed by the installer to an overlay's `Install` step. Mirrors the
/// overlay `InstallOptions` struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallOptions {
    /// Target block device path (e.g. `/dev/mmcblk0`). Informational here.
    pub disk: String,
    /// Architecture being installed.
    pub arch: String,
    /// Extra kernel args the machine config requested.
    pub extra_kernel_args: Vec<String>,
}

impl InstallOptions {
    /// Convenience constructor.
    pub fn new(disk: &str, arch: &str) -> InstallOptions {
        InstallOptions {
            disk: disk.to_string(),
            arch: arch.to_string(),
            extra_kernel_args: Vec::new(),
        }
    }
}

/// A file an overlay wants copied onto the FAT boot partition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootFile {
    /// Destination filename on the boot partition.
    pub name: String,
    /// File contents.
    pub data: Vec<u8>,
}

/// The boot-partition target an overlay writes DTBs / config to. Real installs
/// mount a FAT partition; tests use [`InMemoryBootPartition`].
pub trait BootPartition {
    /// Write (or overwrite) a file on the boot partition.
    fn put(&mut self, name: &str, data: &[u8]) -> Result<()>;
    /// Read a file back, if present.
    fn get(&self, name: &str) -> Option<Vec<u8>>;
}

/// In-memory FAT boot partition used by tests.
#[derive(Debug, Clone, Default)]
pub struct InMemoryBootPartition {
    files: Vec<BootFile>,
}

impl InMemoryBootPartition {
    /// An empty boot partition.
    pub fn new() -> InMemoryBootPartition {
        InMemoryBootPartition::default()
    }

    /// Number of files written.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Whether nothing has been written.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

impl BootPartition for InMemoryBootPartition {
    fn put(&mut self, name: &str, data: &[u8]) -> Result<()> {
        if name.trim().is_empty() {
            return Err(Error::invalid("boot file name must not be empty"));
        }
        if let Some(f) = self.files.iter_mut().find(|f| f.name == name) {
            f.data = data.to_vec();
        } else {
            self.files.push(BootFile {
                name: name.to_string(),
                data: data.to_vec(),
            });
        }
        Ok(())
    }

    fn get(&self, name: &str) -> Option<Vec<u8>> {
        self.files
            .iter()
            .find(|f| f.name == name)
            .map(|f| f.data.clone())
    }
}

/// The overlay contract. An overlay knows how to make a disk bootable for one
/// board family.
pub trait Overlay {
    /// The board this overlay handles.
    fn board_id(&self) -> BoardId;

    /// Kernel args this overlay contributes, given install options.
    fn kernel_args(&self, opts: &InstallOptions) -> Vec<String>;

    /// Partition quirks this overlay imposes.
    fn partition_options(&self) -> PartitionQuirks;

    /// Run the install hook: flash firmware to the raw disk and write DTBs to
    /// the boot partition. Returns the number of bytes flashed.
    fn install(
        &self,
        opts: &InstallOptions,
        disk: &mut dyn RawDisk,
        boot: &mut dyn BootPartition,
    ) -> Result<InstallReport>;
}

/// Summary of what an install hook did.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InstallReport {
    /// Total firmware bytes flashed to raw sectors.
    pub firmware_bytes: u64,
    /// Number of files written to the boot partition.
    pub boot_files: usize,
    /// The full kernel command line produced.
    pub cmdline: String,
}

/// The built-in overlay driven entirely by a [`Board`] definition. This is what
/// the in-tree boards use; external overlays implement [`Overlay`] directly.
#[derive(Debug, Clone)]
pub struct BoardOverlay {
    board: Board,
}

impl BoardOverlay {
    /// Wrap a validated board definition as an overlay.
    pub fn new(board: Board) -> Result<BoardOverlay> {
        board.validate()?;
        Ok(BoardOverlay { board })
    }

    /// Borrow the underlying board.
    pub fn board(&self) -> &Board {
        &self.board
    }
}

impl Overlay for BoardOverlay {
    fn board_id(&self) -> BoardId {
        self.board.id
    }

    fn kernel_args(&self, opts: &InstallOptions) -> Vec<String> {
        let mut args = self.board.kernel_args.clone();
        for e in &opts.extra_kernel_args {
            let e = e.trim();
            if !e.is_empty() && !args.iter().any(|a| a == e) {
                args.push(e.to_string());
            }
        }
        args
    }

    fn partition_options(&self) -> PartitionQuirks {
        self.board.partition.clone()
    }

    fn install(
        &self,
        opts: &InstallOptions,
        disk: &mut dyn RawDisk,
        boot: &mut dyn BootPartition,
    ) -> Result<InstallReport> {
        // Architecture guard: SBC images are arm64.
        let want = self.board.id.arch().as_str();
        if opts.arch != want {
            return Err(Error::unsupported(format!(
                "board {} requires arch {want}, got {}",
                self.board.id.as_str(),
                opts.arch
            )));
        }

        // 1. Flash firmware blobs to their raw offsets.
        let mut firmware_bytes = 0u64;
        for fw in &self.board.firmware {
            flash_image(disk, fw.offset, &fw.image)?;
            firmware_bytes += fw.image.len() as u64;
        }

        // 2. If the board boots from a FAT partition, write the DTB + overlays.
        let mut boot_files = 0usize;
        if self.board.partition.needs_boot_partition || !self.board.dtb.overlays.is_empty() {
            for (name, data) in self.board.dtb.boot_files() {
                boot.put(&name, data)?;
                boot_files += 1;
            }
        } else {
            // Even non-FAT boards keep the base DTB available for the kernel.
            let (name, data) = (
                self.board.dtb.filename.clone(),
                self.board.dtb.data.as_slice(),
            );
            boot.put(&name, data)?;
            boot_files += 1;
        }

        let extra: Vec<&str> = opts.extra_kernel_args.iter().map(|s| s.as_str()).collect();
        let cmdline = self.board.cmdline(&extra);

        Ok(InstallReport {
            firmware_bytes,
            boot_files,
            cmdline,
        })
    }
}

/// A registry of overlays keyed by board, used by the installer to dispatch the
/// right install hook for `talos.board=<id>`.
#[derive(Default)]
pub struct OverlayRegistry {
    overlays: Vec<Box<dyn Overlay>>,
}

impl OverlayRegistry {
    /// An empty overlay registry.
    pub fn new() -> OverlayRegistry {
        OverlayRegistry {
            overlays: Vec::new(),
        }
    }

    /// Register (or replace) an overlay for its board.
    pub fn register(&mut self, overlay: Box<dyn Overlay>) {
        let id = overlay.board_id();
        if let Some(slot) = self.overlays.iter_mut().find(|o| o.board_id() == id) {
            *slot = overlay;
        } else {
            self.overlays.push(overlay);
        }
    }

    /// Look up the overlay for a board.
    pub fn get(&self, id: BoardId) -> Result<&dyn Overlay> {
        self.overlays
            .iter()
            .find(|o| o.board_id() == id)
            .map(|b| b.as_ref())
            .ok_or_else(|| Error::not_found(format!("no overlay for board {}", id.as_str())))
    }

    /// Number of registered overlays.
    pub fn len(&self) -> usize {
        self.overlays.len()
    }

    /// Whether empty.
    pub fn is_empty(&self) -> bool {
        self.overlays.is_empty()
    }

    /// Build a registry from the built-in board set.
    pub fn with_builtins() -> Result<OverlayRegistry> {
        let mut r = OverlayRegistry::new();
        for b in crate::board::builtin_boards() {
            r.register(Box::new(BoardOverlay::new(b)?));
        }
        Ok(r)
    }
}
