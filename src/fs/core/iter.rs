use super::*;

pub struct DirEntryIterator<'a> {
    i_node: Inode,
    counter: usize,
    logic_addr: u64,
    fs: &'a Fs,
}

#[derive(Clone, Debug)]
pub struct Item {
    /// 目录项
    pub entry: DirEntry,
    /// 目录项 entry 在硬盘上的物理地址
    pub real_addr: RealAddr,
}

/// 遍历 DirEntry 的结果：会遍历到之前被删除的目录项：
/// [`DirEntryIterItem::Using`] 表示当前目录项正在被使用。
/// [`DirEntryIterItem::Deleted`] 表示当前目录项已经被删除。
pub enum DirEntryIterItem {
    Using(Item),
    Deleted(Item),
}

impl<'a> Iterator for DirEntryIterator<'a> {
    type Item = DirEntryIterItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.logic_addr / BLOCK_SIZE as u64 >= self.i_node.i_blocks as u64
            || self.counter >= self.i_node.i_size as usize / DIR_ENTRY_SIZE
        {
            return None;
        }

        let real_addr = self
            .i_node
            .convert_addr(&self.fs.disk, self.logic_addr as u64)
            .unwrap();

        let entry = DirEntry::from_disk(&self.fs.disk, real_addr.addr).unwrap();

        let deleted = entry.i_node == 0 && entry.rec_len != 0;
        let item = Item { entry, real_addr };

        self.logic_addr += DIR_ENTRY_SIZE as u64;

        return match deleted {
            true => Some(DirEntryIterItem::Deleted(item)),
            false => {
                self.counter += 1;
                Some(DirEntryIterItem::Using(item))
            }
        };
    }
}

impl DirEntry {
    pub fn iter<'a>(&self, fs: &'a Fs) -> Result<DirEntryIterator<'a>> {
        let i_node = Inode::from_disk(&fs.disk, fs.addr_i_node(self.i_node))?;

        // 检查用户权限，当只有执行权限时才能访问目录项
        if !i_node.i_mode.can_exec(fs.user) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Permission Denied. Need exec permission.",
            ));
        }

        Self::iter_without_limit(&self, fs)
    }

    pub (in crate::fs) fn iter_without_limit<'a>(&self, fs: &'a Fs) -> Result<DirEntryIterator<'a>> {
        // 只有当 DirEntry 是目录时才能进行迭代
        if let FileType::File = self.file_type.into() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("{}: Can't iterate with a file", utils::str(&self.name)),
            ));
        }

        let i_node = Inode::from_disk(&fs.disk, fs.addr_i_node(self.i_node))?;

        Ok(DirEntryIterator {
            i_node,
            counter: 0,
            fs,
            logic_addr: 0,
        })
    }
}

#[test]
fn test_iter() {
    let mut fs = Fs::format().unwrap();
    assert!(fs.mkdir("hello").is_ok());
    assert!(fs.mkdir("world").is_ok());
    assert!(fs.mkdir("test1").is_ok());
    assert!(fs.mkdir("test2").is_ok());
    let iter = fs.cwd.iter(&fs).unwrap();
    let mut i = 0;
    for a in iter {
        if let DirEntryIterItem::Using(Item { entry, .. }) = a {
            i += 1;
            println!(
                "{}",
                std::str::from_utf8(&entry.name[0..(entry.name_len as usize)]).unwrap()
            );
        }
    }
    assert_eq!(i, 8);
}