use protobuf::UnknownValueRef;
use protobuf::reflect::FieldDescriptor;

use crate::{Result, proto};

pub const EXTENSION_NUMBER: u32 = 50_000;

/// Extract the [`proto::Field`] field options, if they exist.
///
/// This inspects unknown fields because `protobuf` 3.x does not support an extension registry.
pub fn get_field_options(field: &FieldDescriptor) -> Result<Option<proto::Field>> {
    use protobuf::Message;
    let field_proto = field.proto();
    let unknown_fields = field_proto.options.special_fields.unknown_fields();
    let mut field = proto::Field::new();
    let mut found = false;
    for (number, val) in unknown_fields.iter() {
        if number == EXTENSION_NUMBER
            && let UnknownValueRef::LengthDelimited(b) = val
        {
            field.merge_from_bytes(b)?;
            found = true;
        }
    }
    Ok(if found { Some(field) } else { None })
}

/// Return `name` if specified, otherwise the field name.
pub fn property_name<'a>(field: &'a FieldDescriptor, options: &'a proto::Field) -> &'a str {
    if options.has_name() {
        return options.name();
    }
    field.name()
}
