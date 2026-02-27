use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic<L = ()> {
    pub kind: DiagnosticKind,
    pub location: Option<L>,
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub file: String,
}

impl Diagnostic<()> {
    pub fn new(kind: DiagnosticKind) -> Self {
        Self {
            kind,
            location: None,
        }
    }

    pub fn locate(self, file: &str) -> Diagnostic<Location> {
        Diagnostic {
            kind: self.kind,
            location: Some(Location {
                file: file.to_string(),
            }),
        }
    }
}

impl Diagnostic<Location> {
    pub fn with_location(kind: DiagnosticKind, file: impl Into<String>) -> Self {
        Self {
            kind,
            location: Some(Location { file: file.into() }),
        }
    }
}

impl fmt::Display for Diagnostic<()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl fmt::Display for Diagnostic<Location> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.location {
            Some(loc) => write!(f, "{}: {}", loc.file, self.kind),
            None => write!(f, "{}", self.kind),
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
        }
    }
}
