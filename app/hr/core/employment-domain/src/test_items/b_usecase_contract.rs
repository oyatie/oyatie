#[cfg(test)]
mod usecase_contract {
    use data_boundary_kernel::DataClass;

    use super::{SensitiveHrDataKind, sensitive_read_payload_data_class};

    #[test]
    fn sensitive_payload_kind_selects_the_required_privacy_class() {
        // Catches a payload-kind branch being assigned the wrong privacy class.
        let cases = [
            (SensitiveHrDataKind::Medical, DataClass::Phi),
            (SensitiveHrDataKind::DisabilityAccommodation, DataClass::Phi),
            (SensitiveHrDataKind::Compensation, DataClass::Financial),
            (
                SensitiveHrDataKind::GovernmentIdentifier,
                DataClass::PiiIdentifying,
            ),
            (
                SensitiveHrDataKind::Disciplinary,
                DataClass::SensitivePipaArticle23,
            ),
        ];

        for (data_kind, expected) in cases {
            assert_eq!(sensitive_read_payload_data_class(data_kind), expected);
        }
    }
}
