//! SVR4 "newc" cpio archives — the format Linux initramfs images use.
//!
//! Talos builds its initramfs as one or more concatenated newc cpio archives
//! (the kernel's `gen_init_cpio` format), optionally compressed. The installer
//! and imager unpack these. This module is a real encoder/decoder for the newc
//! format: each entry is a fixed ASCII-hex header (`070701` magic) followed by
//! a NUL-terminated, 4-byte-aligned name and 4-byte-aligned data, terminated by
//! the special `TRAILER!!!` entry.

use os_kernel::Error;

/// The newc magic string.
pub const NEWC_MAGIC: &[u8; 6] = b"070701";
/// The trailer entry name that ends a cpio stream.
pub const TRAILER: &str = "TRAILER!!!";

/// `S_IFMT` mode-type bits relevant for cpio entries.
pub mod mode {
    /// Regular file.
    pub const IFREG: u32 = 0o100000;
    /// Directory.
    pub const IFDIR: u32 = 0o040000;
    /// Symbolic link.
    pub const IFLNK: u32 = 0o120000;
    /// Type mask.
    pub const IFMT: u32 = 0o170000;
}

/// A single cpio entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpioEntry {
    /// Path name (relative, e.g. `"init"`, `"usr/bin/sh"`).
    pub name: String,
    /// Full mode including type bits (e.g. `IFREG | 0o755`).
    pub mode: u32,
    /// Owner uid.
    pub uid: u32,
    /// Owner gid.
    pub gid: u32,
    /// Modification time.
    pub mtime: u32,
    /// Number of links.
    pub nlink: u32,
    /// File content (for regular files) or symlink target bytes.
    pub data: Vec<u8>,
}

impl CpioEntry {
    /// Build a regular-file entry. `mode` is the permission bits only.
    pub fn file(name: impl Into<String>, data: &[u8], mode: u32) -> Self {
        CpioEntry {
            name: name.into(),
            mode: mode::IFREG | (mode & 0o7777),
            uid: 0,
            gid: 0,
            mtime: 0,
            nlink: 1,
            data: data.to_vec(),
        }
    }

    /// Build a directory entry.
    pub fn dir(name: impl Into<String>, mode: u32) -> Self {
        CpioEntry {
            name: name.into(),
            mode: mode::IFDIR | (mode & 0o7777),
            uid: 0,
            gid: 0,
            mtime: 0,
            nlink: 2,
            data: Vec::new(),
        }
    }

    /// Build a symlink entry; the target is stored as the data payload.
    pub fn symlink(name: impl Into<String>, target: &str) -> Self {
        CpioEntry {
            name: name.into(),
            mode: mode::IFLNK | 0o777,
            uid: 0,
            gid: 0,
            mtime: 0,
            nlink: 1,
            data: target.as_bytes().to_vec(),
        }
    }

    /// The file-type portion of the mode.
    pub fn file_type(&self) -> u32 {
        self.mode & mode::IFMT
    }

    /// Whether this is a directory entry.
    pub fn is_dir(&self) -> bool {
        self.file_type() == mode::IFDIR
    }

    /// Whether this is a symlink entry.
    pub fn is_symlink(&self) -> bool {
        self.file_type() == mode::IFLNK
    }

    /// For a symlink, the decoded target path.
    pub fn link_target(&self) -> Option<String> {
        if self.is_symlink() {
            Some(String::from_utf8_lossy(&self.data).into_owned())
        } else {
            None
        }
    }
}

/// A decoded cpio archive.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CpioArchive {
    /// Entries in archive order (the trailer is not included).
    pub entries: Vec<CpioEntry>,
}

impl CpioArchive {
    /// Find an entry by exact name.
    pub fn find(&self, name: &str) -> Option<&CpioEntry> {
        self.entries.iter().find(|e| e.name == name)
    }
}

fn align4(n: usize) -> usize {
    n.div_ceil(4) * 4
}

fn write_hex8(out: &mut Vec<u8>, value: u32) {
    let s = format!("{value:08X}");
    out.extend_from_slice(s.as_bytes());
}

fn parse_hex8(bytes: &[u8]) -> crate::Result<u32> {
    if bytes.len() != 8 {
        return Err(Error::parse("cpio hex field not 8 bytes".to_string()));
    }
    let s = std::str::from_utf8(bytes)
        .map_err(|_| Error::parse("cpio hex field not ascii".to_string()))?;
    u32::from_str_radix(s, 16).map_err(|_| Error::parse(format!("bad cpio hex field: {s}")))
}

