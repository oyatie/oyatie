use std::fmt;

use crate::ExpansionError;

/// Refusal to admit namespace fragments through [`crate::NamespaceConfig::compile`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NamespaceCompileError {
    DuplicateRelation {
        object_type: String,
        relation: String,
    },
    UnknownRelationReference {
        object_type: String,
        relation: String,
        referenced_relation: String,
    },
    EmptyRewrite {
        object_type: String,
        relation: String,
        kind: &'static str,
    },
    Model(ExpansionError),
}

impl fmt::Display for NamespaceCompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateRelation {
                object_type,
                relation,
            } => write!(f, "duplicate definition for {object_type}#{relation}"),
            Self::UnknownRelationReference {
                object_type,
                relation,
                referenced_relation,
            } => write!(
                f,
                "{object_type}#{relation} references undefined relation {object_type}#{referenced_relation}"
            ),
            Self::EmptyRewrite {
                object_type,
                relation,
                kind,
            } => write!(
                f,
                "{object_type}#{relation} contains an empty {kind} rewrite"
            ),
            Self::Model(error) => write!(f, "namespace model rejected: {error}"),
        }
    }
}

impl std::error::Error for NamespaceCompileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Model(error) => Some(error),
            _ => None,
        }
    }
}
