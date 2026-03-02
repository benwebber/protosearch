use protobuf::plugin::{
    CodeGeneratorRequest, CodeGeneratorResponse,
    code_generator_response::{Feature, File},
};
use protobuf::reflect::{FieldDescriptor, MessageDescriptor, RuntimeFieldType, RuntimeType};
use serde_json::Value;

use crate::context::Context;
use crate::diagnostic::{Diagnostic, DiagnosticKind, Location};
use crate::mapping::{Mapping, Parameters, Property};
use crate::options::{get_field_options, get_index_options, property_name};
use crate::validator::{ValidationContext, validate};
use crate::{Error, Result, Span, proto};

pub fn process(request: CodeGeneratorRequest) -> Result<(CodeGeneratorResponse, Vec<Diagnostic>)> {
    let mut response = CodeGeneratorResponse::new();
    response.set_supported_features(Feature::FEATURE_PROTO3_OPTIONAL as u64);
    let ctx = Context::try_from(request)?;
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    for filename in &ctx.files_to_generate {
        let file_descriptor =
            ctx.get_file_descriptor_by_name(filename)
                .ok_or(Error::InvalidRequest(format!(
                    "missing descriptor for {filename}"
                )))?;
        for message_descriptor in file_descriptor.messages() {
            let validation_ctx = ValidationContext::new(filename, &message_descriptor);
            let mut message_diagnostics: Vec<Diagnostic> = Vec::new();
            let mapping = compile_message(
                &ctx,
                &message_descriptor,
                filename,
                &mut message_diagnostics,
            )?;
            message_diagnostics.extend(validate(&validation_ctx, &mapping));
            let has_errors = message_diagnostics.iter().any(|d| d.is_error());
            if (!mapping.properties.is_empty() || mapping.index.is_some()) && !has_errors {
                let mut file = File::new();
                file.set_name(format!("{}.json", message_descriptor.full_name()));
                file.set_content(serde_json::to_string(&mapping)?);
                response.file.push(file);
            }
            diagnostics.extend(message_diagnostics);
        }
    }
    Ok((response, diagnostics))
}

/// Compile a message as a document mapping.
fn compile_message(
    ctx: &Context,
    message: &MessageDescriptor,
    file: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<Mapping> {
    let mut mapping = Mapping::with_descriptor(message.clone());
    mapping.index = get_index_options(message)?;
    for field in message.fields() {
        if let Some((name, property)) = compile_field(ctx, &field, file, diagnostics)? {
            mapping.properties.insert(name, property);
        }
    }
    Ok(mapping)
}

/// Compile a field as a [`Property`].
///
/// Returns `(name, property)`.
fn compile_field(
    ctx: &Context,
    field: &FieldDescriptor,
    file: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<Option<(String, Property)>> {
    let Some(options) = get_field_options(field)? else {
        return Ok(None);
    };
    let name = property_name(field, &options);
    let location = Location {
        file: file.to_string(),
        span: Span::from_field(field),
    };
    let property = match ctx
        .target()
        .and_then(|label| options.target.iter().find(|t| t.label() == label))
    {
        Some(entry) => match serde_json::from_str::<Value>(entry.json()) {
            Ok(Value::Object(params)) => Property::Leaf(Parameters::Raw(params)),
            Ok(_) => {
                diagnostics.push(
                    Diagnostic::error(DiagnosticKind::InvalidTargetJsonType {
                        message: field.containing_message().name().to_string(),
                        field: field.name().to_string(),
                        label: entry.label().to_string(),
                    })
                    .at(location.clone()),
                );
                return Ok(None);
            }
            Err(_) => {
                diagnostics.push(
                    Diagnostic::error(DiagnosticKind::InvalidTargetJson {
                        message: field.containing_message().name().to_string(),
                        field: field.name().to_string(),
                        label: entry.label().to_string(),
                    })
                    .at(location.clone()),
                );
                return Ok(None);
            }
        },
        None if ctx.target().is_some() => {
            if !options.target.is_empty() {
                // If the field has any targets defined, emit a warning if the provided label does
                // not match any known targets.
                diagnostics.push(
                    Diagnostic::warning(DiagnosticKind::UnknownTarget {
                        message: field.containing_message().name().to_string(),
                        field: field.name().to_string(),
                        label: ctx.target().unwrap().to_string(),
                    })
                    .at(location.clone()),
                );
            }
            // Always return the default mapping.
            property(field, &options)
        }
        None => property(field, &options),
    };
    // A mapping type, as in an object or nested field.
    let mapping = match field.runtime_field_type() {
        RuntimeFieldType::Singular(RuntimeType::Message(desc))
        | RuntimeFieldType::Repeated(RuntimeType::Message(desc)) => Some(desc),
        _ => None,
    }
    .map(|desc| compile_message(ctx, &desc, file, diagnostics))
    .transpose()?
    .unwrap_or_default();
    let property = match (mapping.properties.is_empty(), property) {
        (false, Property::Leaf(parameters)) => Property::Object {
            parameters,
            properties: mapping,
        },
        (_, property) => property,
    };
    Ok(Some((name.to_string(), property)))
}

/// Build a [`Property`] from `FieldMapping`, inferring `type` if absent.
fn property(field: &FieldDescriptor, options: &proto::Field) -> Property {
    let field_mapping = options.mapping.clone().unwrap_or_default();
    let inferred_type = if field_mapping.has_type() {
        None
    } else {
        Some(match field.runtime_field_type() {
            RuntimeFieldType::Singular(t) | RuntimeFieldType::Repeated(t) => {
                infer_type(&t).to_string()
            }
            RuntimeFieldType::Map(_, _) => "object".to_string(),
        })
    };
    Property::Leaf(Parameters::Typed {
        field_mapping: Box::new(field_mapping),
        inferred_type,
    })
}

fn infer_type(t: &RuntimeType) -> &str {
    match t {
        RuntimeType::I32 => "integer",
        RuntimeType::I64 => "long",
        RuntimeType::U32 => "long",
        RuntimeType::U64 => "unsigned_long",
        RuntimeType::F32 => "float",
        RuntimeType::F64 => "double",
        RuntimeType::Bool => "boolean",
        RuntimeType::String => "keyword",
        RuntimeType::VecU8 => "binary",
        RuntimeType::Message(_) => "object",
        RuntimeType::Enum(_) => "keyword",
    }
}
