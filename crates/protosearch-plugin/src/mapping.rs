use std::collections::BTreeMap;
use std::fmt;

use protobuf::reflect::{
    FieldDescriptor, ReflectFieldRef, ReflectValueRef, RuntimeFieldType, RuntimeType,
};
use protobuf::{Enum, MessageDyn};
use serde::Serialize;
use serde_json::{Map, Value, json};

use crate::proto::{Dynamic, FieldMapping, IndexOptions};

/// A document mapping.
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct Mapping {
    pub properties: BTreeMap<String, Property>,
}

/// A mapping property.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Property {
    /// A simple, scalar property.
    Leaf(BTreeMap<String, Value>),
    /// A sub-document mapping, i.e., an `object` or `nested` field.
    Mapping {
        #[serde(flatten)]
        parameters: BTreeMap<String, Value>,
        #[serde(flatten)]
        properties: Mapping,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum InferredType {
    Keyword,
    Boolean,
    Integer,
    Long,
    UnsignedLong,
    Float,
    Double,
    Binary,
    Object,
}

impl TryFrom<&FieldMapping> for Property {
    type Error = crate::Error;

    fn try_from(options: &FieldMapping) -> crate::Result<Self> {
        Ok(Self::Leaf(
            other_to_json(options as &dyn MessageDyn)?
                .into_iter()
                .collect(),
        ))
    }
}

impl fmt::Display for InferredType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Keyword => "keyword",
            Self::Boolean => "boolean",
            Self::Integer => "integer",
            Self::Long => "long",
            Self::UnsignedLong => "unsigned_long",
            Self::Float => "float",
            Self::Double => "double",
            Self::Binary => "binary",
            Self::Object => "object",
        })
    }
}

impl From<RuntimeType> for InferredType {
    fn from(t: RuntimeType) -> Self {
        match t {
            RuntimeType::I32 => Self::Integer,
            RuntimeType::I64 => Self::Long,
            RuntimeType::U32 => Self::Long,
            RuntimeType::U64 => Self::UnsignedLong,
            RuntimeType::F32 => Self::Float,
            RuntimeType::F64 => Self::Double,
            RuntimeType::Bool => Self::Boolean,
            RuntimeType::String => Self::Keyword,
            RuntimeType::VecU8 => Self::Binary,
            RuntimeType::Message(_) => Self::Object,
            RuntimeType::Enum(_) => Self::Keyword,
        }
    }
}

impl From<&FieldDescriptor> for InferredType {
    fn from(field: &FieldDescriptor) -> Self {
        match field.runtime_field_type() {
            RuntimeFieldType::Singular(t) | RuntimeFieldType::Repeated(t) => Self::from(t),
            RuntimeFieldType::Map(_, _) => Self::Object,
        }
    }
}

impl fmt::Display for Dynamic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Dynamic::DYNAMIC_UNSPECIFIED => "",
            Dynamic::DYNAMIC_TRUE => "true",
            Dynamic::DYNAMIC_FALSE => "false",
            Dynamic::DYNAMIC_STRICT => "strict",
            Dynamic::DYNAMIC_RUNTIME => "runtime",
        })
    }
}

impl fmt::Display for IndexOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            IndexOptions::INDEX_OPTIONS_UNSPECIFIED => "",
            IndexOptions::INDEX_OPTIONS_DOCS => "docs",
            IndexOptions::INDEX_OPTIONS_FREQS => "freqs",
            IndexOptions::INDEX_OPTIONS_POSITIONS => "positions",
            IndexOptions::INDEX_OPTIONS_OFFSETS => "offsets",
        })
    }
}

fn to_json(message: &dyn MessageDyn) -> crate::Result<Value> {
    match message.descriptor_dyn().full_name() {
        "google.protobuf.Struct" => struct_to_json(message),
        "google.protobuf.Value" => wkt_value_to_json(message),
        "google.protobuf.ListValue" => list_value_to_json(message),
        _ => Ok(Value::Object(other_to_json(message)?)),
    }
}

