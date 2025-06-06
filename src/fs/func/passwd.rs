use crate::fs::core::*; // 引入文件系统核心模块，包含文件系统操作所需的类型和函数

impl Fs {
    // 定义文件系统的 `passwd` 方法，用于修改用户密码
    pub fn passwd(&mut self, user: usize, passwd: &str) -> Result<()> {
        // 检查当前用户是否有权限修改指定用户的密码
        // 只有 root 用户（user == 0）或用户本人（user == self.user）可以修改密码
        if self.user != 0 && user != self.user {
            return Err(Error::new(ErrorKind::PermissionDenied, "Permission Denied"));
        }

        // 尝试获取指定用户的可变引用
        if let Some(user) = self.fs_desc.users.get_mut(user) {
            // 如果用户存在，将新密码写入用户的密码字段
            user.password = passwd.into_array()?; // 将字符串转换为固定长度的数组
        } else {
            // 如果用户不存在，返回错误
            return Err(Error::new(ErrorKind::PermissionDenied, "User not found"));
        }

        // 将修改后的文件系统描述符写回磁盘
        self.write_fs_desc()?; // 保存文件系统描述符的更改

        // 返回成功
        Ok(())
    }
}