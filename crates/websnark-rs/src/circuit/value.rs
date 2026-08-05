use std::fmt::Display;

use ark_bn254::Fr;
use ark_ff::{AdditiveGroup, BigInteger, PrimeField};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeSeq};

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

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ValueError {
    #[error("expected number")]
    ExpectedNumber,
    #[error("expected array")]
    ExpectedArray,
    #[error("invalid number: {0}")]
    InvalidNumber(String),
    #[error("value out of range")]
    ValueOutOfRange,
}

impl Value {
    pub(crate) fn into_fr(self) -> Result<Fr, ValueError> {
        match self {
            Value::Fr(f) => Ok(f),
            Value::Number(b) => Ok(bigint_to_fr(&b)),
            Value::Array(_) => Err(ValueError::ExpectedNumber),
        }
    }

    pub(crate) fn into_number(self) -> Result<BigInt, ValueError> {
        match self {
            Value::Number(b) => Ok(b),
            Value::Fr(f) => Ok(fr_to_bigint(f)),
            Value::Array(_) => Err(ValueError::ExpectedNumber),
        }
    }

    pub(crate) fn into_u32(self) -> Result<u32, ValueError> {
        match self {
            Value::Fr(f) => f.into_bigint().as_ref()[0]
                .to_u32()
                .ok_or_else(|| ValueError::InvalidNumber(f.to_string())),
            Value::Number(n) => n
                .to_u32()
                .ok_or_else(|| ValueError::InvalidNumber(n.to_string())),
            Value::Array(_) => Err(ValueError::ExpectedNumber),
        }
    }

    pub(crate) fn is_zero(&self) -> Result<bool, ValueError> {
        match self {
            Value::Fr(f) => Ok(f == &Fr::ZERO),
            Value::Number(b) => Ok(b == &BigInt::ZERO),
            Value::Array(_) => Err(ValueError::ExpectedNumber),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Fr(Fr::from(u64::from(value)))
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

/// Serialize in the snarkjs decimal-string JSON format so that
/// `serde_json::to_string(&value)` produces human-readable output.
#[cfg(feature = "serde")]
impl Serialize for Value {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Value::Number(n) => n.to_string().serialize(s),
            Value::Fr(fr) => fr.to_string().serialize(s),
            Value::Array(items) => {
                let mut seq = s.serialize_seq(Some(items.len()))?;
                for item in items {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
        }
    }
}

/// Deserialize from the snarkjs JSON format (number, decimal string, or array).
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(ValueVisitor)
    }
}

#[cfg(feature = "serde")]
struct ValueVisitor;

#[cfg(feature = "serde")]
impl<'de> serde::de::Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("number, decimal string, or array")
    }

    fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
        Ok(Value::Number(BigInt::from(v)))
    }

    fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
        Ok(Value::Number(BigInt::from(v)))
    }

    fn visit_u128<E: serde::de::Error>(self, v: u128) -> Result<Self::Value, E> {
        Ok(Value::Number(BigInt::from(v)))
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        v.parse::<BigInt>()
            .map(Value::Number)
            .map_err(|e| E::custom(e))
    }

    fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
        self.visit_str(&v)
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = seq.next_element::<Value>()? {
            items.push(item);
        }
        Ok(Value::Array(items))
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
