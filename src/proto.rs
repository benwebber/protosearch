//! Protobuf types.
use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::spec;

/// A scalar protobuf field type.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ScalarType {
    Bool,
    String,
    Int64,
    Double,
}

/// A protobuf field value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValueType {
    Scalar(ScalarType),
    Message(String),
}

/// A protobuf field type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldType {
    Optional(ValueType),
    Repeated(ValueType),
    Map(ScalarType, ValueType),
}

/// A protobuf file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct File {
    pub package: String,
    pub extensions: Vec<Extension>,
    pub messages: Vec<Message>,
}

/// A protobuf field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub typ: FieldType,
    pub tag: u32,
}

impl File {
    pub fn new(package: &str) -> Self {
        Self {
            package: package.to_string(),
            extensions: Vec::new(),
            messages: Vec::new(),
        }
    }
}

impl fmt::Display for ScalarType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Bool => "bool",
            Self::String => "string",
            Self::Int64 => "int64",
            Self::Double => "double",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scalar(t) => write!(f, "{}", t),
            Self::Message(t) => write!(f, "{}", t),
        }
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.typ {
            FieldType::Optional(t) => write!(f, "optional {} {} = {}", t, self.name, self.tag),
            FieldType::Repeated(t) => write!(f, "repeated {} {} = {}", t, self.name, self.tag),
            FieldType::Map(kt, vt) => write!(f, "map<{}, {}> {} = {}", kt, vt, self.name, self.tag),
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "message {} {{", self.name)?;
        for field in &self.fields {
            writeln!(f, "  {};", field)?;
        }
        if !self.reserved.is_empty() {
            writeln!(
                f,
                "  reserved {};",
                self.reserved
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            )?;
        }
        writeln!(f, "}}")
    }
}

impl fmt::Display for Extension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "extend {} {{", self.name)?;
        for field in &self.fields {
            writeln!(f, "  {};", field)?;
        }
        writeln!(f, "}}")
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            r#"syntax = "proto2";

package {};

import "google/protobuf/struct.proto";
import "protosearch/protosearch.proto";"#,
            self.package
        )?;
        for extension in &self.extensions {
            write!(f, "\n{}", extension)?;
        }
        for message in &self.messages {
            write!(f, "\n{}", message)?;
        }
        Ok(())
    }
}

impl From<spec::ValueType> for ValueType {
    fn from(t: spec::ValueType) -> Self {
        match t {
            spec::ValueType::Scalar(s) => ValueType::Scalar(s.into()),
            spec::ValueType::Object => ValueType::Message("google.protobuf.Struct".into()),
            spec::ValueType::Definition(name) => {
                ValueType::Message(message_name(&name).to_string())
            }
        }
    }
}

impl From<spec::ScalarType> for ScalarType {
    fn from(t: spec::ScalarType) -> Self {
        match t {
            spec::ScalarType::String => ScalarType::String,
            spec::ScalarType::Boolean => ScalarType::Bool,
            spec::ScalarType::Integer => ScalarType::Int64,
            spec::ScalarType::Double => ScalarType::Double,
        }
    }
}

impl From<spec::Parameter> for FieldType {
    fn from(p: spec::Parameter) -> Self {
        match p {
            spec::Parameter::Value(v) => FieldType::Optional(v.into()),
            spec::Parameter::Repeated(v) => FieldType::Repeated(v.into()),
            spec::Parameter::Map(kt, vt) => FieldType::Map(kt.into(), vt.into()),
        }
    }
}

/// Merge fields from `other` into `fields`.
///
/// If any field in `fields` is *not* in `other`, remove it and add its tag number to `reserved`.
/// Return [`Error::FieldConflict`] if a field in `other` shares the name of a field in `fields`, but differs by tag or type.
fn merge_fields(
    fields: &mut Vec<Field>,
    other: &[Field],
    reserved: &mut Vec<u32>,
    next_tag: &mut u32,
) -> Result<(), Error> {
    let mut current_fields: HashMap<String, (u32, FieldType)> = fields
        .drain(..)
        .map(|field| (field.name, (field.tag, field.typ)))
        .collect();
    let mut new_fields = Vec::with_capacity(other.len());
    for field in other {
        if let Some((current_tag, current_type)) = current_fields.remove(&field.name) {
            if field.typ != current_type {
                return Err(Error::FieldConflict(field.name.clone()));
            }
            new_fields.push(Field {
                name: field.name.clone(),
                typ: field.typ.clone(),
                tag: current_tag,
            });
        } else {
            new_fields.push(Field {
                name: field.name.clone(),
                typ: field.typ.clone(),
                tag: *next_tag,
            });
            *next_tag += 1;
        }
    }
    for (tag, _) in current_fields.values() {
        reserved.push(*tag);
    }
    *fields = new_fields;
    fields.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(())
}

macro_rules! impl_message_like {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct $name {
            pub name: String,
            pub fields: Vec<Field>,
            pub reserved: Vec<u32>,
        }

        impl $name {
            /// Merge fields from `other` into this value.
            ///
            /// New fields start at `tag_offset`.
            pub fn merge(&mut self, other: Self, tag_offset: u32) -> Result<(), Error> {
                let mut next_tag = self.next_tag(tag_offset);
                merge_fields(
                    &mut self.fields,
                    &other.fields,
                    &mut self.reserved,
                    &mut next_tag,
                )?;
                Ok(())
            }

            /// Return the next field tag, considering all defined fields and reserved tags.
            fn next_tag(&self, tag_offset: u32) -> u32 {
                self.fields
                    .iter()
                    .map(|f| f.tag)
                    .chain(self.reserved.iter().copied())
                    .max()
                    .unwrap_or(tag_offset - 1)
                    + 1
            }
        }
    };
}

impl_message_like!(Extension, "A protobuf extension.");
impl_message_like!(Message, "A protobuf message.");

impl File {
    pub fn merge(&mut self, other: Self, tag_offset: u32) -> Result<(), Error> {
        if self.package != other.package {
            return Err(Error::PackageConflict {
                current: self.package.clone(),
                other: other.package,
            });
        }
        for ext in other.extensions {
            if let Some(existing) = self.extensions.iter_mut().find(|e| e.name == ext.name) {
                existing.merge(ext, tag_offset)?;
            } else {
                self.extensions.push(ext);
            }
        }
        for msg in other.messages {
            if let Some(existing) = self.messages.iter_mut().find(|m| m.name == msg.name) {
                existing.merge(msg, 1)?;
            } else {
                self.messages.push(msg);
            }
        }
        Ok(())
    }
}

/// Generate a message name from an OpenAPI schema name.
pub fn message_name(schema_name: &str) -> &str {
    schema_name.rsplit('.').next().unwrap_or(schema_name)
}
