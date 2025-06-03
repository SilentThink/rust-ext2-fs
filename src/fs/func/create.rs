//! # 创建文件和目录
//!
//! 这个文件为文件系统 [`Fs`] 扩充了两个方法：
//!
//! 1. [`Fs::create`]：在当前目录下创建新文件
//! 2. [`Fs::mkdir`]：在当前目录下创建新目录

use crate::fs::core::*;

impl Fs {
    /// 在当前目录下创建目录，`name` 为目录名
    pub fn mkdir(&mut self, name: &str) -> Result<()> {
        self._create(name, FileType::Dir)
    }

    /// 在当前目录下创建新文件
    pub fn create(&mut self, name: &str) -> Result<()> {
        self._create(name, FileType::File)
    }

    /// 创建文件或者目录
    fn _create(&mut self, path: &str, mode: FileType) -> Result<()> {
        let mut deleted_entry_addr = None;

        let (path, name) = path.rsplit_once("/").unwrap_or((".", path));
        let parent_dir_entry = self.path_parse(path)?.dir_entry;

        // 检查对父目录的写权限
        let mut parent_inode = self.get_inode(parent_dir_entry.i_node)?;
        if !parent_inode.i_mode.can_write(self.user) {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "Need write permission to directory",
            ));
        }

        // 遍历当前目录项下的所有文件
        for iter_item in parent_dir_entry.iter(&self)? {
            match iter_item {
                DirEntryIterItem::Using(Item { entry, .. }) => {
                    // 检查同名文件
                    if entry.name == name.into_array()? {
                        return Err(Error::new(
                            ErrorKind::AlreadyExists,
                            "Files has exists",
                        ));
                    }
                }
                DirEntryIterItem::Deleted(Item { real_addr, .. }) => {
                    // 如果找到之前被删除的项，先将它的地址记下来
                    // 后面直接用新的目录项代替
                    deleted_entry_addr = Some(real_addr)
                }
            }
        }

        // 分配索引节点
        let inode_i = self.alloc(BlkType::INode)?;

        let inode = match mode {
            FileType::File => Inode {
                i_mode: FileMode::new(self.user, mode),
                ..Default::default()
            },
            FileType::Dir => {
                // 填充索引节点的内容
                let mut inode = Inode {
                    i_mode: FileMode::new(self.user, mode),
                    i_size: DIR_ENTRY_SIZE as u32 * 2,
                    ..Default::default()
                };

                let data_blk_i = inode.alloc_data_block(self)?;

                // 将 . 和 .. 写入数据块
                self.disk.write_at(
                    DirEntry {
                        i_node: inode_i,
                        rec_len: 0,
                        name_len: 1,
                        file_type: 2,
                        name: ".".into_array()?,
                    }
                    .bytes(),
                    Fs::addr_data_blk(data_blk_i),
                )?;
                self.disk.write_at(
                    DirEntry {
                        i_node: parent_dir_entry.i_node,
                        rec_len: 0,
                        name_len: 2,
                        file_type: 2,
                        name: "..".into_array()?,
                    }
                    .bytes(),
                    Fs::addr_data_blk(data_blk_i) + DIR_ENTRY_SIZE as u64,
                )?;

                inode
            }
            FileType::Symlink => {
                // 软链接的创建在symlink.rs中单独实现
                // 这里不应该被调用
                return Err(Error::new(
                    ErrorKind::Other,
                    "Symlinks should be created using symlink() function",
                ));
            }
        };

        // 保存索引节点
        self.write_inode(inode_i, inode)?;

        // 给当前目录添加 DirEntry
        let dir_entry = DirEntry {
            i_node: inode_i,
            name: name.into_array()?,
            name_len: name.len() as u8,
            file_type: mode.into(),
            rec_len: 1,
        };

        // 如果已经有空位，那就直接将 DirEntry 插入空位
        if let Some(addr) = deleted_entry_addr {
            self.disk.write_at(dir_entry.bytes(), addr.addr)?;
        } else {
            // 没有空位，就只能将 DirEntry 写入新的位置
            let addr = if parent_inode.i_size % BLOCK_SIZE as u32 == 0 {
                // 需要请求新的数据块
                let blk = parent_inode.alloc_data_block(self)?;
                Fs::addr_data_blk(blk)
            } else {
                parent_inode
                    .convert_addr(&mut self.disk, parent_inode.i_size as u64)?
                    .addr
            };
            self.disk.write_at(dir_entry.bytes(), addr)?;
        }

        // 更新当前目录索引节点信息
        parent_inode.i_size += DIR_ENTRY_SIZE as u32;
        if let FileType::Dir = mode {
            self.fs_desc.used_dirs_count += 1;
        }

        // 同步磁盘
        self.write_inode(parent_dir_entry.i_node, parent_inode)?;
        self.write_fs_desc()?;

        Ok(())
    }
}

#[test]
fn mkdir_test() {
    let mut fs = Fs::format().unwrap();
    assert!(fs.mkdir(".").is_err());
    assert!(fs.mkdir("..").is_err());
    println!("{:?}", fs.mkdir("hello"));
    println!("{:?}", fs.mkdir("world"));
    println!("{:?}", fs.mkdir("hello"));
    println!("{:?}", fs.create("hello"));
    println!("{:?}", fs.create("new_file"));
}
