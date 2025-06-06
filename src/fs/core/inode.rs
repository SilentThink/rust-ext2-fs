//! # 索引节点管理模块
//!
//! 此模块实现了 Ext2 文件系统的索引节点（inode）管理功能，包括：
//! - 索引节点的数据结构定义
//! - 逻辑地址到物理地址的转换
//! - 数据块的分配和释放
//! - 多级索引的支持（直接索引、一级索引、二级索引）

use super::*;

/// 索引块类型枚举
/// 
/// 表示不同级别的索引方式，用于文件数据块的寻址
#[derive(Clone, Debug)]
enum IndexedBlk {
    /// 直接索引
    /// 
    /// 直接指向数据块，用于小文件的快速访问
    Directly(u16),
    /// 一次索引
    /// 
    /// 通过一级索引表间接指向数据块
    OnceIndexed(u16),
    /// 二次索引
    /// 
    /// 通过二级索引表间接指向数据块，支持更大的文件
    TwiceIndexed(u16, u16),
}

/// 真实地址结构体
/// 
/// 记录逻辑地址在磁盘上的实际物理位置和索引方式
#[derive(Clone, Debug)]
pub struct RealAddr {
    /// 物理地址（磁盘上的字节偏移量）
    pub(in crate::fs) addr: u64,
    /// 在磁盘上的数据块索引方式
    data_blk: IndexedBlk,
}

/// 索引节点结构体
/// 
/// 存储文件的元数据信息，包括权限、大小、时间戳、数据块指针等
#[repr(align(32))]
#[derive(Clone)]
pub struct Inode {
    /// 文件存取权限和所有者信息
    pub i_mode: FileMode,
    /// 文件数据块个数
    pub i_blocks: u16,
    /// 文件大小（字节数）
    pub i_size: u32,
    /// 创建时间（Unix时间戳）
    pub i_ctime: u32,
    /// 修改时间（Unix时间戳）
    pub i_mtime: u32,
    /// 硬链接数量（引用计数）
    pub i_links_count: u16,
    /// 指向数据块的指针数组（使用多级索引）
    /// 
    /// 索引结构：
    /// - i_block[0-5]: 直接索引，直接指向数据块
    /// - i_block[6]: 一级索引，指向包含数据块地址的索引块
    /// - i_block[7]: 二级索引，指向包含一级索引块地址的索引块
    pub(in crate::fs) i_block: [u16; 8],
}

impl Default for Inode {
    /// 创建默认的索引节点
    /// 
    /// # 返回值
    /// 返回初始化的索引节点，设置当前时间为创建和修改时间
    fn default() -> Self {
        let now = utils::now();
        Self {
            i_mode: Default::default(),
            i_size: 0,
            i_blocks: 0,
            i_block: Default::default(),
            i_ctime: now,
            i_mtime: now,
            i_links_count: 1, // 默认为1个引用（创建时）
        }
    }
}

impl Inode {
    /// 将逻辑地址转换成物理地址
    /// 
    /// # 参数
    /// - `disk`: 磁盘文件引用
    /// - `logicaddr`: 文件内的逻辑地址（字节偏移量）
    /// 
    /// # 返回值
    /// 成功时返回包含物理地址和索引方式的 RealAddr 结构体
    /// 
    /// # 错误
    /// 当磁盘读取失败时返回相应的IO错误
    /// 
    /// # 算法
    /// 1. 计算逻辑地址对应的块号和块内偏移
    /// 2. 根据块号范围确定索引方式：
    ///    - 0-5: 直接索引
    ///    - 6-(6+BLOCK_ADDR_NUM): 一级索引
    ///    - 其他: 二级索引
    /// 3. 根据索引方式计算最终的物理地址
    pub(in crate::fs) fn convert_addr(
        &self,
        disk: &std::fs::File,
        logicaddr: u64,
    ) -> Result<RealAddr> {
        let blk_i = logicaddr / BLOCK_SIZE as u64;
        let blk_offset = logicaddr % BLOCK_SIZE as u64;

        let addr = if blk_i <= 5 {
            // 直接索引：直接从 i_block 数组获取数据块地址
            RealAddr {
                addr: Fs::addr_data_blk(self.i_block[blk_i as usize]) + blk_offset as u64,
                data_blk: IndexedBlk::Directly(self.i_block[blk_i as usize] as u16),
            }
        } else if blk_i - 6 < BLOCK_ADDR_NUM as u64 {
            // 一级索引：通过索引表间接获取数据块地址
            let mut addr = [0u8; 4];
            disk.read_at(
                &mut addr,
                Fs::addr_data_blk(self.i_block[6]) + (blk_i - 6) * 4,
            )?;
            let addr = i32::from_le_bytes(addr) as u16;
            RealAddr {
                addr: Fs::addr_data_blk(addr) + blk_offset,
                data_blk: IndexedBlk::OnceIndexed(addr),
            }
        } else {
            // 二级索引：通过两级索引表获取数据块地址
            let blk_i = blk_i - BLOCK_ADDR_NUM as u64 - 6;

            // 读取一级索引表地址
            let mut addr = [0u8; 4];
            disk.read_at(
                &mut addr,
                Fs::addr_data_blk(self.i_block[7]) + blk_i / BLOCK_ADDR_NUM as u64 * 4,
            )?;
            let addr1 = i32::from_le_bytes(addr) as u16;

            // 读取最终数据块地址
            addr.fill(0);
            disk.read_at(
                &mut addr,
                Fs::addr_data_blk(addr1) + blk_i % BLOCK_ADDR_NUM as u64 * 4,
            )?;
            let addr2 = i32::from_be_bytes(addr) as u16;
            RealAddr {
                addr: Fs::addr_data_blk(addr2) + blk_offset,
                data_blk: IndexedBlk::TwiceIndexed(addr1, addr2),
            }
        };

        Ok(addr)
    }

