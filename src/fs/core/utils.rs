use super::*;
use std::io::Write;

#[inline(always)]
pub fn now() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32
}

/// 返回大小为 BLOCK_SIZE 的空数组
#[inline(always)]
pub fn empty_blk() -> [u8; BLOCK_SIZE as usize] {
    [0u8; BLOCK_SIZE as usize]
}

impl Fs {
    pub fn get_inode(&self, inode_i: u16) -> Result<Inode> {
        Ok(Inode::from_disk(&self.disk, self.addr_i_node(inode_i))?)
    }

    pub fn exit(&mut self) {
        self.write_fs_desc().unwrap();
        self.disk.flush().unwrap()
    }

    pub fn fs_desc(&self) -> &GroupDesc {
        &self.fs_desc
    }

    pub fn current_user(&self) -> usize {
        self.user
    }
}

pub fn str(str: &[u8]) -> &str {
    std::str::from_utf8(str)
        .unwrap_or("[err invaild utf-8]")
        .trim_end_matches('\0')
}
