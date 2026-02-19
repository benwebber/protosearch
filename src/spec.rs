//! Mapping specification.
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// An abstract mapping specification.
#[derive(Debug, Serialize, Deserialize)]
pub struct Spec {
    pub properties: HashMap<String, Property>,
    pub definitions: HashMap<String, Definition>,
}

/// A mapping property.
#[derive(Debug, Serialize, Deserialize)]
pub struct Property {
    pub name: String,
    pub parameters: HashMap<String, Parameter>,
}

/// A named, structured property such as `fielddata`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Definition {
    pub parameters: HashMap<String, Parameter>,
}

/// A mapping parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Parameter {
    Value(ValueType),
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
    Integer,
    Double,
}

impl Parameter {
    pub fn definition_name(&self) -> Option<&str> {
        match self {
            Self::Value(ValueType::Definition(t))
            | Self::Repeated(ValueType::Definition(t))
            | Self::Map(_, ValueType::Definition(t)) => Some(t.as_str()),
            _ => None,
        }
    }
}
