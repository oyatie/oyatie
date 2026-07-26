//! # talos-board
//!
//! Single-board-computer (SBC) and *overlay* support for the operating-system Talos
//! port. This is the Rust model of what used to live in Talos's
//! `internal/app/machined/.../board` registry and now lives in the siderolabs
//! *overlays*:
//!
//! * [`board`] — stable [`BoardId`](board::BoardId)s (Raspberry Pi, Jetson,
//!   Rock64, …), full [`Board`](board::Board) definitions (kernel args,
//!   firmware writes, DTB, partition quirks) and an in-memory
//!   [`BoardRegistry`](board::BoardRegistry) of the built-ins.
//! * [`uboot`] — U-Boot/firmware blobs and the raw-sector flashing boundary
//!   ([`RawDisk`](uboot::RawDisk) + [`InMemoryDisk`](uboot::InMemoryDisk)).
//! * [`dtb`] — device-tree blobs and `.dtbo` overlays copied to the boot
//!   partition.
//! * [`overlay`] — the pluggable [`Overlay`](overlay::Overlay) installer
//!   contract, a built-in [`BoardOverlay`](overlay::BoardOverlay) driven by a
//!   board definition, and an [`InMemoryBootPartition`](overlay::InMemoryBootPartition)
//!   so the whole install hook runs offline in tests.
//!
//! Every OS boundary (disk, boot partition) is a trait with an in-memory
//! implementation; the crate has no external dependencies beyond `talos-core`.

pub mod board;
pub mod dtb;
pub mod overlay;
pub mod uboot;

pub use board::{Arch, Board, BoardId, BoardRegistry, FirmwareWrite, PartitionQuirks};
pub use dtb::{DeviceTree, DtbOverlay};
pub use overlay::{
    BoardOverlay, BootFile, BootPartition, InMemoryBootPartition, InstallOptions, InstallReport,
    Overlay, OverlayRegistry,
};
pub use uboot::{InMemoryDisk, RawDisk, UBootImage, flash_image};

#[cfg(test)]
mod tests {
    use super::*;
    use os_kernel::error::Error;

    #[test]
    fn board_id_roundtrips() {
        for id in [
            BoardId::RpiGeneric,
            BoardId::Rpi4,
            BoardId::JetsonNano,
            BoardId::Rock64,
            BoardId::BananaPiM64,
            BoardId::LibretechAllH3CcH5,
            BoardId::NanoPiR4S,
        ] {
            assert_eq!(BoardId::parse(id.as_str()).unwrap(), id);
            assert_eq!(id.arch(), Arch::Arm64);
        }
        assert!(matches!(BoardId::parse("nope"), Err(Error::NotFound(_))));
    }

    #[test]
    fn registry_has_all_builtins() {
        let r = BoardRegistry::with_builtins();
        assert_eq!(r.len(), 7);
        assert!(!r.is_empty());
        for b in r.iter() {
            b.validate().unwrap();
        }
        assert_eq!(r.get(BoardId::Rock64).unwrap().name, "Pine64 Rock64");
        assert_eq!(r.get_by_name("rpi_4").unwrap().id, BoardId::Rpi4);
    }

    #[test]
    fn registry_get_missing_is_not_found() {
        let r = BoardRegistry::new();
        assert!(matches!(r.get(BoardId::Rpi4), Err(Error::NotFound(_))));
    }

