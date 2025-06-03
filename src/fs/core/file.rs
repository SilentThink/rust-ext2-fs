use super::*;

#[derive(Clone, Copy)]
pub enum FileType {
    File,
    Dir,
    Symlink,
}

impl Into<u8> for FileType {
    fn into(self) -> u8 {
        match self {
            Self::File => 1,
            Self::Dir => 2,
            Self::Symlink => 3,
        }
    }
}

impl From<u8> for FileType {
    fn from(n: u8) -> Self {
        match n {
            1 => Self::File,
            2 => Self::Dir,
            3 => Self::Symlink,
            _ => unreachable!("Unknown file type"),
        }
    }
}

#[derive(Default, Clone)]
pub struct FileMode {
    /// 文件的存取权限，[0rwx:0rwx]
    pub mode: u8,
    /// 文件的拥有者
    pub owner: u8,
}

/// 用户类型
pub enum UserType {
    /// 文件拥有者
    Owner,
    /// 其他用户
    Other,
}

impl FileMode {
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

    pub fn can_read(&self, user: usize) -> bool {
        match self.owner == user as u8 {
            true => self.mode & 0b00_100_000 != 0,
            false => self.mode & 0b00_000_100 != 0 || user == 0,
        }
    }

    pub fn can_write(&self, user: usize) -> bool {
        match self.owner == user as u8 {
            true => self.mode & 0b00_010_000 != 0,
            false => self.mode & 0b00_000_010 != 0 || user == 0,
        }
    }

    pub fn can_exec(&self, user: usize) -> bool {
        match self.owner == user as u8 {
            true => self.mode & 0b00_001_000 != 0,
            false => self.mode & 0b00_000_001 != 0 || user == 0,
        }
    }

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
    /// 将字符串转换成修改权限所需的格式
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
