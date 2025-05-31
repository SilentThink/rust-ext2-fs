use super::*;

#[derive(Clone, Debug)]
enum IndexedBlk {
    /// 直接索引
    Directly(u16),
    /// 一次索引
    OnceIndexed(u16),
    /// 二次索引
    TwiceIndexed(u16, u16),
}

/// 记录逻辑地址在磁盘上的实际位置
#[derive(Clone, Debug)]
pub struct RealAddr {
    /// 物理地址
    pub(in crate::fs) addr: u64,
    /// 在磁盘上的数据块
    data_blk: IndexedBlk,
}

/// 索引节点
#[repr(align(32))]
#[derive(Clone)]
pub struct Inode {
    /// 文件存取权限
    pub i_mode: FileMode,
    /// 文件数据块个数
    pub i_blocks: u16,
    /// 文件大小
    pub i_size: u32,
    /// 创建时间
    pub i_ctime: u32,
    /// 修改时间
    pub i_mtime: u32,
    /// 指向数据块的指针数组（使用二级索引）
    pub(in crate::fs) i_block: [u16; 8],
}

impl Default for Inode {
    fn default() -> Self {
        let now = utils::now();
        Self {
            i_mode: Default::default(),
            i_size: 0,
            i_blocks: 0,
            i_block: Default::default(),
            i_ctime: now,
            i_mtime: now,
        }
    }
}

impl Inode {
    /// 将逻辑地址转换成物理地址
    /// 用来读取索引节点存储的文件内容
    pub(in crate::fs) fn convert_addr(
        &self,
        disk: &std::fs::File,
        logicaddr: u64,
    ) -> Result<RealAddr> {
        let blk_i = logicaddr / BLOCK_SIZE as u64;
        let blk_offset = logicaddr % BLOCK_SIZE as u64;

        let addr = if blk_i <= 5 {
            RealAddr {
                addr: Fs::addr_data_blk(self.i_block[blk_i as usize]) + blk_offset as u64,
                data_blk: IndexedBlk::Directly(self.i_block[blk_i as usize] as u16),
            }
        } else if blk_i - 6 < BLOCK_ADDR_NUM as u64 {
            // 一级索引
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
            // 二级索引
            let blk_i = blk_i - BLOCK_ADDR_NUM as u64 - 6;

            let mut addr = [0u8; 4];
            disk.read_at(
                &mut addr,
                Fs::addr_data_blk(self.i_block[7]) + blk_i / BLOCK_ADDR_NUM as u64 * 4,
            )?;
            let addr1 = i32::from_le_bytes(addr) as u16;

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

    /// 为索引节点分配数据块
    pub(in crate::fs) fn alloc_data_block(&mut self, fs: &mut Fs) -> Result<u16> {
        let blk = if self.i_blocks < 6 {
            // 直接索引
            let addr = fs.alloc(BlkType::DataBlk)?;
            self.i_block[self.i_blocks as usize] = addr;
            addr
        } else if self.i_blocks < 6 + BLOCK_ADDR_NUM as u16 {
            // 一级索引
            let offset = self.i_blocks - 6;
            if offset == 0 {
                self.i_block[6] = fs.alloc(BlkType::DataBlk)?;
            }
            let addr = fs.alloc(BlkType::DataBlk)?;
            fs.disk.write_at(
                &(addr as i32).to_le_bytes(),
                Fs::addr_data_blk(self.i_block[6]) + offset as u64 * 4,
            )?;
            addr
        } else {
            // 二级索引
            let offset = self.i_blocks as u64 - 6 - BLOCK_ADDR_NUM as u64;
            if offset == 0 {
                self.i_block[7] = fs.alloc(BlkType::DataBlk)?;
            }

            if offset % BLOCK_ADDR_NUM as u64 == 0 {
                let addr1 = fs.alloc(BlkType::DataBlk)?;
                fs.disk.write_at(
                    &(addr1 as i32).to_le_bytes(),
                    Fs::addr_data_blk(self.i_block[7]) + offset / BLOCK_ADDR_NUM as u64 * 4,
                )?;
                let addr2 = fs.alloc(BlkType::DataBlk)?;
                fs.disk
                    .write_at(&(addr2 as i32).to_le_bytes(), Fs::addr_data_blk(addr1))?;
                addr2
            } else {
                let mut addr = [0u8; 4];
                fs.disk.read_at(
                    &mut addr,
                    Fs::addr_data_blk(self.i_block[7]) + offset / BLOCK_ADDR_NUM as u64 * 4,
                )?;
                let addr1 = i32::from_le_bytes(addr) as u16;

                let addr2 = fs.alloc(BlkType::DataBlk)?;
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

    /// 清空索引节点，将索引节点的数据块个数设置为 `new_blk_counts`，
    ///
    /// - 如果 `new_blk_counts` 大于等于索引节点记录的数据块个数，那么这个函数不会清除任何数据块
    /// - 如果 `new_blk_counts` 小于索引节点记录的数据块个数，这个函数将删除多余的数据块
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
                    // 一级索引
                    // 额外删除索引节点
                    if i == 6 {
                        blks_to_clean.push(self.i_block[6])
                    }
                    blks_to_clean.push(addr)
                }
                IndexedBlk::TwiceIndexed(addr1, addr2) => {
                    // 二级索引
                    // 额外删除索引节点
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
