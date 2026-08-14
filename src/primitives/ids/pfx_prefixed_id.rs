use std::fmt;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PrefixedId {
    prefix: &'static str,
    suffix: String,
    full: String,
}

impl PrefixedId {
    pub fn parse(
        expected_prefix: &'static str,
        value: impl AsRef<str>,
    ) -> Result<Self, IdParseError> {
        validate_prefix(expected_prefix)?;

        let value = value.as_ref();
        let Some((actual_prefix, suffix)) = value.split_once('-') else {
            return Err(IdParseError::MissingSeparator);
        };

        if actual_prefix != expected_prefix {
            return Err(IdParseError::WrongPrefix {
                expected: expected_prefix,
                actual: actual_prefix.to_owned(),
            });
        }

        Self::from_suffix(expected_prefix, suffix)
    }

    pub fn from_suffix(
        expected_prefix: &'static str,
        suffix: impl AsRef<str>,
    ) -> Result<Self, IdParseError> {
        validate_prefix(expected_prefix)?;

        let suffix = suffix.as_ref();
        validate_suffix(suffix)?;

        Ok(Self {
            prefix: expected_prefix,
            suffix: suffix.to_owned(),
            full: format!("{expected_prefix}-{suffix}"),
        })
    }

    pub fn prefix(&self) -> &'static str {
        self.prefix
    }

    pub fn suffix(&self) -> &str {
        &self.suffix
    }

    pub fn as_str(&self) -> &str {
        &self.full
    }
}

impl fmt::Debug for PrefixedId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for PrefixedId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum IdParseError {
    InvalidPrefix(String),
    MissingSeparator,
    WrongPrefix {
        expected: &'static str,
        actual: String,
    },
    EmptySuffix,
    InvalidSuffix(String),
}

impl fmt::Display for IdParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix(prefix) => write!(
                f,
                "invalid ID prefix `{prefix}`; expected exactly three uppercase ASCII letters"
            ),
            Self::MissingSeparator => f.write_str("invalid ID; expected `XXX-suffix`"),
            Self::WrongPrefix { expected, actual } => {
                write!(f, "wrong ID prefix `{actual}`; expected `{expected}`")
            }
            Self::EmptySuffix => f.write_str("invalid ID; suffix cannot be empty"),
            Self::InvalidSuffix(suffix) => write!(
                f,
                "invalid ID suffix `{suffix}`; expected ASCII letters, digits, or hyphens"
            ),
        }
    }
}

impl std::error::Error for IdParseError {}

fn validate_prefix(prefix: &str) -> Result<(), IdParseError> {
    let valid = prefix.len() == 3 && prefix.bytes().all(|byte| byte.is_ascii_uppercase());

    if valid {
        Ok(())
    } else {
        Err(IdParseError::InvalidPrefix(prefix.to_owned()))
    }
}

fn validate_suffix(suffix: &str) -> Result<(), IdParseError> {
    if suffix.is_empty() {
        return Err(IdParseError::EmptySuffix);
    }

    let valid = suffix
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-');

    if valid {
        Ok(())
    } else {
        Err(IdParseError::InvalidSuffix(suffix.to_owned()))
    }
}
