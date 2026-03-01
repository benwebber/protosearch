use std::collections::BTreeMap;
use std::sync::LazyLock;

use crate::diagnostic::{Diagnostic, DiagnosticKind, Location};
use crate::mapping::{Mapping, Parameters, Property};
use crate::options::{get_field_options, property_name};
use crate::proto::FieldMapping;
use crate::span::Span;
use protobuf::reflect::MessageDescriptor;
use regex::Regex;

static CHECKS: &[&dyn Check] = &[
    &InvalidNameCheck,
    &InvalidIgnoreAboveCheck,
    &InvalidPositionIncrementGapCheck,
    &InvalidIndexPrefixesCheck,
];

pub struct ValidationContext<'a> {
    pub file: &'a str,
    pub message: &'a MessageDescriptor,
    proto_names: BTreeMap<String, String>,
}

impl<'a> ValidationContext<'a> {
    pub fn new(file: &'a str, message: &'a MessageDescriptor) -> Self {
        let proto_names = message
            .fields()
            .filter_map(|f| {
                get_field_options(&f).ok().flatten().map(|opts| {
                    let output = property_name(&f, &opts);
                    (output.to_string(), f.name().to_string())
                })
            })
            .collect();
        Self {
            file,
            message,
            proto_names,
        }
    }

    pub fn proto_name<'b>(&'b self, mapping_name: &'b str) -> &'b str {
        self.proto_names
            .get(mapping_name)
            .map(String::as_str)
            .unwrap_or(mapping_name)
    }

    pub fn field_span(&self, proto_name: &str) -> Option<Span> {
        self.message
            .field_by_name(proto_name)
            .and_then(|f| Span::from_field(&f))
    }

    pub fn location(&self, proto_name: &str) -> Location {
        Location {
            file: self.file.to_string(),
            span: self.field_span(proto_name),
        }
    }
}

pub trait Check: Sync {
    fn check_property(
        &self,
        ctx: &ValidationContext<'_>,
        name: &str,
        property: &Property,
        diagnostics: &mut Vec<Diagnostic>,
    );
}

pub fn validate(ctx: &ValidationContext<'_>, mapping: &Mapping) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for (name, property) in &mapping.properties {
        walk(ctx, name, property, &mut diagnostics);
    }
    diagnostics
}

fn walk(
    ctx: &ValidationContext<'_>,
    name: &str,
    property: &Property,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for check in CHECKS {
        check.check_property(ctx, name, property, diagnostics);
    }
    if let Property::Object { properties, .. } = property {
        let nested_ctx;
        let ctx = if let Some(desc) = &properties.descriptor {
            nested_ctx = ValidationContext::new(ctx.file, desc);
            &nested_ctx
        } else {
            ctx
        };
        for (name, prop) in &properties.properties {
            walk(ctx, name, prop, diagnostics);
        }
    }
}

struct InvalidNameCheck;

impl Check for InvalidNameCheck {
    fn check_property(
        &self,
        ctx: &ValidationContext<'_>,
        name: &str,
        _property: &Property,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let proto_name = ctx.proto_name(name);
        static RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^[@a-z][a-z0-9_]*(\.[a-z0-9_]+)*$").unwrap());
        if !RE.is_match(name) {
            diagnostics.push(
                Diagnostic::warning(DiagnosticKind::InvalidFieldName {
                    message: ctx.message.full_name().to_string(),
                    field: proto_name.to_string(),
                    name: name.to_string(),
                })
                .at(ctx.location(proto_name)),
            );
        }
    }
}

struct InvalidIgnoreAboveCheck;

impl Check for InvalidIgnoreAboveCheck {
    fn check_property(
        &self,
        ctx: &ValidationContext<'_>,
        name: &str,
        property: &Property,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let proto_name = ctx.proto_name(name);
        let Some(field_mapping) = field_mapping(property) else {
            return;
        };
        if field_mapping.has_ignore_above() && field_mapping.ignore_above() <= 0 {
            diagnostics.push(
                Diagnostic::error(DiagnosticKind::InvalidParameterValue {
                    message: ctx.message.full_name().to_string(),
                    field: proto_name.to_string(),
                    parameter: "ignore_above".to_string(),
                    reason: "must be greater than 0".to_string(),
                })
                .at(ctx.location(proto_name)),
            );
        }
    }
}

struct InvalidPositionIncrementGapCheck;

impl Check for InvalidPositionIncrementGapCheck {
    fn check_property(
        &self,
        ctx: &ValidationContext<'_>,
        name: &str,
        property: &Property,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let proto_name = ctx.proto_name(name);
        let Some(field_mapping) = field_mapping(property) else {
            return;
        };
        if field_mapping.has_position_increment_gap() && field_mapping.position_increment_gap() < 0
        {
            diagnostics.push(
                Diagnostic::error(DiagnosticKind::InvalidParameterValue {
                    message: ctx.message.full_name().to_string(),
                    field: proto_name.to_string(),
                    parameter: "position_increment_gap".to_string(),
                    reason: "must be greater than or equal to 0".to_string(),
                })
                .at(ctx.location(proto_name)),
            );
        }
    }
}

struct InvalidIndexPrefixesCheck;

impl Check for InvalidIndexPrefixesCheck {
    fn check_property(
        &self,
        ctx: &ValidationContext<'_>,
        name: &str,
        property: &Property,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let proto_name = ctx.proto_name(name);
        let Some(field_mapping) = field_mapping(property) else {
            return;
        };
        let Some(prefixes) = field_mapping.index_prefixes.as_ref() else {
            return;
        };
        if prefixes.has_min_chars() && prefixes.min_chars() < 0 {
            diagnostics.push(
                Diagnostic::error(DiagnosticKind::InvalidParameterValue {
                    message: ctx.message.full_name().to_string(),
                    field: proto_name.to_string(),
                    parameter: "index_prefixes.min_chars".to_string(),
                    reason: "must be greater than or equal to 0".to_string(),
                })
                .at(ctx.location(proto_name)),
            );
        }
        if prefixes.has_max_chars() && !(0..=20).contains(&prefixes.max_chars()) {
            diagnostics.push(
                Diagnostic::error(DiagnosticKind::InvalidParameterValue {
                    message: ctx.message.full_name().to_string(),
                    field: proto_name.to_string(),
                    parameter: "index_prefixes.max_chars".to_string(),
                    reason: "must be less than or equal to 20".to_string(),
                })
                .at(ctx.location(proto_name)),
            );
        }
    }
}

fn field_mapping(property: &Property) -> Option<&FieldMapping> {
    match property {
        Property::Leaf(Parameters::Typed { field_mapping, .. })
        | Property::Object {
            parameters: Parameters::Typed { field_mapping, .. },
            ..
        } => Some(field_mapping),
        _ => None,
    }
}
