//! Host-safe cryptsetup / dm-crypt command surface.
//!
//! Talos performs encrypted-volume lifecycle work through `cryptsetup`: format
//! a LUKS2 header, enroll keyslots, open the device as `luks2-<volume-id>`, and
//! close that mapper on teardown. This module models the command boundary
//! without executing host disk operations, so controller tests can prove the
//! exact device-mapper intent while staying safe on developer machines and CI.

use std::collections::BTreeMap;

use crate::encryption::Cipher;
use crate::luks::Luks2Header;
use crate::{BlockError, Result};

/// Result of opening an encrypted block device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CryptOpenResult {
    /// Device-mapper name, e.g. `luks2-STATE`.
    pub mapper_name: String,
    /// Canonical `/dev/mapper/...` path.
    pub mapped_path: String,
    /// LUKS keyslot that unlocked the device.
    pub key_slot: u8,
}

/// A redacted command intent for the `cryptsetup` binary.
///
/// Passphrases are represented only by the number of stdin bytes, never by
/// their contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CryptsetupCommand {
    /// Program to execute.
    pub program: String,
    /// Argument vector excluding `program`.
    pub args: Vec<String>,
    /// Number of secret bytes that would be streamed to stdin.
    pub stdin_bytes: usize,
}

impl CryptsetupCommand {
    /// Construct a `cryptsetup luksFormat` command intent.
    pub fn luks_format(
        dev_path: impl Into<String>,
        cipher: Cipher,
        key_slot: u8,
        stdin_bytes: usize,
    ) -> Self {
        CryptsetupCommand {
            program: "cryptsetup".to_string(),
            args: vec![
                "luksFormat".to_string(),
                "--type".to_string(),
                "luks2".to_string(),
                "--cipher".to_string(),
                cipher.as_str().to_string(),
                "--key-size".to_string(),
                cipher.key_bits().to_string(),
                "--key-slot".to_string(),
                key_slot.to_string(),
                "--key-file".to_string(),
                "-".to_string(),
                "--batch-mode".to_string(),
                dev_path.into(),
            ],
            stdin_bytes,
        }
    }

    /// Construct a `cryptsetup luksAddKey` command intent.
    pub fn luks_add_key(
        dev_path: impl Into<String>,
        key_slot: u8,
        existing_key_bytes: usize,
        new_key_bytes: usize,
    ) -> Self {
        CryptsetupCommand {
            program: "cryptsetup".to_string(),
            args: vec![
                "luksAddKey".to_string(),
                "--key-slot".to_string(),
                key_slot.to_string(),
                "--key-file".to_string(),
                "-".to_string(),
                "--batch-mode".to_string(),
                dev_path.into(),
            ],
            stdin_bytes: existing_key_bytes + new_key_bytes,
        }
    }

    /// Construct a `cryptsetup open` command intent.
    pub fn open(
        dev_path: impl Into<String>,
        mapper_name: impl Into<String>,
        key_slot: u8,
        stdin_bytes: usize,
    ) -> Self {
        CryptsetupCommand {
            program: "cryptsetup".to_string(),
            args: vec![
                "open".to_string(),
                "--type".to_string(),
                "luks2".to_string(),
                "--key-slot".to_string(),
                key_slot.to_string(),
                "--key-file".to_string(),
                "-".to_string(),
                dev_path.into(),
                mapper_name.into(),
            ],
            stdin_bytes,
        }
    }

    /// Construct a `cryptsetup close` command intent.
    pub fn close(mapper_name: impl Into<String>) -> Self {
        CryptsetupCommand {
            program: "cryptsetup".to_string(),
            args: vec!["close".to_string(), mapper_name.into()],
            stdin_bytes: 0,
        }
    }
}

