//! Claim-push successor stub (mechanical claim before push).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimReceipt {
    pub integ: String,
    pub tip_sha: String,
}

/// Validate a minimal claim receipt shape (full envelope check lands with claim-mechanical).
pub fn validate_claim_receipt(integ: &str, tip_sha: &str) -> Result<ClaimReceipt, String> {
    if integ.is_empty() || !integ.starts_with("integ/") {
        return Err("claim-push: REFUSE — integ must look like integ/<root>".to_string());
    }
    if tip_sha.len() < 7 || !tip_sha.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("claim-push: REFUSE — tip_sha must be hex".to_string());
    }
    Ok(ClaimReceipt {
        integ: integ.to_string(),
        tip_sha: tip_sha.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_minimal() {
        assert!(validate_claim_receipt("integ/ci", "abc1234").is_ok());
        assert!(validate_claim_receipt("ci", "abc1234").is_err());
        assert!(validate_claim_receipt("integ/ci", "zzzz").is_err());
    }
}
