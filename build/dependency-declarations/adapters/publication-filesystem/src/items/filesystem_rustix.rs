const DESTINATION_NAME: &str = "BUCK";
const STAGE_NAME: &str = ".BUCK.declaration-stage";

#[cfg(target_os = "linux")]
use rustix::fs::{StatxFlags, statx};

struct DirectoryLeaseV1<'a> {
    directory: &'a File,
}

impl<'a> DirectoryLeaseV1<'a> {
    fn try_acquire(directory: &'a File) -> Result<Self, ()> {
        flock(directory, FlockOperation::NonBlockingLockExclusive).map_err(|_| ())?;
        Ok(Self { directory })
    }
}

impl Drop for DirectoryLeaseV1<'_> {
    fn drop(&mut self) {
        let _ = flock(self.directory, FlockOperation::Unlock);
    }
}

struct RustixPublicationTransactionV1<'a> {
    directory: &'a File,
    stage: Option<File>,
}

impl<'a> RustixPublicationTransactionV1<'a> {
    fn new(directory: &'a File) -> Self {
        Self {
            directory,
            stage: None,
        }
    }
}

impl PublicationTransactionV1 for RustixPublicationTransactionV1<'_> {
    fn read_destination(&mut self) -> Result<Option<Vec<u8>>, FailureClassV1> {
        let descriptor = match openat(
            self.directory,
            DESTINATION_NAME,
            OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
            Mode::empty(),
        ) {
            Ok(descriptor) => descriptor,
            Err(rustix::io::Errno::NOENT) => return Ok(None),
            Err(_) => return Err(FailureClassV1::DestinationConflict),
        };
        let stat = fstat(&descriptor).map_err(|_| FailureClassV1::DestinationConflict)?;
        if FileType::from_raw_mode(stat.st_mode) != FileType::RegularFile
            || Mode::from_raw_mode(stat.st_mode) != destination_mode()
        {
            return Err(FailureClassV1::DestinationConflict);
        }
        let length = usize::try_from(stat.st_size)
            .ok()
            .filter(|length| *length <= ValidationBoundsV1::MAX_OUTPUT_BYTES)
            .ok_or(FailureClassV1::DestinationConflict)?;
        let file = File::from(descriptor);
        let mut bytes = Vec::with_capacity(length);
        file.take(ValidationBoundsV1::MAX_OUTPUT_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| FailureClassV1::DestinationConflict)?;
        if bytes.len() != length {
            return Err(FailureClassV1::DestinationConflict);
        }
        Ok(Some(bytes))
    }

    fn write_stage(&mut self, bytes: &[u8]) -> Result<(), FailureClassV1> {
        match unlinkat(self.directory, STAGE_NAME, AtFlags::empty()) {
            Ok(()) | Err(rustix::io::Errno::NOENT) => {}
            Err(_) => return Err(FailureClassV1::StageWriteFailed),
        }
        let descriptor = openat(
            self.directory,
            STAGE_NAME,
            OFlags::WRONLY
                | OFlags::CREATE
                | OFlags::EXCL
                | OFlags::CLOEXEC
                | OFlags::NOFOLLOW,
            destination_mode(),
        )
        .map_err(|_| FailureClassV1::StageWriteFailed)?;
        self.stage = Some(File::from(descriptor));
        let stage = self
            .stage
            .as_mut()
            .ok_or(FailureClassV1::InternalInvariant)?;
        fchmod(&*stage, destination_mode()).map_err(|_| FailureClassV1::StageWriteFailed)?;
        stage
            .write_all(bytes)
            .map_err(|_| FailureClassV1::StageWriteFailed)?;
        Ok(())
    }

    fn sync_stage(&mut self) -> Result<(), FailureClassV1> {
        let stage = self
            .stage
            .as_ref()
            .ok_or(FailureClassV1::InternalInvariant)?;
        sync_stage_file(stage).map_err(|_| FailureClassV1::StageSyncFailed)
    }

    fn ensure_lease(&mut self) -> Result<(), FailureClassV1> {
        flock(self.directory, FlockOperation::NonBlockingLockExclusive)
            .map_err(|_| FailureClassV1::LeaseLost)
    }

    fn replace(&mut self) -> Result<(), FailureClassV1> {
        renameat(
            self.directory,
            STAGE_NAME,
            self.directory,
            DESTINATION_NAME,
        )
        .map_err(|_| FailureClassV1::ReplaceFailed)?;
        self.stage = None;
        Ok(())
    }

    fn sync_directory(&mut self) -> Result<(), FailureClassV1> {
        fsync(self.directory).map_err(|_| FailureClassV1::DirectorySyncFailed)
    }

    fn discard_stage(&mut self) -> Result<(), FailureClassV1> {
        self.stage = None;
        match unlinkat(self.directory, STAGE_NAME, AtFlags::empty()) {
            Ok(()) => {}
            Err(rustix::io::Errno::NOENT) => return Ok(()),
            Err(_) => return Err(FailureClassV1::StageCleanupFailed),
        }
        fsync(self.directory).map_err(|_| FailureClassV1::StageCleanupFailed)
    }
}

