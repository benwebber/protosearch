use protobuf::MessageDyn;
use protobuf::reflect::{ReflectFieldRef, ReflectValueRef};
use serde_json::{Map, Value, json};

/// A wrapper around dynamic message values that provides specific JSON encodings.
/// TODO: Refactor this as a serializer.
pub enum Message<'a> {
    /// A `google.protobuf.Struct` value.
    Struct(&'a dyn MessageDyn),
    /// A `google.protobuf.Value` value.
    Value(&'a dyn MessageDyn),
    /// A `google.protobuf.ListValue` value.
    ListValue(&'a dyn MessageDyn),
    /// Any other Protobuf value.
    Other(&'a dyn MessageDyn),
}

impl<'a> From<&'a dyn MessageDyn> for Message<'a> {
    fn from(m: &'a dyn MessageDyn) -> Self {
        match m.descriptor_dyn().full_name() {
            "google.protobuf.Struct" => Self::Struct(m),
            "google.protobuf.Value" => Self::Value(m),
            "google.protobuf.ListValue" => Self::ListValue(m),
            _ => Self::Other(m),
        }
    }
}

impl Message<'_> {
    pub fn to_json(&self) -> Value {
        match self {
            Self::Struct(m) => struct_to_json(*m),
            Self::Value(m) => wkt_value_to_json(*m),
            Self::ListValue(m) => list_value_to_json(*m),
            Self::Other(m) => other_to_json(*m),
        }
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
        ReflectValueRef::Enum(_, _) => todo!("Choose how to represent enums"),
        ReflectValueRef::Message(m) => Message::from(&*m).to_json(),
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
                _ => continue,
            };
            map.insert(key, reflect_value_to_json(v));
        }
    }
    Value::Object(map)
}

fn wkt_value_to_json(msg: &dyn MessageDyn) -> Value {
    let desc = msg.descriptor_dyn();
    for field in desc.fields() {
        if let ReflectFieldRef::Optional(v) = field.get_reflect(msg)
            && let Some(rv) = v.value()
        {
            match field.name() {
                "null_value" => return Value::Null,
                "number_value" => return reflect_value_to_json(rv),
                "string_value" => return reflect_value_to_json(rv),
                "bool_value" => return reflect_value_to_json(rv),
                "struct_value" => return reflect_value_to_json(rv),
                "list_value" => return reflect_value_to_json(rv),
                _ => {}
            }
        }
    }
    Value::Null
}

fn list_value_to_json(msg: &dyn MessageDyn) -> Value {
    let desc = msg.descriptor_dyn();
    let values_field = desc
        .field_by_name("values")
        .expect("google.protobuf.ListValue must have a 'values' field");
    if let ReflectFieldRef::Repeated(v) = values_field.get_reflect(msg) {
        let arr: Vec<_> = v.into_iter().map(reflect_value_to_json).collect();
        return Value::Array(arr);
    }
    Value::Array(vec![])
}

fn other_to_json(msg: &dyn MessageDyn) -> Value {
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
                        _ => format!("{:?}", k),
                    };
                    obj.insert(key_str, reflect_value_to_json(v));
                }
                map.insert(field.name().to_string(), Value::Object(obj));
            }
            _ => {}
        }
    }
    Value::Object(map)
}
