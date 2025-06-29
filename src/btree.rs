use std::cmp::Ordering;
use crate::page::PageId;

const ORDER: usize = 32; // B+ 树阶数

#[derive(Debug)]
pub enum BTreeNode {
    Internal(InternalNode),
    Leaf(LeafNode),
}

#[derive(Debug)]
pub struct InternalNode {
    keys: Vec<Vec<u8>>,
    children: Vec<PageId>,
}

#[derive(Debug)]
pub struct LeafNode {
    keys: Vec<Vec<u8>>,
    values: Vec<Vec<u8>>,
    next_leaf: Option<PageId>, // 下一个叶子节点的指针
}

#[derive(Debug)]
pub struct BTree {
    root: PageId,

    // 待实现，页面管理器的引用
    // ...
}

impl BTree {
    pub fn new(root_page_id: PageId) -> Self {
        Self { root: root_page_id }
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        // 查找，需要页面管理器
        None
    }

    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        // 插入，需要处理节点分裂
    }

    pub fn delete(&mut self, key: &[u8]) -> bool {
        // 删除，需要处理节点合并
        false
    }

    // 节点分裂（内部节点）
    fn split_internal_node(&mut self, node: &mut InternalNode) -> InternalNode {
        let split_index = ORDER / 2;
        let new_keys = node.keys.drain(split_index..).collect();
        let new_children = node.children.drain(split_index..).collect();

        InternalNode { keys: new_keys, children: new_children }
    }

    fn split_leaf_node(&mut self, node: &mut LeafNode) -> LeafNode {
        let split_index = ORDER / 2;
        let new_keys = node.keys.drain(split_index..).collect();
        let new_values = node.values.drain(split_index..).collect();

        LeafNode { keys: new_keys, values: new_values, next_leaf: node.next_leaf.take() }
    }
}