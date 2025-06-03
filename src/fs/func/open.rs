use crate::fs::core::*;

impl Fs {
    /// 用来打开路径名为 `path` 的文件
    ///
    /// 当文件不存在、或者同时打开的文件数超过 [`FD_LIMIT`]、或者尝试打开一个目录时，文件打开
    /// 失败
    /// ```
    pub fn open(&mut self, path: &str) -> Result<usize> {
        // 当前打开的文件已经到达上限
        if self.opened_len == FD_LIMIT {
            return Err(Error::new(
                ErrorKind::Other,
                "Files decriptions up to limits",
            ));
        }

        // 使用path_parse_with_options函数解析路径，并设置follow_symlinks为true
        let path = self.path_parse(path)?;

        // 不能打开目录
        if let FileType::Dir = path.dir_entry.file_type.into() {
            return Err(Error::new(
                ErrorKind::Other,
                "Can't open directory as file",
            ));
        }

        // 文件的索引节点
        let inode = Inode::from_disk(&self.disk, self.addr_i_node(path.dir_entry.i_node))?;

        // open 需要文件的读权限
        if !inode.i_mode.can_read(self.user) {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "Permission Denied. Need read permission.",
            ));
        }

        // 分配文件描述符
        let mut fd = 0;
        for i in 0..self.fds.len() {
            if self.fds[i].is_none() {
                fd = i;
                break;
            }
        }

        self.fds[fd] = Some(File {
            inode,
            inode_i: path.dir_entry.i_node,
            dir_entry_addr: path.dir_entry_addr,
            parent_inode_i: path.parent_inode_i,
            current_pos: 0,
        });

        self.opened_len += 1;

        return Ok(fd);
    }
}

#[test]
fn open_test() {
    let mut fs = Fs::format().unwrap();

    assert!(fs.open(".").is_err());
    assert!(fs.open("..").is_err());
    assert!(fs.open("////////").is_err());

    assert!(fs.mkdir("dir_a").is_ok());
    assert!(fs.mkdir("dirrrrrrrrrrrrrrrrrrrrr").is_err());

    assert!(fs.open("dir_a").is_err());
    assert!(fs.open("not_exists").is_err());

    for i in 0..fs.fds.len() {
        assert!(fs.create(&format!("file_{}", i)).is_ok());
        assert!(fs
            .open(&format!("./../../../../.././/////file_{}", i))
            .is_ok());
    }
    assert!(fs.open(&format!("file_0")).is_err());
}
