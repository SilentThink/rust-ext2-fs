use crate::fs::core::*;

impl Fs {
    pub fn pwd(&mut self) -> Result<String> {
        let orig_cwd = self.cwd.clone();
        let res = self.find_path();
        self.cwd = orig_cwd;
        return res;
    }

    fn find_path(&mut self) -> Result<String> {
        let mut path = String::new();

        loop {
            let inode_to_find = self.cwd.i_node;

            if self.cwd.i_node == 0 {
                break;
            }

            self.chdir_without_limit("..")?;

            for item in self.cwd.iter_without_limit(self)? {
                if let DirEntryIterItem::Using(Item { entry, .. }) = item {
                    if entry.i_node == inode_to_find {
                        path = format!(
                            "/{}{}",
                            String::from_utf8_lossy(&entry.name).trim_matches('\0'),
                            path
                        );
                    }
                }
            }
        }

        match path.is_empty() {
            true => Ok("/".into()),
            false => Ok(path),
        }
    }
}

#[test]
fn test_pwd() {
    let mut fs = Fs::format().unwrap();
    fs.mkdir("a").unwrap();
    fs.chdir("a").unwrap();
    fs.mkdir("b").unwrap();
    fs.chdir("b").unwrap();
    fs.mkdir("c").unwrap();
    fs.chdir("c").unwrap();
    assert_eq!(fs.pwd().unwrap(), "/a/b/c")
}
