use protobuf::plugin::{
    CodeGeneratorRequest, CodeGeneratorResponse,
    code_generator_response::{Feature, File},
};
use protobuf::reflect::{FieldDescriptor, MessageDescriptor, RuntimeFieldType, RuntimeType};
use protobuf::{Message, UnknownValueRef};
use serde_json::Value;

use crate::context::Context;
use crate::mapping::{InferredType, Mapping, Property};
use crate::{Error, Result, proto};

const EXTENSION_NUMBER: u32 = 50_000;

pub fn process(request: CodeGeneratorRequest) -> Result<CodeGeneratorResponse> {
    let mut response = CodeGeneratorResponse::new();
    response.set_supported_features(Feature::FEATURE_PROTO3_OPTIONAL as u64);
    let ctx = Context::try_from(request)?;
    for filename in &ctx.files_to_generate {
        let file_descriptor =
            ctx.get_file_descriptor_by_name(filename)
                .ok_or(Error::InvalidRequest(format!(
                    "missing descriptor for {filename}"
                )))?;
        for message_descriptor in file_descriptor.messages() {
            let mapping = compile_message(&ctx, &message_descriptor)?;
            if mapping.properties.is_empty() {
                continue;
            }
            let mut file = File::new();
            file.set_name(format!("{}.json", message_descriptor.full_name()));
            file.set_content(serde_json::to_string(&mapping)?);
            response.file.push(file);
        }
    }
    Ok(response)
}

/// Compile a message as a document mapping.
fn compile_message(ctx: &Context, message: &MessageDescriptor) -> Result<Mapping> {
    let mut mapping = Mapping::default();
    for field in message.fields() {
        if let Some((name, property)) = compile_field(ctx, &field)? {
            mapping.properties.insert(name, property);
        }
    }
    Ok(mapping)
}

/// Compile a field as a [`Property`].
///
/// Returns `(name, property)`.
fn compile_field(ctx: &Context, field: &FieldDescriptor) -> Result<Option<(String, Property)>> {
    let Some(options) = get_mapping_options(field)? else {
        return Ok(None);
    };
    let name = property_name(field, &options);
    let property = match ctx
        .target()
        .and_then(|label| options.target.iter().find(|t| t.label() == label))
    {
        Some(entry) => {
            let json: Value =
                serde_json::from_str(entry.json()).map_err(|e| Error::InvalidJson {
                    field: field.name().to_string(),
                    source: e,
                })?;
            let Value::Object(params) = json else {
                return Err(Error::InvalidJsonObject(field.name().into()));
            };
            Property::Leaf(params.into_iter().collect())
        }
        None => {
            let mut property = Property::from(&*options.field);
            if let Property::Leaf(ref mut parameters) = property {
                parameters
                    .entry("type".into())
                    .or_insert_with(|| Value::String(InferredType::from(field).to_string()));
            }
            property
        }
    };
    // A mapping type, as in an object or nested field.
    let mapping = match field.runtime_field_type() {
        RuntimeFieldType::Singular(RuntimeType::Message(desc))
        | RuntimeFieldType::Repeated(RuntimeType::Message(desc)) => Some(desc),
        _ => None,
    }
    .map(|desc| compile_message(ctx, &desc))
    .transpose()?
    .unwrap_or_default();
    let property = match (mapping.properties.is_empty(), property) {
        (false, Property::Leaf(parameters)) => Property::Mapping {
            parameters,
            properties: mapping,
        },
        (_, property) => property,
    };
    Ok(Some((name.to_string(), property)))
}

/// Return `name` if specified, otherwise the field name.
fn property_name<'a>(field: &'a FieldDescriptor, options: &'a proto::Mapping) -> &'a str {
    let name = options.name();
    if !name.is_empty() {
        return name;
    }
    field.name()
}

/// Extract the specified [`proto::Mapping`], if they exist.
///
/// This inspects unknown fields because `protobuf` 3.x does not support an extension registry.
fn get_mapping_options(field: &FieldDescriptor) -> Result<Option<proto::Mapping>> {
    let field_proto = field.proto();
    let unknown_fields = field_proto.options.special_fields.unknown_fields();
    let mut mapping = proto::Mapping::new();
    let mut found = false;
    for (number, val) in unknown_fields.iter() {
        if number == EXTENSION_NUMBER
            && let UnknownValueRef::LengthDelimited(b) = val
        {
            mapping.merge_from_bytes(b)?;
            found = true;
        }
    }
    Ok(if found { Some(mapping) } else { None })
}
