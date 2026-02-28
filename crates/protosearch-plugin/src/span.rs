//! Represent spans in a protobuf source file.
//!
//! The [`descriptor.proto`] file in the `google.protobuf` package represents proto files
//! themselves. That file is thoroughly documented. Read it to better understand this module.
//!
//! [`descriptor.proto`]: https://github.com/protocolbuffers/protobuf/blob/v34.0/src/google/protobuf/descriptor.proto
use protobuf::reflect::{FieldDescriptor, MessageDescriptor};

/// A span of text between two points in a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Point,
    pub end: Point,
}

/// A point in a source file.
///
/// `line` and `column` both start from `1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub line: u32,
    pub column: u32,
}

impl Span {
    pub fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }

    /// Extract the span of a protobuf field.
    pub fn from_field(field: &FieldDescriptor) -> Option<Self> {
        const FIELD_NUMBER: i32 = 2;
        let message = field.containing_message();
        let file_proto = message.file_descriptor_proto();
        let mut path = message_path(&message)?;
        let idx = message
            .fields()
            .position(|f| f.number() == field.number())?;
        path.push(FIELD_NUMBER);
        path.push(idx as i32);
        let source_code_info = file_proto.source_code_info.as_ref()?;
        let location = source_code_info.location.iter().find(|l| l.path == path)?;
        Self::from_proto(&location.span)
    }

    /// Convert a protobuf `SourceCodeInfo.Location` span to a `Span`.
    ///
    /// A protobuf span always has three or four elements: start line, start column, end line
    /// (optional), end column. If the span has three elements, it means the start line and end
    /// line are the same.
    fn from_proto(span: &[i32]) -> Option<Self> {
        match *span {
            [l1, c1, c2] => {
                let start = Point::new(l1 as u32 + 1, c1 as u32 + 1);
                let end = Point::new(l1 as u32 + 1, c2 as u32 + 1);
                Some(Span::new(start, end))
            }
            [l1, c1, l2, c2] => {
                let start = Point::new(l1 as u32 + 1, c1 as u32 + 1);
                let end = Point::new(l2 as u32 + 1, c2 as u32 + 1);
                Some(Span::new(start, end))
            }
            _ => None,
        }
    }
}

impl Point {
    pub fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }
}

/// Build the `SourceCodeInfo` path to `message`.
///
/// A path is a sequence of field numbers and indices in the descriptor, starting from
/// `FileDescriptorProto`.
///
/// Consider this protobuf:
///
/// ```text
/// message Foo {}
///
/// message Bar {
///     message Baz {}
/// }
/// ```
///
/// In `FileDescriptorProto`, the field number for messages (`message_type`) is `4`. A file can
/// contain multiple messages, so this is a `repeated` field. `Foo` is the first message in
/// `message_type`.
///
/// The message path to `Foo` is:
///
/// ```text
/// 4 0
/// ```
///
/// Messages can have nested messages. In `DescriptorProto`, the field number for nested messages
/// (`nested_type`) is `3`. The path to `Bar.Baz` is:
///
/// ```text
/// 4 1 3 0
/// ```
///
/// This function reconstructs the path to a message by following its ancestors upwards, until the
/// message has no parent.
fn message_path(message: &MessageDescriptor) -> Option<Vec<i32>> {
    const MESSAGE_TYPE_NUMBER: i32 = 4;
    const NESTED_TYPE_NUMBER: i32 = 3;
    match message.enclosing_message() {
        None => {
            let idx = message
                .file_descriptor()
                .messages()
                .position(|m| m.full_name() == message.full_name())?;
            Some(vec![MESSAGE_TYPE_NUMBER, idx as i32])
        }
        Some(parent) => {
            let mut path = message_path(&parent)?;
            let idx = parent
                .nested_messages()
                .position(|m| m.full_name() == message.full_name())?;
            path.push(NESTED_TYPE_NUMBER);
            path.push(idx as i32);
            Some(path)
        }
    }
}
