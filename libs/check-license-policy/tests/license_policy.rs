use check_license_policy::{LicensePolicy, LicensePolicyError};

#[test]
fn product_license_policy_accepts_adr_allowed_spdx_identifiers() {
    let policy = LicensePolicy::adr_0013_product_policy();

    assert_eq!(policy.validate_product_license("Apache-2.0"), Ok(()));
    assert_eq!(
        policy.validate_product_license("Apache-2.0 WITH LLVM-exception"),
        Ok(())
    );
    assert_eq!(
        policy.validate_product_license("MIT OR BSD-3-Clause"),
        Ok(())
    );
}

#[test]
fn product_license_policy_blocks_forbidden_and_review_tier_identifiers() {
    let policy = LicensePolicy::adr_0013_product_policy();

    assert_eq!(
        policy.validate_product_license("GPL-3.0"),
        Err(LicensePolicyError::ForbiddenLicense)
    );
    assert_eq!(
        policy.validate_product_license("MIT OR AGPL-3.0"),
        Err(LicensePolicyError::ForbiddenLicense)
    );
    assert_eq!(
        policy.validate_product_license("LGPL-3.0"),
        Err(LicensePolicyError::ReviewRequired)
    );
}

#[test]
fn product_license_policy_rejects_missing_or_unknown_identifiers() {
    let policy = LicensePolicy::adr_0013_product_policy();

    assert_eq!(
        policy.validate_product_license(""),
        Err(LicensePolicyError::MissingLicense)
    );
    assert_eq!(
        policy.validate_product_license("Vendor-Commercial"),
        Err(LicensePolicyError::UnknownLicense)
    );
}
