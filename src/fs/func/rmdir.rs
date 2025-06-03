use crate::utils::str;
use crate::fs::core::*;

impl Fs {
    // 删除当前目录下的空文件夹 `dir_name`
    pub fn rmdir(&mut self, path: &str) -> Result<()> {
        self.rmdir_internal(path, false)
    }

    // 递归删除目录（包括非空目录）
    pub fn rmdir_recursive(&mut self, path: &str) -> Result<()> {
        self.rmdir_internal(path, true)
    }

    // 内部实现，支持递归删除
    fn rmdir_internal(&mut self, path: &str, recursive: bool) -> Result<()> {
        // 寻找要删除的文件夹
        let item_to_delete = self.path_parse(path)?;
        let dir_entry = item_to_delete.dir_entry.clone();
        let dir_name = str(&dir_entry.name);

        let inode = Inode::from_disk(&self.disk, self.addr_i_node(dir_entry.i_node))?;

        // rm 需要对目录的写权限
        if !inode.i_mode.can_write(self.user) {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "Permission Denied",
            ));
        }

        // 跳过 "." 和 ".."
        if dir_name == "." || dir_name == ".." {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                format!(
                    "Persission Denied, can't delete . and .., skip delete {}",
                    str(&dir_entry.name),
                ),
            ));
        }

        if let FileType::File = dir_entry.file_type.into() {
            return Err(Error::new(
                ErrorKind::Other,
                "Not a directory",
            ));
        }

        // 检查目录是否为空（只包含 . 和 ..）
        let is_empty = inode.i_size as usize == 2 * DIR_ENTRY_SIZE;
        
        if !is_empty && !recursive {
            return Err(Error::new(
                ErrorKind::Other,
                "Directory is not empty",
            ));
        }

        // 如果是递归删除且目录不为空，先删除目录中的所有内容
        if !is_empty && recursive {
            self.clear_directory_contents(&dir_entry)?;
        }

        // 删除空目录
        self.delete_empty_directory(item_to_delete)
    }

    // 清空目录中的所有内容（递归删除子目录和文件）
    fn clear_directory_contents(&mut self, dir_entry: &DirEntry) -> Result<()> {
        // 保存当前工作目录
        let original_cwd = self.cwd.clone();
        
        // 切换到要清空的目录
        self.cwd = dir_entry.clone();

        // 收集所有需要删除的项目（避免在迭代时修改）
        let mut items_to_delete = Vec::new();
        
        for entry_item in dir_entry.iter_without_limit(self)? {
            if let iter::DirEntryIterItem::Using(item) = entry_item {
                let entry_name = str(&item.entry.name);
                // 跳过 "." 和 ".."
                if entry_name != "." && entry_name != ".." {
                    items_to_delete.push(item);
                }
            }
        }

        // 删除收集到的所有项目
        for item in items_to_delete {
            let entry_name = str(&item.entry.name);
            match item.entry.file_type.into() {
                FileType::File => {
                    // 递归删除文件
                    let fd = self.open(&entry_name)?;
                    self.rm(fd)?;
                }
                FileType::Dir => {
                    // 递归删除目录
                    self.rmdir_recursive(&entry_name)?;
                }
                FileType::Symlink => {
                    // 删除软链接
                    let fd = self.open(&entry_name)?;
                    self.rm(fd)?;
                }
            }
        }

        // 恢复原来的工作目录
        self.cwd = original_cwd;
        
        Ok(())
    }

    // 删除空目录的具体实现
    fn delete_empty_directory(&mut self, item_to_delete: PathParseRes) -> Result<()> {
        let mut dir_entry = item_to_delete.dir_entry.clone();
        let mut inode = Inode::from_disk(&self.disk, self.addr_i_node(dir_entry.i_node))?;

        // 删除所有数据块
        inode.free_data_block(0, self)?;

        // 删除索引节点
        self.free(BlkType::INode, &[dir_entry.i_node])?;

        // 从当前目录下的目录项里删除目录信息
        dir_entry.i_node = 0;
        self.disk
            .write_at(dir_entry.bytes(), item_to_delete.dir_entry_addr)?;

        // 更新父目录的信息
        let mut inode = self.get_inode(item_to_delete.parent_inode_i)?;
        inode.i_size -= DIR_ENTRY_SIZE as u32;
        self.write_inode(item_to_delete.parent_inode_i, inode)?;

        // 可用目录 + 1
        self.fs_desc.used_dirs_count -= 1;
        self.write_fs_desc()?;
        Ok(())
    }
}

#[test]
fn test_rmdir() {
    use iter::DirEntryIterItem;
    let mut fs = Fs::format().unwrap();
    fs.rmdir(".").expect_err("rmdir can't delete .");
    fs.rmdir(".").expect_err("rmdir can't delete ..");

    fs.mkdir("dir_a").unwrap();
    fs.mkdir("dir_b").unwrap();

    fs.chdir("dir_a").unwrap();
    fs.create("a.txt").unwrap();
    fs.rmdir("a.txt").expect_err("rmdir can't delete file");
    fs.chdir("..").unwrap();

    fs.rmdir("dir_a")
        .expect_err("rmdir only can delete empty directory");
    fs.rmdir("dir_b").unwrap();

    fs.chdir("dir_a").unwrap();
    let fd = fs.open("a.txt").unwrap();
    fs.chdir("..").unwrap();
    fs.rm(fd).unwrap();
    fs.read(fd, &mut [0u8; 1])
        .expect_err("file has been deleted!");

    fs.rmdir("dir_a").unwrap();

    for entry in fs.cwd.iter(&fs).unwrap() {
        match entry {
            DirEntryIterItem::Deleted(Item { entry, .. }) => {
                println!("- {}", std::str::from_utf8(&entry.name).unwrap())
            }
            DirEntryIterItem::Using(Item { entry, .. }) => {
                println!("{}", std::str::from_utf8(&entry.name).unwrap())
            }
        }
    }
}
