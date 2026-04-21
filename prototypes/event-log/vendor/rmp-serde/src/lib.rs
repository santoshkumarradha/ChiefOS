use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt;

pub mod encode {
    pub type Error = crate::Error;
}

pub mod decode {
    pub type Error = crate::Error;
}

#[derive(Debug)]
pub struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self(value.to_string())
    }
}

pub fn to_vec_named<T>(value: &T) -> Result<Vec<u8>, encode::Error>
where
    T: Serialize,
{
    Ok(serde_json::to_vec(value)?)
}

pub fn from_slice<T>(bytes: &[u8]) -> Result<T, decode::Error>
where
    T: DeserializeOwned,
{
    Ok(serde_json::from_slice(bytes)?)
}
