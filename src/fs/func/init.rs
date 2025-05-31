use crate::fs::core::*;

impl Fs {
    pub fn init() -> Result<Fs> {
        let disk = std::fs::File::options()
            .read(true)
            .write(true)
            .open(DISK_PATH)?;

        let fs_desc = GroupDesc::from_disk(&disk, 0)?;

        let cwd_inode = Inode::from_disk(&disk, fs_desc.inode_table as u64 * BLOCK_SIZE as u64)?;
        let cwd = DirEntry::from_disk(&disk, DATA_BEGIN_BLOCK as u64 * BLOCK_SIZE as u64)?;

        if cwd.name != ".".into_array()?
            || cwd.i_node != 0
            || cwd_inode.i_size < 2 * DIR_ENTRY_SIZE as u32
        {
            return Err(Error::new(ErrorKind::Other, "Bad filesystem"));
        }

        Ok(Fs {
            fs_desc,
            cwd,
            disk,
            fds: Default::default(),
            opened_len: 0,
            user: 0,
        })
    }
}

#[test]
fn test_init() {
    Fs::format().unwrap();
    Fs::init().unwrap();
}
