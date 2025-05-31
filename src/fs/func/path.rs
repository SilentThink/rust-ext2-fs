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
    pub fn path_parse(&self, path: &str) -> Result<PathParseRes> {
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
        for name in path.split('/').filter(|s| !s.is_empty()) {
            let mut founded = None;

            for item in dir_entry.iter_without_limit(self)? {
                if let DirEntryIterItem::Using(Item { entry, real_addr }) = item {
                    if entry.name == name.into_array()? {
                        founded = Some(entry);
                        dir_entry_addr = real_addr.addr;
                        parent_inode_i = dir_entry.i_node;
                        break;
                    }
                }
            }

            match founded {
                Some(entry) => dir_entry = entry,
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "File not found",
                    ))
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
