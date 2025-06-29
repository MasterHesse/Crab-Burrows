mod serialization;
mod row;

use crate::serialization::Serializable;
use crate::row::{Row, DataType};

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let row = Row::new(vec![
        DataType::Integer(42),
        DataType::Text("Hello, Crab-Burrows".to_string()),
        DataType::Null,
    ]);

    let bytes = row.serialize()?;
    println!("Serialize size: {} bytes", bytes.len());

    let decode = Row::deserialize(&bytes)?;
    assert_eq!(row, decode);

    Ok(())
}
