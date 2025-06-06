use crate::fs::core::*; // 引入文件系统核心模块，包含文件系统操作所需的类型和函数

impl Fs {
    // 定义文件系统的 `close` 方法，用于关闭指定的文件描述符
    pub fn close(&mut self, fd: usize) -> Result<()> {
        // 检查文件描述符是否有效
        // 如果文件描述符超出范围，或者对应的文件已经关闭，则返回错误
        if fd >= self.fds.len() || self.fds[fd].is_none() {
            return Err(Error::new(ErrorKind::Other, "Bad file description"));
        }

        // 将指定的文件描述符位置设置为 `None`，表示关闭文件
        self.fds[fd] = None;

        // 减少当前打开的文件计数
        self.opened_len -= 1;

        // 返回成功
        Ok(())
    }
}