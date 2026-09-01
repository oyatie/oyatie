#[cfg(test)]
mod filesystem_publisher_tests {
    use super::*;
    use std::fs::{self, File};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn real_transaction_is_atomic_idempotent_and_mode_exact() {
        let fixture = Fixture::new();
        let publisher = FilesystemPublisherV1::from_directory(
            fixture.open_directory(),
            PublisherProfileV1::MacosApfsV1,
        );
        fs::write(fixture.path().join(STAGE_NAME), b"abandoned").unwrap();
        let first = publisher.publish_content(None, b"new");
        assert_eq!(first, PublicationOutcomeV1::Replaced);
        assert_eq!(fs::read(fixture.path().join("BUCK")).unwrap(), b"new");
        assert_eq!(fixture.destination_mode(), 0o644);

        fs::write(fixture.path().join(STAGE_NAME), b"abandoned").unwrap();
        let unchanged = publisher.publish_content(Some(DigestV1::of(b"new")), b"new");
        assert_eq!(unchanged, PublicationOutcomeV1::Unchanged);
        assert_eq!(fixture.entry_names(), ["BUCK"]);

        let conflict = publisher.publish_content(Some(DigestV1::of(b"stale")), b"other");
        assert_failed(conflict, FailureClassV1::DestinationConflict);
        assert_eq!(fs::read(fixture.path().join("BUCK")).unwrap(), b"new");
    }

    #[test]
    fn output_and_preimage_bounds_refuse_before_staging() {
        let output_fixture = Fixture::new();
        let publisher = FilesystemPublisherV1::from_directory(
            output_fixture.open_directory(),
            PublisherProfileV1::MacosApfsV1,
        );
        let oversized = vec![0; ValidationBoundsV1::MAX_OUTPUT_BYTES + 1];
        assert_failed(
            publisher.publish_content(None, &oversized),
            FailureClassV1::InternalInvariant,
        );
        assert!(output_fixture.entry_names().is_empty());

        let preimage_fixture = Fixture::new();
        let preimage = File::create(preimage_fixture.path().join("BUCK")).unwrap();
        rustix::fs::fchmod(&preimage, destination_mode()).unwrap();
        preimage
            .set_len(ValidationBoundsV1::MAX_OUTPUT_BYTES as u64 + 1)
            .unwrap();
        drop(preimage);
        let publisher = FilesystemPublisherV1::from_directory(
            preimage_fixture.open_directory(),
            PublisherProfileV1::MacosApfsV1,
        );
        assert_failed(
            publisher.publish_content(None, b"new"),
            FailureClassV1::DestinationConflict,
        );
        assert_eq!(preimage_fixture.entry_names(), ["BUCK"]);
    }

    #[cfg(unix)]
    #[test]
    fn destination_symlink_and_busy_directory_lease_fail_closed() {
        let symlink_fixture = Fixture::new();
        fs::write(symlink_fixture.path().join("actual"), b"old").unwrap();
        std::os::unix::fs::symlink("actual", symlink_fixture.path().join("BUCK")).unwrap();
        let publisher = FilesystemPublisherV1::from_directory(
            symlink_fixture.open_directory(),
            PublisherProfileV1::MacosApfsV1,
        );
        assert_failed(
            publisher.publish_content(None, b"new"),
            FailureClassV1::DestinationConflict,
        );
        assert_eq!(fs::read(symlink_fixture.path().join("actual")).unwrap(), b"old");

        let lease_fixture = Fixture::new();
        let held = lease_fixture.open_directory();
        rustix::fs::flock(&held, rustix::fs::FlockOperation::LockExclusive).unwrap();
        let publisher = FilesystemPublisherV1::from_directory(
            lease_fixture.open_directory(),
            PublisherProfileV1::MacosApfsV1,
        );
        assert_failed(
            publisher.publish_content(None, b"new"),
            FailureClassV1::DestinationLeaseUnavailable,
        );
        assert!(!lease_fixture.path().join("BUCK").exists());
        rustix::fs::flock(&held, rustix::fs::FlockOperation::Unlock).unwrap();
    }

    #[test]
    fn constructor_requires_a_directory_and_the_observed_filesystem_profile() {
        let fixture = Fixture::new();
        let regular_path = fixture.path().join("regular");
        fs::write(&regular_path, b"not a directory").unwrap();
        assert!(matches!(
            FilesystemPublisherV1::try_new(
                File::open(&regular_path).unwrap(),
                PublisherProfileV1::MacosApfsV1,
            ),
            Err(FilesystemPublisherConfigurationFailureV1::NotDirectory)
        ));

        let observed = detected_publisher_profile(&fixture.open_directory()).unwrap();
        for profile in [
            PublisherProfileV1::LinuxExt4V1,
            PublisherProfileV1::LinuxXfsV1,
            PublisherProfileV1::MacosApfsV1,
        ] {
            let admitted = FilesystemPublisherV1::try_new(fixture.open_directory(), profile);
            assert_eq!(admitted.is_ok(), observed == Some(profile));
            if let Ok(publisher) = admitted {
                assert!(publisher.supports(&profile));
            }
        }
    }

    #[test]
    fn linux_profile_evidence_requires_an_exact_unique_mount() {
        let mountinfo = b"31 23 0:27 / / rw - ext4 /dev/root rw\n\
                          32 23 0:28 / /work rw - xfs /dev/work rw\n\
                          33 23 0:29 / /tmp rw - tmpfs tmpfs rw\n";
        assert_eq!(
            publisher_profile_from_mountinfo(mountinfo, 31),
            Ok(Some(PublisherProfileV1::LinuxExt4V1))
        );
        assert_eq!(
            publisher_profile_from_mountinfo(mountinfo, 32),
            Ok(Some(PublisherProfileV1::LinuxXfsV1))
        );
        assert_eq!(publisher_profile_from_mountinfo(mountinfo, 33), Ok(None));
        assert_eq!(publisher_profile_from_mountinfo(mountinfo, 34), Err(()));

        let duplicate = b"31 23 0:27 / / rw - ext4 /dev/root rw\n\
                          31 23 0:27 / /bind rw - ext4 /dev/root rw\n";
        assert_eq!(publisher_profile_from_mountinfo(duplicate, 31), Err(()));
    }

    fn assert_failed(outcome: PublicationOutcomeV1, expected: FailureClassV1) {
        assert!(matches!(
            outcome,
            PublicationOutcomeV1::Failed {
                failure,
                replacement: ReplacementStateV1::No,
            } if failure.class() == expected
        ));
    }

    static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

    struct Fixture {
        root: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            loop {
                let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
                let root = std::env::temp_dir().join(format!(
                    "dependency-declarations-filesystem-publication-{}-{sequence}",
                    std::process::id()
                ));
                match fs::create_dir(&root) {
                    Ok(()) => return Self { root },
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(error) => panic!("cannot create publication fixture: {error}"),
                }
            }
        }

        fn path(&self) -> &Path {
            &self.root
        }

        fn open_directory(&self) -> File {
            File::open(&self.root).unwrap()
        }

        fn entry_names(&self) -> Vec<String> {
            let mut names: Vec<_> = fs::read_dir(&self.root)
                .unwrap()
                .map(|entry| entry.unwrap().file_name().into_string().unwrap())
                .collect();
            names.sort();
            names
        }

        #[cfg(unix)]
        fn destination_mode(&self) -> u32 {
            use std::os::unix::fs::PermissionsExt as _;
            fs::metadata(self.root.join("BUCK"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }
}
