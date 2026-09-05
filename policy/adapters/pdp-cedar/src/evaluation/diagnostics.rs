use shared_pdp_kernel::PdpError;

pub(super) fn qualification_diagnostic_result<I, E>(diagnostics: I) -> Result<(), PdpError>
where
    I: IntoIterator<Item = E>,
    E: ToString,
{
    let mut diagnostics: Vec<String> = diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.to_string())
        .collect();
    if diagnostics.is_empty() {
        return Ok(());
    }
    diagnostics.sort();
    Err(PdpError::Evaluation {
        detail: format!("Cedar evaluation diagnostics: {}", diagnostics.join("; ")),
    })
}

#[cfg(test)]
mod tests {
    use super::qualification_diagnostic_result;
    use shared_pdp_kernel::PdpError;

    #[test]
    fn qualification_accepts_an_error_free_evaluation() {
        assert_eq!(
            qualification_diagnostic_result(Vec::<String>::new()),
            Ok(())
        );
    }

    #[test]
    fn qualification_refuses_every_diagnostic_in_deterministic_order() {
        assert_eq!(
            qualification_diagnostic_result(vec!["policy-z failed", "policy-a failed"]),
            Err(PdpError::Evaluation {
                detail: "Cedar evaluation diagnostics: policy-a failed; policy-z failed".into(),
            })
        );
    }
}
