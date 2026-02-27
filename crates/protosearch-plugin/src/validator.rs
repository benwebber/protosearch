use std::collections::BTreeMap;
use std::sync::LazyLock;

use regex::Regex;

use crate::diagnostic::{Diagnostic, DiagnosticKind, Location};
use crate::mapping::{Mapping, Property};

macro_rules! checks {
    ($($check:expr),* $(,)?) => {
            vec![$(Box::new($check)),*]
        }
}

pub struct ValidationContext<'a> {
    pub file: &'a str,
    pub message: &'a str,
    proto_names_by_mapping_name: BTreeMap<String, String>,
}

impl<'a> ValidationContext<'a> {
    pub fn new(
        file: &'a str,
        message: &'a str,
        proto_names_by_mapping_name: BTreeMap<String, String>,
    ) -> Self {
        Self {
            file,
            message,
            proto_names_by_mapping_name,
        }
    }

    pub fn proto_name<'b>(&'b self, mapping_name: &'b str) -> &'b str {
        self.proto_names_by_mapping_name
            .get(mapping_name)
            .map(String::as_str)
            .unwrap_or(mapping_name)
    }
}

pub trait Check {
    fn check_property(
        &self,
        ctx: &ValidationContext,
        name: &str,
        property: &Property,
        diagnostics: &mut Vec<Diagnostic<Location>>,
    );
}

pub struct Validator {
    checks: Vec<Box<dyn Check>>,
}

impl Validator {
    pub fn validate(
        &self,
        ctx: &ValidationContext,
        mapping: &Mapping,
    ) -> Vec<Diagnostic<Location>> {
        let mut diagnostics = Vec::new();
        for (name, property) in &mapping.properties {
            self.walk(ctx, name, property, &mut diagnostics);
        }
        diagnostics
    }

    fn walk(
        &self,
        ctx: &ValidationContext,
        name: &str,
        property: &Property,
        diagnostics: &mut Vec<Diagnostic<Location>>,
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
            checks: checks![InvalidNameCheck,],
        }
    }
}

pub fn validate(ctx: &ValidationContext, mapping: &Mapping) -> Vec<Diagnostic<Location>> {
    Validator::default().validate(ctx, mapping)
}

struct InvalidNameCheck;

impl Check for InvalidNameCheck {
    fn check_property(
        &self,
        ctx: &ValidationContext,
        name: &str,
        _property: &Property,
        diagnostics: &mut Vec<Diagnostic<Location>>,
    ) {
        let proto_name = ctx.proto_name(name);
        static RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^[@a-z][a-z0-9_]*(\.[a-z0-9_]+)*$").unwrap());
        if !RE.is_match(name) {
            diagnostics.push(Diagnostic::with_location(
                DiagnosticKind::InvalidFieldName {
                    message: ctx.message.to_string(),
                    field: proto_name.to_string(),
                    name: name.to_string(),
                },
                ctx.file,
            ));
        }
    }
}
