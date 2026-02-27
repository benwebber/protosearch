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

impl fmt::Display for Diagnostic<()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error: {}", self.kind)
    }
}

impl fmt::Display for DiagnosticKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFieldName { field, name } => {
                write!(f, "invalid mapping field name for {:?}: {:?}", field, name)
            }
            Self::InvalidTargetJson { field, label } => {
                write!(f, "invalid target JSON for {:?}: {:?}", field, label)
            }
            Self::InvalidTargetJsonType { field, label } => write!(
                f,
                "target JSON is not an object for {:?}: {:?}",
                field, label
            ),
        }
    }
}
