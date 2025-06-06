use crate::fs::core::*;

impl Fs {
    // 切换当前工作目录
    pub fn chdir(&mut self, path: &str) -> Result<()> {
        // 解析路径
        let entry = self.path_parse(path)?.dir_entry;
        // 获取inode
        let inode = self.get_inode(entry.i_node)?;

        // 如果用户没有读权限，返回错误
        // chdir 需要对目录的读权限
        if !inode.i_mode.can_read(self.user) {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "Need read permission of directory",
            ));
        }

        // 根据文件类型进行处理
        match entry.file_type.into() {
            // 如果文件类型为文件，返回错误
            FileType::File => {
                // 返回错误
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("{}: Not a directory", utils::str(&entry.name)),
                ))
            }
            // 如果文件类型为目录，切换当前工作目录
            FileType::Dir => {
                // 切换当前工作目录
                self.cwd = entry;
                Ok(())
            }
            // 如果文件类型为软链接，切换当前工作目录
            FileType::Symlink => {
                // 对于软链接，我们需要解析目标路径
                // 但path_parse已经处理了软链接的解析，所以这里不需要额外操作
                self.cwd = entry;
                Ok(())
            }
        }
    }

    // 切换当前工作目录，不限制权限
    #[allow(dead_code)]
    pub (in crate::fs) fn chdir_without_limit(&mut self, path: &str) -> Result<()> {
        // 解析路径
        let entry = self.path_parse(path)?.dir_entry;
        // 获取inode
        match entry.file_type.into() {
            // 如果文件类型为文件，返回错误
            FileType::File => {
                // 返回错误
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("{}: Not a directory", utils::str(&entry.name)),
                ))
            }
            // 如果文件类型为目录，切换当前工作目录
            FileType::Dir => {
                // 切换当前工作目录
                self.cwd = entry;
                Ok(())
            }
            // 如果文件类型为软链接，切换当前工作目录
            FileType::Symlink => {
                // 对于软链接，我们需要解析目标路径
                // 但path_parse已经处理了软链接的解析，所以这里不需要额外操作
                self.cwd = entry;
                Ok(())
            }
        }
    }
}

#[test]
fn test_chdir() {
    let mut fs = Fs::format().unwrap();
    fs.mkdir("a").unwrap();
    fs.create("1.txt").unwrap();
    assert!(fs.chdir("1.txt").is_err());
    fs.chdir("a").unwrap();
    fs.create("1.txt").unwrap();
    fs.chdir("..").unwrap();
    assert!(fs.create("1.txt").is_err());
}
