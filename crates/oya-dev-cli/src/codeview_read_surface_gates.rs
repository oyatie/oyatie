use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CodeviewReadSurfaceValidateArgs {
    spec: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CodeviewReadSurfaceReport {
    pub commands_checked: usize,
    pub compatibility_binaries_checked: usize,
    pub provider_env_vars_checked: usize,
    pub rejected_tokens_checked: usize,
}

pub(crate) fn parse_codeview_read_surface_validate_args(
    args: Vec<String>,
) -> Result<CodeviewReadSurfaceValidateArgs, String> {
    let mut spec = PathBuf::from(".omc/specs/codeview-read-surface.json");
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--spec" => {
                spec = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--spec requires a path".to_string())?,
                );
            }
            other => {
                return Err(format!(
                    "unexpected codeview-read-surface argument: {other}"
                ));
            }
        }
    }
    Ok(CodeviewReadSurfaceValidateArgs { spec })
}

pub(crate) fn validate_codeview_read_surface_gate(
    args: CodeviewReadSurfaceValidateArgs,
) -> Result<CodeviewReadSurfaceReport, String> {
    let content = fs::read_to_string(&args.spec)
        .map_err(|error| format!("{}: {error}", args.spec.display()))?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|error| format!("{}: invalid JSON: {error}", args.spec.display()))?;

    let commands_checked = array_len(&json, "commands")?;
    let compatibility_binaries_checked = array_len(&json, "compatibility_binaries")?;
    let provider_env_vars_checked = array_len(&json, "provider_env_vars")?;
    let rejected_tokens_checked = array_len(&json, "rejected_tokens")?;

    if commands_checked == 0
        || compatibility_binaries_checked == 0
        || provider_env_vars_checked == 0
        || rejected_tokens_checked == 0
    {
        return Err("codeview read-surface spec is incomplete".to_string());
    }

    Ok(CodeviewReadSurfaceReport {
        commands_checked,
        compatibility_binaries_checked,
        provider_env_vars_checked,
        rejected_tokens_checked,
    })
}

fn array_len(json: &serde_json::Value, field: &str) -> Result<usize, String> {
    json.get(field)
        .and_then(serde_json::Value::as_array)
        .map(std::vec::Vec::len)
        .ok_or_else(|| format!("codeview read-surface spec missing array field: {field}"))
}