/// Encoder for newc cpio archives, accumulating into a buffer.
#[derive(Debug, Default)]
pub struct CpioWriter {
    buf: Vec<u8>,
    ino: u32,
    finished: bool,
}

impl CpioWriter {
    /// A fresh writer.
    pub fn new() -> Self {
        CpioWriter {
            buf: Vec::new(),
            ino: 0,
            finished: false,
        }
    }

    fn write_record(&mut self, entry: &CpioEntry) -> crate::Result<()> {
        if entry.name.contains('\0') {
            return Err(Error::invalid("cpio name contains NUL"));
        }
        self.ino += 1;
        let name_bytes = entry.name.as_bytes();
        let namesize = name_bytes.len() + 1; // include NUL

        self.buf.extend_from_slice(NEWC_MAGIC);
        write_hex8(&mut self.buf, self.ino); // c_ino
        write_hex8(&mut self.buf, entry.mode); // c_mode
        write_hex8(&mut self.buf, entry.uid); // c_uid
        write_hex8(&mut self.buf, entry.gid); // c_gid
        write_hex8(&mut self.buf, entry.nlink); // c_nlink
        write_hex8(&mut self.buf, entry.mtime); // c_mtime
        write_hex8(&mut self.buf, entry.data.len() as u32); // c_filesize
        write_hex8(&mut self.buf, 0); // c_devmajor
        write_hex8(&mut self.buf, 0); // c_devminor
        write_hex8(&mut self.buf, 0); // c_rdevmajor
        write_hex8(&mut self.buf, 0); // c_rdevminor
        write_hex8(&mut self.buf, namesize as u32); // c_namesize
        write_hex8(&mut self.buf, 0); // c_check

        // name + NUL, padded so that (header=110 + namesize) is 4-aligned.
        self.buf.extend_from_slice(name_bytes);
        self.buf.push(0);
        let header_and_name = 110 + namesize;
        let pad = align4(header_and_name) - header_and_name;
        self.buf.extend(std::iter::repeat_n(0u8, pad));

        // data, padded to 4 bytes.
        self.buf.extend_from_slice(&entry.data);
        let dpad = align4(entry.data.len()) - entry.data.len();
        self.buf.extend(std::iter::repeat_n(0u8, dpad));
        Ok(())
    }

    /// Append an entry.
    pub fn add(&mut self, entry: &CpioEntry) -> crate::Result<()> {
        if self.finished {
            return Err(Error::invalid_state("cpio writer already finished"));
        }
        self.write_record(entry)
    }

    /// Finalize: write the `TRAILER!!!` record and return the bytes.
    pub fn finish(mut self) -> crate::Result<Vec<u8>> {
        if !self.finished {
            let trailer = CpioEntry {
                name: TRAILER.to_string(),
                mode: 0,
                uid: 0,
                gid: 0,
                mtime: 0,
                nlink: 1,
                data: Vec::new(),
            };
            self.write_record(&trailer)?;
            self.finished = true;
        }
        Ok(self.buf)
    }

    /// Bytes written so far.
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Whether nothing has been written.
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}

/// Decoder for newc cpio archives.
pub struct CpioReader;

