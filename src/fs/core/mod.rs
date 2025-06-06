//! # 文件系统核心模块
//!
//! 此模块包含了 Ext2 文件系统实现的核心组件，提供了完整的文件系统功能。
//! 
//! ## 模块结构
//! 
//! - [`inode`] - 索引节点管理，处理文件元数据和数据块索引
//! - [`file`] - 文件类型和权限管理，定义文件权限模式
//! - [`traits`] - 核心特征定义，提供数据序列化和磁盘IO功能
//! - [`iter`] - 目录项迭代器，支持目录内容遍历
//! - [`fs`] - 文件系统主体结构，管理磁盘空间和用户会话
//! - [`utils`] - 工具函数集合，提供常用的辅助功能
//! 
//! ## 主要功能
//! 
//! - **文件管理**: 创建、读取、写入、删除文件
//! - **目录管理**: 创建、遍历、删除目录
//! - **权限控制**: 用户权限验证和文件访问控制
//! - **空间管理**: 磁盘块分配和释放
//! - **索引管理**: 多级索引支持大文件存储
//! 
//! ## 使用示例
//! 
//! ```rust
//! use crate::fs::core::*;
//! 
//! // 创建文件系统
//! let mut fs = Fs::format()?;
//! 
//! // 创建目录
//! fs.mkdir("documents")?;
//! 
//! // 创建文件
//! let fd = fs.create("test.txt")?;
//! fs.write(fd, b"Hello, World!")?;
//! fs.close(fd)?;
//! ```

pub mod inode;
pub mod file;
pub mod traits;
pub mod iter;
pub mod fs;
pub mod utils;

pub use inode::*;
pub use file::*;
pub use traits::*;
pub use iter::*;
pub use fs::*;
pub use super::constant::*;

pub use std::io::Error;
pub use std::io::ErrorKind;
pub use std::io::Result;