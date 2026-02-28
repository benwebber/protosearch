use crate::{Error, Result};

#[derive(Debug)]
pub struct Config {
    pub target: Option<String>,
}

impl TryFrom<&str> for Config {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {
        let mut target = None;
        for param in s.split(',').filter(|s| !s.is_empty()) {
            if let Some(v) = param.strip_prefix("target=") {
                target = Some(v.to_string());
            } else {
                return Err(Error::InvalidRequest(format!("unknown parameter: {param}")));
            }
        }
        Ok(Self { target })
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use crate::Error;

    #[test]
    fn test_target() {
        let config = Config::try_from("target=foo").unwrap();
        assert_eq!(config.target.as_deref(), Some("foo"));
    }

    #[test]
    fn test_empty() {
        let config = Config::try_from("").unwrap();
        assert_eq!(config.target, None);
    }

    #[test]
    fn test_unknown_parameter() {
        assert!(matches!(
            Config::try_from("unknown=bar").unwrap_err(),
            Error::InvalidRequest(_)
        ));
    }

    #[test]
    fn test_target_with_unknown_parameter() {
        assert!(matches!(
            Config::try_from("target=foo,unknown=bar").unwrap_err(),
            Error::InvalidRequest(_)
        ));
    }
}
