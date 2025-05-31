use crate::fs::*;

impl Fs {
    pub fn chmod(&mut self, path: &str, mode: u8) -> Result<()> {
        let inode_i = self.path_parse(path)?.dir_entry.i_node;
        let mut inode = self.get_inode(inode_i)?;

        inode.i_mode.set_mode(self.user, mode)?;

        self.write_inode(inode_i, inode)?;
        Ok(())
    }
}
