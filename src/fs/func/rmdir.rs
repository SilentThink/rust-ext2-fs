use crate::utils::str;
use crate::fs::core::*;

impl Fs {
    // 删除当前目录下的空文件夹 `dir_name`
    pub fn rmdir(&mut self, path: &str) -> Result<()> {
        // 寻找要删除的文件夹
        let item_to_delete = self.path_parse(path)?;
        let mut dir_entry = item_to_delete.dir_entry.clone();
        let dir_name = str(&dir_entry.name);

        let mut inode = Inode::from_disk(&self.disk, self.addr_i_node(dir_entry.i_node))?;

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

        if inode.i_size as usize != 2 * DIR_ENTRY_SIZE {
            return Err(Error::new(
                ErrorKind::Other,
                "Directory is not empty",
            ));
        }

        // 是空目录
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
