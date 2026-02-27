use crate::diagnostic::Diagnostic;
use crate::mapping::{Mapping, Property};

pub trait Check {
    fn check_property(&self, name: &str, property: &Property, diagnostics: &mut Vec<Diagnostic>);
}

#[derive(Default)]
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

pub fn validate(mapping: &Mapping) -> Vec<Diagnostic> {
    Validator::default().validate(mapping)
}
