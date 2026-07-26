//! # talos-archiver
//!
//! A faithful Rust model of Talos's `pkg/archiver` plus the `pkg/tar` / `pkg/xz`
//! helper packages used throughout `siderolabs/talos`.
//!
//! The archiver is the engine behind `talosctl copy` (stream a directory tree
//! off a machine as a tar archive), the machine `Copy`/`List` gRPC services,
//! and image unpacking during installs (extracting a squashed rootfs or an
//! initramfs `cpio` archive). This crate models all of those pieces:
//!
//! * [`walker`] — recursive filesystem walking with include/exclude filters,
//!   max-depth, symlink and special-file handling, mirroring
//!   `pkg/archiver/walker.go`.
//! * [`tar`] — a real (USTAR/PAX-style) tar archive writer and reader operating
//!   over an in-memory byte stream, used by `talosctl copy` and image unpack.
//! * [`cpio`] — the SVR4 "newc" cpio format used for Linux initramfs images.
//! * [`compression`] — decompression abstractions (gzip / xz / zstd) modeled as
//!   a [`compression::Decompressor`] trait with detectable magic numbers, plus a
//!   pass-through identity codec used in tests.
//!
//! Everything is driven through the in-memory [`walker::FileTree`] so the full
//! create -> compress -> extract round trip is exercised offline by the unit
//! tests, with no real disk or external crates involved.

pub mod compression;
pub mod cpio;
pub mod tar;
pub mod walker;

pub use compression::{Codec, Decompressor, GzipCodec, IdentityCodec, XzCodec, ZstdCodec};
pub use cpio::{CpioArchive, CpioEntry, CpioReader, CpioWriter};
pub use tar::{EntryType, TarArchive, TarEntry, TarHeader, TarReader, TarWriter};
pub use walker::{FileKind, FileTree, WalkEntry, WalkOptions, Walker};

/// Result alias used across the archiver, re-exporting the workspace error type.
pub type Result<T> = core::result::Result<T, os_kernel::Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_to_end_walk_tar_roundtrip() {
        // Build a small tree, walk it, write a tar, read it back.
        let mut tree = FileTree::new();
        tree.add_dir("/etc", 0o755);
        tree.add_file("/etc/hostname", b"node-1\n", 0o644);
        tree.add_file("/etc/hosts", b"127.0.0.1 localhost\n", 0o644);
        tree.add_symlink("/etc/resolv.conf", "/run/resolv.conf", 0o777);

        let walker = Walker::new(WalkOptions::default());
        let entries = walker.walk(&tree, "/etc").expect("walk");
        // dir + 2 files + symlink
        assert_eq!(entries.len(), 4);

        let mut writer = TarWriter::new();
        for e in &entries {
            writer.add_walk_entry(&tree, e).expect("add");
        }
        let bytes = writer.finish();

        let archive = TarReader::read(&bytes).expect("read tar");
        let names: Vec<&str> = archive
            .entries
            .iter()
            .map(|e| e.header.name.as_str())
            .collect();
        assert!(names.iter().any(|n| n.ends_with("hostname")));
        let hostname = archive
            .entries
            .iter()
            .find(|e| e.header.name.ends_with("hostname"))
            .unwrap();
        assert_eq!(hostname.data, b"node-1\n");
    }

    #[test]
    fn compression_magic_detection_roundtrip() {
        // identity codec wraps payload; detect picks it up.
        let payload = b"the quick brown fox".to_vec();
        let codec = IdentityCodec;
        let blob = codec.compress(&payload);
        assert_eq!(Codec::detect(&blob), Some(Codec::Identity));
        let out = compression::decompress_auto(&blob).expect("decompress");
        assert_eq!(out, payload);
    }
}
