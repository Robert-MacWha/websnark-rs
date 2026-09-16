use ark_bn254::Fr;
use num_bigint::BigInt;
use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeSeq};

use crate::circuit::{Value, value::bigint_to_fr};

struct ValueVisitor;

/// Serialize in the snarkjs decimal-string JSON format so that
/// `serde_json::to_string(&value)` produces human-readable output.
impl Serialize for Value {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
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
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(ValueVisitor)
    }
}

impl<'de> serde::de::Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("number, decimal string, or array")
    }

    fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
        Ok(Value::Fr(Fr::from(v)))
    }

    fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
        Ok(Value::Fr(Fr::from(v)))
    }

    fn visit_u128<E: serde::de::Error>(self, v: u128) -> Result<Self::Value, E> {
        Ok(Value::Fr(Fr::from(v)))
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        let n = v.parse::<BigInt>().map_err(|e| E::custom(e))?;
        Ok(Value::Fr(bigint_to_fr(&n)))
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

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;

    fn de(s: &str) -> Value {
        serde_json::from_str(s).expect("deserialization failed")
    }

    #[test]
    fn test_single_number() {
        assert!(matches!(de("42"), Value::Fr(f) if f == bigint_to_fr(&BigInt::from(42))));
    }

    #[test]
    fn test_negative_number() {
        assert!(matches!(de("-7"), Value::Fr(f) if f == bigint_to_fr(&BigInt::from(-7))));
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
        assert!(matches!(&items[0], Value::Fr(f) if *f == bigint_to_fr(&BigInt::from(1))));
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
}
