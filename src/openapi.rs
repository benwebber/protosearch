//! Extract data from an OpenAPI specification.
use std::collections::HashMap;

use openapiv3::{
    AdditionalProperties, ArrayType, Components, ReferenceOr, Schema, SchemaKind, Type,
};

use crate::error::{Error, Result};
use crate::spec;

/// Resolve a schema reference to a [`openapiv3::Schema`].
pub fn resolve<'a>(components: &'a Components, reference: &str) -> Result<&'a Schema> {
    let name = schema_name(reference);
    let schema_ref = components
        .schemas
        .get(name)
        .ok_or_else(|| Error::InvalidSpec(format!("schema not found: {name}")))?;
    schema_ref
        .as_item()
        .ok_or_else(|| Error::InvalidSpec(format!("expected item, got reference: {name}")))
}

/// Collect parameters from a property schema into `parameters`.
///
/// Recursively follows `allOf` subschemas to collect all parameters supported by the property.
pub fn collect_parameters_into(
    components: &Components,
    schema: &Schema,
    parameters: &mut HashMap<String, spec::Parameter>,
) -> Result<()> {
    match &schema.schema_kind {
        SchemaKind::Type(Type::Object(obj)) => {
            for (name, prop) in &obj.properties {
                parameters.insert(name.clone(), parameter_from_ref(components, prop)?);
            }
        }
        SchemaKind::AllOf { all_of } => {
            for item in all_of {
                let s = match item {
                    ReferenceOr::Reference { reference } => resolve(components, reference)?,
                    ReferenceOr::Item(s) => s,
                };
                collect_parameters_into(components, s, parameters)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn parameter_from_ref(
    components: &Components,
    reference: &ReferenceOr<Box<Schema>>,
) -> Result<spec::Parameter> {
    match reference {
        ReferenceOr::Reference { reference } => {
            let _schema = resolve(components, reference)?;
            Ok(spec::Parameter::Value(value_type_from_ref(
                components, reference,
            )?))
        }
        ReferenceOr::Item(schema) => parameter_from_schema(components, schema),
    }
}

fn parameter_from_schema(components: &Components, schema: &Schema) -> Result<spec::Parameter> {
    match &schema.schema_kind {
        SchemaKind::Type(Type::Array(arr)) => {
            let item_type = match &arr.items {
                Some(ReferenceOr::Reference { reference }) => {
                    value_type_from_ref(components, reference)?
                }
                Some(ReferenceOr::Item(s)) => value_type_from_schema(components, s)?,
                None => return Err(Error::InvalidSpec("array schema has no items".into())),
            };
            Ok(spec::Parameter::Repeated(item_type))
        }
        SchemaKind::Type(Type::Object(_)) if is_metadata_object(components, schema)? => {
            Ok(spec::Parameter::Map(
                spec::ScalarType::String,
                spec::ValueType::Scalar(spec::ScalarType::String),
            ))
        }
        SchemaKind::OneOf { one_of } | SchemaKind::AnyOf { any_of: one_of } => {
            for item in one_of {
                if let ReferenceOr::Item(s) = item
                    && matches!(&s.schema_kind, SchemaKind::Type(Type::Array(_)))
                {
                    return parameter_from_schema(components, s);
                }
            }
            Ok(spec::Parameter::Value(value_type_from_schema(
                components, schema,
            )?))
        }
        _ => Ok(spec::Parameter::Value(value_type_from_schema(
            components, schema,
        )?)),
    }
}

fn value_type_from_ref(components: &Components, reference: &str) -> Result<spec::ValueType> {
    let schema = resolve(components, reference)?;
    if schema.schema_data.discriminator.is_some() {
        return Ok(spec::ValueType::Object);
    }
    match &schema.schema_kind {
        SchemaKind::Type(Type::Object(obj)) if !obj.properties.is_empty() => Ok(
            spec::ValueType::Definition(schema_name(reference).to_string()),
        ),
        SchemaKind::AllOf { .. } => Ok(spec::ValueType::Definition(
            schema_name(reference).to_string(),
        )),
        _ => value_type_from_schema(components, schema),
    }
}

fn value_type_from_schema(components: &Components, schema: &Schema) -> Result<spec::ValueType> {
    match &schema.schema_kind {
        SchemaKind::Type(typ) => match typ {
            Type::String(_) => Ok(spec::ValueType::Scalar(spec::ScalarType::String)),
            Type::Boolean(_) => Ok(spec::ValueType::Scalar(spec::ScalarType::Boolean)),
            Type::Number(_) => Ok(spec::ValueType::Scalar(spec::ScalarType::Double)),
            Type::Integer(_) => Ok(spec::ValueType::Scalar(spec::ScalarType::Integer)),
            Type::Object(_obj) => Ok(spec::ValueType::Object),
            Type::Array(arr) => array_value_type(components, arr),
        },
        SchemaKind::AllOf { all_of } if all_of.len() == 1 => match &all_of[0] {
            ReferenceOr::Reference { reference } => value_type_from_ref(components, reference),
            ReferenceOr::Item(s) => value_type_from_schema(components, s),
        },
        SchemaKind::OneOf { one_of } | SchemaKind::AnyOf { any_of: one_of } => {
            for item in one_of {
                match item {
                    ReferenceOr::Reference { reference } => {
                        return value_type_from_ref(components, reference);
                    }
                    ReferenceOr::Item(s) if !s.schema_data.nullable => {
                        return value_type_from_schema(components, s);
                    }
                    _ => {}
                }
            }
            Err(Error::InvalidSpec("unresolvable schema type".into()))
        }
        _ => Err(Error::InvalidSpec("unresolvable schema type".into())),
    }
}

fn array_value_type(components: &Components, arr: &ArrayType) -> Result<spec::ValueType> {
    match &arr.items {
        Some(ReferenceOr::Reference { reference }) => value_type_from_ref(components, reference),
        Some(ReferenceOr::Item(s)) => value_type_from_schema(components, s),
        None => Err(Error::InvalidSpec("array schema has no items".into())),
    }
}

fn is_metadata_object(components: &Components, schema: &Schema) -> Result<bool> {
    if let SchemaKind::Type(Type::Object(obj)) = &schema.schema_kind
        && let Some(AdditionalProperties::Schema(boxed)) = &obj.additional_properties
        && let ReferenceOr::Item(s) = boxed.as_ref()
    {
        return Ok(value_type_from_schema(components, s)?
            == spec::ValueType::Scalar(spec::ScalarType::String));
    }
    Ok(false)
}

/// Extract the schema type name from an OpenAPI schema reference.
pub fn schema_name(reference: &str) -> &str {
    reference
        .strip_prefix("#/components/schemas/")
        .unwrap_or(reference)
}
