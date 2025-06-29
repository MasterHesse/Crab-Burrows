use bincode::{Encode, Decode};

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub enum DataType {
    Integer(i64),
    Float(f64),
    Text(String),
    Blob(Vec<u8>),
    Null,
}

#[derive(Debug, PartialEq)]
pub struct Row {
    pub columns: Vec<DataType>,
}

impl Row {
    pub fn new(columns: Vec<DataType>) -> Self {
        Self { columns: columns }
    }
}