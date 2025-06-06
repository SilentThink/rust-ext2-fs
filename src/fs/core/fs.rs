//! # 文件系统核心模块
//!
//! 此模块实现了 Ext2 文件系统的核心功能，包括：
//! - 文件系统描述符管理
//! - 用户管理
//! - 目录项管理
//! - 磁盘块分配和释放
//! - 索引节点管理
//! - 路径解析

use super::*;

/// 块类型枚举
/// 
/// 用于区分不同类型的磁盘块，以便进行相应的分配和释放操作
pub(in crate::fs) enum BlkType {
    /// 索引节点块
    INode,
    /// 数据块
    DataBlk,
}

/// 用户信息结构体
/// 
/// 存储用户的基本信息，包括用户名和密码
#[derive(Default, Debug, Clone, Copy)]
#[repr(align(32))]
pub struct User {
    /// 用户名，最大长度为16字节
    pub name: [u8; 16],
    /// 密码，最大长度为16字节
    pub password: [u8; 16],
}

/// 组描述符结构体
/// 
/// 存储文件系统的元数据信息，包括位图位置、空闲块数量、用户信息等
#[repr(align(32))]
#[derive(Default)]
pub struct GroupDesc {
    /// 卷名，最大长度为16字节
    pub volume_name: [u8; 16],
    /// 保存块位图所在的块号
    pub block_bitmap: u16,
    /// 索引节点位图的块号
    pub inode_bitmap: u16,
    /// 索引表的起始位置
    pub inode_table: u16,
    /// 空闲块的个数
    pub free_blocks_count: u16,
    /// 空闲索引节点的个数    
    pub free_inodes_count: u16,
    /// 目录个数
    pub used_dirs_count: u16,
    /// 用户表，最多支持10个用户
    pub users: [User; 10],
    /// 当前用户数量
    pub users_len: u16,
}

impl GroupDesc {
    /// 创建新的组描述符
    /// 
    /// # 返回值
    /// 返回初始化好的组描述符实例，包含默认的root用户
    /// 
    /// # 默认配置
    /// - 卷名: "Ext2Disk"
    /// - 块位图位置: 块1
    /// - 索引节点位图位置: 块2
    /// - 索引节点表位置: 块3
    /// - 默认用户: root (密码: 123)
    pub(in crate::fs) fn new() -> Self {
        let mut users = [User::default(); 10];

        // 默认用户
        users[0] = User {
            name: "root".into_array().unwrap(),
            password: "123".into_array().unwrap(),
        };

        Self {
            volume_name: "Ext2Disk".into_array().unwrap(),
            block_bitmap: 1,
            inode_bitmap: 2,
            inode_table: 3,
            free_blocks_count: DATA_BLOCKS as u16,
            free_inodes_count: DATA_BLOCKS as u16,
            used_dirs_count: 0,
            users_len: 1,
            users,
        }
    }
}

/// 目录项结构体
/// 
/// 表示目录中的一个条目，包含文件名、索引节点号、文件类型等信息
#[repr(align(32))]
#[derive(Default, PartialEq, Debug, Clone)]
pub struct DirEntry {
    /// 索引节点号
    pub i_node: u16,
    /// 目录项长度
    pub rec_len: u16,
    /// 文件名长度
    pub name_len: u8,
    /// 文件类型
    pub file_type: u8,
    /// 文件名，最大长度为16字节
    pub name: [u8; 16],
}

/// 数据块类型别名
/// 
/// 表示一个完整的数据块，大小为 BLOCK_SIZE
type DataBlk = [u8; BLOCK_SIZE as usize];

/// 文件结构体
/// 
/// 表示一个打开的文件，包含索引节点信息、文件描述符、当前位置等
#[derive(Clone)]
pub(in crate::fs) struct File {
    /// 文件的索引节点
    pub inode: Inode,
    /// 索引节点号
    pub inode_i: u16,
    /// 目录项在磁盘上的地址
    pub dir_entry_addr: u64,
    /// 父目录的索引节点号
    pub parent_inode_i: u16,
    /// 当前文件指针位置
    pub current_pos: usize,
}

