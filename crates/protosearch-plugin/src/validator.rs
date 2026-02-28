use std::collections::BTreeMap;
use std::sync::LazyLock;

use protobuf::reflect::MessageDescriptor;
use regex::Regex;
use serde_json::Value;

use crate::diagnostic::{Diagnostic, DiagnosticKind, Location};
use crate::mapping::{Mapping, Property};
use crate::options::{get_mapping_options, property_name};
use crate::span::Span;

macro_rules! checks {
    ($($check:expr),* $(,)?) => {
            vec![$(Box::new($check)),*]
        }
}

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

pub trait Check {
    fn check_property(
        &self,
        ctx: &ValidationContext<'_>,
        name: &str,
        property: &Property,
        diagnostics: &mut Vec<Diagnostic>,
    );
}

pub struct Validator {
    checks: Vec<Box<dyn Check>>,
}

impl Validator {
    pub fn validate(&self, ctx: &ValidationContext<'_>, mapping: &Mapping) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for (name, property) in &mapping.properties {
            self.walk(ctx, name, property, &mut diagnostics);
        }
        diagnostics
    }

    fn walk(
        &self,
        ctx: &ValidationContext<'_>,
        name: &str,
        property: &Property,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for check in &self.checks {
            check.check_property(ctx, name, property, diagnostics);
        }
        if let Property::Mapping { properties, .. } = property {
            for (name, prop) in &properties.properties {
                self.walk(ctx, name, prop, diagnostics);
            }
        }
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self {
            checks: checks![
                InvalidNameCheck,
                InvalidIgnoreAboveCheck,
                InvalidPositionIncrementGapCheck,
            ],
        }
    }
}

pub fn validate(ctx: &ValidationContext<'_>, mapping: &Mapping) -> Vec<Diagnostic> {
    Validator::default().validate(ctx, mapping)
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
        if let Some(Value::Number(n)) = parameters(property).get("ignore_above")
            && let Some(v) = n.as_i64()
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
        if let Some(Value::Number(n)) = parameters(property).get("position_increment_gap")
            && let Some(v) = n.as_i64()
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
