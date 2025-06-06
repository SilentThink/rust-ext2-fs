use crate::fs::core::*; // 引入文件系统核心模块，包含文件系统操作所需的类型和函数

impl Fs {
    /// 获取当前路径
    pub fn pwd(&self) -> String {
        self.get_current_dir() // 调用私有方法获取当前工作目录的路径
    }

    /// 获取当前工作目录的路径
    pub fn get_current_dir(&self) -> String {
        // 如果是根目录，直接返回 "/"
        if self.cwd.i_node == 0 {
            return "/".to_string();
        }

        // 使用递归方法获取路径
        self.get_path_by_inode(self.cwd.i_node) // 根据当前工作目录的inode号获取路径
    }

    /// 根据inode号获取文件路径
    fn get_path_by_inode(&self, inode_i: u16) -> String {
        if inode_i == 0 {
            return "/".to_string(); // 如果inode号为0，表示根目录
        }

        // 获取当前目录项
        let entry = match self.get_entry_by_inode(inode_i) {
            Ok(entry) => entry, // 成功获取目录项
            Err(_) => return "/".to_string(), // 如果出错，返回根目录路径
        };

        // 获取父目录的inode号
        let parent_inode_i = match self.get_parent_inode(inode_i) {
            Ok(parent) => parent, // 成功获取父目录的inode号
            Err(_) => return "/".to_string(), // 如果出错，返回根目录路径
        };

        // 递归获取父目录的路径
        let parent_path = self.get_path_by_inode(parent_inode_i);
        let name = utils::str(&entry.name); // 获取目录项的名称

        // 拼接路径
        if parent_path == "/" {
            format!("/{}", name) // 如果父路径是根目录，直接拼接
        } else {
            format!("{}/{}", parent_path, name) // 否则在父路径后拼接当前目录名
        }
    }

    /// 获取指定inode的父目录inode号
    fn get_parent_inode(&self, inode_i: u16) -> Result<u16> {
        // 获取inode
        let inode = self.get_inode(inode_i)?; // 获取指定inode号的inode信息

        // 只有目录才有父目录
        if inode.i_blocks == 0 || inode.i_mode.mode & 0b00_000_001 == 0 {
            return Err(Error::new(ErrorKind::Other, "Not a directory")); // 如果不是目录，返回错误
        }

        // 读取目录的第二个目录项（".."）
        let parent_entry = DirEntry::from_disk(
            &self.disk,
            Self::addr_data_blk(inode.i_block[0]) + DIR_ENTRY_SIZE as u64,
        )?;

        Ok(parent_entry.i_node) // 返回父目录的inode号
    }

    /// 根据inode号获取目录项
    fn get_entry_by_inode(&self, inode_i: u16) -> Result<DirEntry> {
        // 从根目录开始查找
        let root_entry = DirEntry::from_disk(&self.disk, Self::addr_data_blk(0))?; // 获取根目录的目录项
        self.find_entry_by_inode(root_entry, inode_i) // 从根目录开始递归查找指定inode的目录项
    }

    /// 在指定目录中查找inode对应的目录项
    fn find_entry_by_inode(&self, dir_entry: DirEntry, inode_i: u16) -> Result<DirEntry> {
        // 如果当前目录就是要找的inode
        if dir_entry.i_node == inode_i {
            return Ok(dir_entry); // 直接返回当前目录项
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
                    return Ok(entry); // 返回找到的目录项
                }

                // 如果是目录，递归查找
                let dir_type: u8 = FileType::Dir.into();
                if entry.file_type == dir_type {
                    match self.find_entry_by_inode(entry, inode_i) {
                        Ok(found) => return Ok(found), // 如果在子目录中找到，返回结果
                        Err(_) => continue, // 如果子目录中未找到，继续查找
                    }
                }
            }
        }

        Err(Error::new(ErrorKind::NotFound, "Entry not found")) // 如果遍历完所有目录仍未找到，返回错误
    }
}

#[test]
fn test_pwd() {
    let mut fs = Fs::format().unwrap(); // 格式化文件系统
    fs.mkdir("a").unwrap(); // 创建目录 "a"
    fs.chdir("a").unwrap(); // 切换到目录 "a"
    fs.mkdir("b").unwrap(); // 创建目录 "b"
    fs.chdir("b").unwrap(); // 切换到目录 "b"
    fs.mkdir("c").unwrap(); // 创建目录 "c"
    fs.chdir("c").unwrap(); // 切换到目录 "c"
    assert_eq!(fs.pwd(), "/a/b/c") // 测试当前路径是否为 "/a/b/c"
}