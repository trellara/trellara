use std::fmt;
use std::fs;
use std::ops::Deref;
use std::path::Path;

use serde::{Deserialize, Deserializer};
use trellara_stream_kafka::KafkaSecretRef;

pub type SecretReference = KafkaSecretRef;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SecretOrigin {
    Inline,
    EnvironmentVariable,
    File,
}

#[derive(Clone, Eq, PartialEq)]
pub struct SensitiveString {
    value: String,
    origin: SecretOrigin,
}

impl SensitiveString {
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.value
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.expose()
    }

    #[must_use]
    pub fn origin(&self) -> SecretOrigin {
        self.origin
    }

    #[must_use]
    pub fn is_reference(&self) -> bool {
        self.origin != SecretOrigin::Inline
    }
}

impl PartialEq<str> for SensitiveString {
    fn eq(&self, other: &str) -> bool {
        self.expose() == other
    }
}

impl PartialEq<&str> for SensitiveString {
    fn eq(&self, other: &&str) -> bool {
        self.expose() == *other
    }
}

impl Deref for SensitiveString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.expose()
    }
}

impl fmt::Debug for SensitiveString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted>")
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum SensitiveStringInput {
    Inline(String),
    Reference(SecretReference),
}

impl<'de> Deserialize<'de> for SensitiveString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let input = SensitiveStringInput::deserialize(deserializer)?;
        let (value, origin) = match input {
            SensitiveStringInput::Inline(value) => (value, SecretOrigin::Inline),
            SensitiveStringInput::Reference(reference) => {
                resolve_reference(&reference).map_err(serde::de::Error::custom)?
            }
        };
        if value.is_empty() {
            return Err(serde::de::Error::custom(
                "resolved secret value must not be empty",
            ));
        }
        Ok(Self { value, origin })
    }
}

fn resolve_reference(reference: &SecretReference) -> Result<(String, SecretOrigin), &'static str> {
    let (value, origin) = match reference {
        KafkaSecretRef::EnvironmentVariable { name } => {
            validate_environment_name(name)?;
            let value =
                std::env::var(name).map_err(|_| "environment secret could not be resolved")?;
            (value, SecretOrigin::EnvironmentVariable)
        }
        KafkaSecretRef::File { path } => {
            if !Path::new(path).is_absolute() {
                return Err("secret file path must be absolute");
            }
            let value = fs::read_to_string(path)
                .map_err(|_| "file secret could not be resolved as UTF-8")?;
            (value, SecretOrigin::File)
        }
    };
    Ok((strip_one_line_ending(value), origin))
}

fn validate_environment_name(name: &str) -> Result<(), &'static str> {
    let mut bytes = name.bytes();
    let valid_start = bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_');
    let valid_tail = bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_');
    if valid_start && valid_tail {
        Ok(())
    } else {
        Err("environment secret name must use an ASCII shell identifier")
    }
}

fn strip_one_line_ending(mut value: String) -> String {
    if value.ends_with("\r\n") {
        value.truncate(value.len() - 2);
    } else if value.ends_with('\n') {
        value.pop();
    }
    value
}
