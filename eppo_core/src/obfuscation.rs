use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::{ufc::ValueWire, Str};

/// `md5::Digest` that implements `Serialize` and `Deserialize` (by converting to hex-encoded
/// string).
#[serde_as]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub(crate) struct Md5HashedStr(#[serde_as(as = "serde_with::hex::Hex")] [u8; 16]);

impl Md5HashedStr {
    pub fn new(salt: &[u8], input: &[u8]) -> Md5HashedStr {
        let mut ctx = md5::Context::new();
        ctx.consume(salt);
        ctx.consume(input);
        ctx.compute().into()
    }
}

impl From<md5::Digest> for Md5HashedStr {
    fn from(value: md5::Digest) -> Self {
        Md5HashedStr(value.0)
    }
}

/// Same as [`Str`] but serializes as base64-encoded string.
#[serde_as]
#[derive(
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Clone,
    Serialize,
    Deserialize,
    derive_more::From,
    derive_more::Into,
)]
#[from(Str, String, std::borrow::Cow<'_, str>)]
pub(crate) struct Base64Str(#[serde_as(as = "serde_with::base64::Base64")] pub(crate) Str);

impl From<ValueWire> for Base64Str {
    /// Convert value to base64. Booleans and numbers are converted to string first.
    fn from(value: ValueWire) -> Base64Str {
        let s = match value {
            ValueWire::Boolean(b) => Str::from_static_str(if b { "true" } else { "false" }),
            ValueWire::Number(n) => n.to_string().into(),
            ValueWire::String(s) => s,
        };
        Base64Str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_md5() {
        let md5_digest = md5::compute(b"hello");
        let s = Md5HashedStr::from(md5_digest);

        let json = serde_json::to_string(&s).unwrap();

        assert_eq!(json, "\"5d41402abc4b2a76b9719d911017c592\"");
    }
}
