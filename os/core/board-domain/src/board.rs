//! Board definitions and the in-memory board registry.
//!
//! Mirrors Talos's historical `internal/pkg/board` registry (now folded into
//! the siderolabs *overlays*): each single-board computer (SBC) has a stable
//! board ID, a human name, the kernel command-line arguments it requires, the
//! U-Boot/firmware blobs that must be flashed to fixed disk offsets, and the
//! device-tree (DTB) and partition quirks that the installer must honour.

use crate::dtb::DeviceTree;
use crate::uboot::UBootImage;
use os_kernel::error::{Error, Result};

/// Stable board identifiers.
///
/// These match the IDs Talos uses on the kernel command line as
/// `talos.board=<id>` and in `pkg/machinery/constants` (e.g. `rpi_generic`,
/// `jetson_nano`, `rock64`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoardId {
    /// Raspberry Pi (generic / CM4 / Pi 4 family).
    RpiGeneric,
    /// Raspberry Pi 4 (4 GB+) specific tuning.
    Rpi4,
    /// NVIDIA Jetson Nano.
    JetsonNano,
    /// Pine64 Rock64.
    Rock64,
    /// BananaPi M64.
    BananaPiM64,
    /// Libre Computer "Le Potato" (AML-S905X-CC).
    LibretechAllH3CcH5,
    /// NanoPi R4S.
    NanoPiR4S,
}

impl BoardId {
    /// Parse the on-the-wire board id (`talos.board=<id>`).
    pub fn parse(s: &str) -> Result<BoardId> {
        Ok(match s {
            "rpi_generic" => BoardId::RpiGeneric,
            "rpi_4" => BoardId::Rpi4,
            "jetson_nano" => BoardId::JetsonNano,
            "rock64" => BoardId::Rock64,
            "bananapi_m64" => BoardId::BananaPiM64,
            "libretech_all_h3_cc_h5" => BoardId::LibretechAllH3CcH5,
            "nanopi_r4s" => BoardId::NanoPiR4S,
            other => return Err(Error::not_found(format!("unknown board id: {other}"))),
        })
    }

    /// The canonical on-the-wire id string.
    pub fn as_str(self) -> &'static str {
        match self {
            BoardId::RpiGeneric => "rpi_generic",
            BoardId::Rpi4 => "rpi_4",
            BoardId::JetsonNano => "jetson_nano",
            BoardId::Rock64 => "rock64",
            BoardId::BananaPiM64 => "bananapi_m64",
            BoardId::LibretechAllH3CcH5 => "libretech_all_h3_cc_h5",
            BoardId::NanoPiR4S => "nanopi_r4s",
        }
    }

    /// CPU architecture the board ships. Talos only builds arm64 SBC images.
    pub fn arch(self) -> Arch {
        Arch::Arm64
    }
}

/// CPU architecture of a board image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    /// 64-bit ARM (aarch64) — every supported SBC.
    Arm64,
    /// 64-bit x86, here only for completeness/validation.
    Amd64,
}

impl Arch {
    /// The Linux/Talos arch token (`arm64`, `amd64`).
    pub fn as_str(self) -> &'static str {
        match self {
            Arch::Arm64 => "arm64",
            Arch::Amd64 => "amd64",
        }
    }
}

/// How the firmware/U-Boot blob is written relative to the start of the disk.
///
/// SBCs boot from raw-sector firmware that lives *before* the GPT partitions,
/// so the installer dd's the blob to a fixed byte offset rather than into a
/// filesystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirmwareWrite {
    /// Byte offset from the start of the disk to write the blob to.
    pub offset: u64,
    /// The U-Boot/firmware image to write there.
    pub image: UBootImage,
}

/// A complete board definition: everything the installer needs to make a disk
/// bootable on this specific SBC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    /// Stable identifier.
    pub id: BoardId,
    /// Human-readable name shown in logs.
    pub name: String,
    /// Kernel command-line args this board always needs (e.g. serial console).
    pub kernel_args: Vec<String>,
    /// Firmware/U-Boot blobs to flash to raw disk offsets, in write order.
    pub firmware: Vec<FirmwareWrite>,
    /// Device tree this board boots with.
    pub dtb: DeviceTree,
    /// Partition layout quirks.
    pub partition: PartitionQuirks,
}

