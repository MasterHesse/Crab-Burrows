use std::io;
use crate::page::{Page, PageId, PageType, PAGE_SIZE};
use crate::storage::buffer_pool::BufferPool;

const BPLUS_TREE_ORDER: usize = 32; // B+树阶数

/// B+树内部节点结构
#[derive(Debug)]
struct InternalNode {
    keys: Vec<Vec<u8>>,
    children: Vec<PageId>,
}

/// B+树叶节点结构
#[derive(Debug)]
struct LeafNode {
    keys: Vec<Vec<u8>>,
    values: Vec<Vec<u8>>,
    next_leaf: Option<PageId>, // 下一个叶子节点的指针
}

/// B+树索引结构
#[derive(Debug)]
pub struct BTree {
    root_page_id: PageId,
}

impl BTree {
    /// 创建新的B+树实例
    pub fn new(root_page_id: PageId) -> Self {
        Self { root_page_id }
    }
    
    /// 获取根页面ID
    pub fn get_root_page_id(&self) -> PageId {
        self.root_page_id
    }
    
    /// 设置根页面ID
    pub fn set_root_page_id(&mut self, root_page_id: PageId) {
        self.root_page_id = root_page_id;
    }

    /// 根据键查找对应的值
    pub fn get_value(
        &self, 
        buffer_pool: &mut BufferPool, 
        key: &[u8]
    ) -> io::Result<Option<Vec<u8>>> {
        let mut current_page_id = self.root_page_id;
        
        loop {
            let page = buffer_pool.fetch_page(current_page_id)
                .ok_or_else(|| io::Error::new(
                    io::ErrorKind::NotFound, 
                    format!("Page {} not found", current_page_id)
                ))?;
            
            match page.page_type {
                PageType::Leaf => {
                    let leaf = self.deserialize_leaf_node(&page)?;
                    if let Some(pos) = leaf.keys.iter().position(|k| k.as_slice() == key) {
                        return Ok(Some(leaf.values[pos].clone()));
                    }
                    return Ok(None);
                }
                PageType::Internal => {
                    let internal = self.deserialize_internal_node(&page)?;
                    let pos = internal.keys.iter()
                        .position(|k| key < k.as_slice())
                        .unwrap_or(internal.keys.len());
                    
                    current_page_id = internal.children[pos];
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData, 
                        format!("Invalid page type {:?} in B+Tree traversal", page.page_type)
                    ));
                }
            }
        }
    }

    /// 插入键值对
    pub fn insert_key_value(
        &mut self, 
        buffer_pool: &mut BufferPool,
        key: Vec<u8>, 
        value: Vec<u8>
    ) -> io::Result<()> {
        let mut current_page_id = self.root_page_id;
        let mut path = Vec::new(); // 记录访问路径 (page_id, child_index)
        
        // 查找插入位置并记录路径
        loop {
            let mut page = buffer_pool.fetch_page(current_page_id)
                .ok_or_else(|| io::Error::new(
                    io::ErrorKind::NotFound, 
                    format!("Page {} not found", current_page_id)
                ))?;
            
            match page.page_type {
                PageType::Leaf => {
                    // 找到叶子节点，进行插入
                    let mut leaf = self.deserialize_leaf_node(&page)?;
                    let pos = leaf.keys.iter()
                        .position(|k| k.as_slice() >= key.as_slice())
                        .unwrap_or(leaf.keys.len());
                    
                    // 插入键值对
                    leaf.keys.insert(pos, key.clone());
                    leaf.values.insert(pos, value);
                    
                    // 检查是否需要分裂
                    if leaf.keys.len() > BPLUS_TREE_ORDER {
                        let (new_leaf, promote_key) = self.split_leaf_node(leaf);
                        let mut new_page = buffer_pool.create_page();
                        let new_page_id = new_page.id;
                        
                        // 序列化新叶子节点
                        self.serialize_leaf_node(&mut new_page, new_leaf)?;
                        buffer_pool.unpin_page(new_page, true);
                        
                        // 创建新内部节点或更新父节点
                        self.handle_node_split(
                            buffer_pool, 
                            &path, 
                            promote_key, 
                            new_page_id
                        )?;
                    } else {
                        // 更新当前叶子节点
                        self.serialize_leaf_node(&mut page, leaf)?;
                        buffer_pool.unpin_page(page, true);
                    }
                    
                    return Ok(());
                }
                PageType::Internal => {
                    let internal = self.deserialize_internal_node(&page)?;
                    let child_index = internal.keys.iter()
                        .position(|k| key.as_slice() < k.as_slice())
                        .unwrap_or(internal.keys.len());
                    
                    // 记录路径 (当前页面ID, 子节点索引)
                    path.push((current_page_id, child_index));
                    current_page_id = internal.children[child_index];
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData, 
                        format!("Invalid page type {:?} in B+Tree traversal", page.page_type)
                    ));
                }
            }
        }
    }

    /// 处理节点分裂
    fn handle_node_split(
        &mut self,
        buffer_pool: &mut BufferPool,
        path: &[(PageId, usize)],
        mut promote_key: Vec<u8>, // 改为可变
        mut new_page_id: PageId   // 改为可变
    ) -> io::Result<()> {
        // 从路径末尾开始处理分裂
        for &(page_id, child_index) in path.iter().rev() {
            let mut page = buffer_pool.fetch_page(page_id)
                .ok_or_else(|| io::Error::new(
                    io::ErrorKind::NotFound, 
                    format!("Page {} not found", page_id)
                ))?;
            
            let mut internal = self.deserialize_internal_node(&page)?;
            
            // 插入提升的键和新的页面ID
            internal.keys.insert(child_index, promote_key.clone());
            internal.children.insert(child_index + 1, new_page_id);
            
            // 检查是否需要继续分裂
            if internal.keys.len() <= BPLUS_TREE_ORDER {
                // 更新当前节点并返回
                self.serialize_internal_node(&mut page, internal)?;
                buffer_pool.unpin_page(page, true);
                return Ok(());
            }
            
            // 分裂内部节点
            let (new_internal, new_promote_key) = self.split_internal_node(internal);
            let mut new_page = buffer_pool.create_page();
            let new_page_id_temp = new_page.id;
            
            // 序列化新内部节点
            self.serialize_internal_node(&mut new_page, new_internal)?;
            buffer_pool.unpin_page(new_page, true);
            
            // 更新提升键和新页面ID
            promote_key = new_promote_key;
            new_page_id = new_page_id_temp;
        }
        
        // 如果路径已处理完但根节点需要分裂
        self.create_new_root_node(buffer_pool, promote_key, new_page_id)
    }

    /// 创建新的根节点
    fn create_new_root_node(
        &mut self,
        buffer_pool: &mut BufferPool,
        first_key: Vec<u8>,
        new_page_id: PageId
    ) -> io::Result<()> {
        // 创建新根节点
        let mut new_root = buffer_pool.create_page();
        new_root.page_type = PageType::Internal;
        
        let root_node = InternalNode {
            keys: vec![first_key],
            children: vec![self.root_page_id, new_page_id],
        };
        
        // 序列化新根节点
        self.serialize_internal_node(&mut new_root, root_node)?;
        
        // 更新根页面ID
        self.root_page_id = new_root.id;
        buffer_pool.unpin_page(new_root, true);
        
        Ok(())
    }

    /// 分裂叶子节点
    fn split_leaf_node(&self, mut node: LeafNode) -> (LeafNode, Vec<u8>) {
        let split_index = BPLUS_TREE_ORDER / 2;
        let new_keys: Vec<_> = node.keys.drain(split_index..).collect();
        let new_values = node.values.drain(split_index..).collect();
        
        // 新节点的第一个键作为提升键
        let promote_key = new_keys[0].clone();
        
        let new_leaf = LeafNode {
            keys: new_keys,
            values: new_values,
            next_leaf: node.next_leaf.take(),
        };
        
        (new_leaf, promote_key)
    }

    /// 分裂内部节点
    fn split_internal_node(&self, mut node: InternalNode) -> (InternalNode, Vec<u8>) {
        let split_index = BPLUS_TREE_ORDER / 2;
        let promote_key = node.keys.remove(split_index);
        
        let new_keys = node.keys.drain(split_index..).collect();
        let new_children = node.children.drain(split_index..).collect();
        
        let new_internal = InternalNode {
            keys: new_keys,
            children: new_children,
        };
        
        (new_internal, promote_key)
    }

    // === 序列化/反序列化方法 ===
    
    /// 序列化叶子节点到页面
    fn serialize_leaf_node(
        &self,
        page: &mut Page,
        node: LeafNode
    ) -> io::Result<()> {
        // 设置页面类型
        page.page_type = PageType::Leaf;
        
        // 获取可变引用
        let data = page.data_mut();

        // 清空页面
        data.fill(0);
        
        // 写入键值对数量
        let count = node.keys.len() as u16;
        data[0..2].copy_from_slice(&count.to_be_bytes());

        // 写入下一个叶子节点指针
        let next_leaf = node.next_leaf.unwrap_or(0);
        data[2..10].copy_from_slice(&next_leaf.to_be_bytes());

        // 写入键值对
        let mut offset = 10;
        for (key, value) in node.keys.iter().zip(node.values.iter()) {
            // 写入键长度
            let key_len = key.len() as u16;
            data[offset..offset+2].copy_from_slice(&key_len.to_be_bytes());
            offset += 2;

            // 写入键数据
            data[offset..offset+key.len()].copy_from_slice(key);
            offset += key.len();

            // 写入值长度
            let value_len = value.len() as u16;
            data[offset..offset+2].copy_from_slice(&value_len.to_be_bytes());
            offset += 2;

            // 写入值数据
            data[offset..offset+value.len()].copy_from_slice(value);
            offset += value.len();

            // 检查是否超出页面大小
            if offset > PAGE_SIZE {
                return Err(io::Error::new(
                    io::ErrorKind::OutOfMemory,
                    "Leaf node content exceeds page size"
                ));
            }
        }

        Ok(())
    }

    /// 从页面反序列化叶子节点
    fn deserialize_leaf_node(&self, page: &Page) -> io::Result<LeafNode> {
        if page.page_type != PageType::Leaf {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Page is not a leaf node"
            ));
        }

        let data = page.data();
        
        // 读取键值对数量
        let count = u16::from_be_bytes([data[0], data[1]]) as usize;

        // 读取下一个叶子节点指针
        let next_leaf = u64::from_be_bytes([
            data[2], data[3], data[4], data[5], 
            data[6], data[7], data[8], data[9]
        ]);
        let next_leaf = if next_leaf == 0 { None } else { Some(next_leaf) };

        let mut keys = Vec::with_capacity(count);
        let mut values = Vec::with_capacity(count);
        let mut offset = 10;

        for _ in 0..count {
            // 读取键长度
            let key_len = u16::from_be_bytes([data[offset], data[offset+1]]) as usize;
            offset += 2;
            
            // 读取键数据
            let key = data[offset..offset+key_len].to_vec();
            offset += key_len;
            
            // 读取值长度
            let value_len = u16::from_be_bytes([data[offset], data[offset+1]]) as usize;
            offset += 2;
            
            // 读取值数据
            let value = data[offset..offset+value_len].to_vec();
            offset += value_len;
            
            keys.push(key);
            values.push(value);
        }

        Ok(LeafNode {
            keys,
            values,
            next_leaf,
        })
    }

    /// 序列化内部节点到页面
    fn serialize_internal_node(
        &self,
        page: &mut Page,
        node: InternalNode
    ) -> io::Result<()> {
        // 设置页面类型
        page.page_type = PageType::Internal;

        let data = page.data_mut();

        // 清空页面
        data.fill(0);

        // 写入键数量
        let key_count = node.keys.len() as u16;
        data[0..2].copy_from_slice(&key_count.to_be_bytes());
        
        // 写入子节点数量 (应该等于键数量+1)
        let child_count = node.children.len() as u16;
        data[2..4].copy_from_slice(&child_count.to_be_bytes());
        
        let mut offset = 4;

        // 写入键
        for key in &node.keys {
            // 写入键长度
            let key_len = key.len() as u16;
            data[offset..offset+2].copy_from_slice(&key_len.to_be_bytes());
            offset += 2;
            
            // 写入键数据
            data[offset..offset+key.len()].copy_from_slice(key);
            offset += key.len();
        }
        
        // 写入子节点指针
        for child in &node.children {
            data[offset..offset+8].copy_from_slice(&child.to_be_bytes());
            offset += 8;
        }

        // 检查是否超出页面大小
        if offset > PAGE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::OutOfMemory,
                "Internal node content exceeds page size"
            ));
        }

        Ok(())
    }

    /// 反序列化内部节点
    fn deserialize_internal_node(&self, page: &Page) -> io::Result<InternalNode> {
        if page.page_type != PageType::Internal {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData, 
                "Page is not an internal node"
            ));
        }

        let data = page.data();
        
        // 读取键数量
        let key_count = u16::from_be_bytes([data[0], data[1]]) as usize;
        
        // 读取子节点数量
        let child_count = u16::from_be_bytes([data[2], data[3]]) as usize;

        // 验证子节点数量
        if child_count != key_count + 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData, 
                format!("Invalid child count: expected {}, got {}", key_count + 1, child_count)
            ));
        }

        let mut keys = Vec::with_capacity(key_count);
        let mut children = Vec::with_capacity(child_count);
        let mut offset = 4;

        // 读取键
        for _ in 0..key_count {
            // 读取键长度
            let key_len = u16::from_be_bytes([data[offset], data[offset+1]]) as usize;
            offset += 2;
            
            // 读取键数据
            let key = data[offset..offset+key_len].to_vec();
            offset += key_len;
            
            keys.push(key);
        }

        // 读取子节点指针
        for _ in 0..child_count {
            let child = u64::from_be_bytes([
                data[offset], data[offset+1], data[offset+2], data[offset+3],
                data[offset+4], data[offset+5], data[offset+6], data[offset+7]
            ]);
            offset += 8;
            
            children.push(child);
        }

        Ok(InternalNode { keys, children })
    }
}