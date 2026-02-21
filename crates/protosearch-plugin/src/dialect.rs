#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dialect {
    package: String,
}

impl Dialect {
    pub fn new(package: impl Into<String>) -> Self {
        Self {
            package: package.into(),
        }
    }

    pub fn package(&self) -> &str {
        &self.package
    }

    pub fn suffix(&self) -> &str {
        self.package
            .strip_prefix("protosearch.")
            .unwrap_or(&self.package)
    }
}
