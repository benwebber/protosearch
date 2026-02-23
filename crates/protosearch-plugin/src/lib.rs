mod context;
mod error;
mod mapping;
mod plugin;

#[allow(warnings, dead_code, unused_mut, unused_results, clippy::all)]
mod proto {
    include!(concat!(env!("OUT_DIR"), "/protosearch.rs"));
}

pub use error::{Error, Result};
pub use plugin::process;

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use protobuf::Message;
    use protobuf::descriptor::FileDescriptorSet;
    use protobuf::plugin::CodeGeneratorRequest;
    use serde_json::Value;

    macro_rules! test_snapshot {
        ($name:ident, $test:expr, $target:expr) => {
            #[test]
            fn $name() {
                let req = make_request("tests/tests.proto", $target);
                let resp = crate::process(&req).unwrap();
                insta::assert_json_snapshot!(output_for(&resp, $test));
            }
        };
    }

    fn make_request(file_to_generate: &str, target: Option<&str>) -> CodeGeneratorRequest {
        let pb = PathBuf::from(env!("OUT_DIR")).join("tests.pb");
        let bytes = std::fs::read(pb).unwrap();
        let fds = FileDescriptorSet::parse_from_bytes(&bytes).unwrap();
        let mut req = CodeGeneratorRequest::new();
        if let Some(t) = target {
            req.set_parameter(format!("target={t}"))
        }
        req.file_to_generate.push(file_to_generate.to_string());
        req.proto_file = fds.file;
        req
    }

    fn output_for(
        resp: &protobuf::plugin::CodeGeneratorResponse,
        message: &str,
    ) -> BTreeMap<String, Value> {
        resp.file
            .iter()
            .filter(|f| f.name().starts_with(&format!("{message}.")))
            .map(|f| {
                let content: Value = serde_json::from_str(f.content()).unwrap();
                (f.name().to_string(), content)
            })
            .collect()
    }

    test_snapshot!(test_no_target, "tests.TestCase", None);
    test_snapshot!(test_target, "tests.TestCase", Some("foo"));
    test_snapshot!(test_struct_fielddata, "tests.StructFielddataCase", None);
    test_snapshot!(test_list_fielddata, "tests.ListFielddataCase", None);
    test_snapshot!(test_null_fielddata, "tests.NullFielddataCase", None);
    test_snapshot!(test_nested, "tests.NestedCase", None);
    test_snapshot!(test_enum, "tests.EnumCase", None);
    test_snapshot!(test_infer_type, "tests.InferTypeTestCase", None);

    #[test]
    fn test_invalid_json_target() {
        let req = make_request("tests/tests.proto", Some("invalid-json"));
        assert!(matches!(
            crate::process(&req).unwrap_err(),
            crate::Error::InvalidJson { .. }
        ));
    }

    #[test]
    fn test_non_object_json_target() {
        let req = make_request("tests/tests.proto", Some("non-object"));
        assert!(matches!(
            crate::process(&req).unwrap_err(),
            crate::Error::InvalidRequest(_)
        ));
    }

    #[test]
    fn test_missing_descriptor() {
        let req = make_request("tests/nonexistent.proto", None);
        assert!(matches!(
            crate::process(&req).unwrap_err(),
            crate::Error::InvalidRequest(_)
        ));
    }
}
