use std::io::Write;

use crate::fs::core::*;

impl Fs {
    /// 格式化磁盘
    pub fn format() -> Result<Self> {
        let mut disk = std::fs::File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(DISK_PATH)?;
        let mut fs_desc = GroupDesc::new();

        for _ in 0..BLOCKS {
            disk.write_all(&utils::empty_blk())?;
        }

        let cwd = Self::init_root(&disk, &mut fs_desc)?;

        let mut fs = Self {
            fs_desc,
            cwd,
            disk,
            fds: Default::default(),
            opened_len: 0,
            user: 0,
        };

        fs.mkdir("/home")?;
        fs.mkdir("/root")?;

        Ok(fs)
    }

    /// 初始化根目录
    fn init_root(disk: &std::fs::File, fs_desc: &mut GroupDesc) -> Result<DirEntry> {
        // 初始化位图
        let mut blk = utils::empty_blk();
        blk[0] = 0b1000_0000;
        disk.write_at(&blk, fs_desc.inode_bitmap as u64 * BLOCK_SIZE as u64)?;
        disk.write_at(&blk, fs_desc.block_bitmap as u64 * BLOCK_SIZE as u64)?;

        // 写入根目录的索引节点
        let now = utils::now();
        let inode = Inode {
            i_mode: FileMode::new(0, FileType::Dir),
            i_blocks: 1,
            i_size: 2 * DIR_ENTRY_SIZE as u32,
            i_ctime: now,
            i_mtime: now,
            i_block: Default::default(),
            i_links_count: 1,
        };
        disk.write_at(
            (&inode).bytes(),
            fs_desc.inode_table as u64 * BLOCK_SIZE as u64,
        )?;

        // 将根目录的目录项写入对应的磁盘块
        let dir_entry = DirEntry {
            i_node: 0,
            rec_len: 0,
            name_len: 1,
            file_type: 2,
            name: ".".into_array()?,
        };
        disk.write_at(dir_entry.bytes(), Fs::addr_data_blk(0))?;
        disk.write_at(
            DirEntry {
                i_node: 0,
                rec_len: 0,
                name_len: 2,
                file_type: 2,
                name: "..".into_array()?,
            }
            .bytes(),
            Fs::addr_data_blk(0) + DIR_ENTRY_SIZE as u64,
        )?;

        fs_desc.free_blocks_count -= 1;
        fs_desc.free_inodes_count -= 1;
        fs_desc.used_dirs_count = 1;

        // 将更新后的 fs_desc 写回磁盘
        disk.write_at(fs_desc.bytes(), 0)?;

        Ok(dir_entry)
    }
}

#[test]
fn test_format() {
    assert!(Fs::format().is_ok())
}
