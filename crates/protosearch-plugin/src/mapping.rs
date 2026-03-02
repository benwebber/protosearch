use std::collections::BTreeMap;
use std::fmt;

use protobuf::reflect::{MessageDescriptor, ReflectFieldRef, ReflectValueRef};
use protobuf::{Enum, MessageDyn};
use serde::Serialize;
use serde::ser::{Error, Serializer};
use serde_json::{Map, Value, json};

use crate::Result;
use crate::proto::{Dynamic, FieldMapping, Index, IndexOptions, SourceMode, TermVector};

/// A document mapping.
#[derive(Debug, Default)]
pub struct Mapping {
    pub descriptor: Option<MessageDescriptor>,
    pub index: Option<Index>,
    pub properties: BTreeMap<String, Property>,
}

/// A mapping property.
#[derive(Debug)]
pub enum Property {
    /// A simple, scalar property.
    Leaf(Parameters),
    /// A sub-document mapping, i.e., an `object` or `nested` field.
    Object {
        parameters: Parameters,
        properties: Mapping,
    },
}

#[derive(Debug)]
pub enum Parameters {
    Typed {
        field_mapping: Box<FieldMapping>,
        inferred_type: Option<String>,
    },
    Raw(Map<String, Value>),
}

impl Mapping {
    pub fn with_descriptor(descriptor: MessageDescriptor) -> Self {
        Self {
            descriptor: Some(descriptor),
            index: None,
            properties: Default::default(),
        }
    }
}

impl Serialize for Mapping {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map: BTreeMap<String, Value> = self
            .index
            .as_ref()
            .map(|i| other_to_json(i as &dyn MessageDyn))
            .transpose()
            .map_err(S::Error::custom)?
            .unwrap_or_default()
            .into_iter()
            .collect();
        if !self.properties.is_empty() {
            map.insert(
                "properties".to_string(),
                serde_json::to_value(&self.properties).map_err(S::Error::custom)?,
            );
        }
        map.serialize(serializer)
    }
}

impl Serialize for Property {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Leaf(p) => p.serialize(serializer),
            Self::Object {
                parameters,
                properties,
            } => {
                let mut map = parameters_to_map(parameters).map_err(S::Error::custom)?;
                map.insert(
                    "properties".to_string(),
                    serde_json::to_value(&properties.properties).map_err(S::Error::custom)?,
                );
                map.serialize(serializer)
            }
        }
    }
}

impl Serialize for Parameters {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        parameters_to_map(self)
            .map_err(S::Error::custom)?
            .serialize(serializer)
    }
}

fn parameters_to_map(parameters: &Parameters) -> Result<BTreeMap<String, Value>> {
    match parameters {
        Parameters::Raw(m) => Ok(m.clone().into_iter().collect()),
        Parameters::Typed {
            field_mapping,
            inferred_type,
        } => {
            let mut map: BTreeMap<String, Value> =
                other_to_json(field_mapping.as_ref() as &dyn MessageDyn)?
                    .into_iter()
                    .collect();
            if let Some(t) = inferred_type {
                map.entry("type".to_string())
                    .or_insert_with(|| Value::String(t.clone()));
            }
            Ok(map)
        }
    }
}

impl fmt::Display for Dynamic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::DYNAMIC_UNSPECIFIED => "",
            Self::DYNAMIC_TRUE => "true",
            Self::DYNAMIC_FALSE => "false",
            Self::DYNAMIC_STRICT => "strict",
            Self::DYNAMIC_RUNTIME => "runtime",
        })
    }
}

impl fmt::Display for IndexOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::INDEX_OPTIONS_UNSPECIFIED => "",
            Self::INDEX_OPTIONS_DOCS => "docs",
            Self::INDEX_OPTIONS_FREQS => "freqs",
            Self::INDEX_OPTIONS_POSITIONS => "positions",
            Self::INDEX_OPTIONS_OFFSETS => "offsets",
        })
    }
}

impl fmt::Display for SourceMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::SOURCE_MODE_UNSPECIFIED => "",
            Self::SOURCE_MODE_DISABLED => "disabled",
            Self::SOURCE_MODE_STORED => "stored",
            Self::SOURCE_MODE_SYNTHETIC => "synthetic",
        })
    }
}

