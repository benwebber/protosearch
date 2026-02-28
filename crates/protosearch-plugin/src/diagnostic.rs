use std::fmt;

use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub kind: DiagnosticKind,
    pub location: Option<Location>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Warning,
    Error,
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
    pub fn error(kind: DiagnosticKind) -> Self {
        Self {
            severity: Severity::Error,
            kind,
            location: None,
        }
    }

    pub fn warning(kind: DiagnosticKind) -> Self {
        Self {
            severity: Severity::Warning,
            kind,
            location: None,
        }
    }

    pub fn at(self, location: Location) -> Self {
        Self {
            severity: self.severity,
            kind: self.kind,
            location: Some(location),
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self.severity, Severity::Error)
    }

    pub fn is_warning(&self) -> bool {
        matches!(self.severity, Severity::Warning)
    }
}

impl Severity {
    pub fn prefix(&self) -> char {
        match self {
            Self::Warning => 'W',
            Self::Error => 'E',
        }
    }
}

impl DiagnosticKind {
    pub fn number(&self) -> u32 {
        match self {
            Self::InvalidFieldName { .. } => 1,
            Self::InvalidTargetJson { .. } => 2,
            Self::InvalidTargetJsonType { .. } => 3,
            Self::InvalidParameterValue { .. } => 100,
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
            Some(loc) => write!(
                f,
                "{}: {}{:0>3} {}",
                loc,
                self.severity.prefix(),
                self.kind.number(),
                self.kind
            ),
            None => write!(
                f,
                "{}{:0>3} {}",
                self.severity.prefix(),
                self.kind.number(),
                self.kind
            ),
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
                write!(f, "{message}.{field}: '{name}' is not a valid field name")
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
