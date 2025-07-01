use crate::row::{Row, DataType};
use std::io;
use bincode::{config, Decode, Encode};
use bincode::error::{EncodeError, DecodeError};
use bincode::{enc, de};
use bincode::config::standard;

pub trait Serializable {
    fn serialize(&self) -> Result<Vec<u8>, EncodeError>;
    fn deserialize(data: &[u8]) -> Result<Self, DecodeError>
    where 
        Self: Sized;
}

// Row 类型序列化
impl Serializable for Row {
    fn serialize(&self) -> Result<Vec<u8>, EncodeError> {
        let config = standard().with_fixed_int_encoding();
        bincode::encode_to_vec(&self.columns, config)
    }

    fn deserialize(data: &[u8]) -> Result<Self, DecodeError> {
        let config = standard().with_fixed_int_encoding();
        let (columns, _) = bincode::decode_from_slice(data, config)?;
        Ok(Row::new(columns))
    }
}

// 为 DataType 实现序列化
impl Serializable for DataType {
    fn serialize(&self) -> Result<Vec<u8>, EncodeError> {
        let config = standard().with_fixed_int_encoding();
        match self {
            DataType::Integer(i) => bincode::encode_to_vec(i, config),
            DataType::Float(f) => bincode::encode_to_vec(f, config),
            DataType::Text(s) => bincode::encode_to_vec(s, config),
            DataType::Blob(b) => bincode::encode_to_vec(b, config),
            DataType::Null => bincode::encode_to_vec(0u8, config),
        }
    }

    fn deserialize(data: &[u8]) -> Result<Self, DecodeError> {
        let config = standard().with_fixed_int_encoding();

        if let Ok(i) = bincode::decode_from_slice(data, config) {
            return Ok(DataType::Integer(i.0));
        }
        if let Ok(f) = bincode::decode_from_slice(data, config) {
            return Ok(DataType::Float(f.0));
        }
        if let Ok(s) = bincode::decode_from_slice(data, config) {
            return Ok(DataType::Text(s.0));
        }
        if let Ok(b) = bincode::decode_from_slice(data, config) {
            return Ok(DataType::Blob(b.0));
        }
        if let Ok(null) = bincode::decode_from_slice::<u8, _>(data, config) {
            if null.0 == 0 {
                return Ok(DataType::Null);
            }
        }        

        Err(DecodeError::Other("Invaild data type"))
    }
}