impl fmt::Display for TermVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::TERM_VECTOR_UNSPECIFIED => "",
            Self::TERM_VECTOR_NO => "no",
            Self::TERM_VECTOR_YES => "yes",
            Self::TERM_VECTOR_WITH_POSITIONS => "with_positions",
            Self::TERM_VECTOR_WITH_OFFSETS => "with_offsets",
            Self::TERM_VECTOR_WITH_POSITIONS_OFFSETS => "with_positions_offsets",
            Self::TERM_VECTOR_WITH_POSITIONS_PAYLOADS => "with_positions_payloads",
            Self::TERM_VECTOR_WITH_POSITIONS_OFFSETS_PAYLOADS => "with_positions_offsets_payloads",
        })
    }
}

fn to_json(message: &dyn MessageDyn) -> Result<Value> {
    match message.descriptor_dyn().full_name() {
        "google.protobuf.Value" => wkt_value_to_json(message),
        "google.protobuf.ListValue" => list_value_to_json(message),
        _ => Ok(Value::Object(other_to_json(message)?)),
    }
}

fn reflect_value_to_json(v: ReflectValueRef) -> Result<Value> {
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
            "protosearch.Dynamic" => proto_enum_to_json::<Dynamic>(i),
            "protosearch.IndexOptions" => proto_enum_to_json::<IndexOptions>(i),
            "protosearch.SourceMode" => proto_enum_to_json::<SourceMode>(i),
            "protosearch.TermVector" => proto_enum_to_json::<TermVector>(i),
            _ => unreachable!(
                "unknown enum type '{}': implement Display and add match arm",
                desc.full_name()
            ),
        },
        ReflectValueRef::Message(m) => to_json(&*m),
    }
}

fn wkt_value_to_json(msg: &dyn MessageDyn) -> Result<Value> {
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

fn list_value_to_json(msg: &dyn MessageDyn) -> Result<Value> {
    let desc = msg.descriptor_dyn();
    let values_field = desc
        .field_by_name("values")
        .expect("google.protobuf.ListValue must have a 'values' field");
    match values_field.get_reflect(msg) {
        ReflectFieldRef::Repeated(v) => Ok(Value::Array(
            v.into_iter()
                .map(reflect_value_to_json)
                .collect::<Result<_>>()?,
        )),
        _ => unreachable!("google.protobuf.ListValue values are always repeated"),
    }
}

fn other_to_json(msg: &dyn MessageDyn) -> Result<Map<String, Value>> {
    let desc = msg.descriptor_dyn();
    let mut map = Map::new();
    for field in desc.fields() {
        match field.get_reflect(msg) {
            ReflectFieldRef::Optional(v) => {
                if let Some(rv) = v.value() {
                    // Always ignore the conventional default/zero value (UNSPECIFIED).
                    const UNSPECIFIED: i32 = 0;
                    if let ReflectValueRef::Enum(_, UNSPECIFIED) = rv {
                        continue;
                    }
                    map.insert(field.name().to_string(), reflect_value_to_json(rv)?);
                }
            }
            ReflectFieldRef::Repeated(v) if !v.is_empty() => {
                let arr: Result<Vec<_>> = v.into_iter().map(reflect_value_to_json).collect();
                map.insert(field.name().to_string(), Value::Array(arr?));
            }
            ReflectFieldRef::Map(m) if !m.is_empty() => {
                let mut obj = Map::new();
                for (k, v) in m.into_iter() {
                    let ReflectValueRef::String(s) = k else {
                        unreachable!("all protosearch.FieldMapping maps have string keys")
                    };
                    obj.insert(s.to_string(), reflect_value_to_json(v)?);
                }
                map.insert(field.name().to_string(), Value::Object(obj));
            }
            _ => {}
        }
    }
    Ok(map)
}

fn proto_enum_to_json<T: Enum + fmt::Display>(i: i32) -> Result<Value> {
    Ok(Value::String(T::from_i32(i).unwrap().to_string()))
}
