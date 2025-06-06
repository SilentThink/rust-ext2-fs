use crate::fs::core::*;

impl Fs {
    // 修改文件所有者
    pub fn chown(&mut self, path: &str, user: &str) -> Result<()> {
        // 解析路径
        let entry = self.path_parse(path)?;

        // 获取用户ID
        let mut user_id = None;
        // 遍历用户
        for (i, u) in self.fs_desc.users.iter().enumerate() {
            // 如果用户名匹配
            if u.name == user.into_array()? {
                // 设置用户ID
                user_id = Some(i);
                // 跳出循环
                break;
            }
        }

        // 根据用户ID进行处理
        match user_id {
            // 如果用户ID不存在，返回错误
            None => {
                // 返回错误
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Can't find user {}", user),
                ))
            }
            // 如果用户ID存在，修改文件所有者
            Some(user) => {
                // 获取inode
                let mut inode = self.get_inode(entry.dir_entry.i_node)?;

                // 如果用户不是所有者且不是root，返回错误
                if inode.i_mode.owner as usize != self.user && self.user != 0 {
                    return Err(Error::new(ErrorKind::PermissionDenied, "Permission Denied"));
                }
                inode.i_mode.owner = user as u8;
                self.write_inode(entry.dir_entry.i_node, inode)?;
                Ok(())
            }
        }
    }
}
