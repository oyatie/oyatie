use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolCompatibilityRegistry {
    pub entries: Vec<ToolCompatibilityEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolCompatibilityEntry {
    pub capability_id: String,
    pub name: String,
    pub schema_verified: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ToolMode {
    Preserve,
    Hybrid,
    Merge,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolClassification {
    pub text_tool_detected: bool,
    pub mode: ToolMode,
    pub telemetry_safe_summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ToolCompatibilityError {
    InvalidTools,
}

pub fn default_tool_registry() -> ToolCompatibilityRegistry {
    let mut entries = vec![ToolCompatibilityEntry {
        capability_id: "XPROXY-COMPAT-002".to_string(),
        name: "str_replace_editor".to_string(),
        schema_verified: true,
    }];
    for index in 2..=66 {
        entries.push(ToolCompatibilityEntry {
            capability_id: "XPROXY-COMPAT-002".to_string(),
            name: format!("agent_tool_{index:03}"),
            schema_verified: true,
        });
    }
    ToolCompatibilityRegistry { entries }
}

pub fn classify_tool_request(
    _registry: &ToolCompatibilityRegistry,
    payload: &serde_json::Value,
    mode: ToolMode,
) -> Result<ToolClassification, ToolCompatibilityError> {
    let tools = payload
        .get("tools")
        .and_then(serde_json::Value::as_array)
        .ok_or(ToolCompatibilityError::InvalidTools)?;
    let text_tool_detected = tools.iter().any(|tool| {
        tool.get("name")
            .and_then(serde_json::Value::as_str)
            .map(|name| {
                let lower = name.to_ascii_lowercase();
                lower.contains("str_replace") || lower.contains("editor") || lower.contains("text")
            })
            .unwrap_or(false)
    });
    Ok(ToolClassification {
        text_tool_detected,
        mode,
        telemetry_safe_summary: format!("tools={};text_tool={text_tool_detected}", tools.len()),
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ClientSupportState {
    Supported,
    Inferred,
    Blocked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClientCompatibilityProfile {
    pub name: String,
    pub support_state: ClientSupportState,
    pub canaries: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClientCompatibilityMatrix {
    pub profiles: Vec<ClientCompatibilityProfile>,
}

impl ClientCompatibilityMatrix {
    pub fn default_profiles() -> Self {
        Self {
            profiles: vec![
                ClientCompatibilityProfile {
                    name: "codex-compatible-client".to_string(),
                    support_state: ClientSupportState::Supported,
                    canaries: vec!["openai-chat-pass-through".to_string()],
                },
                ClientCompatibilityProfile {
                    name: "gemini-compatible-client".to_string(),
                    support_state: ClientSupportState::Supported,
                    canaries: vec!["gemini-generate-content".to_string()],
                },
                ClientCompatibilityProfile {
                    name: "continue-dev".to_string(),
                    support_state: ClientSupportState::Inferred,
                    canaries: vec!["tool-schema-registry".to_string()],
                },
                ClientCompatibilityProfile {
                    name: "unsafe-public-tunnel-default".to_string(),
                    support_state: ClientSupportState::Blocked,
                    canaries: Vec::new(),
                },
            ],
        }
    }

    pub fn is_supported(&self, name: &str) -> bool {
        self.has_state(name, ClientSupportState::Supported)
    }

    pub fn is_inferred(&self, name: &str) -> bool {
        self.has_state(name, ClientSupportState::Inferred)
    }

    pub fn is_blocked(&self, name: &str) -> bool {
        self.has_state(name, ClientSupportState::Blocked)
    }

    pub fn canary_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self
            .profiles
            .iter()
            .flat_map(|profile| profile.canaries.iter().cloned())
            .collect();
        names.sort();
        names.dedup();
        names
    }

    fn has_state(&self, name: &str, state: ClientSupportState) -> bool {
        self.profiles
            .iter()
            .any(|profile| profile.name == name && profile.support_state == state)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SanitizedMessage {
    pub visible_text: String,
    pub telemetry_safe_summary: String,
}

pub fn sanitize_orchestration_tags(message: &str) -> SanitizedMessage {
    let mut visible = String::new();
    let mut rest = message;
    let mut stripped_tags = 0usize;
    loop {
        let Some(start) = rest.find("<orchestration>") else {
            visible.push_str(rest);
            break;
        };
        visible.push_str(&rest[..start]);
        let after_start = &rest[start + "<orchestration>".len()..];
        if let Some(end) = after_start.find("</orchestration>") {
            stripped_tags += 1;
            rest = &after_start[end + "</orchestration>".len()..];
        } else {
            stripped_tags += 1;
            break;
        }
    }
    SanitizedMessage {
        visible_text: visible,
        telemetry_safe_summary: format!("stripped_tags={stripped_tags}"),
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExternalClientProfile {
    pub name: String,
    pub requires_https: bool,
    pub recommends_public_tunnel_default: bool,
    pub provider_prefix_workaround: Option<String>,
}

impl ExternalClientProfile {
    pub fn cloud_safe(name: &str, provider_prefix_workaround: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            requires_https: true,
            recommends_public_tunnel_default: false,
            provider_prefix_workaround: provider_prefix_workaround.map(str::to_string),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AgentProfilePackage {
    pub name: String,
    pub surface: &'static str,
    pub gateway_hot_path: bool,
    pub installs_local_hook: bool,
}

impl AgentProfilePackage {
    pub fn status_only(name: &str) -> Self {
        Self {
            name: name.to_string(),
            surface: "cloud-dashboard-profile",
            gateway_hot_path: false,
            installs_local_hook: false,
        }
    }
}
