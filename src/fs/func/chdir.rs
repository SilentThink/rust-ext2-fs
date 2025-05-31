use crate::fs::core::*;

impl Fs {
    pub fn chdir(&mut self, path: &str) -> Result<()> {
        let entry = self.path_parse(path)?.dir_entry;
        let inode = self.get_inode(entry.i_node)?;

        // chdir 需要对目录的读权限
        if !inode.i_mode.can_read(self.user) {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "Need read permission of directory",
            ));
        }

        match entry.file_type.into() {
            FileType::File => Err(Error::new(ErrorKind::Other, "Not a directory")),
            FileType::Dir => {
                self.cwd = entry;
                Ok(())
            }
        }
    }

    pub (in crate::fs) fn chdir_without_limit(&mut self, path: &str) -> Result<()> {
        let entry = self.path_parse(path)?.dir_entry;
        match entry.file_type.into() {
            FileType::File => Err(Error::new(ErrorKind::Other, "Not a directory")),
            FileType::Dir => {
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
