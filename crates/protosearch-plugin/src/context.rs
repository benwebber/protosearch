use std::collections::HashMap;

use protobuf::descriptor::FileDescriptorProto;
use protobuf::reflect::FileDescriptor;

use crate::Result;

pub struct Context {
    file_descriptors_by_name: HashMap<String, FileDescriptor>,
    target: Option<String>,
}

impl Context {
    pub fn new(protos: Vec<FileDescriptorProto>, target: Option<&str>) -> Result<Self> {
        let file_descriptors = FileDescriptor::new_dynamic_fds(protos, &[])?;
        let mut file_descriptors_by_name: HashMap<String, FileDescriptor> = HashMap::new();
        for fd in file_descriptors {
            file_descriptors_by_name.insert(fd.name().to_string(), fd);
        }
        Ok(Context {
            file_descriptors_by_name,
            target: target.map(str::to_string),
        })
    }

    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    pub fn get_file_descriptor_by_name(&self, name: &str) -> Option<&FileDescriptor> {
        self.file_descriptors_by_name.get(name)
    }
}
