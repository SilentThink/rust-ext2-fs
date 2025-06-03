use crate::fs::core::*;

impl Fs {
    /// 路径解析器，用来支持其他函数，如 `[Self::open]`，`[Self::create]` 等函数就可以使用
    /// 这个函数进行路径解析
    ///
    /// `path` 为路径名。类 Unix 系统里有两种路径格式，绝对路径和相对路径：
    ///
    /// - 绝对路径是指以 `/` 开头的路径，比如如 `/etc/passwd` 表示的是根目录下的 `etc`
    ///   文件夹内的 `passwd` 文件
    /// - 相对路径就是不以 `/` 开头的路径，如 `../hello` 表示的是工作目录的上一级目录下
    ///   的 hello 文件
    ///
    /// 如果路径有效，就返回路径对应的 [`PathParserRes`]
    /// 
    /// `follow_symlinks` 参数控制是否解析软链接，默认为true
    pub fn path_parse(&self, path: &str) -> Result<PathParseRes> {
        self.path_parse_with_options(path, true)
    }

    /// 路径解析器的内部实现，带有选项控制是否解析软链接
    pub fn path_parse_with_options(&self, path: &str, follow_symlinks: bool) -> Result<PathParseRes> {
        let mut path = String::from(path);
        // 确定开始查找的目录
        let mut dir_entry = if path.starts_with("/") {
            // 返回根节点的 DirEntry
            path.remove(0);
            DirEntry::from_disk(&self.disk, Self::addr_data_blk(0))?
        } else {
            // 返回工作目录的 DirEntry
            self.cwd.clone()
        };

        let inode = self.get_inode(dir_entry.i_node)?;

        let mut dir_entry_addr = Self::addr_data_blk(inode.i_block[0]);
        let mut parent_inode_i = DirEntry::from_disk(
            &self.disk,
            Self::addr_data_blk(inode.i_block[0]) + DIR_ENTRY_SIZE as u64,
        )?
        .i_node;

        // 用 `/` 分隔路径名，开始查找
        for name in path.split("/") {
            if name.is_empty() {
                continue;
            }

            match name {
                "." => continue,
                ".." => {
                    // 返回父目录
                    let parent_inode = self.get_inode(parent_inode_i)?;
                    dir_entry_addr = Self::addr_data_blk(parent_inode.i_block[0]);
                    dir_entry = DirEntry::from_disk(&self.disk, dir_entry_addr)?;

                    parent_inode_i = DirEntry::from_disk(
                        &self.disk,
                        Self::addr_data_blk(parent_inode.i_block[0]) + DIR_ENTRY_SIZE as u64,
                    )?
                    .i_node;
                }
                _ => {
                    // 检查当前目录的执行权限
                    let inode = self.get_inode(dir_entry.i_node)?;
                    if !inode.i_mode.can_exec(self.user) {
                        return Err(Error::new(
                            ErrorKind::PermissionDenied,
                            "Permission Denied. Need exec permission.",
                        ));
                    }

                    // 在当前目录下查找名为 name 的文件
                    let mut found = false;
                    let mut found_entry = dir_entry.clone();
                    let mut found_entry_addr = dir_entry_addr;
                    let mut found_parent_inode_i = parent_inode_i;

                    for item in dir_entry.iter(self)? {
                        if let DirEntryIterItem::Using(Item { entry, real_addr }) = item {
                            if entry.name == name.into_array()? {
                                found = true;
                                found_entry = entry;
                                found_entry_addr = real_addr.addr;
                                found_parent_inode_i = dir_entry.i_node;
                                break;
                            }
                        }
                    }

                    if !found {
                        return Err(Error::new(
                            ErrorKind::NotFound,
                            format!("{}: No such file or directory", name),
                        ));
                    }

                    // 处理软链接
                    let symlink_type: u8 = FileType::Symlink.into();
                    if follow_symlinks && found_entry.file_type == symlink_type {
                        // 读取软链接的目标路径
                        let symlink_inode = self.get_inode(found_entry.i_node)?;
                        let mut target_path = vec![0u8; symlink_inode.i_size as usize];
                        
                        // 读取软链接数据块中存储的目标路径
                        if symlink_inode.i_blocks > 0 {
                            let data_blk = symlink_inode.i_block[0];
                            self.disk.read_at(&mut target_path, Self::addr_data_blk(data_blk))?;
                        }
                        
                        let target = String::from_utf8(target_path)
                            .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid symlink target"))?;
                        
                        // 递归解析目标路径
                        // 如果目标路径是绝对路径，从根目录开始解析
                        // 如果是相对路径，从当前目录开始解析
                        let resolved_path = if target.starts_with("/") {
                            // 绝对路径
                            target
                        } else {
                            // 相对路径，需要拼接当前路径
                            let current_dir = self.get_current_dir();
                            if current_dir == "/" {
                                format!("/{}", target)
                            } else {
                                format!("{}/{}", current_dir, target)
                            }
                        };
                        
                        // 如果这是路径的最后一个组件，直接解析整个目标路径
                        // 否则，需要继续解析剩余的路径组件
                        let remaining_path = path.split("/").skip_while(|&p| p != name).skip(1).collect::<Vec<_>>();
                        if remaining_path.is_empty() {
                            // 这是最后一个组件，直接解析整个目标
                            return self.path_parse_with_options(&resolved_path, follow_symlinks);
                        } else {
                            // 还有更多组件，先解析目标，然后继续处理剩余路径
                            let target_res = self.path_parse_with_options(&resolved_path, follow_symlinks)?;
                            dir_entry = target_res.dir_entry;
                            dir_entry_addr = target_res.dir_entry_addr;
                            parent_inode_i = target_res.parent_inode_i;
                            continue;
                        }
                    }

                    // 更新当前目录
                    dir_entry = found_entry;
                    dir_entry_addr = found_entry_addr;
                    parent_inode_i = found_parent_inode_i;
                }
            }
        }

        Ok(PathParseRes {
            dir_entry,
            dir_entry_addr,
            parent_inode_i,
        })
    }
}

#[test]
fn test_path_parse() -> Result<()> {
    let mut fs = Fs::format()?;

    fs.mkdir("a")?;
    fs.mkdir("b")?;
    fs.create("c")?;

    fs.chdir("a")?;
    fs.mkdir("b")?;
    fs.chdir("b")?;

    fs.path_parse("/")?;
    fs.path_parse("/../../../../")?;
    fs.path_parse("../a").expect_err("err");
    fs.path_parse("../b")?;
    fs.path_parse("../c").expect_err("should return err");

    fs.path_parse("../../b")?;
    fs.path_parse("../../a/b")?;

    fs.path_parse("/c/d/e/f").expect_err("err path");
    fs.path_parse("/a/ddddddd/fffff").expect_err("not founded");
    assert_eq!(fs.path_parse("/c")?.dir_entry.name, "c".into_array()?);
    assert_eq!(fs.path_parse("../../c")?.dir_entry.name, "c".into_array()?);
    fs.path_parse("../../c/d/e/f").expect_err("not founded");
    Ok(())
}