impl CpioReader {
    /// Parse an entire archive, stopping at the `TRAILER!!!` entry.
    pub fn read(bytes: &[u8]) -> crate::Result<CpioArchive> {
        let mut entries = Vec::new();
        let mut off = 0;
        loop {
            if off + 110 > bytes.len() {
                return Err(Error::parse("cpio truncated header".to_string()));
            }
            if &bytes[off..off + 6] != NEWC_MAGIC {
                return Err(Error::parse(format!("bad cpio magic at offset {off}")));
            }
            let field = |i: usize| -> crate::Result<u32> {
                let start = off + 6 + i * 8;
                parse_hex8(&bytes[start..start + 8])
            };
            let mode = field(1)?;
            let uid = field(2)?;
            let gid = field(3)?;
            let nlink = field(4)?;
            let mtime = field(5)?;
            let filesize = field(6)? as usize;
            let namesize = field(11)? as usize;

            let name_start = off + 110;
            if name_start + namesize > bytes.len() {
                return Err(Error::parse("cpio truncated name".to_string()));
            }
            // name includes trailing NUL.
            let raw_name = &bytes[name_start..name_start + namesize];
            let name_end = raw_name.iter().position(|&b| b == 0).unwrap_or(namesize);
            let name = String::from_utf8_lossy(&raw_name[..name_end]).into_owned();

            let header_and_name = 110 + namesize;
            let data_start = off + align4(header_and_name);

            if name == TRAILER {
                break;
            }

            if data_start + filesize > bytes.len() {
                return Err(Error::parse("cpio truncated data".to_string()));
            }
            let data = bytes[data_start..data_start + filesize].to_vec();

            entries.push(CpioEntry {
                name,
                mode,
                uid,
                gid,
                mtime,
                nlink,
                data,
            });

            off = data_start + align4(filesize);
        }
        Ok(CpioArchive { entries })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_archive_is_just_trailer() {
        let w = CpioWriter::new();
        let bytes = w.finish().unwrap();
        let arc = CpioReader::read(&bytes).unwrap();
        assert!(arc.entries.is_empty());
    }

    #[test]
    fn file_roundtrip() {
        let mut w = CpioWriter::new();
        w.add(&CpioEntry::file("init", b"#!/bin/sh\n", 0o755))
            .unwrap();
        let bytes = w.finish().unwrap();
        let arc = CpioReader::read(&bytes).unwrap();
        assert_eq!(arc.entries.len(), 1);
        let e = arc.find("init").unwrap();
        assert_eq!(e.data, b"#!/bin/sh\n");
        assert_eq!(e.mode & 0o7777, 0o755);
        assert_eq!(e.file_type(), mode::IFREG);
    }

    #[test]
    fn dir_and_symlink_types() {
        let mut w = CpioWriter::new();
        w.add(&CpioEntry::dir("usr", 0o755)).unwrap();
        w.add(&CpioEntry::symlink("usr/sh", "bin/busybox")).unwrap();
        let bytes = w.finish().unwrap();
        let arc = CpioReader::read(&bytes).unwrap();
        assert!(arc.find("usr").unwrap().is_dir());
        let link = arc.find("usr/sh").unwrap();
        assert!(link.is_symlink());
        assert_eq!(link.link_target().unwrap(), "bin/busybox");
    }

    #[test]
    fn multiple_entries_preserve_order() {
        let mut w = CpioWriter::new();
        w.add(&CpioEntry::file("a", b"1", 0o644)).unwrap();
        w.add(&CpioEntry::file("bb", b"22", 0o644)).unwrap();
        w.add(&CpioEntry::file("ccc", b"333", 0o644)).unwrap();
        let bytes = w.finish().unwrap();
        let arc = CpioReader::read(&bytes).unwrap();
        let names: Vec<&str> = arc.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["a", "bb", "ccc"]);
        assert_eq!(arc.find("ccc").unwrap().data, b"333");
    }

    #[test]
    fn alignment_is_four_bytes() {
        // odd-length name and data exercise padding.
        let mut w = CpioWriter::new();
        w.add(&CpioEntry::file("abc", b"xyzab", 0o644)).unwrap();
        let bytes = w.finish().unwrap();
        // Each record offset must remain 4-aligned; reading back must succeed.
        assert_eq!(bytes.len() % 4, 0);
        let arc = CpioReader::read(&bytes).unwrap();
        assert_eq!(arc.find("abc").unwrap().data, b"xyzab");
    }

    #[test]
    fn bad_magic_rejected() {
        let mut bytes = {
            let mut w = CpioWriter::new();
            w.add(&CpioEntry::file("x", b"y", 0o644)).unwrap();
            w.finish().unwrap()
        };
        bytes[0] = b'9';
        let err = CpioReader::read(&bytes).unwrap_err();
        assert_eq!(err.kind(), "parse");
    }

    #[test]
    fn nul_in_name_rejected() {
        let mut w = CpioWriter::new();
        let err = w.add(&CpioEntry::file("a\0b", b"x", 0o644)).unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }

    #[test]
    fn truncated_archive_rejected() {
        let mut w = CpioWriter::new();
        w.add(&CpioEntry::file("init", b"data", 0o644)).unwrap();
        let bytes = w.finish().unwrap();
        let truncated = &bytes[..bytes.len() - 40];
        let err = CpioReader::read(truncated).unwrap_err();
        assert_eq!(err.kind(), "parse");
    }
}
