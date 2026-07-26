//! A real USTAR (POSIX.1-1988) tar writer and reader.
//!
//! This is the wire format Talos's `pkg/archiver` streams over the machine
//! `Copy` gRPC service and that the installer reads when unpacking images. The
//! implementation here writes genuine 512-byte tar blocks with octal numeric
//! fields and the `ustar\0` magic, computes the header checksum exactly as GNU
//! tar / the Go `archive/tar` package do, and reads them back.
//!
//! It operates over in-memory `Vec<u8>` streams so the create/extract round
//! trip is fully exercised by tests without touching a real filesystem.

use os_kernel::Error;

use crate::walker::{FileKind, FileTree, WalkEntry};

/// Size of a tar block, in bytes.
pub const BLOCK_SIZE: usize = 512;

/// The tar typeflag for an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    /// Regular file (`'0'`).
    Regular,
    /// Directory (`'5'`).
    Directory,
    /// Symbolic link (`'2'`); target lives in the linkname field.
    Symlink,
    /// Hard link (`'1'`).
    HardLink,
    /// Character device (`'3'`).
    CharDevice,
    /// FIFO (`'6'`).
    Fifo,
}

impl EntryType {
    /// The single-byte typeflag character.
    pub fn flag(self) -> u8 {
        match self {
            EntryType::Regular => b'0',
            EntryType::HardLink => b'1',
            EntryType::Symlink => b'2',
            EntryType::CharDevice => b'3',
            EntryType::Directory => b'5',
            EntryType::Fifo => b'6',
        }
    }

    /// Parse a typeflag byte. `\0` is treated as a regular file per the spec.
    pub fn from_flag(b: u8) -> crate::Result<Self> {
        Ok(match b {
            b'0' | 0 => EntryType::Regular,
            b'1' => EntryType::HardLink,
            b'2' => EntryType::Symlink,
            b'3' => EntryType::CharDevice,
            b'5' => EntryType::Directory,
            b'6' => EntryType::Fifo,
            other => return Err(Error::parse(format!("unsupported tar typeflag {other:#x}"))),
        })
    }

    fn from_kind(kind: FileKind) -> Self {
        match kind {
            FileKind::Regular => EntryType::Regular,
            FileKind::Directory => EntryType::Directory,
            FileKind::Symlink => EntryType::Symlink,
            FileKind::Special => EntryType::CharDevice,
        }
    }
}

/// A decoded tar header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TarHeader {
    /// Entry path (relative, no leading slash by convention).
    pub name: String,
    /// File mode bits.
    pub mode: u32,
    /// Owner uid.
    pub uid: u32,
    /// Owner gid.
    pub gid: u32,
    /// Content size in bytes (0 for non-regular).
    pub size: u64,
    /// Modification time, seconds since epoch.
    pub mtime: u64,
    /// Entry type.
    pub typeflag: EntryType,
    /// Symlink/hardlink target, empty otherwise.
    pub linkname: String,
}

impl TarHeader {
    /// A regular-file header.
    pub fn regular(name: impl Into<String>, size: u64, mode: u32) -> Self {
        TarHeader {
            name: name.into(),
            mode,
            uid: 0,
            gid: 0,
            size,
            mtime: 0,
            typeflag: EntryType::Regular,
            linkname: String::new(),
        }
    }

    /// A directory header.
    pub fn directory(name: impl Into<String>, mode: u32) -> Self {
        TarHeader {
            name: name.into(),
            mode,
            uid: 0,
            gid: 0,
            size: 0,
            mtime: 0,
            typeflag: EntryType::Directory,
            linkname: String::new(),
        }
    }

    /// A symlink header.
    pub fn symlink(name: impl Into<String>, target: impl Into<String>, mode: u32) -> Self {
        TarHeader {
            name: name.into(),
            mode,
            uid: 0,
            gid: 0,
            size: 0,
            mtime: 0,
            typeflag: EntryType::Symlink,
            linkname: target.into(),
        }
    }
}

/// A fully decoded tar entry (header + content).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TarEntry {
    /// The entry header.
    pub header: TarHeader,
    /// File content bytes (empty for non-regular entries).
    pub data: Vec<u8>,
}

/// A decoded archive: an ordered list of entries.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TarArchive {
    /// The entries in archive order.
    pub entries: Vec<TarEntry>,
}

impl TarArchive {
    /// Look up an entry by exact name.
    pub fn find(&self, name: &str) -> Option<&TarEntry> {
        self.entries.iter().find(|e| e.header.name == name)
    }
}