fn reflect_value_to_json(v: ReflectValueRef) -> crate::Result<Value> {
    match v {
        ReflectValueRef::Bool(b) => Ok(json!(b)),
        ReflectValueRef::I32(i) => Ok(json!(i)),
        ReflectValueRef::I64(i) => Ok(json!(i)),
        ReflectValueRef::U32(u) => Ok(json!(u)),
        ReflectValueRef::U64(u) => Ok(json!(u)),
        ReflectValueRef::F32(f) => Ok(json!(f)),
        ReflectValueRef::F64(f) => Ok(json!(f)),
        ReflectValueRef::String(s) => Ok(json!(s)),
        ReflectValueRef::Bytes(b) => Ok(json!(b)),
        ReflectValueRef::Enum(desc, i) => match desc.full_name() {
            "protosearch.Dynamic" => {
                let dynamic =
                    Dynamic::from_i32(i).ok_or(crate::Error::UnsupportedFieldValueType)?;
                Ok(Value::String(dynamic.to_string()))
            }
            "protosearch.IndexOptions" => {
                let index_options =
                    IndexOptions::from_i32(i).ok_or(crate::Error::UnsupportedFieldValueType)?;
                Ok(Value::String(index_options.to_string()))
            }
            _ => Err(crate::Error::UnsupportedFieldValueType),
        },
        ReflectValueRef::Message(m) => to_json(&*m),
    }
}

fn struct_to_json(msg: &dyn MessageDyn) -> crate::Result<Value> {
    let desc = msg.descriptor_dyn();
    let fields_field = desc
        .field_by_name("fields")
        .expect("google.protobuf.Struct must have a 'fields' field");
    let mut map = Map::new();
    if let ReflectFieldRef::Map(m) = fields_field.get_reflect(msg) {
        for (k, v) in m.into_iter() {
            let key = match k {
                ReflectValueRef::String(s) => s.to_string(),
                _ => unreachable!("google.protobuf.Struct keys must be strings"),
            };
            map.insert(key, reflect_value_to_json(v)?);
        }
    }
    Ok(Value::Object(map))
}

fn wkt_value_to_json(msg: &dyn MessageDyn) -> crate::Result<Value> {
    let desc = msg.descriptor_dyn();
    let oneof = desc
        .oneof_by_name("kind")
        .expect("google.protobuf.Value must have a oneof kind");
    for field in oneof.fields() {
        if let ReflectFieldRef::Optional(v) = field.get_reflect(msg)
            && let Some(rv) = v.value()
        {
            return match field.name() {
                "null_value" => Ok(Value::Null),
                _ => reflect_value_to_json(rv),
            };
        }
    }
    Ok(Value::Null)
}

fn list_value_to_json(msg: &dyn MessageDyn) -> crate::Result<Value> {
    let desc = msg.descriptor_dyn();
    let values_field = desc
        .field_by_name("values")
        .expect("google.protobuf.ListValue must have a 'values' field");
    match values_field.get_reflect(msg) {
        ReflectFieldRef::Repeated(v) => {
            let items: crate::Result<Vec<_>> = v.into_iter().map(reflect_value_to_json).collect();
            Ok(Value::Array(items?))
        }
        _ => unreachable!("google.protobuf.ListValue values are always repeated"),
    }
}

fn other_to_json(msg: &dyn MessageDyn) -> crate::Result<Map<String, Value>> {
    let desc = msg.descriptor_dyn();
    let mut map = Map::new();
    for field in desc.fields() {
        match field.get_reflect(msg) {
            ReflectFieldRef::Optional(v) => {
                if let Some(rv) = v.value() {
                    const UNSPECIFIED: i32 = 0;
                    if let ReflectValueRef::Enum(_, UNSPECIFIED) = rv {
                        continue;
                    }
                    map.insert(field.name().to_string(), reflect_value_to_json(rv)?);
                }
            }
            ReflectFieldRef::Repeated(v) if !v.is_empty() => {
                let arr: crate::Result<Vec<_>> = v.into_iter().map(reflect_value_to_json).collect();
                map.insert(field.name().to_string(), Value::Array(arr?));
            }
            ReflectFieldRef::Map(m) if !m.is_empty() => {
                let mut obj = Map::new();
                for (k, v) in m.into_iter() {
                    let key_str = match k {
                        ReflectValueRef::String(s) => s.to_string(),
                        _ => unreachable!("all protosearch.FieldMapping maps have string keys"),
                    };
                    obj.insert(key_str, reflect_value_to_json(v)?);
                }
                map.insert(field.name().to_string(), Value::Object(obj));
            }
            _ => {}
        }
    }
    Ok(map)
}
