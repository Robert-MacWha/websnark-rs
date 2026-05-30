use std::fmt::Display;

use anyhow::{anyhow, bail};
use ark_bn254::Fr;
use ark_ff::{AdditiveGroup, BigInteger, PrimeField};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use serde::{Deserialize, Deserializer};
use serde_json::Value as JsonValue;

/// Circuit value, representing either a number or an array of values.
///
/// Internally the Number and Fr variants are used to more efficiently compute various
/// operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Number(BigInt),
    #[doc(hidden)]
    Fr(Fr),
    Array(Vec<Value>),
}

impl Value {
    pub(crate) fn into_fr(self) -> Result<Fr, anyhow::Error> {
        match self {
            Value::Fr(f) => Ok(f),
            Value::Number(b) => Ok(bigint_to_fr(&b)),
            Value::Array(_) => bail!("expected number, got array"),
        }
    }

    pub(crate) fn into_number(self) -> Result<BigInt, anyhow::Error> {
        match self {
            Value::Number(b) => Ok(b),
            Value::Fr(f) => Ok(fr_to_bigint(f)),
            Value::Array(_) => bail!("expected number, got array"),
        }
    }

    pub(crate) fn into_u64(self) -> Result<u64, anyhow::Error> {
        match self {
            Value::Fr(f) => f.into_bigint().as_ref()[0]
                .to_u64()
                .ok_or_else(|| anyhow!("selector out of u64 range")),
            Value::Number(n) => n
                .to_u64()
                .ok_or_else(|| anyhow!("selector out of u64 range")),
            Value::Array(_) => bail!("expected number, got array"),
        }
    }

    pub(crate) fn is_zero(&self) -> Result<bool, anyhow::Error> {
        match self {
            Value::Fr(f) => Ok(f == &Fr::ZERO),
            Value::Number(b) => Ok(b == &BigInt::ZERO),
            Value::Array(_) => bail!("expected number, got array"),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Fr(Fr::from(value as u64))
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Value::Fr(Fr::from(value))
    }
}

impl From<BigInt> for Value {
    fn from(value: BigInt) -> Self {
        Value::Number(value)
    }
}

impl From<Fr> for Value {
    fn from(value: Fr) -> Self {
        Value::Fr(value)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{n}"),
            Value::Fr(fr) => write!(f, "{fr}"),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = JsonValue::deserialize(d)?;
        Self::from_json(&raw).map_err(serde::de::Error::custom)
    }
}

impl Value {
    fn from_json(v: &JsonValue) -> Result<Self, String> {
        match v {
            JsonValue::Number(n) => {
                let s = n.to_string();
                let big = s.parse::<BigInt>().map_err(|e| e.to_string())?;
                Ok(Value::Number(big))
            }
            JsonValue::String(s) => {
                // Fallback for large numbers serialized as strings
                let big = s.parse::<BigInt>().map_err(|e| e.to_string())?;
                Ok(Value::Number(big))
            }
            JsonValue::Array(arr) => {
                let items = arr.iter().map(Self::from_json).collect::<Result<_, _>>()?;
                Ok(Value::Array(items))
            }
            other => Err(format!("expected number or array, got {other}")),
        }
    }
}

pub fn bigint_to_fr(n: &BigInt) -> Fr {
    if let Some(u) = n.to_u64() {
        return Fr::from(u);
    }
    if let Some(i) = n.to_i64() {
        return -Fr::from(i.unsigned_abs());
    }
    let bytes = n.to_signed_bytes_le();
    Fr::from_le_bytes_mod_order(&bytes)
}

fn fr_to_bigint(f: Fr) -> BigInt {
    let bytes = f.into_bigint().to_bytes_le();
    BigInt::from_bytes_le(num_bigint::Sign::Plus, &bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;

    fn de(s: &str) -> Value {
        serde_json::from_str(s).expect("deserialization failed")
    }

    #[test]
    fn test_single_number() {
        assert!(matches!(de("42"), Value::Number(n) if n == BigInt::from(42)));
    }

    #[test]
    fn test_negative_number() {
        assert!(matches!(de("-7"), Value::Number(n) if n == BigInt::from(-7)));
    }

    #[test]
    fn test_flat_array() {
        let v = de("[1, 2, 3]");
        assert!(matches!(v, Value::Array(ref a) if a.len() == 3));
    }

    #[test]
    fn test_mixed_nesting() {
        let v = de("[1, [2, 3]]");
        let Value::Array(items) = v else {
            panic!("expected array")
        };
        assert!(matches!(&items[0], Value::Number(n) if *n == BigInt::from(1)));
        assert!(matches!(&items[1], Value::Array(a) if a.len() == 2));
    }

    #[test]
    fn test_empty_array() {
        let v = de("[]");
        assert!(matches!(v, Value::Array(a) if a.is_empty()));
    }

    #[test]
    fn test_invalid_input_fails() {
        assert!(serde_json::from_str::<Value>("\"not a number\"").is_err());
        assert!(serde_json::from_str::<Value>("true").is_err());
        assert!(serde_json::from_str::<Value>("null").is_err());
    }

    #[test]
    fn test_fr_bigint_conversion() {
        let big = BigInt::parse_bytes(b"123456789012345678901234567890", 10).unwrap();
        let fr = bigint_to_fr(&big);
        let big_back = fr_to_bigint(fr);
        assert_eq!(big, big_back);
    }
}