/// 文件系统主结构体
/// 
/// 管理整个文件系统的状态，包括磁盘访问、文件描述符、用户信息等
pub struct Fs {
    /// 维护的文件系统描述结构体
    pub(in crate::fs) fs_desc: GroupDesc,
    /// 写入/读取 虚拟磁盘的文件
    pub(in crate::fs) disk: std::fs::File,
    /// 用来记录当前打开的文件
    pub(in crate::fs) fds: [Option<File>; FD_LIMIT],
    /// 当前文件打开的个数
    pub(in crate::fs) opened_len: usize,
    /// 当前登录用户
    pub(in crate::fs) user: usize,
    /// 指向当前目录的 DirEntry 节点
    pub cwd: DirEntry,
}

impl Fs {
    /// 获取索引节点在磁盘上的物理地址
    /// 
    /// # 参数
    /// - `i_node`: 索引节点号
    /// 
    /// # 返回值
    /// 返回索引节点在磁盘上的字节偏移量
    #[inline(always)]
    pub(in crate::fs) fn addr_i_node(&self, i_node: u16) -> u64 {
        BLOCK_SIZE as u64 * (self.fs_desc.inode_table as u64 + i_node as u64)
    }

    /// 获取数据块在磁盘上的物理地址
    /// 
    /// # 参数
    /// - `data_blk`: 数据块号
    /// 
    /// # 返回值
    /// 返回数据块在磁盘上的字节偏移量
    pub(in crate::fs) fn addr_data_blk(data_blk: u16) -> u64 {
        BLOCK_SIZE as u64 * (DATA_BEGIN_BLOCK as u64 + data_blk as u64)
    }

    /// 将文件系统描述符写入磁盘
    /// 
    /// # 返回值
    /// 成功时返回 Ok(())，失败时返回错误信息
    /// 
    /// # 错误
    /// 当磁盘写入失败时返回相应的IO错误
    pub(in crate::fs) fn write_fs_desc(&mut self) -> Result<()> {
        self.disk.write_at(self.fs_desc.bytes(), 0)?;
        Ok(())
    }

    /// 从位图中寻找空闲位并标记为已使用
    /// 
    /// # 参数
    /// - `map`: 位图数据块的可变引用
    /// 
    /// # 返回值
    /// 成功时返回找到的空闲位的位置，失败时返回错误
    /// 
    /// # 错误
    /// 当位图已满时返回 NotFound 错误
    /// 
    /// # 算法
    /// 遍历位图的每个字节，检查每一位是否为0（空闲），
    /// 找到后将该位设置为1（已使用）并返回位置
    fn find_free_bit(map: &mut DataBlk) -> Result<u16> {
        let mut blk = 0u16;
        for i in 0..map.len() as usize {
            let mut to_match: u8 = 0b1000_0000;
            for _ in 0..8 {
                // 找到空闲节点
                if map[i] & to_match == 0 {
                    // 将位图里的节点设置为 1
                    map[i] = map[i] | to_match;
                    return Ok(blk);
                }
                to_match = to_match >> 1;
                blk += 1;
            }
        }
        Err(Error::new(ErrorKind::NotFound, "Inode table is full"))
    }

    /// 清除位图中指定位的使用标记
    /// 
    /// # 参数
    /// - `map`: 位图数据块的可变引用
    /// - `bit_i`: 要清除的位的索引
    /// 
    /// # 算法
    /// 计算位在字节中的位置，创建掩码并清除对应的位
    fn clear_used_bit(map: &mut DataBlk, bit_i: u16) {
        let i = bit_i / 8;
        let bit = bit_i % 8;
        let mask = 0b1000_0000 >> bit;
        map[i as usize] = map[i as usize] & !mask;
    }

    /// 将索引节点写入磁盘
    /// 
    /// # 参数
    /// - `inode_no`: 索引节点号
    /// - `inode`: 要写入的索引节点
    /// 
    /// # 返回值
    /// 成功时返回 Ok(())，失败时返回错误信息
    /// 
    /// # 错误
    /// - 当索引节点号超出范围时返回 OutOfMemory 错误
    /// - 当磁盘写入失败时返回相应的IO错误
    pub(in crate::fs) fn write_inode(&mut self, inode_no: u16, inode: Inode) -> Result<()> {
        let offset = self.addr_i_node(inode_no);
        match offset >= Fs::addr_data_blk(0) {
            true => Err(Error::new(
                ErrorKind::OutOfMemory,
                "the inode_no out of bounds",
            )),
            false => {
                self.disk.write_at(inode.bytes(), offset)?;
                Ok(())
            }
        }
    }

