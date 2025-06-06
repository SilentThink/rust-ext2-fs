//! # 目录项迭代器模块
//!
//! 此模块实现了目录项的迭代功能，允许遍历目录中的所有条目。
//! 主要功能包括：
//! - 目录项的迭代器实现
//! - 已删除目录项的识别和处理
//! - 权限检查和访问控制
//! - 目录内容的安全遍历

use super::*;

/// 目录项迭代器
/// 
/// 用于遍历目录中的所有条目，包括正在使用的和已删除的目录项
pub struct DirEntryIterator<'a> {
    /// 目录对应的索引节点
    i_node: Inode,
    /// 已遍历的有效目录项计数器
    counter: usize,
    /// 当前逻辑地址（文件内偏移量）
    logic_addr: u64,
    /// 文件系统引用
    fs: &'a Fs,
}

/// 迭代器返回的条目项
/// 
/// 包含目录项和其在磁盘上的物理地址信息
#[derive(Clone, Debug)]
pub struct Item {
    /// 目录项数据
    pub entry: DirEntry,
    /// 目录项在硬盘上的物理地址
    pub real_addr: RealAddr,
}

/// 目录项迭代结果枚举
/// 
/// 区分正在使用的目录项和已删除的目录项
pub enum DirEntryIterItem {
    /// 正在使用的目录项
    /// 
    /// 表示该目录项当前有效，包含有效的文件或子目录信息
    Using(Item),
    /// 已删除的目录项
    /// 
    /// 表示该目录项已被删除，但在磁盘上仍有残留数据
    Deleted(Item),
}

impl<'a> Iterator for DirEntryIterator<'a> {
    type Item = DirEntryIterItem;

    /// 获取下一个目录项
    /// 
    /// # 返回值
    /// - `Some(DirEntryIterItem)`: 下一个目录项（可能是使用中的或已删除的）
    /// - `None`: 已遍历完所有目录项
    /// 
    /// # 算法
    /// 1. 检查是否已遍历完所有数据块或达到目录项数量限制
    /// 2. 将当前逻辑地址转换为物理地址
    /// 3. 从磁盘读取目录项数据
    /// 4. 根据目录项的状态判断是否已删除
    /// 5. 更新迭代器状态并返回结果
    fn next(&mut self) -> Option<Self::Item> {
        // 检查是否已遍历完所有数据块或目录项
        if self.logic_addr / BLOCK_SIZE as u64 >= self.i_node.i_blocks as u64
            || self.counter >= self.i_node.i_size as usize / DIR_ENTRY_SIZE
        {
            return None;
        }

        // 将逻辑地址转换为物理地址
        let real_addr = self
            .i_node
            .convert_addr(&self.fs.disk, self.logic_addr as u64)
            .unwrap();

        // 从磁盘读取目录项
        let entry = DirEntry::from_disk(&self.fs.disk, real_addr.addr).unwrap();

        // 判断目录项是否已删除
        // 已删除的目录项特征：i_node为0但rec_len不为0
        let deleted = entry.i_node == 0 && entry.rec_len != 0;
        let item = Item { entry, real_addr };

        // 移动到下一个目录项
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
    /// 创建目录项迭代器（带权限检查）
    /// 
    /// # 参数
    /// - `fs`: 文件系统引用
    /// 
    /// # 返回值
    /// 成功时返回目录项迭代器，失败时返回错误
    /// 
    /// # 错误
    /// - 当用户没有执行权限时返回 PermissionDenied 错误
    /// - 当目录项不是目录类型时返回相应错误
    /// 
    /// # 权限要求
    /// 用户必须对目录具有执行权限才能遍历其内容
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

    /// 创建目录项迭代器（无权限检查）
    /// 
    /// # 参数
    /// - `fs`: 文件系统引用
    /// 
    /// # 返回值
    /// 成功时返回目录项迭代器，失败时返回错误
    /// 
    /// # 错误
    /// 当目录项不是目录类型时返回错误
    /// 
    /// # 注意
    /// 此方法跳过权限检查，主要用于系统内部操作
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