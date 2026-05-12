//! Foundry license-policy kernel.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LicensePolicyError {
    MissingLicense,
    UnknownLicense,
    ForbiddenLicense,
    ReviewRequired,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LicensePolicy;

impl LicensePolicy {
    pub fn adr_0013_product_policy() -> Self {
        Self
    }

    pub fn validate_product_license(&self, license: &str) -> Result<(), LicensePolicyError> {
        let identifiers = spdx_identifiers(license)?;
        let mut requires_review = false;
        for identifier in identifiers {
            if is_forbidden(identifier) {
                return Err(LicensePolicyError::ForbiddenLicense);
            }
            if is_review_required(identifier) {
                requires_review = true;
                continue;
            }
            if !is_allowed(identifier) {
                return Err(LicensePolicyError::UnknownLicense);
            }
        }
        if requires_review {
            Err(LicensePolicyError::ReviewRequired)
        } else {
            Ok(())
        }
    }
}

fn spdx_identifiers(license: &str) -> Result<Vec<&str>, LicensePolicyError> {
    let trimmed = license.trim();
    if trimmed.is_empty() {
        return Err(LicensePolicyError::MissingLicense);
    }
    if trimmed == "Apache-2.0 WITH LLVM-exception" {
        return Ok(vec![trimmed]);
    }
    let identifiers = trimmed
        .split(|character: char| character.is_whitespace() || matches!(character, '(' | ')'))
        .filter(|token| !token.is_empty())
        .filter(|token| !matches!(*token, "AND" | "OR" | "WITH"))
        .collect::<Vec<_>>();
    if identifiers.is_empty() {
        Err(LicensePolicyError::MissingLicense)
    } else {
        Ok(identifiers)
    }
}

fn is_allowed(identifier: &str) -> bool {
    matches!(
        identifier,
        "Apache-2.0"
            | "Apache-2.0 WITH LLVM-exception"
            | "MIT"
            | "MIT-0"
            | "BSD-2-Clause"
            | "BSD-3-Clause"
            | "BSD-3-Clause-Clear"
            | "ISC"
            | "0BSD"
            | "Unlicense"
            | "CC0-1.0"
            | "MPL-2.0"
            | "Unicode-DFS-2016"
            | "Unicode-3.0"
            | "Zlib"
            | "libpng-2.0"
    )
}

fn is_forbidden(identifier: &str) -> bool {
    matches!(
        identifier,
        "GPL-2.0"
            | "GPL-3.0"
            | "GPL-2.0-or-later"
            | "GPL-3.0-or-later"
            | "AGPL-3.0"
            | "AGPL-3.0-or-later"
    )
}

fn is_review_required(identifier: &str) -> bool {
    matches!(
        identifier,
        "LGPL-2.0"
            | "LGPL-2.1"
            | "LGPL-3.0"
            | "SSPL-1.0"
            | "BUSL-1.1"
            | "Elastic-2.0"
            | "RSAL-1.0"
            | "TSL-2.0"
            | "FSL-1.1"
    )
}
