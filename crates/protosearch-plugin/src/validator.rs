use std::sync::LazyLock;

use regex::Regex;

use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::mapping::{Mapping, Property};

macro_rules! checks {
    ($($check:expr),* $(,)?) => {
            vec![$(Box::new($check)),*]
        }
}

pub trait Check {
    fn check_property(&self, name: &str, property: &Property, diagnostics: &mut Vec<Diagnostic>);
}

pub struct Validator {
    checks: Vec<Box<dyn Check>>,
}

impl Validator {
    pub fn validate(&self, mapping: &Mapping) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for (name, property) in &mapping.properties {
            self.walk(name, property, &mut diagnostics);
        }
        diagnostics
    }

    fn walk(&self, name: &str, property: &Property, diagnostics: &mut Vec<Diagnostic>) {
        for check in &self.checks {
            check.check_property(name, property, diagnostics);
        }
        if let Property::Mapping { properties, .. } = property {
            for (name, prop) in &properties.properties {
                self.walk(name, prop, diagnostics);
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

pub fn validate(mapping: &Mapping) -> Vec<Diagnostic> {
    Validator::default().validate(mapping)
}

struct InvalidNameCheck;

impl Check for InvalidNameCheck {
    fn check_property(&self, name: &str, _property: &Property, diagnostics: &mut Vec<Diagnostic>) {
        static RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^[@a-z][a-z0-9_]*(\.[a-z0-9_]+)*$").unwrap());
        if !RE.is_match(name) {
            diagnostics.push(Diagnostic::new(DiagnosticKind::InvalidFieldName {
                field: name.to_string(),
                name: name.to_string(),
            }));
        }
    }
}
