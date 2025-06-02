use crate::fs::core::*;

impl Fs {
    /// 将文件的长度裁剪到 `new_len` 字节，并清空 `new_len` 之后的数据块
    /// 如果 `new_len` 等于文件长度，则这个函数不起作用
    pub fn cut(&mut self, fd: usize, new_len: u64) -> Result<()> {
        if fd >= self.fds.len() || self.fds[fd].is_none() {
            return Err(Error::new(ErrorKind::Other, "Bad file description"));
        }

        // 计算 new_len 字节占用的块数
        let new_blk_counts = match new_len % BLOCK_SIZE as u64 {
            0 => new_len / BLOCK_SIZE as u64,
            _ => new_len / BLOCK_SIZE as u64 + 1,
        } as u16;

        let mut file = self.fds[fd].clone().unwrap();

        // 需要写权限
        if !file.inode.i_mode.can_write(self.user) {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "Need write permission",
            ));
        }

        if new_blk_counts >= file.inode.i_blocks {
            return Ok(());
        }

        // 删除多余的数据块
        file.inode.free_data_block(new_blk_counts, self)?;

        // 完成文件大小的剪裁，更新索引节点
        file.inode.i_mtime = utils::now();
        file.inode.i_size = new_len as u32;

        self.fds[fd] = Some(file);
        Ok(())
    }

    // 删除文件
    pub fn rm(&mut self, fd: usize) -> Result<()> {
        if fd >= self.fds.len() || self.fds[fd].is_none() {
            return Err(Error::new(ErrorKind::Other, "Bad file description"));
        }

        let mut file = self.fds[fd].clone().unwrap();

        // rm 需要对目录的写权限
        if !file.inode.i_mode.can_write(self.user) {
            return Err(Error::new(ErrorKind::PermissionDenied, "Permission Denied"));
        }

        // 减少硬链接计数
        file.inode.i_links_count -= 1;
        
        // 如果还有其他硬链接引用，只更新inode并删除当前目录项
        if file.inode.i_links_count > 0 {
            // 更新inode
            self.write_inode(file.inode_i, file.inode)?;
            
            // 删除当前目录项
            let mut dir_entry = DirEntry::from_disk(&self.disk, file.dir_entry_addr)?;
            dir_entry.i_node = 0;
            dir_entry.rec_len = 1;
            self.disk.write_at(dir_entry.bytes(), file.dir_entry_addr)?;
            
            // 更新父目录inode
            let mut cwd_inode = Inode::from_disk(&self.disk, self.addr_i_node(file.parent_inode_i))?;
            cwd_inode.i_size -= DIR_ENTRY_SIZE as u32;
            cwd_inode.i_mtime = utils::now();
            self.write_inode(file.parent_inode_i, cwd_inode)?;
            
            // 回收文件描述符
            self.fds[fd] = None;
            self.opened_len -= 1;
            
            return Ok(());
        }

        // 如果没有其他硬链接引用，清空文件存储的数据块
        self.cut(fd, 0)?;

        // 删除文件的索引节点
        self.free(BlkType::INode, &[file.inode_i])?;

        // 编辑目录项
        let mut dir_entry = DirEntry::from_disk(&self.disk, file.dir_entry_addr)?;
        dir_entry.i_node = 0;
        dir_entry.rec_len = 1;
        self.disk.write_at(dir_entry.bytes(), file.dir_entry_addr)?;

        let mut cwd_inode = Inode::from_disk(&self.disk, self.addr_i_node(file.parent_inode_i))?;
        cwd_inode.i_size -= DIR_ENTRY_SIZE as u32;
        cwd_inode.i_mtime = utils::now();
        self.write_inode(file.parent_inode_i, cwd_inode)?;

        // 回收文件描述符
        self.fds[fd] = None;
        self.opened_len -= 1;

        Ok(())
    }
}

#[test]
fn test_rm_file() {
    use crate::fs::core::Item;

    let mut fs = Fs::format().unwrap();
    fs.create("1.txt").unwrap();
    fs.create("2.txt").unwrap();
    fs.mkdir("dir").unwrap();

    let fd = fs.open("1.txt").unwrap();
    fs.write(fd, b"hello world").unwrap();
    fs.rm(fd).unwrap();
    let fd = fs.open("2.txt").unwrap();
    fs.rm(fd).unwrap();
    fs.mkdir("hello").unwrap();

    let mut len = 0;
    for entry in fs.cwd.iter(&fs).unwrap() {
        len += 1;
        match entry {
            iter::DirEntryIterItem::Using(Item { entry, .. }) => {
                println!("{}", std::str::from_utf8(&entry.name).unwrap())
            }
            iter::DirEntryIterItem::Deleted(Item { entry, .. }) => {
                println!("- {}", std::str::from_utf8(&entry.name).unwrap())
            }
        }
    }

    assert_eq!(len, 7);
}
