use super::*;

pub(in crate::fs) enum BlkType {
    INode,
    DataBlk,
}

#[derive(Default, Debug, Clone, Copy)]
#[repr(align(32))]
pub struct User {
    pub name: [u8; 16],
    pub password: [u8; 16],
}

#[repr(align(32))]
#[derive(Default)]
pub struct GroupDesc {
    /// 卷名
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
    /// 用户表
    pub users: [User; 10],
    pub users_len: u16,
}

impl GroupDesc {
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
    /// 文件名
    pub name: [u8; 16],
}

type DataBlk = [u8; BLOCK_SIZE as usize];

#[derive(Clone)]
pub(in crate::fs) struct File {
    pub inode: Inode,
    pub inode_i: u16,
    pub dir_entry_addr: u64,
    pub parent_inode_i: u16,
    pub current_pos: usize,
}

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
    /// 获取索引节点 i_node 在磁盘的位置
    #[inline(always)]
    pub(in crate::fs) fn addr_i_node(&self, i_node: u16) -> u64 {
        BLOCK_SIZE as u64 * (self.fs_desc.inode_table as u64 + i_node as u64)
    }

    /// 获取数据块 data_blk 在磁盘的位置
    pub(in crate::fs) fn addr_data_blk(data_blk: u16) -> u64 {
        BLOCK_SIZE as u64 * (DATA_BEGIN_BLOCK as u64 + data_blk as u64)
    }

    pub(in crate::fs) fn write_fs_desc(&mut self) -> Result<()> {
        self.disk.write_at(self.fs_desc.bytes(), 0)?;
        Ok(())
    }

    /// 从位图里寻找为 0 的位，并将这个位设置为 1，返回这个位的位置
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

    /// 从位图的位设置为 0
    fn clear_used_bit(map: &mut DataBlk, bit_i: u16) {
        let i = bit_i / 8;
        let bit = bit_i % 8;
        let mask = 0b1000_0000 >> bit;
        map[i as usize] = map[i as usize] & !mask;
    }

    /// 将索引节点写入磁盘
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

    /// 分配空闲的数据块 / 索引节点
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

    /// 删除数据块 / 索引节点
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

/// 路径解析结果，包含当前路径对应的目录项
#[derive(Debug)]
pub struct PathParseRes {
    /// 当前路径名对应的目录项
    pub dir_entry: DirEntry,
    pub(in crate::fs) dir_entry_addr: u64,
    /// 父目录对应的索引节点
    pub(in crate::fs) parent_inode_i: u16,
}
