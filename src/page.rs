// src/page.rs

use std::mem;

// 4KB 页面大小
pub const PAGE_SIZE: usize = 4096;
pub type PageId = u64;

// 头页面元数据位置
pub const MAGIC_NUMBER_OFFSET: usize = 0;     // 8字节 → 存储数据库标识符"CRABDBv1"，用于验证文件类型
pub const FREE_LIST_START_OFFSET: usize = 8;  // 8字节 → 空闲页面链表的起始页面ID
pub const ROOT_PAGE_ID_OFFSET: usize = 16;    // 8字节 → B+树根节点的页面ID 
pub const PAGE_SIZE_OFFSET: usize = 24;       // 4字节 → 存储页面大小(4096)，用于兼容性检查
pub const CHECKSUM_OFFSET: usize = 28;        // 4字节 → 存储头页面的校验和，用于数据完整性验证

#[derive(Debug, PartialEq, Clone)]
pub enum PageType {
    Free,       // 空闲页 
    Internal,   // B+树内部节点
    Leaf,       // B+树叶子节点
    Overflow,   // 行溢出页
    Header,     // 头页面
}

#[derive(Clone)]
pub struct Page {
    pub id: PageId,
    pub page_type: PageType,
    data: [u8; PAGE_SIZE],
}

impl Page {
    pub fn new(id: PageId, page_type: PageType) -> Self {
        let page_type_clone = page_type.clone();
        
        let mut page = Self {
            id,
            page_type,
            data: [0; PAGE_SIZE],
        };
        

        if page_type_clone == PageType::Header {
            page.data[MAGIC_NUMBER_OFFSET..MAGIC_NUMBER_OFFSET+8]
                .copy_from_slice(b"CRABDBv1");

            page.data[PAGE_SIZE_OFFSET..PAGE_SIZE_OFFSET+4]
                .copy_from_slice(&(PAGE_SIZE as u32).to_be_bytes());
        }

        page
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
            4 => PageType::Header,
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
            PageType::Header => 4,           
        });
        result.extend_from_slice(&self.data[1..]);
        result
    }

    // === Header 方法 ===

    /// 验证魔术数字 （检查数据库文件有效性）
    pub fn validate_magic_number(&self) -> bool {
        if self.page_type != PageType::Header {
            return false;
        }

        &self.data[MAGIC_NUMBER_OFFSET..MAGIC_NUMBER_OFFSET+8] == b"CRABDBv1"
    }

    /// 获取空闲列表起始页面ID
    pub fn free_list_start(&self) -> Option<PageId> {
        if self.page_type != PageType::Header {
            return None;
        }

        let bytes: [u8; 8] = self.data[FREE_LIST_START_OFFSET..FREE_LIST_START_OFFSET+8]
            .try_into()
            .ok()?;

        Some(PageId::from_be_bytes(bytes))
    }
    
    /// 设置空闲列表起始页面ID
    pub fn set_free_list_start(&mut self, page_id: PageId) {
        if self.page_type == PageType::Header {
            self.data[FREE_LIST_START_OFFSET..FREE_LIST_START_OFFSET+8]
                .copy_from_slice(&page_id.to_be_bytes());
        }
    }
    
    /// 获取根页面ID
    pub fn root_page_id(&self) -> Option<PageId> {
        if self.page_type != PageType::Header {
            return None;
        }

        let bytes: [u8; 8] = self.data[ROOT_PAGE_ID_OFFSET..ROOT_PAGE_ID_OFFSET+8]
            .try_into()
            .ok()?;

        Some(PageId::from_be_bytes(bytes))
    }
    
    /// 设置根页面ID
    pub fn set_root_page_id(&mut self, page_id: PageId) {
        if self.page_type == PageType::Header {
            self.data[ROOT_PAGE_ID_OFFSET..ROOT_PAGE_ID_OFFSET+8]
                .copy_from_slice(&page_id.to_be_bytes());
        }
    }
    
    /// 获取页面大小 （PAGE_SIZE）
    pub fn page_size(&self) -> Option<u32> {
        if self.page_type != PageType::Header {
            return None;
        }

        let bytes: [u8; 4] = self.data[PAGE_SIZE_OFFSET..PAGE_SIZE_OFFSET+4]
            .try_into()
            .ok()?;

        Some(u32::from_be_bytes(bytes))
    }
    
    /// 校验和计算方法（简单版）
    pub fn calculate_checksum(&self) -> u32 {
        use std::hash::Hasher;
        let mut hasher = crc32fast::Hasher::new();
        
        hasher.write(&self.data[..CHECKSUM_OFFSET]);
        hasher.write(&self.data[CHECKSUM_OFFSET+4..]);

        hasher.finish() as u32
    }

    /// 计算并设置校验和
    pub fn update_checksums(&mut self) {
        if self.page_type == PageType::Header {
            let checksum = self.calculate_checksum();
            self.data[CHECKSUM_OFFSET..CHECKSUM_OFFSET+4]
                .copy_from_slice(&checksum.to_be_bytes());
        }
    }
    
    /// 验证校验和
    pub fn validate_checksum(&self) -> bool {
        if self.page_type != PageType::Header {
            return false;
        }

        let stored_checksum = u32::from_be_bytes(
            self.data[CHECKSUM_OFFSET..CHECKSUM_OFFSET+4].try_into().unwrap()
        );

        stored_checksum == self.calculate_checksum() 
    }
    
    


}