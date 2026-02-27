use std::collections::HashMap;

use protobuf::plugin::CodeGeneratorRequest;
use protobuf::reflect::FileDescriptor;

use crate::Result;

pub struct Context {
    file_descriptors_by_name: HashMap<String, FileDescriptor>,
    pub files_to_generate: Vec<String>,
    parameter: String,
}

impl TryFrom<CodeGeneratorRequest> for Context {
    type Error = crate::Error;

    fn try_from(request: CodeGeneratorRequest) -> Result<Self> {
        let file_descriptors_by_name = FileDescriptor::new_dynamic_fds(request.proto_file, &[])?
            .into_iter()
            .map(|fd| (fd.name().into(), fd))
            .collect();
        Ok(Context {
            file_descriptors_by_name,
            files_to_generate: request.file_to_generate,
            parameter: request.parameter.unwrap_or_default(),
        })
    }
}

impl Context {
    pub fn target(&self) -> Option<&str> {
        parse_target(&self.parameter)
    }

    pub fn get_file_descriptor_by_name(&self, name: &str) -> Option<&FileDescriptor> {
        self.file_descriptors_by_name.get(name)
    }
}

fn parse_target(parameter: &str) -> Option<&str> {
    parameter
        .split(',')
        .find_map(|kv| kv.strip_prefix("target="))
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
