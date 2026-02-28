use std::collections::HashMap;

use protobuf::plugin::CodeGeneratorRequest;
use protobuf::reflect::FileDescriptor;

use crate::Result;
use crate::config::Config;

pub struct Context {
    file_descriptors_by_name: HashMap<String, FileDescriptor>,
    pub files_to_generate: Vec<String>,
    config: Config,
}

impl TryFrom<CodeGeneratorRequest> for Context {
    type Error = crate::Error;

    fn try_from(request: CodeGeneratorRequest) -> Result<Self> {
        let config = Config::try_from(request.parameter.as_deref().unwrap_or_default())?;
        let file_descriptors_by_name = FileDescriptor::new_dynamic_fds(request.proto_file, &[])?
            .into_iter()
            .map(|fd| (fd.name().into(), fd))
            .collect();
        Ok(Context {
            file_descriptors_by_name,
            files_to_generate: request.file_to_generate,
            config,
        })
    }
}

impl Context {
    pub fn target(&self) -> Option<&str> {
        self.config.target.as_deref()
    }

    pub fn get_file_descriptor_by_name(&self, name: &str) -> Option<&FileDescriptor> {
        self.file_descriptors_by_name.get(name)
    }
}
