use data_ontology_kernel::EntityTypeId;

pub(super) struct PointQuery {
    pub(super) entity_type: EntityTypeId,
    pub(super) revision: u32,
}

impl PointQuery {
    pub(super) fn parse(raw: Option<&str>, object_ref: &str) -> Option<Self> {
        let raw = raw.filter(|query| query.len() <= 4096)?;
        if !identifier(object_ref, "ent_") {
            return None;
        }
        let mut url = reqwest::Url::parse("https://point.invalid/").ok()?;
        url.set_query(Some(raw));
        let pairs: Vec<_> = url.query_pairs().into_owned().collect();
        let mut canonical = reqwest::Url::parse("https://point.invalid/").ok()?;
        canonical.query_pairs_mut().extend_pairs(&pairs);
        // A decode-and-encode round trip rejects aliases and lossy parser repairs.
        if canonical.query()? != raw || pairs.len() != 2 {
            return None;
        }
        let (mut entity_type, mut revision) = (None, None);
        for (key, value) in pairs {
            let slot = match key.as_str() {
                "type" => &mut entity_type,
                "revision" => &mut revision,
                _ => return None,
            };
            if slot.replace(value).is_some() {
                return None;
            }
        }
        let entity_type = entity_type.filter(|value| identifier(value, "ety_"))?;
        let revision = revision
            .filter(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))?;
        Some(Self {
            entity_type: EntityTypeId::new(entity_type).ok()?,
            revision: revision.parse().ok()?,
        })
    }
}

fn identifier(value: &str, prefix: &str) -> bool {
    value.starts_with(prefix)
        && value.len() > prefix.len()
        && value.len() <= 256
        && value.trim() == value
        && !value.chars().any(char::is_control)
}
