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

pub fn process(request: &CodeGeneratorRequest) -> Result<CodeGeneratorResponse> {
    let mut response = CodeGeneratorResponse::new();
    response.set_supported_features(Feature::FEATURE_PROTO3_OPTIONAL as u64);
    let target = parse_target(request.parameter());
    let ctx = Context::new(request.proto_file.clone(), target)?;
    for filename in &request.file_to_generate {
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
    let Some(options) = get_field_mapping_options(field)? else {
        return Ok(None);
    };
    let name = property_name(field, &options);
    let property = match ctx
        .target()
        .and_then(|label| options.output.target.iter().find(|t| t.label() == label))
    {
        Some(entry) => {
            let json: Value =
                serde_json::from_str(entry.json()).map_err(|e| Error::InvalidJson {
                    field: field.name().to_string(),
                    source: e,
                })?;
            let Value::Object(mut params) = json else {
                return Err(Error::InvalidJsonObject(field.name().into()));
            };
            let typ = params
                .remove("type")
                .and_then(|v| v.as_str().map(str::to_string))
                .unwrap_or_default();
            Property::Leaf {
                typ,
                parameters: params.into_iter().collect(),
            }
        }
        None => {
            let typ = if !options.type_().is_empty() {
                options.type_().to_string()
            } else {
                infer_type(field).to_string()
            };
            Property::from_options(&options, typ)
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
        (false, Property::Leaf { typ, .. }) => Property::Mapping {
            typ,
            properties: mapping,
        },
        (_, property) => property,
    };
    Ok(Some((name.to_string(), property)))
}

fn parse_target(parameter: &str) -> Option<&str> {
    parameter
        .split(',')
        .find_map(|kv| kv.strip_prefix("target="))
}

/// Return `output.name` if specified, otherwise the field name.
fn property_name<'a>(field: &'a FieldDescriptor, options: &'a proto::FieldMapping) -> &'a str {
    let name = options.output.name();
    if !name.is_empty() {
        return name;
    }
    field.name()
}

/// Extract the specified [`proto::FieldMapping`], if they exist.
///
/// This inspects unknown fields because `protobuf` 3.x does not support an extension registry.
fn get_field_mapping_options(field: &FieldDescriptor) -> Result<Option<proto::FieldMapping>> {
    let field_proto = field.proto();
    let unknown_fields = field_proto.options.special_fields.unknown_fields();
    let mut bytes: Vec<u8> = Vec::new();
    let mut found = false;
    for (number, val) in unknown_fields.iter() {
        if number == EXTENSION_NUMBER
            && let UnknownValueRef::LengthDelimited(b) = val
        {
            bytes.extend_from_slice(b);
            found = true;
        }
    }
    if !found {
        return Ok(None);
    }
    Ok(Some(proto::FieldMapping::parse_from_bytes(&bytes)?))
}

fn infer_type(field: &FieldDescriptor) -> InferredType {
    let rt = match field.runtime_field_type() {
        RuntimeFieldType::Singular(t) | RuntimeFieldType::Repeated(t) => t,
        RuntimeFieldType::Map(_, _) => return InferredType::Object,
    };
    InferredType::from(rt)
}

#[cfg(test)]
mod tests {
    macro_rules! test_parse_target {
        ($name:ident, $param:expr, $expected:expr) => {
            #[test]
            fn $name() {
                assert_eq!(super::parse_target($param), $expected);
            }
        };
    }

    test_parse_target!(parse_target, "target=elasticsearch", Some("elasticsearch"));
    test_parse_target!(parse_target_missing, "foo=bar", None);
    test_parse_target!(parse_target_empty, "", None);
    test_parse_target!(
        parse_target_multiple_parameters,
        "foo=bar,target=elasticsearch",
        Some("elasticsearch")
    );
}
