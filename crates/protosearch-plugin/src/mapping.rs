use std::collections::BTreeMap;
use std::fmt;

use protobuf::MessageDyn;
use protobuf::reflect::{ReflectFieldRef, ReflectValueRef, RuntimeType};
use serde::Serialize;
use serde_json::{Map, Value, json};

use crate::proto::FieldMapping;

const OUTPUT_FIELD_NAME: &str = "output";

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
    Leaf {
        #[serde(rename = "type")]
        typ: String,
        #[serde(default, flatten)]
        parameters: BTreeMap<String, Value>,
    },
    /// A sub-document mapping, i.e., an `object` or `nested` field.
    Mapping {
        #[serde(rename = "type")]
        typ: String,
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

impl Property {
    pub fn from_options(options: &FieldMapping, typ: String) -> Self {
        let mut params = other_to_json(options as &dyn MessageDyn);
        params.remove("type");
        params.remove(OUTPUT_FIELD_NAME);
        Self::Leaf {
            typ,
            parameters: params.into_iter().collect(),
        }
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

fn to_json(message: &dyn MessageDyn) -> Value {
    match message.descriptor_dyn().full_name() {
        "google.protobuf.Struct" => struct_to_json(message),
        "google.protobuf.Value" => wkt_value_to_json(message),
        "google.protobuf.ListValue" => list_value_to_json(message),
        _ => Value::Object(other_to_json(message)),
    }
}

fn reflect_value_to_json(v: ReflectValueRef) -> Value {
    match v {
        ReflectValueRef::Bool(b) => json!(b),
        ReflectValueRef::I32(i) => json!(i),
        ReflectValueRef::I64(i) => json!(i),
        ReflectValueRef::U32(u) => json!(u),
        ReflectValueRef::U64(u) => json!(u),
        ReflectValueRef::F32(f) => json!(f),
        ReflectValueRef::F64(f) => json!(f),
        ReflectValueRef::String(s) => json!(s),
        ReflectValueRef::Bytes(b) => json!(b),
        ReflectValueRef::Enum(_, _) => {
            unimplemented!("enum value mapping parameters are not supported")
        }
        ReflectValueRef::Message(m) => to_json(&*m),
    }
}

fn struct_to_json(msg: &dyn MessageDyn) -> Value {
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
            map.insert(key, reflect_value_to_json(v));
        }
    }
    Value::Object(map)
}

fn wkt_value_to_json(msg: &dyn MessageDyn) -> Value {
    let desc = msg.descriptor_dyn();
    let oneof = desc
        .oneof_by_name("kind")
        .expect("google.protobuf.Value must have a oneof kind");
    for field in oneof.fields() {
        if let ReflectFieldRef::Optional(v) = field.get_reflect(msg)
            && let Some(rv) = v.value()
        {
            return match field.name() {
                "null_value" => Value::Null,
                _ => reflect_value_to_json(rv),
            };
        }
    }
    Value::Null
}

fn list_value_to_json(msg: &dyn MessageDyn) -> Value {
    let desc = msg.descriptor_dyn();
    let values_field = desc
        .field_by_name("values")
        .expect("google.protobuf.ListValue must have a 'values' field");
    match values_field.get_reflect(msg) {
        ReflectFieldRef::Repeated(v) => {
            Value::Array(v.into_iter().map(reflect_value_to_json).collect())
        }
        _ => unreachable!("google.protobuf.ListValue values are always repeated"),
    }
}

fn other_to_json(msg: &dyn MessageDyn) -> Map<String, Value> {
    let desc = msg.descriptor_dyn();
    let mut map = Map::new();
    for field in desc.fields() {
        match field.get_reflect(msg) {
            ReflectFieldRef::Optional(v) => {
                if let Some(rv) = v.value() {
                    map.insert(field.name().to_string(), reflect_value_to_json(rv));
                }
            }
            ReflectFieldRef::Repeated(v) if !v.is_empty() => {
                let arr: Vec<_> = v.into_iter().map(reflect_value_to_json).collect();
                map.insert(field.name().to_string(), Value::Array(arr));
            }
            ReflectFieldRef::Map(m) if !m.is_empty() => {
                let mut obj = Map::new();
                for (k, v) in m.into_iter() {
                    let key_str = match k {
                        ReflectValueRef::String(s) => s.to_string(),
                        _ => unreachable!("all protosearch.FieldMapping maps have string keys"),
                    };
                    obj.insert(key_str, reflect_value_to_json(v));
                }
                map.insert(field.name().to_string(), Value::Object(obj));
            }
            _ => {}
        }
    }
    map
}
