//! # 读取文件

use crate::fs::core::*;

impl Fs {
    /// 返回已经读取的字符个数
    pub fn read(&mut self, fd: usize, buf: &mut [u8]) -> Result<usize> {
        // 如果文件描述符无效，返回错误
        if fd >= self.fds.len() || self.fds[fd].is_none() {
            // 返回错误
            return Err(Error::new(ErrorKind::Other, "Bad file description"));
        }

        // 获取文件
        let file = self.fds[fd].as_mut().unwrap();

        // 如果用户没有读权限，返回错误
        if !file.inode.i_mode.can_read(self.user) {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "Permission Denied.",
            ));
        }

        // 计数器
        let mut counter = 0;

        // 循环读取
        loop {
            if file.current_pos >= file.inode.i_size as usize || counter == buf.len() {
                break;
            }

            // 创建一个缓冲区用于存储读取的字符
            let mut c = [0u8; 1];
            // 获取文件地址
            let addr = file
                .inode
                .convert_addr(&self.disk, file.current_pos as u64)?;
            self.disk.read_at(&mut c, addr.addr)?;

            // 将读取的字符写入缓冲区
            buf[counter] = c[0];

            // 计数器加 1
            counter += 1;
            // 当前位置加 1
            file.current_pos += 1;
        }

        // 返回已经读取的字符个数
        Ok(counter)
    }
}
