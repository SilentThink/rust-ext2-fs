//! # 文件类型和权限管理模块
//!
//! 此模块定义了文件系统中的文件类型、权限模式以及用户访问控制。
//! 主要包含以下功能：
//! - 文件类型的定义和转换
//! - 文件权限模式的管理
//! - 用户访问权限的检查
//! - 文件权限的字符串表示和解析

use super::*;

/// 文件类型
/// 
/// 表示文件系统中的三种基本文件类型
#[derive(Clone, Copy)]
pub enum FileType {
    /// 普通文件
    File,
    /// 目录
    Dir,
    /// 符号链接
    Symlink,
}

impl Into<u8> for FileType {
    /// 将文件类型转换为数字表示
    /// 
    /// # 返回值
    /// - 1: 普通文件
    /// - 2: 目录
    /// - 3: 符号链接
    fn into(self) -> u8 {
        match self {
            Self::File => 1,
            Self::Dir => 2,
            Self::Symlink => 3,
        }
    }
}

impl From<u8> for FileType {
    /// 从数字表示转换为文件类型
    /// 
    /// # 参数
    /// - `n`: 数字表示的文件类型
    /// 
    /// # 返回值
    /// 对应的文件类型枚举
    /// 
    /// # Panics
    /// 当传入未知的文件类型数字时会 panic
    fn from(n: u8) -> Self {
        match n {
            1 => Self::File,
            2 => Self::Dir,
            3 => Self::Symlink,
            _ => unreachable!("Unknown file type"),
        }
    }
}

/// 文件权限模式
/// 
/// 存储文件的访问权限和所有者信息
#[derive(Default, Clone)]
pub struct FileMode {
    /// 文件的存取权限位图，格式为 [0rwx:0rwx]
    /// 高3位表示所有者权限，低3位表示其他用户权限
    pub mode: u8,
    /// 文件的拥有者ID
    pub owner: u8,
}

/// 用户类型
/// 
/// 用于区分不同的用户类型以进行权限检查
pub enum UserType {
    /// 文件拥有者
    Owner,
    /// 其他用户
    Other,
}

impl FileMode {
    /// 创建新的文件权限模式
    /// 
    /// # 参数
    /// - `owner`: 文件拥有者的用户ID
    /// - `file_type`: 文件类型
    /// 
    /// # 返回值
    /// 根据文件类型设置默认权限的 FileMode 实例
    /// 
    /// # 默认权限
    /// - 普通文件: rwxr--
    /// - 目录: rwxr-x
    /// - 符号链接: rwxr--
    pub fn new(owner: usize, file_type: FileType) -> Self {
        Self {
            mode: match file_type {
                FileType::File => 0b00_111_100,
                FileType::Dir => 0b00_111_101,
                FileType::Symlink => 0b00_111_100,
            },
            owner: owner as u8,
        }
    }

    /// 检查用户是否有读权限
    /// 
    /// # 参数
    /// - `user`: 用户ID
    /// 
    /// # 返回值
    /// 如果用户有读权限则返回 true，否则返回 false
    /// 
    /// # 规则
    /// - root用户（ID为0）总是有读权限
    /// - 文件拥有者检查拥有者权限位
    /// - 其他用户检查其他用户权限位
    pub fn can_read(&self, user: usize) -> bool {
        match self.owner == user as u8 {
            true => self.mode & 0b00_100_000 != 0,
            false => self.mode & 0b00_000_100 != 0 || user == 0,
        }
    }

    /// 检查用户是否有写权限
    /// 
    /// # 参数
    /// - `user`: 用户ID
    /// 
    /// # 返回值
    /// 如果用户有写权限则返回 true，否则返回 false
    /// 
    /// # 规则
    /// - root用户（ID为0）总是有写权限
    /// - 文件拥有者检查拥有者权限位
    /// - 其他用户检查其他用户权限位
    pub fn can_write(&self, user: usize) -> bool {
        match self.owner == user as u8 {
            true => self.mode & 0b00_010_000 != 0,
            false => self.mode & 0b00_000_010 != 0 || user == 0,
        }
    }

    /// 检查用户是否有执行权限
    /// 
    /// # 参数
    /// - `user`: 用户ID
    /// 
    /// # 返回值
    /// 如果用户有执行权限则返回 true，否则返回 false
    /// 
    /// # 规则
    /// - root用户（ID为0）总是有执行权限
    /// - 文件拥有者检查拥有者权限位
    /// - 其他用户检查其他用户权限位
    pub fn can_exec(&self, user: usize) -> bool {
        match self.owner == user as u8 {
            true => self.mode & 0b00_001_000 != 0,
            false => self.mode & 0b00_000_001 != 0 || user == 0,
        }
    }

    /// 设置文件权限模式
    /// 
    /// # 参数
    /// - `user`: 执行操作的用户ID
    /// - `mode`: 新的权限模式
    /// 
    /// # 返回值
    /// 成功时返回 Ok(())，失败时返回错误信息
    /// 
    /// # 错误
    /// - 非文件拥有者且非root用户尝试修改权限
    /// - 权限模式超出有效范围（0-127）
    pub fn set_mode(&mut self, user: usize, mode: u8) -> Result<()> {
        if self.owner != user as u8 && user != 0 {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "Permission Denied. Can't set mode to file",
            ));
        }

        if mode > 0b00_111_111 {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "Permission Denied. Invaild file mode",
            ));
        }

        self.mode = mode;

        Ok(())
    }
}

impl std::fmt::Display for FileMode {
    /// 格式化文件权限为字符串表示
    /// 
    /// # 返回值
    /// 格式为 "rwx:rwx" 的权限字符串
    /// - 第一部分是拥有者权限
    /// - 第二部分是其他用户权限
    /// - r表示读权限，w表示写权限，x表示执行权限
    /// - 无权限时用 '-' 表示
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut mask = 0b00_100_000;
        let mut str = String::new();
        for i in 0..6 {
            match mask & self.mode {
                0 => str.push('-'),
                _ => match i % 3 {
                    0 => str.push('r'),
                    1 => str.push('w'),
                    2 => str.push('x'),
                    _ => unreachable!(),
                },
            };
            if i == 2 {
                str.push(':')
            }
            mask = mask >> 1;
        }
        write!(f, "{}", str)
    }
}

impl FileMode {
    /// 将字符串格式的权限转换为数字模式
    /// 
    /// # 参数
    /// - `str`: 权限字符串，格式为 "rwx:rwx"
    /// 
    /// # 返回值
    /// 成功时返回权限的数字表示，失败时返回错误
    /// 
    /// # 错误
    /// - 字符串长度不为7
    /// - 字符串格式不正确
    /// - 包含无效字符
    /// 
    /// # 示例
    /// ```
    /// let mode = FileMode::str_to_mode("rwx:r-x").unwrap();
    /// assert_eq!(mode, 0b00_111_101);
    /// ```
    pub fn str_to_mode(str: &str) -> Result<u8> {
        let mut err = str.len() != 7;

        let mut str = str.as_bytes().to_vec();
        str.remove(3);

        let mut mode = 0b00_000_000;
        let mut mask = 0b00_100_000;
        let temp = [b'r', b'w', b'x'];

        for i in 0..6 {
            match str[i] {
                b'-' => {}
                a if a == temp[i % 3] => mode = mode | mask,
                _ => err = true,
            };
            mask = mask >> 1;
        }

        match err {
            true => Err(Error::new(ErrorKind::InvalidData, "Wrong file mode format")),
            false => Ok(mode),
        }
    }
}
