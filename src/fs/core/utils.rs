//! # 工具函数模块
//!
//! 此模块提供了文件系统操作中常用的工具函数，包括：
//! - 时间戳获取
//! - 内存块操作
//! - 字符串处理
//! - 文件系统状态查询

use super::*;
use std::io::Write;

/// 获取当前Unix时间戳
/// 
/// # 返回值
/// 返回当前时间的Unix时间戳（秒数），类型为u32
/// 
/// # 用途
/// 主要用于设置文件的创建时间和修改时间
#[inline(always)]
pub fn now() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32
}

/// 创建空的数据块
/// 
/// # 返回值
/// 返回大小为 BLOCK_SIZE 的零填充字节数组
/// 
/// # 用途
/// 用于初始化新的数据块或清空现有数据块
#[inline(always)]
pub fn empty_blk() -> [u8; BLOCK_SIZE as usize] {
    [0u8; BLOCK_SIZE as usize]
}

impl Fs {
    /// 获取指定索引节点
    /// 
    /// # 参数
    /// - `inode_i`: 索引节点号
    /// 
    /// # 返回值
    /// 成功时返回索引节点，失败时返回错误
    /// 
    /// # 错误
    /// 当磁盘读取失败时返回相应的IO错误
    pub fn get_inode(&self, inode_i: u16) -> Result<Inode> {
        Ok(Inode::from_disk(&self.disk, self.addr_i_node(inode_i))?)
    }

    /// 安全退出文件系统
    /// 
    /// # 功能
    /// 1. 将文件系统描述符写入磁盘
    /// 2. 刷新磁盘缓冲区，确保所有数据都已写入
    /// 
    /// # 注意
    /// 此方法会强制刷新所有缓冲区，确保数据持久化
    pub fn exit(&mut self) {
        self.write_fs_desc().unwrap();
        self.disk.flush().unwrap()
    }

    /// 获取文件系统描述符的只读引用
    /// 
    /// # 返回值
    /// 返回文件系统描述符的不可变引用
    /// 
    /// # 用途
    /// 用于查询文件系统的元数据信息，如空闲块数量、用户信息等
    pub fn fs_desc(&self) -> &GroupDesc {
        &self.fs_desc
    }

    /// 获取当前登录用户的ID
    /// 
    /// # 返回值
    /// 返回当前用户的ID
    /// 
    /// # 用途
    /// 用于权限检查和文件所有权管理
    pub fn current_user(&self) -> usize {
        self.user
    }
}

/// 将字节数组转换为UTF-8字符串
/// 
/// # 参数
/// - `str`: 字节数组切片
/// 
/// # 返回值
/// 返回转换后的字符串切片，如果转换失败则返回错误提示
/// 
/// # 功能
/// 1. 尝试将字节数组转换为UTF-8字符串
/// 2. 移除字符串末尾的空字符（'\0'）
/// 3. 如果转换失败，返回错误提示字符串
/// 
/// # 用途
/// 主要用于处理文件名和其他存储在字节数组中的字符串数据
pub fn str(str: &[u8]) -> &str {
    std::str::from_utf8(str)
        .unwrap_or("[err invaild utf-8]")
        .trim_end_matches('\0')
}