fn destination_mode() -> Mode {
    Mode::RUSR | Mode::WUSR | Mode::RGRP | Mode::ROTH
}

#[cfg(target_os = "macos")]
fn sync_stage_file(file: &File) -> rustix::io::Result<()> {
    rustix::fs::fcntl_fullfsync(file)
}

#[cfg(not(target_os = "macos"))]
fn sync_stage_file(file: &File) -> rustix::io::Result<()> {
    fsync(file)
}

#[cfg(target_os = "linux")]
fn detected_publisher_profile(directory: &File) -> Result<Option<PublisherProfileV1>, ()> {
    const EXT_FAMILY_MAGIC: u64 = 0xef53;
    const XFS_MAGIC: u64 = 0x5846_5342;
    let filesystem = fstatfs(directory).map_err(|_| ())?;
    let stat = statx(
        directory,
        "",
        AtFlags::EMPTY_PATH,
        StatxFlags::MNT_ID,
    )
    .map_err(|_| ())?;
    if !StatxFlags::from_bits_retain(stat.stx_mask).contains(StatxFlags::MNT_ID) {
        return Err(());
    }

    const MAX_MOUNTINFO_BYTES: u64 = 1024 * 1024;
    let mut mountinfo = Vec::new();
    File::open("/proc/self/mountinfo")
        .map_err(|_| ())?
        .take(MAX_MOUNTINFO_BYTES + 1)
        .read_to_end(&mut mountinfo)
        .map_err(|_| ())?;
    if mountinfo.len() as u64 > MAX_MOUNTINFO_BYTES {
        return Err(());
    }
    let observed = publisher_profile_from_mountinfo(&mountinfo, stat.stx_mnt_id)?;
    Ok(match (filesystem.f_type as u64, observed) {
        (EXT_FAMILY_MAGIC, Some(PublisherProfileV1::LinuxExt4V1))
        | (XFS_MAGIC, Some(PublisherProfileV1::LinuxXfsV1)) => observed,
        _ => None,
    })
}

#[cfg(any(target_os = "linux", test))]
fn publisher_profile_from_mountinfo(
    mountinfo: &[u8],
    mount_id: u64,
) -> Result<Option<PublisherProfileV1>, ()> {
    let mountinfo = std::str::from_utf8(mountinfo).map_err(|_| ())?;
    let mut matched = None;
    for line in mountinfo.lines() {
        let mut fields = line.split_ascii_whitespace();
        let current_id = fields.next().ok_or(())?.parse::<u64>().map_err(|_| ())?;
        if current_id != mount_id {
            continue;
        }
        if matched.is_some() {
            return Err(());
        }
        if !fields.any(|field| field == "-") {
            return Err(());
        }
        matched = Some(match fields.next().ok_or(())? {
            "ext4" => Some(PublisherProfileV1::LinuxExt4V1),
            "xfs" => Some(PublisherProfileV1::LinuxXfsV1),
            _ => None,
        });
    }
    matched.ok_or(())
}

#[cfg(target_os = "macos")]
fn detected_publisher_profile(directory: &File) -> Result<Option<PublisherProfileV1>, ()> {
    let filesystem = fstatfs(directory).map_err(|_| ())?;
    let name = filesystem
        .f_fstypename
        .iter()
        .copied()
        .map(|byte| byte as u8)
        .take_while(|byte| *byte != 0);
    Ok(name
        .eq(b"apfs".iter().copied())
        .then_some(PublisherProfileV1::MacosApfsV1))
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn detected_publisher_profile(_: &File) -> Result<Option<PublisherProfileV1>, ()> {
    Ok(None)
}
