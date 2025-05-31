//! 用来定义文件系统的常量

use std::mem::size_of;

use super::{Inode, DirEntry};

/// 每个磁盘块的大小：512 字节
pub const BLOCK_SIZE: usize = 512;

/// 数据块的块数为 BLOCK_SIZE * 8
pub const DATA_BLOCKS: usize = BLOCK_SIZE * 8;

/// i 节点需要的块数
const INODE_BLOCKS: usize = 8 * size_of::<Inode>();

/// 数据块在磁盘上的起始位置。
/// 前三块被用来存储 位图 和 组描述符 [`GroupDesc`](crate::fs::GroupDesc)，
/// 之后的 [`INODE_BLOCKS`] 被 i 节点占用
pub const DATA_BEGIN_BLOCK: usize = (3 + INODE_BLOCKS) as usize;

/// 磁盘块的总块数
pub const BLOCKS: usize = DATA_BEGIN_BLOCK + DATA_BLOCKS;

/// 目录项的大小
pub const DIR_ENTRY_SIZE: usize = size_of::<DirEntry>();

/// 每一个数据块可以存储的目录项个数
pub const BLOCK_ADDR_NUM: usize = BLOCK_SIZE / DIR_ENTRY_SIZE;

/// 虚拟磁盘的路径名
pub const DISK_PATH: &str = "disk.bin";

/// 整个文件系统可以同时打开的文件个数
pub const FD_LIMIT: usize = 20;

#[test]
fn test_sizes() {
    use super::GroupDesc;

    /// 检查一个数是否是以2为底的幂
    fn check_log2(mut num: usize) -> bool {
        while num != 1 {
            if num % 2 != 0 {
                return false;
            }
            num /= 2
        }
        return true;
    }

    assert!(size_of::<GroupDesc>() <= BLOCK_SIZE);
    assert!(check_log2(BLOCK_SIZE));
    assert!(check_log2(size_of::<DirEntry>()));
    assert!(check_log2(size_of::<Inode>()));
}