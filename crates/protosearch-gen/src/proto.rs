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
    Int32,
    Int64,
    Float,
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
    pub extensions: Vec<ExtendBlock>,
    pub messages: Vec<Message>,
}

/// A protobuf field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub typ: FieldType,
    pub number: u32,
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
            Self::Int32 => "int32",
            Self::Int64 => "int64",
            Self::Float => "float",
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
            FieldType::Optional(t) => write!(f, "optional {} {} = {}", t, self.name, self.number),
            FieldType::Repeated(t) => write!(f, "repeated {} {} = {}", t, self.name, self.number),
            FieldType::Map(kt, vt) => {
                write!(f, "map<{}, {}> {} = {}", kt, vt, self.name, self.number)
            }
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

impl fmt::Display for ExtendBlock {
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
            spec::ScalarType::Int32 => ScalarType::Int32,
            spec::ScalarType::Int64 => ScalarType::Int64,
            spec::ScalarType::Float => ScalarType::Float,
            spec::ScalarType::Double => ScalarType::Double,
        }
    }
}

impl From<spec::Parameter> for FieldType {
    fn from(p: spec::Parameter) -> Self {
        match p {
            spec::Parameter::Optional(v) => FieldType::Optional(v.into()),
            spec::Parameter::Repeated(v) => FieldType::Repeated(v.into()),
            spec::Parameter::Map(kt, vt) => FieldType::Map(kt.into(), vt.into()),
        }
    }
}

/// Merge fields from `other` into `fields`.
///
/// If any field in `fields` is *not* in `other`, remove it and add its number number to `reserved`.
/// Return [`Error::FieldConflict`] if a field in `other` shares the name of a field in `fields`, but differs by number or type.
fn merge_fields(
    fields: &mut Vec<Field>,
    other: &[Field],
    reserved: &mut Vec<u32>,
    next_number: &mut u32,
) -> Result<(), Error> {
    let mut current_fields: HashMap<String, (u32, FieldType)> = fields
        .drain(..)
        .map(|field| (field.name, (field.number, field.typ)))
        .collect();
    let mut new_fields = Vec::with_capacity(other.len());
    for field in other {
        if let Some((current_number, current_type)) = current_fields.remove(&field.name) {
            if field.typ != current_type {
                return Err(Error::FieldConflict(field.name.clone()));
            }
            new_fields.push(Field {
                name: field.name.clone(),
                typ: field.typ.clone(),
                number: current_number,
            });
        } else {
            new_fields.push(Field {
                name: field.name.clone(),
                typ: field.typ.clone(),
                number: *next_number,
            });
            *next_number += 1;
        }
    }
    for (number, _) in current_fields.values() {
        reserved.push(*number);
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
            /// New fields start at `number_offset`.
            pub fn merge(&mut self, other: Self, number_offset: u32) -> Result<(), Error> {
                let mut next_number = self.next_number(number_offset);
                merge_fields(
                    &mut self.fields,
                    &other.fields,
                    &mut self.reserved,
                    &mut next_number,
                )?;
                Ok(())
            }

            /// Return the next field number, considering all defined fields and reserved numbers.
            fn next_number(&self, number_offset: u32) -> u32 {
                self.fields
                    .iter()
                    .map(|f| f.number)
                    .chain(self.reserved.iter().copied())
                    .max()
                    .unwrap_or(number_offset - 1)
                    + 1
            }
        }
    };
}

impl_message_like!(ExtendBlock, "A protobuf extension.");
impl_message_like!(Message, "A protobuf message.");

impl File {
    pub fn merge(&mut self, other: Self, number_offset: u32) -> Result<(), Error> {
        if self.package != other.package {
            return Err(Error::PackageConflict {
                current: self.package.clone(),
                other: other.package,
            });
        }
        for ext in other.extensions {
            if let Some(existing) = self.extensions.iter_mut().find(|e| e.name == ext.name) {
                existing.merge(ext, number_offset)?;
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
