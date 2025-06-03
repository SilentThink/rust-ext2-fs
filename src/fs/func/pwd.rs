use crate::fs::core::*;

impl Fs {
    /// 获取当前路径
    pub fn pwd(&self) -> String {
        self.get_current_dir()
    }

    /// 获取当前工作目录的路径
    pub fn get_current_dir(&self) -> String {
        // 如果是根目录，直接返回 "/"
        if self.cwd.i_node == 0 {
            return "/".to_string();
        }

        // 使用递归方法获取路径
        self.get_path_by_inode(self.cwd.i_node)
    }

    /// 根据inode号获取文件路径
    fn get_path_by_inode(&self, inode_i: u16) -> String {
        if inode_i == 0 {
            return "/".to_string();
        }

        // 获取当前目录项
        let entry = match self.get_entry_by_inode(inode_i) {
            Ok(entry) => entry,
            Err(_) => return "/".to_string(),
        };

        // 获取父目录的inode号
        let parent_inode_i = match self.get_parent_inode(inode_i) {
            Ok(parent) => parent,
            Err(_) => return "/".to_string(),
        };

        // 递归获取父目录的路径
        let parent_path = self.get_path_by_inode(parent_inode_i);
        let name = utils::str(&entry.name);

        // 拼接路径
        if parent_path == "/" {
            format!("/{}", name)
        } else {
            format!("{}/{}", parent_path, name)
        }
    }

    /// 获取指定inode的父目录inode号
    fn get_parent_inode(&self, inode_i: u16) -> Result<u16> {
        // 获取inode
        let inode = self.get_inode(inode_i)?;
        
        // 只有目录才有父目录
        if inode.i_blocks == 0 || inode.i_mode.mode & 0b00_000_001 == 0 {
            return Err(Error::new(ErrorKind::Other, "Not a directory"));
        }

        // 读取目录的第二个目录项（".."）
        let parent_entry = DirEntry::from_disk(
            &self.disk,
            Self::addr_data_blk(inode.i_block[0]) + DIR_ENTRY_SIZE as u64,
        )?;

        Ok(parent_entry.i_node)
    }

    /// 根据inode号获取目录项
    fn get_entry_by_inode(&self, inode_i: u16) -> Result<DirEntry> {
        // 从根目录开始查找
        let root_entry = DirEntry::from_disk(&self.disk, Self::addr_data_blk(0))?;
        self.find_entry_by_inode(root_entry, inode_i)
    }

    /// 在指定目录中查找inode对应的目录项
    fn find_entry_by_inode(&self, dir_entry: DirEntry, inode_i: u16) -> Result<DirEntry> {
        // 如果当前目录就是要找的inode
        if dir_entry.i_node == inode_i {
            return Ok(dir_entry);
        }

        // 遍历目录下的所有项
        for item in dir_entry.iter(self)? {
            if let DirEntryIterItem::Using(Item { entry, .. }) = item {
                // 跳过. 和 ..
                let name = utils::str(&entry.name);
                if name == "." || name == ".." {
                    continue;
                }

                // 如果找到了匹配的inode
                if entry.i_node == inode_i {
                    return Ok(entry);
                }

                // 如果是目录，递归查找
                let dir_type: u8 = FileType::Dir.into();
                if entry.file_type == dir_type {
                    match self.find_entry_by_inode(entry, inode_i) {
                        Ok(found) => return Ok(found),
                        Err(_) => continue,
                    }
                }
            }
        }

        Err(Error::new(ErrorKind::NotFound, "Entry not found"))
    }
}

#[test]
fn test_pwd() {
    let mut fs = Fs::format().unwrap();
    fs.mkdir("a").unwrap();
    fs.chdir("a").unwrap();
    fs.mkdir("b").unwrap();
    fs.chdir("b").unwrap();
    fs.mkdir("c").unwrap();
    fs.chdir("c").unwrap();
    assert_eq!(fs.pwd(), "/a/b/c")
}