/// Board-specific partition quirks the installer must respect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartitionQuirks {
    /// The disk sector at which the first GPT partition may begin. Boards that
    /// reserve raw space for firmware push this out (e.g. Rock64 reserves the
    /// first 16 MiB).
    pub first_partition_sector: u64,
    /// Whether a dedicated FAT firmware/boot partition is required (Raspberry
    /// Pi reads `config.txt`/`*.dtb` from a FAT partition).
    pub needs_boot_partition: bool,
}

impl PartitionQuirks {
    /// The reserved prefix in bytes assuming 512-byte sectors.
    pub fn reserved_bytes(&self) -> u64 {
        self.first_partition_sector * 512
    }
}

impl Board {
    /// Validate internal consistency of a board definition. Used both by the
    /// built-in registry (as a self-check) and by externally supplied
    /// overlays.
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::invalid("board name must not be empty"));
        }
        // Firmware writes must not overlap.
        let mut writes: Vec<&FirmwareWrite> = self.firmware.iter().collect();
        writes.sort_by_key(|w| w.offset);
        for pair in writes.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let a_end = a.offset + a.image.len() as u64;
            if a_end > b.offset {
                return Err(Error::invalid(format!(
                    "firmware writes overlap: [{}, {}) and [{}, ..)",
                    a.offset, a_end, b.offset
                )));
            }
        }
        // Firmware must fit before the first partition.
        let reserved = self.partition.reserved_bytes();
        for w in &self.firmware {
            let end = w.offset + w.image.len() as u64;
            if end > reserved && reserved != 0 {
                return Err(Error::invalid(format!(
                    "firmware at offset {} ({} bytes) overruns reserved region of {} bytes",
                    w.offset,
                    w.image.len(),
                    reserved
                )));
            }
        }
        self.dtb.validate()?;
        Ok(())
    }

    /// The full kernel command line for this board, combining board args with
    /// any caller-supplied extra args, de-duplicated and stable-ordered.
    pub fn cmdline(&self, extra: &[&str]) -> String {
        let mut out: Vec<String> = Vec::new();
        for a in self
            .kernel_args
            .iter()
            .map(|s| s.as_str())
            .chain(extra.iter().copied())
        {
            let a = a.trim();
            if !a.is_empty() && !out.iter().any(|e| e == a) {
                out.push(a.to_string());
            }
        }
        out.join(" ")
    }
}

/// In-memory registry of the boards Talos ships, keyed by [`BoardId`].
#[derive(Debug, Clone, Default)]
pub struct BoardRegistry {
    boards: Vec<Board>,
}

impl BoardRegistry {
    /// An empty registry; use [`BoardRegistry::with_builtins`] for the real set.
    pub fn new() -> BoardRegistry {
        BoardRegistry { boards: Vec::new() }
    }

    /// Registry pre-populated with all built-in board definitions.
    pub fn with_builtins() -> BoardRegistry {
        let mut r = BoardRegistry::new();
        for b in builtin_boards() {
            // Built-ins are validated; a panic here is a programming error.
            r.register(b).expect("built-in board is valid");
        }
        r
    }

    /// Register (or replace) a board after validating it.
    pub fn register(&mut self, board: Board) -> Result<()> {
        board.validate()?;
        if let Some(slot) = self.boards.iter_mut().find(|b| b.id == board.id) {
            *slot = board;
        } else {
            self.boards.push(board);
        }
        Ok(())
    }

    /// Look up a board by id.
    pub fn get(&self, id: BoardId) -> Result<&Board> {
        self.boards
            .iter()
            .find(|b| b.id == id)
            .ok_or_else(|| Error::not_found(format!("board not registered: {}", id.as_str())))
    }

    /// Look up by on-the-wire id string (`talos.board=`).
    pub fn get_by_name(&self, name: &str) -> Result<&Board> {
        self.get(BoardId::parse(name)?)
    }

    /// Number of registered boards.
    pub fn len(&self) -> usize {
        self.boards.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.boards.is_empty()
    }

    /// Iterate over registered boards.
    pub fn iter(&self) -> impl Iterator<Item = &Board> {
        self.boards.iter()
    }
}

