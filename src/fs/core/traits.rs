//! # traits.rs
//!
//! 为文件系统定义了多种 trait，以满足对元素的读写功能：
//!
//! 1. [`IntoBytes`] 用来将 Rust 结构体转换成字符数组，这样就可以将
//!   整个结构写入磁盘了
//! 2. [`ByteArray`] 主要是用来将 `&str` 字符串转换成字符数组
//! 3. [`FromDisk`] 将文件里读取对象
//!

use super::*;
use std::mem::size_of;

pub trait IntoBytes: Sized {
    fn bytes(&self) -> &[u8] {
        convert(self)
    }
}

#[inline(always)]
fn convert<'b, 'a: 'b, T>(ref_val: &'a T) -> &'b [u8] {
    unsafe { std::slice::from_raw_parts(ref_val as *const T as *const u8, size_of::<T>()) }
}

impl IntoBytes for GroupDesc {}
impl IntoBytes for Inode {}
impl IntoBytes for DirEntry {}

pub trait ByteArray {
    fn into_array<const LEN: usize>(self) -> Result<[u8; LEN]>;
}

impl ByteArray for &str {
    fn into_array<const LEN: usize>(self) -> Result<[u8; LEN]> {
        if self.len() >= LEN {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Too long!! Should less {} bytes", LEN).to_string(),
            ));
        }

        if self.len() == 0 {
            return Err(Error::new(ErrorKind::Other, "can't receive empty string"));
        }

        if self.contains("/") {
            return Err(Error::new(
                ErrorKind::Other,
                "Can't contains char '/'",
            ));
        }

        let mut arr = [0u8; LEN];
        unsafe {
            std::ptr::copy(self.as_ptr(), arr.as_mut_ptr(), self.len());
        }
        Ok(arr)
    }
}

pub trait FromDisk: Sized + Default {
    fn from_disk(disk: &std::fs::File, offset: u64) -> Result<Self> {
        let mut obj = Self::default();
        let slice = unsafe {
            std::slice::from_raw_parts_mut(&mut obj as *mut Self as *mut u8, size_of::<Self>())
        };
        disk.read_at(slice, offset)?;
        Ok(obj)
    }
}

impl FromDisk for DirEntry {}
impl FromDisk for Inode {}
impl FromDisk for GroupDesc {}

pub trait FsFileExt {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize>;
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize>;
}

impl FsFileExt for std::fs::File {
    #[inline(always)]
    #[cfg(target_family = "unix")]
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        std::os::unix::fs::FileExt::read_at(self, buf, offset)
    }
    #[inline(always)]
    #[cfg(target_family = "unix")]
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        std::os::unix::fs::FileExt::write_at(self, buf, offset)
    }

    #[inline(always)]
    #[cfg(target_family = "windows")]
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        std::os::windows::fs::FileExt::seek_read(self, buf, offset)
    }
    #[inline(always)]
    #[cfg(target_family = "windows")]
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        std::os::windows::fs::FileExt::seek_write(self, buf, offset)
    }
}
