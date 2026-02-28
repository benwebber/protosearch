use std::fmt;

use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub location: Option<Location>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum DiagnosticKind {
    InvalidTargetJson {
        message: String,
        field: String,
        label: String,
    },
    InvalidTargetJsonType {
        message: String,
        field: String,
        label: String,
    },
    InvalidFieldName {
        message: String,
        field: String,
        name: String,
    },
    InvalidParameterValue {
        message: String,
        field: String,
        parameter: String,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub file: String,
    pub span: Option<Span>,
}

impl Diagnostic {
    pub fn new(kind: DiagnosticKind) -> Self {
        Self {
            kind,
            location: None,
        }
    }

    pub fn with_location(kind: DiagnosticKind, location: Location) -> Self {
        Self {
            kind,
            location: Some(location),
        }
    }
}

impl DiagnosticKind {
    pub fn code(&self) -> &str {
        match self {
            Self::InvalidFieldName { .. } => "E001",
            Self::InvalidTargetJson { .. } => "E002",
            Self::InvalidTargetJsonType { .. } => "E003",
            Self::InvalidParameterValue { .. } => "E100",
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.span {
            Some(span) => write!(f, "{}:{}:{}", self.file, span.start.line, span.start.column,),
            None => write!(f, "{}", self.file),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.location {
            Some(loc) => write!(f, "{}: {} {}", loc, self.kind.code(), self.kind),
            None => write!(f, "{} {}", self.kind.code(), self.kind),
        }
    }
}

impl fmt::Display for DiagnosticKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTargetJson {
                message,
                field,
                label,
            } => {
                write!(f, "{message}.{field}: target '{label}' is not valid JSON")
            }
            Self::InvalidTargetJsonType {
                message,
                field,
                label,
            } => {
                write!(
                    f,
                    "{message}.{field}: target '{label}' must be a JSON object"
                )
            }
            Self::InvalidFieldName {
                message,
                field,
                name,
            } => {
                write!(
                    f,
                    "{message}.{field}: name '{name}' is not a valid field name"
                )
            }
            Self::InvalidParameterValue {
                message,
                field,
                parameter,
                reason,
            } => write!(f, "{message}.{field}: '{parameter}' {reason}"),
        }
    }
}