/// Canonical Talos dm-crypt mapper name for a volume id.
pub fn mapper_name(volume_id: &str) -> Result<String> {
    if volume_id.is_empty() {
        return Err(BlockError::InvalidDevice(
            "empty encrypted volume id".to_string(),
        ));
    }
    if volume_id
        .bytes()
        .any(|byte| byte == b'/' || byte == 0 || byte.is_ascii_whitespace())
    {
        return Err(BlockError::InvalidDevice(format!(
            "volume id {volume_id:?} cannot be used as a mapper name"
        )));
    }
    Ok(format!("luks2-{volume_id}"))
}

/// Canonical `/dev/mapper/...` path for a mapper name.
pub fn mapper_path(mapper_name: &str) -> Result<String> {
    if mapper_name.is_empty()
        || mapper_name
            .bytes()
            .any(|byte| byte == b'/' || byte == 0 || byte.is_ascii_whitespace())
    {
        return Err(BlockError::InvalidDevice(format!(
            "invalid mapper name {mapper_name:?}"
        )));
    }
    Ok(format!("/dev/mapper/{mapper_name}"))
}

fn validate_device_path(dev_path: &str) -> Result<()> {
    if dev_path.is_empty() || dev_path.bytes().any(|byte| byte == 0) {
        return Err(BlockError::InvalidDevice(format!(
            "invalid encrypted device path {dev_path:?}"
        )));
    }
    Ok(())
}

/// Format request for a LUKS2 device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LuksFormatRequest {
    /// Raw block device path.
    pub dev_path: String,
    /// Header UUID/model identifier.
    pub uuid: String,
    /// Cipher to configure.
    pub cipher: Cipher,
    /// Slot to seed during format.
    pub key_slot: u8,
    /// Passphrase bytes for the seeded slot.
    pub passphrase: Vec<u8>,
}

/// Add-key request for an existing LUKS2 device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LuksAddKeyRequest {
    /// Raw block device path.
    pub dev_path: String,
    /// Existing passphrase that already opens the device.
    pub existing_passphrase: Vec<u8>,
    /// New slot to enroll.
    pub new_key_slot: u8,
    /// New slot passphrase.
    pub new_passphrase: Vec<u8>,
}

/// Open request for an existing LUKS2 device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LuksOpenRequest {
    /// Raw block device path.
    pub dev_path: String,
    /// Device-mapper name.
    pub mapper_name: String,
    /// Keyslot that should be tried.
    pub key_slot: u8,
    /// Passphrase bytes for `key_slot`.
    pub passphrase: Vec<u8>,
}

/// Close request for an opened dm-crypt mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LuksCloseRequest {
    /// Device-mapper name.
    pub mapper_name: String,
}

/// Boundary implemented by real cryptsetup runners and host-safe fakes.
pub trait CryptsetupBackend {
    /// Format a device as LUKS2.
    fn format(&mut self, request: LuksFormatRequest) -> Result<()>;
    /// Enroll an additional keyslot.
    fn add_key(&mut self, request: LuksAddKeyRequest) -> Result<()>;
    /// Open an encrypted device.
    fn open(&mut self, request: LuksOpenRequest) -> Result<CryptOpenResult>;
    /// Close an encrypted mapper.
    fn close(&mut self, request: LuksCloseRequest) -> Result<()>;
}

/// In-memory cryptsetup backend used by tests.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MemCryptsetupBackend {
    headers: BTreeMap<String, Luks2Header>,
    opened: BTreeMap<String, CryptOpenResult>,
    commands: Vec<CryptsetupCommand>,
}

impl MemCryptsetupBackend {
    /// Recorded command intents.
    pub fn commands(&self) -> &[CryptsetupCommand] {
        &self.commands
    }

    /// Inspect a formatted header.
    pub fn header(&self, dev_path: &str) -> Option<&Luks2Header> {
        self.headers.get(dev_path)
    }

    /// Inspect an opened mapper.
    pub fn opened(&self, mapper_name: &str) -> Option<&CryptOpenResult> {
        self.opened.get(mapper_name)
    }

