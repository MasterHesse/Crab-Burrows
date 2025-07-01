use std::fs::{File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Read, Write};
use std::path::Path;
use crate::page::{self, Page, PageId, PageType, PAGE_SIZE};

pub struct DiskManager {
    file: File,
    next_free_page: PageId,
    file_size: u64,
}

impl DiskManager {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let file_size = file.metadata()?.len();

        // 空文件则初始化头页面
        if file_size == 0 {
            let header = Page::new(0, PageType::Header);
            file.write_all(&header.to_bytes())?;
            file.flush()?;
            return Ok(
                Self { file, next_free_page: 1, file_size: PAGE_SIZE as u64 }
            );
        }

        // 读取空闲列表信息
        let mut header_data = vec![0; PAGE_SIZE];
        file.seek(SeekFrom::Start(0))?;
        file.read_exact(&mut header_data)?;

        let header = Page::from_bytes(0, &header_data).ok_or_else(
            || io::Error::new(io::ErrorKind::InvalidData, "Invaild header page")
        )?;

        // 解析空闲列表指针
        let next_free_page = u64::from_be_bytes(header.data()[..8].try_into().unwrap());

        Ok(Self{
            file,
            next_free_page,
            file_size
        })
    }

    pub fn read_page(&mut self, page_id: PageId) -> io::Result<Page> {
        let offset = page_id * PAGE_SIZE as u64;
        if offset >= self.file_size {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Page out of bounds"
            ));
        }

        let mut data = vec![0; PAGE_SIZE];
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.read_exact(&mut data)?;

        Page::from_bytes(page_id, &data).ok_or_else(
            || io::Error::new(io::ErrorKind::InvalidData, 
                "Invail page data")
        )
    }

    pub fn write_page(&mut self, page: &Page) -> io::Result<()> {
        let offset = page.id * PAGE_SIZE as u64;

        // 若超出大小则拓展文件
        if offset + PAGE_SIZE as u64 > self.file_size {
            self.file.set_len(offset + PAGE_SIZE as u64)?;
            self.file_size = offset + PAGE_SIZE as u64;
        }

        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(&page.to_bytes())?;
        Ok(())
    }

    pub fn allocate_page(&mut self) -> PageId {
        let new_page_id = self.next_free_page;
        self.next_free_page += 1;
        new_page_id
    }

    pub fn deallocate_page(&mut self, page_id: PageId) {
        // 简化实现：将页面添加到空闲列表末尾
        // 实际实现应维护空闲链表
    }

    // 同步
    pub fn sync(&mut self) -> io::Result<()> {
        self.file.flush()?;
        self.file.sync_all()
    }

    pub fn shutdown(&mut self) -> io::Result<()> {
        // 更新头页面中的空闲列表指针
        let mut header = Page::new(0, PageType::Header);
        header.data_mut()[..8].copy_from_slice(&self.next_free_page.to_be_bytes());
        self.write_page(&header)?;
        self.sync()
    }    


}