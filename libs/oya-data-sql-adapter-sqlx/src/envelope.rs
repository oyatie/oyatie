//! Envelope-encrypted tenant-row write helper for the owned data SQL port.

#[cfg(test)]
mod tests {
    use super::*;
    use oya_data_sql_kernel::clock::HlcTimestamp;
    use oya_data_sql_kernel::{
        CommitReceipt, DataSession, DataSqlError, DataStore, ReadQuery, RowSet, SessionDescriptor,
        SqlValue, WriteBatch,
    };

    #[derive(Debug)]
    struct RecordingSession {
        descriptor: SessionDescriptor,
        committed: Vec<WriteBatch>,
    }

    impl RecordingSession {
        fn tenant(tenant_id: &str) -> Self {
            Self {
                descriptor: SessionDescriptor::tenant_data(
                    tenant_id,
                    "cell-001",
                    "oya-data-envelope-test",
                )
                .unwrap(),
                committed: Vec::new(),
            }
        }
    }

    impl DataSession for RecordingSession {
        fn descriptor(&self) -> &SessionDescriptor {
            &self.descriptor
        }

        fn execute_write(&mut self, batch: &WriteBatch) -> Result<CommitReceipt, DataSqlError> {
            self.committed.push(batch.clone());
            Ok(CommitReceipt {
                store: DataStore::TenantData,
                commit_timestamp: HlcTimestamp::new(self.committed.len() as u64, 0),
                statement_names: batch.statement_names(),
            })
        }

        fn execute_read(&mut self, _query: &ReadQuery) -> Result<RowSet, DataSqlError> {
            Ok(RowSet::default())
        }
    }

    #[derive(Default)]
    struct RotatingKmsFixture {
        version: u32,
        calls: usize,
    }

    impl RotatingKmsFixture {
        fn rotate(&mut self) {
            self.version += 1;
        }
    }

    impl EnvelopeKmsPort for RotatingKmsFixture {
        fn encrypt(
            &mut self,
            request: EnvelopeEncryptRequest<'_>,
        ) -> Result<EnvelopeEncryptedPayload, EnvelopeKmsError> {
            self.calls += 1;
            let version = self.version.max(1);
            let ciphertext = request
                .plaintext
                .iter()
                .rev()
                .map(|byte| byte ^ (version as u8))
                .collect();
            Ok(EnvelopeEncryptedPayload {
                kms_key_id: request.kms_key_id.to_owned(),
                key_version: version,
                wrapped_dek: format!("wrapped-dek-v{version}").into_bytes(),
                ciphertext,
            })
        }
    }

    #[test]
    fn quota_row_write_uses_rotated_kms_versions_without_plaintext_params() {
        let mut kms = RotatingKmsFixture { version: 1, calls: 0 };
        let mut session = RecordingSession::tenant("ten_acme");
        let first = EnvelopeTenantRowWrite::new(
            "ten_acme",
            "quota/ten_acme",
            "kms/us-east-1/ten_acme/quota",
            b"quota:clusters=5;nodes=10",
        )
        .unwrap();
        write_encrypted_tenant_row(&mut session, &mut kms, &first).unwrap();

        kms.rotate();
        let second = EnvelopeTenantRowWrite::new(
            "ten_acme",
            "quota/ten_acme/v2",
            "kms/us-east-1/ten_acme/quota",
            b"quota:clusters=7;nodes=12",
        )
        .unwrap();
        write_encrypted_tenant_row(&mut session, &mut kms, &second).unwrap();

        assert_eq!(kms.calls, 2);
        assert_eq!(session.committed.len(), 2);
        assert_eq!(stored_key_version(&session.committed[0]), 1);
        assert_eq!(stored_key_version(&session.committed[1]), 2);
        assert_ne!(stored_wrapped_dek(&session.committed[0]), stored_wrapped_dek(&session.committed[1]));
        assert_no_param_contains_plaintext(&session.committed, b"quota:clusters");
    }

    fn stored_key_version(batch: &WriteBatch) -> i64 {
        match &batch.statements[0].params[3] {
            SqlValue::Int64(value) => *value,
            other => panic!("expected key version param, got {other:?}"),
        }
    }

    fn stored_wrapped_dek(batch: &WriteBatch) -> &[u8] {
        match &batch.statements[0].params[4] {
            SqlValue::Bytes(value) => value,
            other => panic!("expected wrapped DEK param, got {other:?}"),
        }
    }

    fn assert_no_param_contains_plaintext(batches: &[WriteBatch], needle: &[u8]) {
        for batch in batches {
            for statement in &batch.statements {
                for param in &statement.params {
                    match param {
                        SqlValue::Text(value) => assert!(!value.as_bytes().windows(needle.len()).any(|window| window == needle)),
                        SqlValue::Bytes(value) => assert!(!value.windows(needle.len()).any(|window| window == needle)),
                        _ => {}
                    }
                }
            }
        }
    }
}