    #[test]
    fn register_replaces_same_id() {
        let mut r = BoardRegistry::new();
        let mut b = board::builtin_boards()
            .into_iter()
            .find(|b| b.id == BoardId::Rock64)
            .unwrap();
        r.register(b.clone()).unwrap();
        b.name = "Renamed".to_string();
        r.register(b).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r.get(BoardId::Rock64).unwrap().name, "Renamed");
    }

    #[test]
    fn cmdline_dedups_and_appends_extra() {
        let r = BoardRegistry::with_builtins();
        let b = r.get(BoardId::RpiGeneric).unwrap();
        let cl = b.cmdline(&["console=tty0", "talos.platform=metal"]);
        // "console=tty0" already present in board args -> not duplicated.
        assert_eq!(cl.matches("console=tty0").count(), 1);
        assert!(cl.contains("talos.platform=metal"));
    }

    #[test]
    fn validate_rejects_overlapping_firmware() {
        let mut b = board::builtin_boards()
            .into_iter()
            .find(|b| b.id == BoardId::Rock64)
            .unwrap();
        // Add a second firmware write that overlaps the first.
        let first = b.firmware[0].clone();
        b.firmware.push(FirmwareWrite {
            offset: first.offset + 1,
            image: first.image.clone(),
        });
        assert!(matches!(b.validate(), Err(Error::Invalid(_))));
    }

    #[test]
    fn validate_rejects_firmware_overrunning_reserved() {
        let mut b = board::builtin_boards()
            .into_iter()
            .find(|b| b.id == BoardId::BananaPiM64)
            .unwrap();
        // Reserved region is first_partition_sector*512 = 2048*512. Push a huge
        // blob past it.
        let big = vec![0xAB; (b.partition.reserved_bytes() + 16) as usize];
        b.firmware[0].image = UBootImage::new("big.bin", &big);
        assert!(matches!(b.validate(), Err(Error::Invalid(_))));
    }

    #[test]
    fn in_memory_disk_bounds_checked() {
        let mut d = InMemoryDisk::new(64);
        assert_eq!(d.size(), 64);
        d.write_at(0, &[1, 2, 3]).unwrap();
        assert_eq!(d.read_at(0, 3).unwrap(), vec![1, 2, 3]);
        assert!(d.write_at(62, &[1, 2, 3, 4]).is_err());
        assert!(d.read_at(60, 10).is_err());
    }

    #[test]
    fn flash_image_writes_and_verifies() {
        let mut d = InMemoryDisk::new(1024);
        let img = UBootImage::new("u-boot.bin", b"\xd0\x0d\xfe\xedHELLO");
        flash_image(&mut d, 512, &img).unwrap();
        assert_eq!(d.read_at(512, img.len()).unwrap(), img.data);
        assert!(img.has_fdt_magic());
    }

    #[test]
    fn flash_empty_image_rejected() {
        let mut d = InMemoryDisk::new(16);
        let img = UBootImage::new("empty.bin", b"");
        assert!(flash_image(&mut d, 0, &img).is_err());
    }

    #[test]
    fn dtb_validation_and_overlays() {
        let mut dt = DeviceTree::new("rk3328-rock64.dtb", b"\xd0\x0d\xfe\xedxyz");
        assert!(dt.has_magic());
        dt.validate().unwrap();
        dt.add_overlay("disable-bt.dtbo", b"ovl").unwrap();
        assert_eq!(dt.boot_files().len(), 2);
        assert!(dt.add_overlay("bad.txt", b"x").is_err());

        let bad = DeviceTree::new("no-magic.dtb", b"junk");
        assert!(matches!(bad.validate(), Err(Error::Parse(_))));

        let bad_ext = DeviceTree::new("file.bin", b"\xd0\x0d\xfe\xed");
        assert!(matches!(bad_ext.validate(), Err(Error::Invalid(_))));
    }

    #[test]
    fn overlay_install_flashes_firmware_and_writes_dtb() {
        let board = board::builtin_boards()
            .into_iter()
            .find(|b| b.id == BoardId::Rock64)
            .unwrap();
        let ov = BoardOverlay::new(board.clone()).unwrap();
        let mut disk = InMemoryDisk::new(64 * 1024 * 1024);
        let mut boot = InMemoryBootPartition::new();
        let opts = InstallOptions::new("/dev/mmcblk0", "arm64");

        let report = ov.install(&opts, &mut disk, &mut boot).unwrap();
        assert!(report.firmware_bytes > 0);
        assert_eq!(report.boot_files, 1);
        assert!(report.cmdline.contains("ttyS2"));

        // The firmware really landed at the board's offset.
        let fw = &board.firmware[0];
        assert_eq!(
            disk.read_at(fw.offset, fw.image.len()).unwrap(),
            fw.image.data
        );
        // The DTB really landed on the boot partition.
        assert!(boot.get("rk3328-rock64.dtb").is_some());
    }

    #[test]
    fn overlay_install_rejects_wrong_arch() {
        let board = board::builtin_boards()
            .into_iter()
            .find(|b| b.id == BoardId::JetsonNano)
            .unwrap();
        let ov = BoardOverlay::new(board).unwrap();
        let mut disk = InMemoryDisk::new(1024 * 1024);
        let mut boot = InMemoryBootPartition::new();
        let opts = InstallOptions::new("/dev/mmcblk0", "amd64");
        assert!(matches!(
            ov.install(&opts, &mut disk, &mut boot),
            Err(Error::Unsupported(_))
        ));
    }

    #[test]
    fn overlay_kernel_args_merge_extra() {
        let board = board::builtin_boards()
            .into_iter()
            .find(|b| b.id == BoardId::Rpi4)
            .unwrap();
        let ov = BoardOverlay::new(board).unwrap();
        let mut opts = InstallOptions::new("/dev/mmcblk0", "arm64");
        opts.extra_kernel_args = vec!["console=tty0".into(), "quiet".into()];
        let args = ov.kernel_args(&opts);
        // existing "console=tty0" not duplicated, "quiet" appended.
        assert_eq!(args.iter().filter(|a| *a == "console=tty0").count(), 1);
        assert!(args.iter().any(|a| a == "quiet"));
    }

    #[test]
    fn overlay_registry_dispatch() {
        let reg = OverlayRegistry::with_builtins().unwrap();
        assert_eq!(reg.len(), 7);
        assert!(!reg.is_empty());
        let ov = reg.get(BoardId::NanoPiR4S).unwrap();
        assert_eq!(ov.board_id(), BoardId::NanoPiR4S);
        assert_eq!(ov.partition_options().first_partition_sector, 32768);
    }

    #[test]
    fn rpi_boot_partition_required() {
        let r = BoardRegistry::with_builtins();
        assert!(
            r.get(BoardId::RpiGeneric)
                .unwrap()
                .partition
                .needs_boot_partition
        );
        assert!(
            !r.get(BoardId::Rock64)
                .unwrap()
                .partition
                .needs_boot_partition
        );
    }
}
