mod config;
mod context;
mod diagnostic;
mod error;
mod mapping;
mod options;
mod plugin;
mod span;
mod validator;

#[allow(warnings, clippy::all)]
mod proto {
    include!(concat!(env!("OUT_DIR"), "/protosearch.rs"));
}

pub use diagnostic::{Diagnostic, DiagnosticKind, Location};
pub use error::{Error, Result};
pub use plugin::process;
pub use span::{Point, Span};
pub use validator::validate;

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use std::sync::LazyLock;

    use protobuf::Message;
    use protobuf::descriptor::{FileDescriptorProto, FileDescriptorSet};
    use protobuf::plugin::CodeGeneratorRequest;
    use serde_json::Value;

    use crate::diagnostic::DiagnosticKind;

    static DESCRIPTORS: LazyLock<Vec<FileDescriptorProto>> = LazyLock::new(|| {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let proto_dir = manifest_dir.join("../../proto");
        let out = PathBuf::from(env!("OUT_DIR")).join("tests.pb");
        let protoc = protoc_bin_vendored::protoc_bin_path().expect("cannot find bundled protoc");
        let status = std::process::Command::new(&protoc)
            .arg("-I")
            .arg(&proto_dir)
            .arg("--include_imports")
            .arg("--include_source_info")
            .arg("--descriptor_set_out")
            .arg(&out)
            .arg("tests/tests.proto")
            .status()
            .expect("failed to execute protoc");
        assert!(status.success(), "protoc failed with status {status}");
        let bytes = std::fs::read(&out).unwrap();
        FileDescriptorSet::parse_from_bytes(&bytes).unwrap().file
    });

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
        let mut req = CodeGeneratorRequest::new();
        if let Some(t) = target {
            req.set_parameter(format!("target={t}"))
        }
        req.file_to_generate.push(file_to_generate.to_string());
        req.proto_file = DESCRIPTORS.clone();
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

    test_snapshot!(test_no_target, "tests.FieldTestCase", None);
    test_snapshot!(test_infer_type, "tests.InferTypeTestCase", None);
    test_snapshot!(test_target, "tests.FieldTestCase", Some("foo"));
    test_snapshot!(test_index_params, "tests.IndexTestCase", None);
    test_snapshot!(test_enum, "tests.EnumTestCase", None);
    test_snapshot!(test_nested, "tests.MessageTestCase", None);
    test_snapshot!(test_dynamic, "tests.DynamicTestCase", None);
    test_snapshot!(test_index_options, "tests.IndexOptionsTestCase", None);
    test_snapshot!(test_term_vector, "tests.TermVectorTestCase", None);

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

    macro_rules! test_parameter_value {
        ($test_name:ident, $field:literal, $parameter:literal, true) => {
            #[test]
            fn $test_name() {
                let req = make_request("tests/tests.proto", None);
                let (_resp, diagnostics) = crate::process(req).unwrap();
                assert!(!diagnostics.iter().any(|d| matches!(
                    &d.kind,
                    DiagnosticKind::InvalidParameterValue { field, parameter, .. } if field == $field && parameter == $parameter
                )));
            }
        };
        ($test_name:ident, $field:literal, $parameter:literal, false) => {
            #[test]
            fn $test_name() {
                let req = make_request("tests/tests.proto", None);
                let (_resp, diagnostics) = crate::process(req).unwrap();
                assert!(diagnostics.iter().any(|d| matches!(
                    &d.kind,
                    DiagnosticKind::InvalidParameterValue { field, parameter, .. } if field == $field && parameter == $parameter
                )));
            }
        };
    }

    test_parameter_value!(test_ignore_above_valid, "valid", "ignore_above", true);
    test_parameter_value!(test_ignore_above_zero, "zero", "ignore_above", false);
    test_parameter_value!(
        test_ignore_above_negative,
        "negative",
        "ignore_above",
        false
    );

    test_parameter_value!(
        test_position_increment_gap_zero,
        "zero",
        "position_increment_gap",
        true
    );
    test_parameter_value!(
        test_position_increment_gap_valid,
        "valid",
        "position_increment_gap",
        true
    );
    test_parameter_value!(
        test_position_increment_gap_negative,
        "negative",
        "position_increment_gap",
        false
    );

    test_parameter_value!(
        test_index_prefixes_valid_min_chars,
        "valid",
        "index_prefixes.min_chars",
        true
    );
    test_parameter_value!(
        test_index_prefixes_valid_max_chars,
        "valid",
        "index_prefixes.max_chars",
        true
    );
    test_parameter_value!(
        test_index_prefixes_min_chars_negative,
        "min_chars_negative",
        "index_prefixes.min_chars",
        false
    );
    test_parameter_value!(
        test_index_prefixes_max_chars_negative,
        "max_chars_negative",
        "index_prefixes.max_chars",
        false
    );

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
    fn test_unknown_target() {
        let req = make_request("tests/tests.proto", Some("bar"));
        let (_resp, diagnostics) = crate::process(req).unwrap();
        let expected = DiagnosticKind::UnknownTarget {
            message: "FieldTestCase".to_string(),
            field: "output_target".to_string(),
            label: "bar".to_string(),
        };
        assert!(diagnostics.iter().any(|d| d.kind == expected));
    }

    #[test]
    fn test_nested_field_location() {
        let req = make_request("tests/tests.proto", None);
        let (_resp, diagnostics) = crate::process(req).unwrap();
        let diagnostic = diagnostics.iter().find(|d| {
            matches!(
                &d.kind,
                DiagnosticKind::InvalidFieldName { message, name, .. }
                    if message == "tests.NestedValidationTestCase.Inner" && name == "BadField"
            )
        });
        assert!(
            diagnostic.is_some(),
            "expected InvalidFieldName for nested field"
        );
        assert!(
            diagnostic
                .unwrap()
                .location
                .as_ref()
                .and_then(|l| l.span.as_ref())
                .is_some(),
            "diagnostic has no span",
        );
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
