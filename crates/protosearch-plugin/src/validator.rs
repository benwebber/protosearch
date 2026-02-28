use std::collections::BTreeMap;
use std::sync::LazyLock;

use protobuf::reflect::MessageDescriptor;
use regex::Regex;
use serde_json::Value;

use crate::diagnostic::{Diagnostic, DiagnosticKind, Location};
use crate::mapping::{Mapping, Property};
use crate::options::{get_mapping_options, property_name};
use crate::span::Span;

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
                get_mapping_options(&f).ok().flatten().map(|opts| {
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
    if let Property::Mapping { properties, .. } = property {
        for (name, prop) in &properties.properties {
            walk(ctx, name, prop, diagnostics);
        }
    }
}

fn parameters(property: &Property) -> &BTreeMap<String, Value> {
    match property {
        Property::Leaf(p) | Property::Mapping { parameters: p, .. } => p,
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
            diagnostics.push(Diagnostic::with_location(
                DiagnosticKind::InvalidFieldName {
                    message: ctx.message.full_name().to_string(),
                    field: proto_name.to_string(),
                    name: name.to_string(),
                },
                ctx.location(proto_name),
            ));
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
        if let Some(v) = get_i64(property, "ignore_above")
            && v <= 0
        {
            diagnostics.push(Diagnostic::with_location(
                DiagnosticKind::InvalidParameterValue {
                    message: ctx.message.full_name().to_string(),
                    field: proto_name.to_string(),
                    parameter: "ignore_above".to_string(),
                    reason: "'ignore_above' must be greater than 0".to_string(),
                },
                ctx.location(proto_name),
            ));
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
        if let Some(v) = get_i64(property, "position_increment_gap")
            && v < 0
        {
            diagnostics.push(Diagnostic::with_location(
                DiagnosticKind::InvalidParameterValue {
                    message: ctx.message.full_name().to_string(),
                    field: proto_name.to_string(),
                    parameter: "position_increment_gap".to_string(),
                    reason: "must be greater than or equal to 0".to_string(),
                },
                ctx.location(proto_name),
            ));
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
        let Some(prefixes) = parameters(property)
            .get("index_prefixes")
            .and_then(Value::as_object)
        else {
            return;
        };
        if let Some(v) = prefixes.get("min_chars").and_then(Value::as_i64)
            && v < 0
        {
            diagnostics.push(Diagnostic::with_location(
                DiagnosticKind::InvalidParameterValue {
                    message: ctx.message.full_name().to_string(),
                    field: proto_name.to_string(),
                    parameter: "index_prefixes.min_chars".to_string(),
                    reason: "must be greater than or equal to 0".to_string(),
                },
                ctx.location(proto_name),
            ));
        }
        if let Some(v) = prefixes.get("max_chars").and_then(Value::as_i64)
            && (!(0..=20).contains(&v))
        {
            diagnostics.push(Diagnostic::with_location(
                DiagnosticKind::InvalidParameterValue {
                    message: ctx.message.full_name().to_string(),
                    field: proto_name.to_string(),
                    parameter: "index_prefixes.max_chars".to_string(),
                    reason: "must be less than or equal to 20".to_string(),
                },
                ctx.location(proto_name),
            ));
        }
    }
}

fn get_i64(property: &Property, key: &str) -> Option<i64> {
    parameters(property).get(key)?.as_i64()
}
