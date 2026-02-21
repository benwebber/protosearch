use std::collections::{HashMap, HashSet};

use protobuf::descriptor::FileDescriptorProto;
use protobuf::{UnknownValueRef, reflect};

use crate::dialect::Dialect;
use crate::{Error, Result};

pub struct FieldExtension {
    pub field_name: String,
    pub package: String,
    pub message: Box<dyn protobuf::MessageDyn>,
}

pub struct Context {
    file_descriptors_by_name: HashMap<String, reflect::FileDescriptor>,
    extensions_by_number: HashMap<u32, reflect::MessageDescriptor>,
    extension_names_by_number: HashMap<u32, String>,
    field_mapping_options_descriptor: reflect::MessageDescriptor,
}

impl Context {
    pub fn new(protos: Vec<FileDescriptorProto>) -> Result<Self> {
        let file_descriptors = reflect::FileDescriptor::new_dynamic_fds(protos, &[])?;
        let mut file_descriptors_by_name: HashMap<String, reflect::FileDescriptor> = HashMap::new();
        let mut extensions_by_number: HashMap<u32, reflect::MessageDescriptor> = HashMap::new();
        let mut extension_names_by_number: HashMap<u32, String> = HashMap::new();
        for fd in file_descriptors.into_iter() {
            for ext in fd.extensions() {
                if ext.containing_message().full_name() == crate::EXTENSION_MESSAGE_NAME
                    && let reflect::RuntimeType::Message(message_descriptor) =
                        ext.singular_runtime_type()
                {
                    let number = ext.number() as u32;
                    extensions_by_number.insert(number, message_descriptor);
                    extension_names_by_number.insert(number, ext.name().to_string());
                }
            }
            file_descriptors_by_name.insert(fd.name().to_string(), fd);
        }
        let field_mapping_options_descriptor = &file_descriptors_by_name
            .values()
            .find_map(|fd| fd.message_by_full_name(&format!(".{}", crate::EXTENSION_MESSAGE_NAME)))
            .ok_or(Error::InvalidRequest(
                "protosInvalidRequestMappingOptions not found".to_string(),
            ))?;
        Ok(Context {
            file_descriptors_by_name,
            extensions_by_number,
            extension_names_by_number,
            // TODO: Avoid clone by storing map of all extensions in struct, something like (u32, Option<u32>).
            field_mapping_options_descriptor: field_mapping_options_descriptor.clone(),
        })
    }

    pub fn get_file_descriptor_by_name(&self, name: &str) -> Option<&reflect::FileDescriptor> {
        self.file_descriptors_by_name.get(name)
    }

    pub fn get_field_mapping_options(
        &self,
        field: &reflect::FieldDescriptor,
    ) -> Result<Option<Box<dyn protobuf::MessageDyn>>> {
        let proto = field.proto();
        let unknown_fields = proto.options.special_fields.unknown_fields();
        match unknown_fields.get(crate::EXTENSION_NUMBER) {
            Some(protobuf::UnknownValueRef::LengthDelimited(bytes)) => Ok(Some(
                self.field_mapping_options_descriptor
                    .parse_from_bytes(bytes)?,
            )),
            _ => Ok(None),
        }
    }

    pub fn get_extension_by_number(&self, number: u32) -> Option<&reflect::MessageDescriptor> {
        self.extensions_by_number.get(&number)
    }

    pub fn get_property_name(
        &self,
        field: &reflect::FieldDescriptor,
        options: &dyn protobuf::MessageDyn,
    ) -> String {
        let name_field = self.field_mapping_options_descriptor.field_by_name("name");
        if let Some(name_field) = name_field
            && let reflect::ReflectFieldRef::Optional(v) = name_field.get_reflect(options)
            && let Some(reflect::ReflectValueRef::String(name)) = v.value()
            && !name.is_empty()
        {
            return name.to_string();
        }
        field.name().to_string()
    }

    pub fn get_property(
        &self,
        options: &dyn protobuf::MessageDyn,
    ) -> Result<Option<FieldExtension>> {
        let unknown_fields = options.special_fields_dyn().unknown_fields();
        for (tag, val) in unknown_fields.iter() {
            if let Some(msg_desc) = self.extensions_by_number.get(&tag)
                && let UnknownValueRef::LengthDelimited(bytes) = val
                && let Some(field_name) = self.extension_names_by_number.get(&tag)
            {
                let message = msg_desc.parse_from_bytes(bytes)?;
                let package = msg_desc.file_descriptor().proto().package().to_string();
                return Ok(Some(FieldExtension {
                    field_name: field_name.clone(),
                    package,
                    message,
                }));
            }
        }
        Ok(None)
    }

    pub fn get_dialects(&self, message: &reflect::MessageDescriptor) -> Result<HashSet<Dialect>> {
        let mut dialects = HashSet::new();
        for field in message.fields() {
            if let Some(options) = self.get_field_mapping_options(&field)?
                && let Some(ext) = self.get_property(&*options)?
            {
                dialects.insert(Dialect::new(ext.package));
            }
        }
        Ok(dialects)
    }
}
