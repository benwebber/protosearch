//! Mapping specification.
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// An abstract mapping specification.
#[derive(Debug, Serialize, Deserialize)]
pub struct MappingSpec {
    pub types: HashMap<String, PropertyType>,
    pub shared_types: HashMap<String, SharedType>,
}

/// A mapping property.
#[derive(Debug, Serialize, Deserialize)]
pub struct PropertyType {
    pub name: String,
    pub parameters: HashMap<String, Parameter>,
}

/// A named, structured property such as `fielddata`.
#[derive(Debug, Serialize, Deserialize)]
pub struct SharedType {
    pub parameters: HashMap<String, Parameter>,
}

/// A mapping parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Parameter {
    Optional(ValueType),
    Repeated(ValueType),
    Map(ScalarType, ValueType),
}

/// A parameter value type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValueType {
    /// A scalar value.
    Scalar(ScalarType),
    /// An unstructured object, such as metadata.
    Object,
    /// A named, structured type.
    Definition(String),
}

/// A simple scalar value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScalarType {
    Boolean,
    String,
    Int32,
    Int64,
    Float,
    Double,
}

impl Parameter {
    pub fn definition_name(&self) -> Option<&str> {
        match self {
            Self::Optional(ValueType::Definition(t))
            | Self::Repeated(ValueType::Definition(t))
            | Self::Map(_, ValueType::Definition(t)) => Some(t.as_str()),
            _ => None,
        }
    }
}