/// The built-in board definitions Talos ships for arm64 SBCs.
pub fn builtin_boards() -> Vec<Board> {
    use crate::dtb::DeviceTree;
    use crate::uboot::UBootImage;

    vec![
        Board {
            id: BoardId::RpiGeneric,
            name: "Raspberry Pi (generic)".to_string(),
            kernel_args: vec![
                "console=tty0".to_string(),
                "console=ttyAMA0,115200".to_string(),
                "sun50i_a64.power_off=1".to_string(),
            ],
            // Pi loads firmware from the FAT boot partition, not raw sectors.
            firmware: vec![],
            dtb: DeviceTree::new("bcm2711-rpi-4-b.dtb", b"\xd0\x0d\xfe\xed---rpi"),
            partition: PartitionQuirks {
                first_partition_sector: 2048,
                needs_boot_partition: true,
            },
        },
        Board {
            id: BoardId::Rpi4,
            name: "Raspberry Pi 4 Model B".to_string(),
            kernel_args: vec![
                "console=tty0".to_string(),
                "console=ttyAMA0,115200".to_string(),
            ],
            firmware: vec![],
            dtb: DeviceTree::new("bcm2711-rpi-4-b.dtb", b"\xd0\x0d\xfe\xed---rpi4"),
            partition: PartitionQuirks {
                first_partition_sector: 2048,
                needs_boot_partition: true,
            },
        },
        Board {
            id: BoardId::JetsonNano,
            name: "NVIDIA Jetson Nano".to_string(),
            kernel_args: vec![
                "console=tty0".to_string(),
                "console=ttyS0,115200n8".to_string(),
            ],
            firmware: vec![FirmwareWrite {
                offset: 0,
                image: UBootImage::new("jetson-nano-spl.bin", b"\xd0\x0d\xfe\xedjetson"),
            }],
            dtb: DeviceTree::new("tegra210-p3450-0000.dtb", b"\xd0\x0d\xfe\xedjetson"),
            partition: PartitionQuirks {
                first_partition_sector: 8192,
                needs_boot_partition: false,
            },
        },
        Board {
            id: BoardId::Rock64,
            name: "Pine64 Rock64".to_string(),
            kernel_args: vec!["console=ttyS2,1500000n8".to_string()],
            firmware: vec![FirmwareWrite {
                // Rockchip idbloader at sector 64.
                offset: 64 * 512,
                image: UBootImage::new("idbloader.img", b"\xd0\x0d\xfe\xedrock64idb"),
            }],
            dtb: DeviceTree::new("rk3328-rock64.dtb", b"\xd0\x0d\xfe\xedrock64"),
            partition: PartitionQuirks {
                first_partition_sector: 32768,
                needs_boot_partition: false,
            },
        },
        Board {
            id: BoardId::BananaPiM64,
            name: "BananaPi M64".to_string(),
            kernel_args: vec!["console=ttyS0,115200".to_string()],
            firmware: vec![FirmwareWrite {
                // Allwinner U-Boot at 8 KiB.
                offset: 8192,
                image: UBootImage::new("u-boot-sunxi-with-spl.bin", b"\xd0\x0d\xfe\xedbpim64"),
            }],
            dtb: DeviceTree::new("sun50i-a64-bananapi-m64.dtb", b"\xd0\x0d\xfe\xedbpim64"),
            partition: PartitionQuirks {
                first_partition_sector: 2048,
                needs_boot_partition: false,
            },
        },
        Board {
            id: BoardId::LibretechAllH3CcH5,
            name: "Libre Computer ALL-H3-CC (H5)".to_string(),
            kernel_args: vec!["console=ttyS0,115200".to_string()],
            firmware: vec![FirmwareWrite {
                offset: 8192,
                image: UBootImage::new("u-boot-sunxi-with-spl.bin", b"\xd0\x0d\xfe\xedlepotato"),
            }],
            dtb: DeviceTree::new("sun50i-h5-libretech-all-h3-cc.dtb", b"\xd0\x0d\xfe\xedh5"),
            partition: PartitionQuirks {
                first_partition_sector: 2048,
                needs_boot_partition: false,
            },
        },
        Board {
            id: BoardId::NanoPiR4S,
            name: "FriendlyElec NanoPi R4S".to_string(),
            kernel_args: vec!["console=ttyS2,1500000n8".to_string()],
            firmware: vec![FirmwareWrite {
                offset: 64 * 512,
                image: UBootImage::new("idbloader.img", b"\xd0\x0d\xfe\xednanopir4s"),
            }],
            dtb: DeviceTree::new("rk3399-nanopi-r4s.dtb", b"\xd0\x0d\xfe\xednanopir4s"),
            partition: PartitionQuirks {
                first_partition_sector: 32768,
                needs_boot_partition: false,
            },
        },
    ]
}
