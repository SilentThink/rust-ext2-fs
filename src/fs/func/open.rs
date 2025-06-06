use crate::fs::core::*; // 引入文件系统核心模块

impl Fs {
    /// 用来打开路径名为 `path` 的文件
    ///
    /// 当文件不存在、或者同时打开的文件数超过 [`FD_LIMIT`]、或者尝试打开一个目录时，文件打开
    /// 失败
    pub fn open(&mut self, path: &str) -> Result<usize> {
        // 当前打开的文件已经到达上限
        if self.opened_len == FD_LIMIT {
            return Err(Error::new(
                ErrorKind::Other,
                "Files descriptions up to limits",
            )); // 如果打开的文件数达到上限，返回错误
        }

        // 使用path_parse_with_options函数解析路径，并设置follow_symlinks为true
        let path = self.path_parse(path)?; // 解析路径，返回路径解析结果

        // 不能打开目录
        if let FileType::Dir = path.dir_entry.file_type.into() {
            return Err(Error::new(
                ErrorKind::Other,
                "Can't open directory as file",
            )); // 如果路径指向的是目录，返回错误
        }

        // 文件的索引节点
        let inode = Inode::from_disk(&self.disk, self.addr_i_node(path.dir_entry.i_node))?; // 从磁盘加载文件的索引节点

        // open 需要文件的读权限
        if !inode.i_mode.can_read(self.user) {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "Permission Denied. Need read permission.",
            )); // 检查当前用户是否有读权限，如果没有则返回错误
        }

        // 分配文件描述符
        let mut fd = 0; // 初始化文件描述符变量
        for i in 0..self.fds.len() {
            if self.fds[i].is_none() {
                fd = i; // 找到一个空闲的文件描述符位置
                break;
            }
        }

        self.fds[fd] = Some(File {
            inode, // 文件的索引节点
            inode_i: path.dir_entry.i_node, // 索引节点编号
            dir_entry_addr: path.dir_entry_addr, // 目录项地址
            parent_inode_i: path.parent_inode_i, // 父目录的索引节点编号
            current_pos: 0, // 文件的当前读写位置
        });

        self.opened_len += 1; // 增加打开的文件计数

        return Ok(fd); // 返回分配的文件描述符
    }
}

#[test]
fn open_test() {
    let mut fs = Fs::format().unwrap(); // 格式化文件系统

    // 测试打开无效路径
    assert!(fs.open(".").is_err()); // 尝试打开当前目录，应该失败
    assert!(fs.open("..").is_err()); // 尝试打开父目录，应该失败
    assert!(fs.open("////////").is_err()); // 尝试打开无效路径，应该失败

    // 测试创建和打开目录
    assert!(fs.mkdir("dir_a").is_ok()); // 创建目录 "dir_a"，应该成功
    assert!(fs.mkdir("dirrrrrrrrrrrrrrrrrrrrr").is_err()); // 创建过长的目录名，应该失败

    // 测试打开目录
    assert!(fs.open("dir_a").is_err()); // 尝试打开目录 "dir_a"，应该失败
    assert!(fs.open("not_exists").is_err()); // 尝试打开不存在的文件，应该失败

    // 测试打开文件
    for i in 0..fs.fds.len() {
        assert!(fs.create(&format!("file_{}", i)).is_ok()); // 创建文件，应该成功
        assert!(fs
            .open(&format!("./../../../../.././/////file_{}", i))
            .is_ok()); // 打开文件，即使路径包含冗余部分，也应该成功
    }
    assert!(fs.open(&format!("file_0")).is_err()); // 尝试打开超出文件描述符限制的文件，应该失败
}