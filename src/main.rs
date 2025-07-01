// src/main.rs

mod storage;
mod page;
mod btree;
mod serialization;
mod row;

use crate::page::{Page, PageType};
use crate::btree::BTree;
use crate::serialization::Serializable;
use crate::row::{Row, DataType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Crab-Burrows Database System");
    println!("Version 0.1.0 (Stage 1: Basic Data Structures)");
    
    // 行数据序列化演示
    println!("\n=== Row Serialization Demo ===");
    let row = Row::new(vec![
        DataType::Integer(42),
        DataType::Text("Hello, Crab-Burrows".to_string()),
        DataType::Null,
    ]);
    
    let bytes = row.serialize()?;
    println!("Serialized row size: {} bytes", bytes.len());
    
    let decoded = Row::deserialize(&bytes)?;
    println!("Deserialized row: {:?}", decoded);
    assert_eq!(row, decoded);
    
    // 页面管理演示
    println!("\n=== Page Management Demo ===");
    let page = Page::new(1, PageType::Leaf);
    println!("Created page: ID={}, Type={:?}", page.id, page.page_type);
    
    // B+树演示
    println!("\n=== B+Tree Index Demo ===");
    let root_page_id = 1;
    let btree = BTree::new(root_page_id);
    println!("Created B+Tree with root page ID: {}", btree.get_root_page_id());
    
    println!("\nAll tests completed successfully!");
    Ok(())
}