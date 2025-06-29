// src/page.rs

use std::mem;

// 4KB 页面大小
pub const PAGE_SIZE: usize = 4096;
pub type PageId = u64;

#[derive(Debug, PartialEq)]
pub enum PageType {
    Free,       // 空闲页 
    Internal,   // B+树内部节点
    Leaf,       // B+树叶子节点
    Overflow,   // 行溢出页
}

pub struct Page {
    pub id: PageId,
    pub page_type: PageType,
    data: [u8; PAGE_SIZE],
}

impl Page {
    pub fn new(id: PageId, page_type: PageType) -> Self {
        Self {
            id,
            page_type,
            data: [0; PAGE_SIZE],
        }
    }

    // 获得页面数据切片
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    // 获得可修改的页面数据切片
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    // 从字节流反序列化页面
    pub fn from_bytes(id: PageId, bytes: &[u8]) -> Option<Self> {
        if bytes.len() != PAGE_SIZE {
            return None;
        }

        let page_type = match bytes[0] {
            0 => PageType::Free,
            1 => PageType::Internal,
            2 => PageType::Leaf,
            3 => PageType::Overflow,
            _ => return None,
        };

        let mut data = [0; PAGE_SIZE];
        data.copy_from_slice(bytes);

        Some(Self { 
            id, 
            page_type, 
            data, 
        })
    }

    // 序列化页面到字节流
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(PAGE_SIZE);
        result.push(match self.page_type {
            PageType::Free => 0,
            PageType::Internal => 1,
            PageType::Leaf => 2,
            PageType::Overflow => 3,            
        });
        result.extend_from_slice(&self.data[1..]);
        result
    }
}