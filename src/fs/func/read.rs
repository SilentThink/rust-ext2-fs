//! # 读取文件

use crate::fs::core::*;

impl Fs {
    /// 返回已经读取的字符个数
    pub fn read(&mut self, fd: usize, buf: &mut [u8]) -> Result<usize> {
        if fd >= self.fds.len() || self.fds[fd].is_none() {
            return Err(Error::new(ErrorKind::Other, "Bad file description"));
        }

        let file = self.fds[fd].as_mut().unwrap();

        // read 需要读权限
        if !file.inode.i_mode.can_read(self.user) {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "Permission Denied.",
            ));
        }

        let mut counter = 0;

        loop {
            if file.current_pos >= file.inode.i_size as usize || counter == buf.len() {
                break;
            }

            let mut c = [0u8; 1];
            let addr = file
                .inode
                .convert_addr(&self.disk, file.current_pos as u64)?;
            self.disk.read_at(&mut c, addr.addr)?;

            buf[counter] = c[0];

            counter += 1;
            file.current_pos += 1;
        }

        Ok(counter)
    }
}
