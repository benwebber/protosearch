mod context;
mod diagnostic;
mod error;
mod mapping;
mod plugin;
mod validator;

#[allow(warnings, clippy::all)]
mod proto {
    include!(concat!(env!("OUT_DIR"), "/protosearch.rs"));
}

pub use diagnostic::{Diagnostic, DiagnosticKind, Location};
pub use error::{Error, Result};
pub use plugin::process;
pub use validator::validate;

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use protobuf::Message;
    use protobuf::descriptor::FileDescriptorSet;
    use protobuf::plugin::CodeGeneratorRequest;
    use serde_json::Value;

    use crate::diagnostic::DiagnosticKind;

    macro_rules! test_snapshot {
        ($name:ident, $test:expr, $target:expr) => {
            #[test]
            fn $name() {
                let req = make_request("tests/tests.proto", $target);
                let (resp, _diagnostics) = crate::process(req).unwrap();
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
    test_snapshot!(test_infer_type, "tests.InferTypeTestCase", None);
    test_snapshot!(test_target, "tests.TestCase", Some("foo"));
    test_snapshot!(test_fielddata, "tests.FieldDataTestCase", None);
    test_snapshot!(test_enum, "tests.EnumTestCase", None);
    test_snapshot!(test_nested, "tests.MessageTestCase", None);

    macro_rules! test_field_name {
        ($test_name:ident, $name:literal, true) => {
            #[test]
            fn $test_name() {
                let req = make_request("tests/tests.proto", None);
                let (_resp, diagnostics) = crate::process(req).unwrap();
                assert!(!diagnostics.iter().any(|d| matches!(
                    &d.kind,
                    DiagnosticKind::InvalidFieldName { name, .. } if name == $name
                )));
            }
        };
        ($test_name:ident, $name:literal, false) => {
            #[test]
            fn $test_name() {
                let req = make_request("tests/tests.proto", None);
                let (_resp, diagnostics) = crate::process(req).unwrap();
                assert!(diagnostics.iter().any(|d| matches!(
                    &d.kind,
                    DiagnosticKind::InvalidFieldName { name, .. } if name == $name
                )));
            }
        };
    }

    test_field_name!(test_field_name_valid, "valid", true);
    test_field_name!(test_field_name_with_underscore, "valid_name", true);
    test_field_name!(test_field_name_with_digit, "field1", true);
    test_field_name!(test_field_name_at_prefix, "@timestamp", true);
    test_field_name!(test_field_name_dotted, "object.field", true);

    test_field_name!(test_field_name_empty, "", false);
    test_field_name!(test_field_name_uppercase, "Title", false);
    test_field_name!(test_field_name_leading_digit, "1field", false);
    test_field_name!(test_field_name_hyphen, "field-name", false);

    #[test]
    fn test_invalid_json_target_string() {
        let req = make_request("tests/tests.proto", Some("invalid-json-string"));
        let (_resp, diagnostics) = crate::process(req).unwrap();
        let expected = DiagnosticKind::InvalidTargetJson {
            message: "InvalidTargetJsonTestCase".to_string(),
            field: "invalid_json".to_string(),
            label: "invalid-json-string".to_string(),
        };
        assert!(diagnostics.iter().any(|d| d.kind == expected));
    }

    #[test]
    fn test_non_object_json_target() {
        let req = make_request("tests/tests.proto", Some("invalid-json-array"));
        let (_resp, diagnostics) = crate::process(req).unwrap();
        let expected = DiagnosticKind::InvalidTargetJsonType {
            message: "InvalidTargetJsonTestCase".to_string(),
            field: "invalid_json".to_string(),
            label: "invalid-json-array".to_string(),
        };
        assert!(diagnostics.iter().any(|d| d.kind == expected));
    }

    #[test]
    fn test_missing_descriptor() {
        let req = make_request("missing.proto", None);
        assert!(matches!(
            crate::process(req).unwrap_err(),
            crate::Error::InvalidRequest(_)
        ));
    }
}
