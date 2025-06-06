use crate::fs::core::*; // 引入文件系统核心模块

impl Fs {
    pub fn write(&mut self, fd: usize, buf: &[u8]) -> Result<usize> {
        // 定义文件系统的写入函数，接收文件描述符和要写入的数据缓冲区
        if fd >= self.fds.len() || self.fds[fd].is_none() {
            // 检查文件描述符是否有效
            return Err(Error::new(ErrorKind::Other, "Bad file descriptor"));
        }

        let mut file = self.fds[fd].clone().unwrap(); // 获取文件描述符对应的文件对象

        if !file.inode.i_mode.can_write(self.user) {
            // 检查当前用户是否有写权限
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "Permission Denied.",
            ));
        }

        let mut counter = 0; // 初始化写入计数器

        loop {
            if counter == buf.len() {
                // 如果已经写完所有数据，退出循环
                break;
            }

            if file.current_pos >= file.inode.i_size as usize {
                // 如果当前写入位置超过文件大小
                file.inode.i_size = (file.current_pos + 1) as u32; // 更新文件大小

                let mut new_blocks = (file.inode.i_size / BLOCK_SIZE as u32) as u16; // 计算所需的新块数
                if file.inode.i_size % BLOCK_SIZE as u32 != 0 {
                    new_blocks += 1; // 如果有剩余部分，分配一个额外的块
                }

                let block_to_alloc = new_blocks - file.inode.i_blocks; // 计算需要分配的新块数
                for _ in 0..block_to_alloc {
                    file.inode.alloc_data_block(self)?; // 分配数据块
                }
            }

            let addr = file
                .inode
                .convert_addr(&self.disk, file.current_pos as u64)?; // 获取当前写入位置的磁盘地址
            self.disk.write_at(&[buf[counter]], addr.addr)?; // 将数据写入磁盘

            counter += 1; // 更新写入计数器
            file.current_pos += 1; // 更新文件的当前写入位置
        }

        file.inode.i_mtime = utils::now(); // 更新文件的修改时间

        // 写入更新后的索引节点
        self.write_inode(file.inode_i, file.inode.clone())?;

        self.fds[fd] = Some(file); // 更新文件描述符中的文件对象

        Ok(counter) // 返回成功写入的字节数
    }
}

#[test]
fn test_read_write() {
    let part1 = r#"ghjgky;;...fygeyrgfierwygw"#; // 定义测试用的字符串

    let mut fs = Fs::format().unwrap(); // 格式化文件系统
    fs.create("test.txt").unwrap(); // 创建一个测试文件
    let fd = fs.open("test.txt").unwrap(); // 打开文件并获取文件描述符
    fs.write(fd, part1.as_bytes()).unwrap(); // 向文件写入测试字符串
    fs.write(fd, part1.as_bytes()).unwrap(); // 再次写入相同的字符串
    let fd2 = fs.open("test.txt").unwrap(); // 重新打开文件以进行读取

    let mut str: Vec<u8> = Vec::new(); // 初始化一个字符串缓冲区
    let mut buf = [0u8; 13]; // 定义一个读取缓冲区
    while fs.read(fd2, &mut buf).unwrap() != 0 {
        // 循环读取文件内容
        str.extend(buf.iter()); // 将读取到的数据追加到字符串缓冲区
        buf.fill(0); // 清空读取缓冲区
    }
    let str = std::str::from_utf8(&str).unwrap_or("invalid utf-8 string"); // 将字节数组转换为字符串
    println!("{}", str); // 打印读取到的内容
    assert_eq!(str.trim_matches('\0'), format!("{}{}", part1, part1)); // 断言读取的内容是否正确
}