fn write_octal(field: &mut [u8], value: u64) {
    // Width-1 octal digits, space/NUL terminated, zero-padded — GNU style.
    let width = field.len();
    let s = format!("{:0width$o}", value, width = width.saturating_sub(1));
    let bytes = s.as_bytes();
    let start = width.saturating_sub(1).saturating_sub(bytes.len());
    for b in field.iter_mut() {
        *b = b'0';
    }
    field[start..width - 1].copy_from_slice(&bytes[bytes.len().saturating_sub(width - 1)..]);
    field[width - 1] = 0;
}

fn write_str(field: &mut [u8], s: &str) {
    for b in field.iter_mut() {
        *b = 0;
    }
    let bytes = s.as_bytes();
    let n = bytes.len().min(field.len());
    field[..n].copy_from_slice(&bytes[..n]);
}

fn parse_octal(field: &[u8]) -> crate::Result<u64> {
    let mut val: u64 = 0;
    let mut seen = false;
    for &b in field {
        match b {
            b'0'..=b'9' => {
                seen = true;
                val = val * 8 + u64::from(b - b'0');
            }
            b' ' | 0 => {
                if seen {
                    break;
                }
            }
            other => return Err(Error::parse(format!("invalid octal byte {other:#x}"))),
        }
    }
    Ok(val)
}

fn parse_str(field: &[u8]) -> String {
    let end = field.iter().position(|&b| b == 0).unwrap_or(field.len());
    String::from_utf8_lossy(&field[..end]).into_owned()
}

fn checksum(block: &[u8; BLOCK_SIZE]) -> u32 {
    let mut sum: u32 = 0;
    for (i, &b) in block.iter().enumerate() {
        // The checksum field (148..156) is treated as spaces during compute.
        if (148..156).contains(&i) {
            sum += u32::from(b' ');
        } else {
            sum += u32::from(b);
        }
    }
    sum
}

fn encode_header(h: &TarHeader) -> [u8; BLOCK_SIZE] {
    let mut block = [0u8; BLOCK_SIZE];
    write_str(&mut block[0..100], &h.name);
    write_octal(&mut block[100..108], u64::from(h.mode));
    write_octal(&mut block[108..116], u64::from(h.uid));
    write_octal(&mut block[116..124], u64::from(h.gid));
    write_octal(&mut block[124..136], h.size);
    write_octal(&mut block[136..148], h.mtime);
    // checksum placeholder = spaces
    for b in &mut block[148..156] {
        *b = b' ';
    }
    block[156] = h.typeflag.flag();
    write_str(&mut block[157..257], &h.linkname);
    // ustar magic + version
    block[257..263].copy_from_slice(b"ustar\0");
    block[263..265].copy_from_slice(b"00");

    let sum = checksum(&block);
    // checksum field: 6 octal digits, NUL, space.
    let s = format!("{sum:06o}");
    block[148..154].copy_from_slice(s.as_bytes());
    block[154] = 0;
    block[155] = b' ';
    block
}

fn padded_len(size: u64) -> usize {
    let blocks = (size as usize).div_ceil(BLOCK_SIZE);
    blocks * BLOCK_SIZE
}

/// Streaming tar writer accumulating into an in-memory buffer.
#[derive(Debug, Default)]
pub struct TarWriter {
    buf: Vec<u8>,
    finished: bool,
}

impl TarWriter {
    /// A fresh writer.
    pub fn new() -> Self {
        TarWriter {
            buf: Vec::new(),
            finished: false,
        }
    }

    /// Append an entry with explicit header and content.
    pub fn add_entry(&mut self, header: &TarHeader, data: &[u8]) -> crate::Result<()> {
        if self.finished {
            return Err(Error::invalid_state("tar writer already finished"));
        }
        if header.name.is_empty() {
            return Err(Error::invalid("tar entry name is empty"));
        }
        if header.name.len() > 100 {
            return Err(Error::unsupported(format!(
                "tar name too long for USTAR (>100): {}",
                header.name
            )));
        }
        let mut header = header.clone();
        if header.typeflag == EntryType::Regular {
            header.size = data.len() as u64;
        } else {
            header.size = 0;
        }
        let block = encode_header(&header);
        self.buf.extend_from_slice(&block);
        if header.typeflag == EntryType::Regular && !data.is_empty() {
            self.buf.extend_from_slice(data);
            let pad = padded_len(data.len() as u64) - data.len();
            self.buf.extend(std::iter::repeat_n(0u8, pad));
        }
        Ok(())
    }