    /// Seed a header for a device path, modelling a previously formatted
    /// encrypted volume discovered by the controller.
    pub fn insert_header(
        &mut self,
        dev_path: impl Into<String>,
        header: Luks2Header,
    ) -> Result<()> {
        let dev_path = dev_path.into();
        validate_device_path(&dev_path)?;
        if !header.is_valid() {
            return Err(BlockError::KeyFailure(format!(
                "invalid LUKS2 header for {dev_path}"
            )));
        }
        self.headers.insert(dev_path, header);
        Ok(())
    }
}

impl CryptsetupBackend for MemCryptsetupBackend {
    fn format(&mut self, request: LuksFormatRequest) -> Result<()> {
        validate_device_path(&request.dev_path)?;
        if self.headers.contains_key(&request.dev_path) {
            return Err(BlockError::KeyFailure(format!(
                "LUKS2 header already exists on {}",
                request.dev_path
            )));
        }
        let mut header = Luks2Header::format(request.uuid, request.cipher);
        header.add_key(request.key_slot, &request.passphrase)?;
        self.commands.push(CryptsetupCommand::luks_format(
            request.dev_path.clone(),
            request.cipher,
            request.key_slot,
            request.passphrase.len(),
        ));
        self.headers.insert(request.dev_path, header);
        Ok(())
    }

    fn add_key(&mut self, request: LuksAddKeyRequest) -> Result<()> {
        validate_device_path(&request.dev_path)?;
        let header = self.headers.get_mut(&request.dev_path).ok_or_else(|| {
            BlockError::NotFound(format!("LUKS2 header for {} not found", request.dev_path))
        })?;
        let _existing_slot = header.open(&request.existing_passphrase)?;
        header.add_key(request.new_key_slot, &request.new_passphrase)?;
        self.commands.push(CryptsetupCommand::luks_add_key(
            request.dev_path,
            request.new_key_slot,
            request.existing_passphrase.len(),
            request.new_passphrase.len(),
        ));
        Ok(())
    }

    fn open(&mut self, request: LuksOpenRequest) -> Result<CryptOpenResult> {
        validate_device_path(&request.dev_path)?;
        mapper_path(&request.mapper_name)?;
        if self.opened.contains_key(&request.mapper_name) {
            return Err(BlockError::InvalidDevice(format!(
                "mapper {} is already open",
                request.mapper_name
            )));
        }
        let header = self.headers.get(&request.dev_path).ok_or_else(|| {
            BlockError::NotFound(format!("LUKS2 header for {} not found", request.dev_path))
        })?;
        let key_slot = header.open(&request.passphrase)?;
        if key_slot != request.key_slot {
            return Err(BlockError::KeyFailure(format!(
                "passphrase unlocked slot {key_slot}, not requested slot {}",
                request.key_slot
            )));
        }
        let mapped_path = mapper_path(&request.mapper_name)?;
        let result = CryptOpenResult {
            mapper_name: request.mapper_name.clone(),
            mapped_path,
            key_slot,
        };
        self.commands.push(CryptsetupCommand::open(
            request.dev_path,
            request.mapper_name.clone(),
            request.key_slot,
            request.passphrase.len(),
        ));
        self.opened.insert(request.mapper_name, result.clone());
        Ok(result)
    }

