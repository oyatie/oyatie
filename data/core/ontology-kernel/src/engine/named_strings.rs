use crate::{
    NamedStringTypeDefinition, NamedStringTypeRef, OntologyEngine, OntologyEngineError,
    ValueTypeDeclaration,
};

impl OntologyEngine {
    pub fn register_named_string_type(
        &mut self,
        definition: NamedStringTypeDefinition,
    ) -> Result<NamedStringTypeRef, OntologyEngineError> {
        let reference = definition.reference().clone();
        if reference.revision() != 1 {
            return Err(OntologyEngineError::InvalidNamedStringRevision);
        }
        if self.named_string_types.contains_key(&reference) {
            return Err(OntologyEngineError::DuplicateNamedStringType);
        }
        self.named_string_types
            .insert(reference.clone(), definition);
        Ok(reference)
    }
    pub fn named_string_type(
        &self,
        reference: &NamedStringTypeRef,
    ) -> Option<&NamedStringTypeDefinition> {
        self.named_string_types.get(reference)
    }
    pub fn named_string_types<'a>(
        &'a self,
        tenant: &'a str,
    ) -> impl Iterator<Item = &'a NamedStringTypeDefinition> {
        self.named_string_types
            .iter()
            .filter(move |(reference, _)| reference.tenant_id() == tenant)
            .map(|(_, definition)| definition)
    }
    pub fn named_string_declaration(
        &self,
        reference: &NamedStringTypeRef,
    ) -> Result<ValueTypeDeclaration, OntologyEngineError> {
        self.named_string_type(reference)
            .cloned()
            .map(ValueTypeDeclaration::NamedString)
            .ok_or(OntologyEngineError::UnknownNamedStringType)
    }
}
