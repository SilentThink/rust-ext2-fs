//! # 特征定义模块
//!
//! 为文件系统定义了多种 trait，以满足对元素的读写功能：
//!
//! 1. [`IntoBytes`] 用来将 Rust 结构体转换成字节数组，这样就可以将
//!   整个结构写入磁盘了
//! 2. [`ByteArray`] 主要是用来将 `&str` 字符串转换成字符数组
//! 3. [`FromDisk`] 从磁盘文件里读取对象
//! 4. [`FsFileExt`] 为标准库的 File 类型提供跨平台的定位读写功能

use super::*;
use std::mem::size_of;

/// 字节转换特征
/// 
/// 为结构体提供转换为字节数组的能力，用于将数据写入磁盘
pub trait IntoBytes: Sized {
    /// 将结构体转换为字节数组
    /// 
    /// # 返回值
    /// 返回结构体的字节表示，可直接写入磁盘
    /// 
    /// # 安全性
    /// 此方法使用 unsafe 代码将结构体内存直接转换为字节数组
    fn bytes(&self) -> &[u8] {
        convert(self)
    }
}

/// 内部转换函数
/// 
/// # 参数
/// - `ref_val`: 要转换的结构体引用
/// 
/// # 返回值
/// 返回结构体的字节数组表示
/// 
/// # 安全性
/// 使用 unsafe 代码直接访问内存，调用者需要确保结构体布局稳定
#[inline(always)]
fn convert<'b, 'a: 'b, T>(ref_val: &'a T) -> &'b [u8] {
    unsafe { std::slice::from_raw_parts(ref_val as *const T as *const u8, size_of::<T>()) }
}

// 为核心数据结构实现 IntoBytes 特征
impl IntoBytes for GroupDesc {}
impl IntoBytes for Inode {}
impl IntoBytes for DirEntry {}

/// 字节数组转换特征
/// 
/// 为字符串类型提供转换为固定长度字节数组的能力
pub trait ByteArray {
    /// 将字符串转换为固定长度的字节数组
    /// 
    /// # 类型参数
    /// - `LEN`: 目标数组的长度
    /// 
    /// # 返回值
    /// 成功时返回固定长度的字节数组，失败时返回错误
    fn into_array<const LEN: usize>(self) -> Result<[u8; LEN]>;
}

impl ByteArray for &str {
    /// 将字符串转换为固定长度的字节数组
    /// 
    /// # 类型参数
    /// - `LEN`: 目标数组的长度
    /// 
    /// # 返回值
    /// 成功时返回以零填充的字节数组，失败时返回错误
    /// 
    /// # 错误
    /// - 字符串长度 >= LEN 时返回错误
    /// - 字符串为空时返回错误
    /// - 字符串包含 '/' 字符时返回错误
    /// 
    /// # 安全性
    /// 使用 unsafe 代码进行内存复制，但已确保边界检查
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

/// 从磁盘读取特征
/// 
/// 为结构体提供从磁盘文件中读取数据的能力
pub trait FromDisk: Sized + Default {
    /// 从磁盘指定位置读取结构体数据
    /// 
    /// # 参数
    /// - `disk`: 磁盘文件引用
    /// - `offset`: 读取位置的字节偏移量
    /// 
    /// # 返回值
    /// 成功时返回读取的结构体实例，失败时返回IO错误
    /// 
    /// # 算法
    /// 1. 创建默认的结构体实例
    /// 2. 将结构体内存作为字节数组
    /// 3. 从磁盘指定位置读取数据填充结构体
    /// 
    /// # 安全性
    /// 使用 unsafe 代码直接操作结构体内存
    fn from_disk(disk: &std::fs::File, offset: u64) -> Result<Self> {
        let mut obj = Self::default();
        let slice = unsafe {
            std::slice::from_raw_parts_mut(&mut obj as *mut Self as *mut u8, size_of::<Self>())
        };
        disk.read_at(slice, offset)?;
        Ok(obj)
    }
}

// 为核心数据结构实现 FromDisk 特征
impl FromDisk for DirEntry {}
impl FromDisk for Inode {}
impl FromDisk for GroupDesc {}

/// 文件扩展特征
/// 
/// 为标准库的 File 类型提供跨平台的定位读写功能
pub trait FsFileExt {
    /// 在指定位置读取数据
    /// 
    /// # 参数
    /// - `buf`: 读取数据的缓冲区
    /// - `offset`: 读取位置的字节偏移量
    /// 
    /// # 返回值
    /// 成功时返回实际读取的字节数，失败时返回IO错误
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize>;
    
    /// 在指定位置写入数据
    /// 
    /// # 参数
    /// - `buf`: 要写入的数据
    /// - `offset`: 写入位置的字节偏移量
    /// 
    /// # 返回值
    /// 成功时返回实际写入的字节数，失败时返回IO错误
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize>;
}

impl FsFileExt for std::fs::File {
    /// Unix系统的定位读取实现
    #[inline(always)]
    #[cfg(target_family = "unix")]
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        std::os::unix::fs::FileExt::read_at(self, buf, offset)
    }
    
    /// Unix系统的定位写入实现
    #[inline(always)]
    #[cfg(target_family = "unix")]
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        std::os::unix::fs::FileExt::write_at(self, buf, offset)
    }

    /// Windows系统的定位读取实现
    #[inline(always)]
    #[cfg(target_family = "windows")]
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        std::os::windows::fs::FileExt::seek_read(self, buf, offset)
    }
    
    /// Windows系统的定位写入实现
    #[inline(always)]
    #[cfg(target_family = "windows")]
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        std::os::windows::fs::FileExt::seek_write(self, buf, offset)
    }
}
