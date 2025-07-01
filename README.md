# Crab-Burrows 蟹洞

[![Crates.io](https://img.shields.io/crates/v/rudis)](https://crates.io/crates/rudis) 
[![Rust](https://img.shields.io/badge/rust-1.70+-orange)](https://www.rust-lang.org/) 
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)

一个用 Rust 实现的，类似 SQLite 的嵌入式数据库（我们称它为 Crab-Burrows 蟹洞）。

---

## 目录
- [Crab-Burrows 蟹洞](#crab-burrows-蟹洞)
  - [目录](#目录)
  - [什么是 Crab-Burrows](#什么是-crab-burrows)
  - [为什么开发 Crab-Burrows](#为什么开发-crab-burrows)
  - [功能特性](#功能特性)
  - [构建与使用](#构建与使用)
    - [安装](#安装)
    - [运行示例测试](#运行示例测试)
  - [项目设计](#项目设计)
    - [文件架构](#文件架构)
    - [页结构设计](#页结构设计)
  - [贡献](#贡献)
  - [许可证](#许可证)

---

## 什么是 Crab-Burrows
`Crab-Burrows` 是一个用 Rust 从零实现的轻量级, 个人学习性质的嵌入式数据库。包括SQL解析，查询优化，执行引擎，存储引擎，事务操作等内容。

---

## 为什么开发 Crab-Burrows
- 个人基于学习的深入实践 Rust 核心特性：所有权, 借用, 异步, 零拷贝等  
- 理解 SQL 内部实现

---

## 功能特性

- **轻量级嵌入式数据库**：单文件数据库设计，无外部依赖
- **B+树索引**：高效的数据存储和检索

---

## 构建与使用
### 安装
```bash
git clone https://github.com/MasterHesse/Crab-Burrows.git
cd crab-burrows
cargo build --release
```
---
### 运行示例测试
```bash
cargo run --release
```

---

## 项目设计
### 文件架构
```
src
│  btree.rs # B+树设计
│  lib.rs 
│  main.rs 
│  page.rs # 页结构设计
│  row.rs # 行结构设计
│  serialization.rs # 序列化
│  
└─storage
        buffer_pool.rs # 缓冲池
        disk_manager.rs # 磁盘管理
        mod.rs
        wal.rs 
```

### 页结构设计
```
[文件布局]
0-4KB:     Header Page (元数据，空闲列表指针)
4-8KB:     Free List Header (空闲页面管理)
8-12KB:    B+Tree Root Page (索引根节点)
12KB+:     Data Pages (实际数据存储)

[Header Page 结构]
0-8:       魔数 "CRABDBv1"
8-16:      空闲列表起始页ID
16-24:     根页面ID
24-28:     页面大小 (固定4096)
28-32:     校验和
32-4096:   保留空间

[Free List 结构]
使用链表管理空闲页面：
- 每个空闲页面存储下一个空闲页面的ID
- 最后一个空闲页面存储0
```

---

## 贡献

非常欢迎 Issue, PR 与讨论。  
本项目以学习为主，优先「简单, 可读」的实现。

---

## 许可证
- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)