    /// 分配空闲的数据块或索引节点
    /// 
    /// # 参数
    /// - `alloc_type`: 分配类型（数据块或索引节点）
    /// 
    /// # 返回值
    /// 成功时返回分配的块号，失败时返回错误信息
    /// 
    /// # 错误
    /// - 当没有空闲空间时返回 Other 错误
    /// - 当位图操作失败时返回相应错误
    /// 
    /// # 算法
    /// 1. 根据分配类型选择对应的位图和计数器
    /// 2. 检查是否有空闲空间
    /// 3. 读取位图，寻找空闲位并标记
    /// 4. 更新计数器和文件系统描述符
    pub(in crate::fs) fn alloc(&mut self, alloc_type: BlkType) -> Result<u16> {
        let (map_blk, free_count) = match alloc_type {
            BlkType::DataBlk => (
                self.fs_desc.block_bitmap,
                &mut self.fs_desc.free_blocks_count,
            ),
            BlkType::INode => (
                self.fs_desc.inode_bitmap,
                &mut self.fs_desc.free_inodes_count,
            ),
        };

        if *free_count == 0 {
            return Err(Error::new(ErrorKind::Other, "No space to alloc"));
        }

        // 读取位图
        let mut bit_map = utils::empty_blk();
        self.disk
            .read_at(&mut bit_map, map_blk as u64 * BLOCK_SIZE as u64)?;

        // 寻找空的数据块，将对应的位设置成 1
        let blk = Self::find_free_bit(&mut bit_map)?;
        self.disk.write_at(&bit_map, map_blk as u64 * BLOCK_SIZE as u64)?;

        *free_count -= 1;
        self.write_fs_desc()?;

        Ok(blk)
    }

    /// 释放数据块或索引节点
    /// 
    /// # 参数
    /// - `free_type`: 释放类型（数据块或索引节点）
    /// - `nodes_i`: 要释放的块号数组
    /// 
    /// # 返回值
    /// 成功时返回 Ok(())，失败时返回错误信息
    /// 
    /// # 错误
    /// 当位图操作或磁盘写入失败时返回相应错误
    /// 
    /// # 算法
    /// 1. 根据释放类型选择对应的位图和计数器
    /// 2. 读取位图
    /// 3. 清除指定位的使用标记
    /// 4. 更新计数器和文件系统描述符
    pub(in crate::fs) fn free(&mut self, free_type: BlkType, nodes_i: &[u16]) -> Result<()> {
        let (map_blk, free_count) = match free_type {
            BlkType::DataBlk => (
                self.fs_desc.block_bitmap,
                &mut self.fs_desc.free_blocks_count,
            ),
            BlkType::INode => (
                self.fs_desc.inode_bitmap,
                &mut self.fs_desc.free_inodes_count,
            ),
        };

        let mut bit_map = utils::empty_blk();
        self.disk
            .read_at(&mut bit_map, map_blk as u64 * BLOCK_SIZE as u64)?;

        for &bit_i in nodes_i {
            Self::clear_used_bit(&mut bit_map, bit_i);
        }

        self.disk.write_at(&bit_map, map_blk as u64 * BLOCK_SIZE as u64)?;

        *free_count += nodes_i.len() as u16;
        self.write_fs_desc()?;

        Ok(())
    }
}

/// 路径解析结果
/// 
/// 包含路径解析后得到的目录项信息和相关地址
#[derive(Debug)]
pub struct PathParseRes {
    /// 当前路径名对应的目录项
    pub dir_entry: DirEntry,
    /// 目录项在磁盘上的物理地址
    pub(in crate::fs) dir_entry_addr: u64,
    /// 父目录对应的索引节点号
    pub(in crate::fs) parent_inode_i: u16,
}
