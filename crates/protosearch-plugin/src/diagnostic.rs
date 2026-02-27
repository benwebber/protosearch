use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic<L = ()> {
    pub kind: DiagnosticKind,
    pub location: Option<L>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum DiagnosticKind {
    InvalidTargetJson { field: String, label: String },
    InvalidTargetJsonType { field: String, label: String },
    InvalidFieldName { field: String, name: String },
}

impl Diagnostic<()> {
    pub fn new(kind: DiagnosticKind) -> Self {
        Self {
            kind,
            location: None,
        }
    }
}

impl fmt::Display for Diagnostic<()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error: {}", self.kind)
    }
}

impl fmt::Display for DiagnosticKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTargetJson { field, label } => {
                write!(f, "field `{field}`: target `{label}` has invalid JSON")
            }
            Self::InvalidTargetJsonType { field, label } => {
                write!(f, "field `{field}`: target `{label}` must be a JSON object")
            }
            Self::InvalidFieldName { field, name } => {
                write!(
                    f,
                    "field `{field}`: name `{name}` is not a valid identifier"
                )
            }
        }
    }
}