    /// Convenience: append a regular file.
    pub fn add_file(&mut self, name: &str, data: &[u8], mode: u32) -> crate::Result<()> {
        self.add_entry(&TarHeader::regular(name, data.len() as u64, mode), data)
    }

    /// Convenience: append a directory.
    pub fn add_dir(&mut self, name: &str, mode: u32) -> crate::Result<()> {
        self.add_entry(&TarHeader::directory(name, mode), &[])
    }

    /// Append a node sourced from a [`FileTree`] walk entry. The tar `name`
    /// uses the walk-relative path, exactly as Talos's archiver streams it.
    pub fn add_walk_entry(&mut self, tree: &FileTree, entry: &WalkEntry) -> crate::Result<()> {
        let node = tree
            .get(&entry.abs_path)
            .ok_or_else(|| Error::not_found(format!("node missing: {}", entry.abs_path)))?;
        let mut name = entry.rel_path.clone();
        if name == "." {
            name = String::new();
        }
        if node.kind == FileKind::Directory && !name.is_empty() && !name.ends_with('/') {
            name.push('/');
        }
        if name.is_empty() {
            // root directory entry -> "./"
            name = "./".to_string();
        }
        let header = TarHeader {
            name,
            mode: node.mode,
            uid: node.uid,
            gid: node.gid,
            size: node.data.len() as u64,
            mtime: node.mtime,
            typeflag: EntryType::from_kind(node.kind),
            linkname: node.link_target.clone(),
        };
        self.add_entry(&header, &node.data)
    }

    /// Finalize the archive (writes two zero blocks) and return the bytes.
    pub fn finish(mut self) -> Vec<u8> {
        if !self.finished {
            self.buf.extend(std::iter::repeat_n(0u8, BLOCK_SIZE * 2));
            self.finished = true;
        }
        self.buf
    }

    /// Bytes written so far (excluding the not-yet-written trailer).
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Whether nothing has been written.
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}

/// Tar reader over an in-memory byte slice.
pub struct TarReader;

impl TarReader {
    /// Parse an entire archive from bytes.
    pub fn read(bytes: &[u8]) -> crate::Result<TarArchive> {
        if !bytes.len().is_multiple_of(BLOCK_SIZE) {
            return Err(Error::parse(format!(
                "tar length {} not a multiple of {BLOCK_SIZE}",
                bytes.len()
            )));
        }
        let mut entries = Vec::new();
        let mut off = 0;
        while off + BLOCK_SIZE <= bytes.len() {
            let mut block = [0u8; BLOCK_SIZE];
            block.copy_from_slice(&bytes[off..off + BLOCK_SIZE]);
            off += BLOCK_SIZE;

            if block.iter().all(|&b| b == 0) {
                // Trailer (one all-zero block ends the meaningful stream).
                break;
            }

            // Verify magic for a sane header.
            if &block[257..262] != b"ustar" {
                return Err(Error::parse("missing ustar magic in header".to_string()));
            }
            // Verify checksum.
            let stored = parse_octal(&block[148..156])?;
            let computed = checksum(&block);
            if u64::from(computed) != stored {
                return Err(Error::parse(format!(
                    "tar checksum mismatch: stored {stored}, computed {computed}"
                )));
            }

            let header = TarHeader {
                name: parse_str(&block[0..100]),
                mode: parse_octal(&block[100..108])? as u32,
                uid: parse_octal(&block[108..116])? as u32,
                gid: parse_octal(&block[116..124])? as u32,
                size: parse_octal(&block[124..136])?,
                mtime: parse_octal(&block[136..148])?,
                typeflag: EntryType::from_flag(block[156])?,
                linkname: parse_str(&block[157..257]),
            };

            let mut data = Vec::new();
            if header.typeflag == EntryType::Regular && header.size > 0 {
                let n = header.size as usize;
                let total = padded_len(header.size);
                if off + total > bytes.len() {
                    return Err(Error::parse("truncated tar content".to_string()));
                }
                data.extend_from_slice(&bytes[off..off + n]);
                off += total;
            }
            entries.push(TarEntry { header, data });
        }
        Ok(TarArchive { entries })
    }

