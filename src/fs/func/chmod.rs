use crate::fs::*;

impl Fs {
    // 修改文件权限
    pub fn chmod(&mut self, path: &str, mode: u8) -> Result<()> {
        // 解析路径
        let inode_i = self.path_parse(path)?.dir_entry.i_node;
        // 获取inode
        let mut inode = self.get_inode(inode_i)?;

        // 设置文件权限
        inode.i_mode.set_mode(self.user, mode)?;

        // 写入inode
        self.write_inode(inode_i, inode)?;
        Ok(())
    }
}
