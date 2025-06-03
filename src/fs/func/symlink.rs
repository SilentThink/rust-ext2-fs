//! # 软链接
//!
//! 这个文件为文件系统 [`Fs`] 扩充了软链接功能：
//!
//! 1. [`Fs::symlink`]：创建指向目标路径的软链接

use crate::fs::core::*;

impl Fs {
    /// 创建软链接
    /// 
    /// 参数:
    /// - `target`: 目标路径
    /// - `link_name`: 软链接的名称
    /// 
    /// 返回:
    /// - `Result<()>`: 操作结果
    pub fn symlink(&mut self, target: &str, link_name: &str) -> Result<()> {
        // 解析软链接路径
        let (link_dir_path, link_file_name) = link_name.rsplit_once("/").unwrap_or((".", link_name));
        let link_dir_res = self.path_parse(link_dir_path)?;
        let link_dir_entry = link_dir_res.dir_entry.clone();
        
        // 检查对链接目录的写权限
        let link_dir_inode = self.get_inode(link_dir_entry.i_node)?;
        if !link_dir_inode.i_mode.can_write(self.user) {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "Need write permission to directory",
            ));
        }
        
        // 检查链接名是否已存在
        for iter_item in link_dir_entry.iter(self)? {
            if let DirEntryIterItem::Using(Item { entry, .. }) = iter_item {
                if entry.name == link_file_name.into_array()? {
                    return Err(Error::new(
                        ErrorKind::AlreadyExists,
                        "Link name already exists",
                    ));
                }
            }
        }
        
        // 查找可能被删除的目录项位置
        let mut deleted_entry_addr = None;
        for iter_item in link_dir_entry.iter_without_limit(self)? {
            if let DirEntryIterItem::Deleted(Item { real_addr, .. }) = iter_item {
                deleted_entry_addr = Some(real_addr);
                break;
            }
        }
        
        // 分配索引节点
        let inode_i = self.alloc(BlkType::INode)?;
        
        // 创建软链接的inode
        let mut inode = Inode {
            i_mode: FileMode::new(self.user, FileType::Symlink),
            ..Default::default()
        };
        
        // 将目标路径写入软链接的数据块
        let data_blk_i = inode.alloc_data_block(self)?;
        let target_bytes = target.as_bytes();
        self.disk.write_at(target_bytes, Fs::addr_data_blk(data_blk_i))?;
        
        // 更新inode大小为目标路径的长度
        inode.i_size = target_bytes.len() as u32;
        self.write_inode(inode_i, inode)?;
        
        // 创建新的目录项
        let dir_entry = DirEntry {
            i_node: inode_i,
            name: link_file_name.into_array()?,
            name_len: link_file_name.len() as u8,
            file_type: FileType::Symlink.into(),
            rec_len: 1,
        };
        
        // 将新的目录项写入磁盘
        if let Some(addr) = deleted_entry_addr {
            // 使用之前删除的目录项的位置
            self.disk.write_at(dir_entry.bytes(), addr.addr)?;
        } else {
            // 在目录末尾添加新的目录项
            let mut link_dir_inode = self.get_inode(link_dir_entry.i_node)?;
            
            // 检查是否需要分配新的数据块
            let logic_addr = link_dir_inode.i_size as u64;
            let blk_i = logic_addr / BLOCK_SIZE as u64;
            
            if blk_i >= link_dir_inode.i_blocks as u64 {
                // 需要分配新的数据块
                link_dir_inode.alloc_data_block(self)?;
            }
            
            // 计算新目录项的物理地址
            let real_addr = link_dir_inode.convert_addr(&self.disk, logic_addr)?;
            
            // 写入新的目录项
            self.disk.write_at(dir_entry.bytes(), real_addr.addr)?;
            
            // 更新目录的大小
            link_dir_inode.i_size += DIR_ENTRY_SIZE as u32;
            link_dir_inode.i_mtime = utils::now();
            self.write_inode(link_dir_entry.i_node, link_dir_inode)?;
        }
        
        Ok(())
    }

    /// 读取软链接的目标路径
    pub fn read_symlink_target(&self, symlink_path: &str) -> Result<String> {
        // 解析软链接路径，但不跟随链接
        let path_res = self.path_parse_with_options(symlink_path, false)?;
        let entry = path_res.dir_entry.clone();
        
        // 检查是否为软链接
        let symlink_type: u8 = FileType::Symlink.into();
        if entry.file_type != symlink_type {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Not a symbolic link",
            ));
        }
        
        // 读取软链接的目标路径
        let symlink_inode = self.get_inode(entry.i_node)?;
        let mut target_path = vec![0u8; symlink_inode.i_size as usize];
        
        // 读取软链接数据块中存储的目标路径
        if symlink_inode.i_blocks > 0 {
            let data_blk = symlink_inode.i_block[0];
            self.disk.read_at(&mut target_path, Self::addr_data_blk(data_blk))?;
        }
        
        String::from_utf8(target_path)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid symlink target"))
    }
}

#[test]
fn test_symlink() {
    // 测试内容
    let content = "Hello, symlink!";
    
    // 创建文件系统
    let mut fs = Fs::format().unwrap();
    
    // 创建测试目录和文件
    fs.mkdir("test_dir").unwrap();
    fs.create("test_file.txt").unwrap();
    let fd = fs.open("test_file.txt").unwrap();
    fs.write(fd, content.as_bytes()).unwrap();
    fs.close(fd).unwrap();
    
    // 创建软链接到文件
    fs.symlink("test_file.txt", "file_link").unwrap();
    
    // 创建软链接到目录
    fs.symlink("test_dir", "dir_link").unwrap();
    
    // 通过软链接读取文件内容
    let fd = fs.open("file_link").unwrap();
    let mut buffer = Vec::new();
    let mut buf = [0u8; 20];
    let n = fs.read(fd, &mut buf).unwrap();
    buffer.extend_from_slice(&buf[0..n]);
    fs.close(fd).unwrap();
    
    // 验证内容正确
    let read_content = std::str::from_utf8(&buffer).unwrap();
    assert_eq!(read_content, content);
    
    // 测试删除原始文件后，软链接失效
    let fd = fs.open("test_file.txt").unwrap();
    fs.rm(fd).unwrap();
    
    // 尝试打开软链接应该失败（因为目标不存在）
    assert!(fs.open("file_link").is_err());
} 