    fn close(&mut self, request: LuksCloseRequest) -> Result<()> {
        mapper_path(&request.mapper_name)?;
        if self.opened.remove(&request.mapper_name).is_none() {
            return Err(BlockError::NotFound(format!(
                "mapper {} is not open",
                request.mapper_name
            )));
        }
        self.commands
            .push(CryptsetupCommand::close(request.mapper_name));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cryptsetup_command_builders_are_source_shaped() {
        assert_eq!(mapper_name("DATA").unwrap(), "luks2-DATA");
        assert_eq!(mapper_path("luks2-DATA").unwrap(), "/dev/mapper/luks2-DATA");
        assert!(mapper_name("bad/name").is_err());
        assert!(mapper_name("bad name").is_err());
        assert!(mapper_path("bad/name").is_err());

        assert_eq!(
            CryptsetupCommand::luks_format("/dev/sda1", Cipher::AesXtsPlain64, 3, 12),
            CryptsetupCommand {
                program: "cryptsetup".to_string(),
                args: vec![
                    "luksFormat",
                    "--type",
                    "luks2",
                    "--cipher",
                    "aes-xts-plain64",
                    "--key-size",
                    "512",
                    "--key-slot",
                    "3",
                    "--key-file",
                    "-",
                    "--batch-mode",
                    "/dev/sda1",
                ]
                .into_iter()
                .map(str::to_string)
                .collect(),
                stdin_bytes: 12,
            }
        );
        assert_eq!(
            CryptsetupCommand::open("/dev/sda1", "luks2-DATA", 3, 12).args,
            vec![
                "open",
                "--type",
                "luks2",
                "--key-slot",
                "3",
                "--key-file",
                "-",
                "/dev/sda1",
                "luks2-DATA",
            ]
        );
        assert_eq!(
            CryptsetupCommand::luks_add_key("/dev/sda1", 4, 7, 9),
            CryptsetupCommand {
                program: "cryptsetup".to_string(),
                args: vec![
                    "luksAddKey",
                    "--key-slot",
                    "4",
                    "--key-file",
                    "-",
                    "--batch-mode",
                    "/dev/sda1",
                ]
                .into_iter()
                .map(str::to_string)
                .collect(),
                stdin_bytes: 16,
            }
        );
        assert_eq!(
            CryptsetupCommand::close("luks2-DATA"),
            CryptsetupCommand {
                program: "cryptsetup".to_string(),
                args: vec!["close".to_string(), "luks2-DATA".to_string()],
                stdin_bytes: 0,
            }
        );
    }

    #[test]
    fn mem_cryptsetup_backend_formats_adds_opens_and_closes() {
        let mut backend = MemCryptsetupBackend::default();
        backend
            .format(LuksFormatRequest {
                dev_path: "/dev/sda1".to_string(),
                uuid: "DATA:/dev/sda1".to_string(),
                cipher: Cipher::AesXtsPlain64,
                key_slot: 1,
                passphrase: b"primary".to_vec(),
            })
            .unwrap();
        backend
            .add_key(LuksAddKeyRequest {
                dev_path: "/dev/sda1".to_string(),
                existing_passphrase: b"primary".to_vec(),
                new_key_slot: 4,
                new_passphrase: b"secondary".to_vec(),
            })
            .unwrap();
        let opened = backend
            .open(LuksOpenRequest {
                dev_path: "/dev/sda1".to_string(),
                mapper_name: "luks2-DATA".to_string(),
                key_slot: 1,
                passphrase: b"primary".to_vec(),
            })
            .unwrap();

        assert_eq!(opened.mapped_path, "/dev/mapper/luks2-DATA");
        assert_eq!(opened.key_slot, 1);
        let header = backend.header("/dev/sda1").unwrap();
        assert_eq!(header.active_slots(), 2);
        assert_eq!(header.open(b"secondary").unwrap(), 4);
        assert!(backend.opened("luks2-DATA").is_some());
        assert_eq!(
            backend
                .commands()
                .iter()
                .map(|cmd| &cmd.args[0])
                .collect::<Vec<_>>(),
            vec!["luksFormat", "luksAddKey", "open"]
        );
        assert!(
            backend
                .open(LuksOpenRequest {
                    dev_path: "/dev/sda1".to_string(),
                    mapper_name: "luks2-DATA".to_string(),
                    key_slot: 1,
                    passphrase: b"primary".to_vec(),
                })
                .is_err()
        );

        backend
            .close(LuksCloseRequest {
                mapper_name: "luks2-DATA".to_string(),
            })
            .unwrap();
        assert!(backend.opened("luks2-DATA").is_none());
        assert_eq!(
            backend.commands().last().unwrap().args,
            vec!["close", "luks2-DATA"]
        );
        assert!(
            backend
                .close(LuksCloseRequest {
                    mapper_name: "luks2-DATA".to_string(),
                })
                .is_err()
        );
    }
}
