// src/storage/wal.rs

use std::fs::{OpenOptions, File};
use std::io::{self, Write, Seek, SeekFrom};
use std::path::Path;
use bincode::{encode_into_std_write, Decode, Encode};
use crate::storage::disk_manager::{DiskManager};

#[derive(Debug, Encode, Decode)]
pub enum LogEntry {
    Insert {
        table_id: u32,
        key: Vec<u8>,
        value: Vec<u8>,
    },
    Delete {
        table_id: u32,
        key: Vec<u8>,
    },
    BeginTransaction,
    CommitTransaction,
    RollbackTransaction,
}

pub struct WriteAheadLog {
    log_file: File,
    log_sequence_number: u64,
}

impl WriteAheadLog {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let log_file = OpenOptions::new()
            .read(true)
            .truncate(true)
            .create(true)
            .append(true)
            .open(path)?;

        // 计算日志序列号
        let log_sequence_number = log_file.metadata()?.len();

        Ok(Self { log_file, log_sequence_number, })
    }

    pub fn log(&mut self, entry: &LogEntry) -> Result<u64, Box<dyn std::error::Error>> {
        let start_lsn = self.log_sequence_number;

        // 序列化日志条目
        encode_into_std_write(
            entry, 
            &mut self.log_file, 
            bincode::config::standard()
        )?;

        // 更新 LSN
        let current_pos = self.log_file.stream_position()?;
        self.log_sequence_number = current_pos;

        // 持久化，写入磁盘
        self.log_file.flush()?;

        Ok(start_lsn)
    }

    pub fn checkpoint(&mut self, disk_manager: &mut DiskManager) -> io::Result<()> {
        // 刷新所有脏页到磁盘
        disk_manager.sync()?;

        // 记录检查点
        let _ = self.log(&LogEntry::CommitTransaction);

        // 截断日志文件
        self.log_file.set_len(0)?;
        self.log_sequence_number = 0;

        Ok(())
    }

    pub fn rollback(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 此处简化实现为： 回滚当前事务
        // 实际上需要解析日志并撤销操作
        self.log(&LogEntry::RollbackTransaction)?;
        Ok(())
    }
}