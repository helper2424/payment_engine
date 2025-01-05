use rust_decimal::{ Decimal, RoundingStrategy };
use serde::{Serializer, Deserialize, Deserializer};
use serde::de::Error as SerdeError;
use std::str::FromStr;

pub const DECIMAL_PRECISION: u32 = 4;

pub fn serialize_decimal<S>(decimal: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{    
    let rounded = decimal.round_dp_with_strategy(DECIMAL_PRECISION, RoundingStrategy::ToZero);
    serializer.serialize_str(rounded.to_string().as_str())
}

pub fn deserialize_option_decimal<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
where
    D: Deserializer<'de>,
{    
    let decimal_str = String::deserialize(deserializer)?;
    if decimal_str.is_empty() {
        return Ok(None);
    }
    
    let result = Decimal::from_str(&decimal_str)
        .map_err(D::Error::custom)?;

    Ok(Some(result.round_dp_with_strategy(DECIMAL_PRECISION, RoundingStrategy::ToZero)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use csv::{ReaderBuilder, WriterBuilder};
    use serde::{Serialize, Deserialize};

    fn serialize_to_string(dummy: &DummyStruct) -> String {
        let mut wtr = WriterBuilder::new().from_writer(vec![]);
        wtr.serialize(dummy).unwrap();
        String::from_utf8(wtr.into_inner().unwrap()).unwrap()
    }

    fn deserialize_from_string(input: &str) -> DummyStruct {
        let mut rdr = ReaderBuilder::new().from_reader(input.as_bytes());
        let first_record = rdr.deserialize::<DummyStruct>().next();
        
        if first_record.is_none() {
            panic!("No record found");
        }
        
        let first_record_res = first_record.unwrap();

        if first_record_res.is_err() {
            panic!("Error deserializing record: {}", first_record_res.err().unwrap());
        }

        first_record_res.unwrap()
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct DummyStruct {
        #[serde(serialize_with = "serialize_decimal")]
        amount: Decimal,

        #[serde(deserialize_with = "deserialize_option_decimal", skip_serializing)]
        option_amount: Option<Decimal>
    }

    #[test]
    fn test_serialize_decimal() {
        let dummy = DummyStruct {
            amount: dec!(1234.5678),
            option_amount: None
        };

        assert_eq!(serialize_to_string(&dummy), "amount\n1234.5678\n");
    }

    #[test]
    fn test_serialize_zero() {
        let dummy = DummyStruct {
            amount: dec!(0.0),
            option_amount: None
        };
        assert_eq!(serialize_to_string(&dummy), "amount\n0.0\n");
    }

    #[test]
    fn test_serialize_negative() {
        let dummy = DummyStruct {
            amount: dec!(-1.2345),
            option_amount: None
        };
        assert_eq!(serialize_to_string(&dummy), "amount\n-1.2345\n");
    }

    #[test]
    fn test_serialize_with_high_precision() {
        let dummy = DummyStruct {
            amount: dec!(1.23456789),
            option_amount: None
        };
        assert_eq!(serialize_to_string(&dummy), "amount\n1.2345\n");
    }

    #[test]
    fn test_deserialize_option_decimal() {
        let dummy = DummyStruct {
            amount: dec!(0),
            option_amount: Some(dec!(1.2345))
        };
        assert_eq!(deserialize_from_string("amount,option_amount\n0,1.2345\n"), dummy);
    }

    #[test]
    fn test_deserialize_option_decimal_zero() {
        assert_eq!(deserialize_from_string("amount,option_amount\n0,0\n").option_amount, Some(dec!(0)));
    }

    #[test]
    fn test_deserialize_option_decimal_none() {
        assert_eq!(deserialize_from_string("amount,option_amount\n0,\n").option_amount, None);
    }

    #[test]
    fn test_deserialize_option_decimal_high_precision() {
        assert_eq!(deserialize_from_string("amount,option_amount\n0,1.23456789\n").option_amount, Some(dec!(1.2345)));
    }

    #[test]
    fn test_deserialize_option_decimal_negative() {
        assert_eq!(deserialize_from_string("amount,option_amount\n0,-1.2345\n").option_amount, Some(dec!(-1.2345)));
    }
} 
