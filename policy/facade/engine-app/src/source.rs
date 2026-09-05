use std::collections::BTreeMap;

use policy_pdp_kernel::{EntitySlice, PolicyBundle, TemplateLink, TemplateSrc};
use serde::{Deserialize, Serialize};
use shared_audit_digest_adapter_awslc::Sha256Digester;
use shared_audit_event_kernel::Digester;
use shared_platform_contracts_kernel::pdp::{
    AuthorizationRequest, Decision, Obligation, PolicyVersion,
};

use crate::QualificationError;

/// Complete authored serving inputs. Version is derived, never caller assigned.
/// Source whitespace and vector order are significant; map insertion order is not.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySource {
    pub schema_src: String,
    pub policies_src: String,
    pub tenant_policies: BTreeMap<String, String>,
    pub templates: Vec<TemplateSrc>,
    pub template_links: Vec<TemplateLink>,
    pub action_map: BTreeMap<String, String>,
}

impl PolicySource {
    /// Identity covers every serving input, independent of qualification cases.
    /// This does not parse or admit the source.
    ///
    /// # Errors
    /// Returns a serialization or version-contract refusal.
    pub fn content_version(&self) -> Result<PolicyVersion, QualificationError> {
        let digest = content_digest(b"oyatie-policy-source/v1\0", self)?;
        PolicyVersion::new(digest).map_err(|violations| QualificationError::Encoding {
            detail: format!("content identity violates version contract: {violations:?}"),
        })
    }

    pub(crate) fn candidate(&self) -> Result<PolicyBundle, QualificationError> {
        Ok(PolicyBundle {
            version: self.content_version()?,
            schema_src: self.schema_src.clone(),
            policies_src: self.policies_src.clone(),
            tenant_policies: self.tenant_policies.clone(),
            templates: self.templates.clone(),
            template_links: self.template_links.clone(),
            action_map: self.action_map.clone(),
        })
    }
}

impl From<&PolicyBundle> for PolicySource {
    fn from(bundle: &PolicyBundle) -> Self {
        Self {
            schema_src: bundle.schema_src.clone(),
            policies_src: bundle.policies_src.clone(),
            tenant_policies: bundle.tenant_policies.clone(),
            templates: bundle.templates.clone(),
            template_links: bundle.template_links.clone(),
            action_map: bundle.action_map.clone(),
        }
    }
}

/// Expected complete enforcement content; random correlation IDs are excluded.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionExpectation {
    pub decision: Decision,
    pub determining_policy_ids: Vec<String>,
    pub obligations: Vec<Obligation>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyCase {
    pub name: String,
    pub request: AuthorizationRequest,
    pub entities: EntitySlice,
    pub expected: DecisionExpectation,
}

/// Closed source-and-tests input to offline qualification.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyProject {
    pub source: PolicySource,
    pub cases: Vec<PolicyCase>,
}

pub(crate) fn content_digest(
    domain: &[u8],
    value: &impl Serialize,
) -> Result<String, QualificationError> {
    let mut bytes = domain.to_vec();
    serde_json::to_writer(&mut bytes, value).map_err(|error| QualificationError::Encoding {
        detail: error.to_string(),
    })?;
    Ok(Sha256Digester.digest_hex(&bytes))
}
