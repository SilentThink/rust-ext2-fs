//! # 硬链接
//!
//! 这个文件为文件系统 [`Fs`] 扩充了硬链接功能：
//!
//! 1. [`Fs::link`]：创建指向已有文件的硬链接

use crate::fs::core::*;

impl Fs {
    /// 创建硬链接
    /// 
    /// 参数:
    /// - `target`: 目标文件路径
    /// - `link_name`: 硬链接的名称
    /// 
    /// 返回:
    /// - `Result<()>`: 操作结果
    pub fn link(&mut self, target: &str, link_name: &str) -> Result<()> {
        // 解析目标文件路径
        let target_path_res = self.path_parse(target)?;
        let target_dir_entry = target_path_res.dir_entry.clone();
        
        // 检查目标是否为文件（硬链接只能指向文件，不能指向目录）
        if let FileType::Dir = target_dir_entry.file_type.into() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Hard links to directories are not allowed",
            ));
        }
        
        // 解析硬链接路径
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
        
        // 创建新的目录项，指向同一个inode
        let dir_entry = DirEntry {
            i_node: target_dir_entry.i_node, // 使用相同的inode
            name: link_file_name.into_array()?,
            name_len: link_file_name.len() as u8,
            file_type: target_dir_entry.file_type, // 保持相同的文件类型
            rec_len: 1,
        };
        
        // 增加目标文件的硬链接计数
        let mut target_inode = self.get_inode(target_dir_entry.i_node)?;
        target_inode.i_links_count += 1;
        self.write_inode(target_dir_entry.i_node, target_inode)?;
        
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
}

#[test]
fn test_link() {
    // 测试内容
    let content = "Hello, hard link!";
    let modified = "Modified content!";
    
    // 创建文件系统
    let mut fs = Fs::format().unwrap();
    
    // 创建测试文件并写入内容
    fs.create("test_file.txt").unwrap();
    let fd = fs.open("test_file.txt").unwrap();
    fs.write(fd, content.as_bytes()).unwrap();
    fs.close(fd).unwrap();
    
    // 创建硬链接
    fs.link("test_file.txt", "hard_link.txt").unwrap();
    
    // 验证两个文件指向同一个inode
    let original_path = fs.path_parse("test_file.txt").unwrap();
    let link_path = fs.path_parse("hard_link.txt").unwrap();
    assert_eq!(original_path.dir_entry.i_node, link_path.dir_entry.i_node);
    
    // 通过硬链接读取内容
    let fd = fs.open("hard_link.txt").unwrap();
    let mut buffer = Vec::new();
    let mut buf = [0u8; 10];
    while fs.read(fd, &mut buf).unwrap() != 0 {
        buffer.extend_from_slice(&buf);
        buf.fill(0);
    }
    fs.close(fd).unwrap();
    
    // 验证内容正确
    let read_content = std::str::from_utf8(&buffer).unwrap_or("invalid utf-8").trim_matches('\0');
    assert_eq!(read_content, content);
    
    // 通过硬链接修改内容
    let fd = fs.open("hard_link.txt").unwrap();
    fs.write(fd, modified.as_bytes()).unwrap();
    fs.close(fd).unwrap();
    
    // 通过原始文件读取修改后的内容
    let fd = fs.open("test_file.txt").unwrap();
    let mut buffer = Vec::new();
    let mut buf = [0u8; 10];
    while fs.read(fd, &mut buf).unwrap() != 0 {
        buffer.extend_from_slice(&buf);
        buf.fill(0);
    }
    fs.close(fd).unwrap();
    
    // 验证修改后的内容正确
    let read_content = std::str::from_utf8(&buffer).unwrap_or("invalid utf-8").trim_matches('\0');
    assert_eq!(read_content, modified);
    
    // 测试删除原始文件后，硬链接仍然可以访问内容
    let fd = fs.open("test_file.txt").unwrap();
    fs.rm(fd).unwrap();
    
    // 尝试打开原文件应该失败
    assert!(fs.open("test_file.txt").is_err());
    
    // 通过硬链接仍然可以访问内容
    let fd = fs.open("hard_link.txt").unwrap();
    let mut buffer = Vec::new();
    let mut buf = [0u8; 10];
    while fs.read(fd, &mut buf).unwrap() != 0 {
        buffer.extend_from_slice(&buf);
        buf.fill(0);
    }
    fs.close(fd).unwrap();
    
    // 验证内容仍然正确
    let read_content = std::str::from_utf8(&buffer).unwrap_or("invalid utf-8").trim_matches('\0');
    assert_eq!(read_content, modified);
} 