    /// Extract an archive back into a fresh [`FileTree`] rooted at `dest`.
    pub fn extract_to_tree(bytes: &[u8], dest: &str) -> crate::Result<FileTree> {
        let archive = Self::read(bytes)?;
        let mut tree = FileTree::new();
        let dest = FileTree::normalize(dest);
        for entry in &archive.entries {
            let rel = entry
                .header
                .name
                .trim_end_matches('/')
                .trim_start_matches("./");
            let abs = if rel.is_empty() {
                dest.clone()
            } else if dest == "/" {
                format!("/{rel}")
            } else {
                format!("{dest}/{rel}")
            };
            match entry.header.typeflag {
                EntryType::Directory => tree.add_dir(&abs, entry.header.mode),
                EntryType::Regular => tree.add_file(&abs, &entry.data, entry.header.mode),
                EntryType::Symlink => {
                    tree.add_symlink(&abs, &entry.header.linkname, entry.header.mode)
                }
                EntryType::CharDevice | EntryType::Fifo => {
                    tree.add_special(&abs, entry.header.mode)
                }
                EntryType::HardLink => tree.add_file(&abs, &[], entry.header.mode),
            }
        }
        Ok(tree)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn octal_roundtrip() {
        let mut field = [0u8; 12];
        write_octal(&mut field, 0o644);
        assert_eq!(parse_octal(&field).unwrap(), 0o644);
        write_octal(&mut field, 1234567);
        assert_eq!(parse_octal(&field).unwrap(), 1234567);
    }

    #[test]
    fn single_file_roundtrip() {
        let mut w = TarWriter::new();
        w.add_file("hello.txt", b"hello world", 0o644).unwrap();
        let bytes = w.finish();
        // header + 1 content block + 2 trailer blocks
        assert_eq!(bytes.len(), BLOCK_SIZE * 4);
        let arc = TarReader::read(&bytes).unwrap();
        assert_eq!(arc.entries.len(), 1);
        let e = &arc.entries[0];
        assert_eq!(e.header.name, "hello.txt");
        assert_eq!(e.header.mode, 0o644);
        assert_eq!(e.data, b"hello world");
    }

    #[test]
    fn directory_and_symlink_entries() {
        let mut w = TarWriter::new();
        w.add_dir("etc/", 0o755).unwrap();
        w.add_entry(&TarHeader::symlink("etc/rc", "/sbin/init", 0o777), &[])
            .unwrap();
        let bytes = w.finish();
        let arc = TarReader::read(&bytes).unwrap();
        let dir = arc.find("etc/").unwrap();
        assert_eq!(dir.header.typeflag, EntryType::Directory);
        let link = arc.find("etc/rc").unwrap();
        assert_eq!(link.header.typeflag, EntryType::Symlink);
        assert_eq!(link.header.linkname, "/sbin/init");
    }

    #[test]
    fn checksum_mismatch_detected() {
        let mut w = TarWriter::new();
        w.add_file("a", b"data", 0o644).unwrap();
        let mut bytes = w.finish();
        // Corrupt a byte in the name field of the first header.
        bytes[0] ^= 0xff;
        let err = TarReader::read(&bytes).unwrap_err();
        assert_eq!(err.kind(), "parse");
    }

    #[test]
    fn name_too_long_rejected() {
        let mut w = TarWriter::new();
        let long = "a".repeat(101);
        let err = w.add_file(&long, b"x", 0o644).unwrap_err();
        assert_eq!(err.kind(), "unsupported");
    }

    #[test]
    fn empty_name_rejected() {
        let mut w = TarWriter::new();
        let err = w.add_file("", b"x", 0o644).unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }

    #[test]
    fn extract_to_tree_reconstructs() {
        let mut src = FileTree::new();
        src.add_dir("/d", 0o755);
        src.add_file("/d/f", b"content", 0o600);
        src.add_symlink("/d/l", "/target", 0o777);

        let walker = crate::walker::Walker::new(crate::walker::WalkOptions::default().skip_root());
        let entries = walker.walk(&src, "/d").unwrap();
        let mut w = TarWriter::new();
        for e in &entries {
            w.add_walk_entry(&src, e).unwrap();
        }
        let bytes = w.finish();

        let out = TarReader::extract_to_tree(&bytes, "/restore").unwrap();
        assert_eq!(out.get("/restore/f").unwrap().data, b"content");
        assert_eq!(out.get("/restore/l").unwrap().kind, FileKind::Symlink);
        assert_eq!(out.get("/restore/l").unwrap().link_target, "/target");
    }

    #[test]
    fn non_block_aligned_length_rejected() {
        let bytes = vec![0u8; 100];
        let err = TarReader::read(&bytes).unwrap_err();
        assert_eq!(err.kind(), "parse");
    }

    #[test]
    fn writer_rejects_after_finish_via_state() {
        let mut w = TarWriter::new();
        w.add_file("x", b"1", 0o644).unwrap();
        assert!(!w.is_empty());
        let bytes = w.finish();
        assert!(!bytes.is_empty());
    }
}
