use super::model::Refusal;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Node {
    pub object: super::model::Object,
    pub relation: String,
}

impl Node {
    pub fn new(object: super::model::Object, relation: impl Into<String>) -> Self {
        Self {
            object,
            relation: relation.into(),
        }
    }
}

pub enum Formula {
    Literal(bool),
    Refuse(Refusal),
    Union(Vec<Self>),
    Intersection(Vec<Self>),
    Difference(Box<Self>, Box<Self>),
}

impl Formula {
    pub fn evaluate(&self) -> Result<bool, Refusal> {
        match self {
            Self::Literal(value) => Ok(*value),
            Self::Refuse(refusal) => Err(refusal.clone()),
            Self::Union(children) => {
                for child in children {
                    if child.evaluate()? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Self::Intersection(children) => {
                for child in children {
                    if !child.evaluate()? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Self::Difference(base, subtract) => {
                if !base.evaluate()? {
                    return Ok(false);
                }
                Ok(!subtract.evaluate()?)
            }
        }
    }
}