    /// 为索引节点分配新的数据块
    /// 
    /// # 参数
    /// - `fs`: 文件系统的可变引用
    /// 
    /// # 返回值
    /// 成功时返回新分配的数据块号
    /// 
    /// # 错误
    /// 当磁盘空间不足或磁盘操作失败时返回相应错误
    /// 
    /// # 算法
    /// 根据当前已分配的块数选择合适的索引方式：
    /// 1. 前6个块使用直接索引
    /// 2. 接下来的块使用一级索引
    /// 3. 更多的块使用二级索引
    pub(in crate::fs) fn alloc_data_block(&mut self, fs: &mut Fs) -> Result<u16> {
        let blk = if self.i_blocks < 6 {
            // 直接索引：直接在 i_block 数组中存储数据块地址
            let addr = fs.alloc(BlkType::DataBlk)?;
            self.i_block[self.i_blocks as usize] = addr;
            addr
        } else if self.i_blocks < 6 + BLOCK_ADDR_NUM as u16 {
            // 一级索引：需要索引表来存储数据块地址
            let offset = self.i_blocks - 6;
            if offset == 0 {
                // 第一次使用一级索引，需要分配索引表
                self.i_block[6] = fs.alloc(BlkType::DataBlk)?;
            }
            let addr = fs.alloc(BlkType::DataBlk)?;
            // 将数据块地址写入索引表
            fs.disk.write_at(
                &(addr as i32).to_le_bytes(),
                Fs::addr_data_blk(self.i_block[6]) + offset as u64 * 4,
            )?;
            addr
        } else {
            // 二级索引：需要两级索引表
            let offset = self.i_blocks as u64 - 6 - BLOCK_ADDR_NUM as u64;
            if offset == 0 {
                // 第一次使用二级索引，需要分配二级索引表
                self.i_block[7] = fs.alloc(BlkType::DataBlk)?;
            }

            if offset % BLOCK_ADDR_NUM as u64 == 0 {
                // 需要新的一级索引表
                let addr1 = fs.alloc(BlkType::DataBlk)?;
                // 将一级索引表地址写入二级索引表
                fs.disk.write_at(
                    &(addr1 as i32).to_le_bytes(),
                    Fs::addr_data_blk(self.i_block[7]) + offset / BLOCK_ADDR_NUM as u64 * 4,
                )?;
                let addr2 = fs.alloc(BlkType::DataBlk)?;
                // 将数据块地址写入一级索引表
                fs.disk
                    .write_at(&(addr2 as i32).to_le_bytes(), Fs::addr_data_blk(addr1))?;
                addr2
            } else {
                // 使用现有的一级索引表
                let mut addr = [0u8; 4];
                fs.disk.read_at(
                    &mut addr,
                    Fs::addr_data_blk(self.i_block[7]) + offset / BLOCK_ADDR_NUM as u64 * 4,
                )?;
                let addr1 = i32::from_le_bytes(addr) as u16;

                let addr2 = fs.alloc(BlkType::DataBlk)?;
                // 将数据块地址写入一级索引表
                fs.disk.write_at(
                    &(addr2 as i32).to_le_bytes(),
                    Fs::addr_data_blk(addr1) + offset % BLOCK_ADDR_NUM as u64 * 4,
                )?;
                addr2
            }
        };

        self.i_blocks += 1;

        Ok(blk)
    }

    /// 释放索引节点的数据块
    /// 
    /// # 参数
    /// - `new_blk_counts`: 新的数据块数量
    /// - `fs`: 文件系统的可变引用
    /// 
    /// # 返回值
    /// 成功时返回 Ok(())，失败时返回错误信息
    /// 
    /// # 行为
    /// - 如果 `new_blk_counts` >= 当前块数，不执行任何操作
    /// - 如果 `new_blk_counts` < 当前块数，释放多余的数据块
    /// 
    /// # 算法
    /// 1. 遍历需要释放的数据块
    /// 2. 根据索引方式确定需要释放的块（包括索引块）
    /// 3. 批量释放所有相关的数据块
    /// 4. 更新索引节点的块计数
    pub(in crate::fs) fn free_data_block(
        &mut self,
        new_blk_counts: u16,
        fs: &mut Fs,
    ) -> Result<()> {
        if new_blk_counts >= self.i_blocks {
            return Ok(());
        }

        let mut blks_to_clean: Vec<u16> = Vec::new();
        for i in new_blk_counts..self.i_blocks {
            // 要删除的数据块号
            match self.convert_addr(&fs.disk, i as u64 * BLOCK_SIZE as u64)?.data_blk {
                IndexedBlk::Directly(addr) => blks_to_clean.push(addr),
                IndexedBlk::OnceIndexed(addr) => {
                    // 一级索引：需要额外删除索引块
                    if i == 6 {
                        blks_to_clean.push(self.i_block[6])
                    }
                    blks_to_clean.push(addr)
                }
                IndexedBlk::TwiceIndexed(addr1, addr2) => {
                    // 二级索引：需要额外删除索引块
                    let offset = i - 6 - BLOCK_ADDR_NUM as u16;
                    if offset == 0 {
                        blks_to_clean.push(self.i_block[7]);
                    }
                    if offset % BLOCK_ADDR_NUM as u16 == 0 {
                        blks_to_clean.push(addr1);
                    }
                    blks_to_clean.push(addr2);
                }
            }
        }

        fs.free(BlkType::DataBlk, &blks_to_clean)?;

        self.i_blocks = new_blk_counts;
        Ok(())
    }
}
