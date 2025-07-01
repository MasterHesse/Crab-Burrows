// src/storage/buffer_pool.rs

use std::collections::HashMap;
use std::num::NonZero;
use lru::LruCache;
use crate::page::{self, Page, PageId};
use crate::storage::disk_manager::{self, DiskManager};
use std::sync::{Arc, Mutex};
use std::io;

pub struct BufferPool {
    cache: LruCache<PageId, Page>,
    disk_manager: Arc<Mutex<DiskManager>>,
    dirty_pages: HashMap<PageId, Page>,
}

impl BufferPool {
    pub fn new(capacity: usize, disk_manager: Arc<Mutex<DiskManager>>) -> Self {
        Self {
            cache: LruCache::new(NonZero::new(capacity).unwrap()),
            disk_manager,
            dirty_pages: HashMap::new(),
        }
    }

    pub fn fetch_page(&mut self, page_id: PageId) -> Option<Page> {
        // 检查缓存
        if let Some(page) = self.cache.get(&page_id) {
            return Some(page.clone());
        }

        //磁盘加载
        let mut dm = self.disk_manager.lock().unwrap();
        if let Ok(page) = dm.read_page(page_id) {
            self.cache.put(page_id, page.clone());
            return Some(page);
        }

        None
    }

    pub fn create_page(&mut self) -> Page {
        let mut dm = self.disk_manager.lock().unwrap();
        let page_id = dm.allocate_page();
        Page::new(page_id, page::PageType::Leaf) // 默认叶子节点
    }

    pub fn unpin_page(&mut self, page: Page, is_dirty: bool) {
        self.cache.put(page.id, page.clone());
        if is_dirty {
            self.dirty_pages.insert(page.id, page);
        }
    }

    pub fn flush_page(&mut self, page_id: PageId) -> io::Result<()> {
        if let Some(page) = self.dirty_pages.remove(&page_id) {
            let mut dm = self.disk_manager.lock().unwrap();
            dm.write_page(&page)?;
        }

        Ok(())
    }

    pub fn flush_all(&mut self) -> io::Result<()> {
        for (_, page) in self.dirty_pages.drain() {
            let mut dm = self.disk_manager.lock().unwrap();
            dm.write_page(&page)?;
        }

        self.disk_manager.lock().unwrap().sync()
    }
}