use protobuf::plugin::{
    CodeGeneratorRequest, CodeGeneratorResponse,
    code_generator_response::{Feature, File},
};
use protobuf::reflect::{FieldDescriptor, MessageDescriptor, RuntimeFieldType, RuntimeType};
use serde_json::{Map, Value, json};

use crate::context::Context;
use crate::message::Message;
use crate::{Error, Result};

pub fn process(req: &CodeGeneratorRequest) -> Result<CodeGeneratorResponse> {
    let mut resp = CodeGeneratorResponse::new();
    resp.set_supported_features(Feature::FEATURE_PROTO3_OPTIONAL as u64);
    let ctx = Context::new(req.proto_file.clone())?;
    for filename in &req.file_to_generate {
        let file_descriptor =
            ctx.get_file_descriptor_by_name(filename)
                .ok_or(Error::InvalidRequest(format!(
                    "missing descriptor for {filename}"
                )))?;
        for message_descriptor in file_descriptor.messages() {
            let dialects = ctx.get_dialects(&message_descriptor)?;
            for dialect in dialects {
                let properties = compile(&ctx, &message_descriptor, dialect.package())?;
                if properties.is_empty() {
                    continue;
                }
                let mapping = json!({ "properties": properties });
                let mut file = File::new();
                file.set_name(format!(
                    "{}.{}.json",
                    message_descriptor.name(),
                    dialect.suffix()
                ));
                file.set_content(serde_json::to_string(&mapping)?);
                resp.file.push(file);
            }
        }
    }
    Ok(resp)
}

fn compile(
    ctx: &Context,
    message: &MessageDescriptor,
    package: &str,
) -> Result<Map<String, Value>> {
    let mut properties = Map::new();
    for field in message.fields() {
        let Some(options) = ctx.get_field_mapping_options(&field)? else {
            continue;
        };
        let Some(ext) = ctx.get_property(&*options)? else {
            continue;
        };
        if ext.package != package {
            continue;
        }
        let name = ctx.get_property_name(&field, &*options);
        let mut parameters = Map::new();
        parameters.insert("type".to_string(), Value::String(ext.field_name));
        if let Value::Object(m) = Message::from(&*ext.message).to_json() {
            for (k, v) in m {
                parameters.insert(k, v);
            }
        }
        if let Some(sub_desc) = sub_message_descriptor(&field) {
            let sub_properties = compile(ctx, &sub_desc, package)?;
            if !sub_properties.is_empty() {
                parameters.insert("properties".into(), Value::Object(sub_properties));
            }
        }
        properties.insert(name, Value::Object(parameters));
    }
    Ok(properties)
}

fn sub_message_descriptor(field: &FieldDescriptor) -> Option<MessageDescriptor> {
    match field.runtime_field_type() {
        RuntimeFieldType::Singular(RuntimeType::Message(desc))
        | RuntimeFieldType::Repeated(RuntimeType::Message(desc)) => Some(desc),
        _ => None,
    }
}
