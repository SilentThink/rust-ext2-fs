use crate::fs::core::*;

impl Fs {
    pub fn userdel(&mut self, name: &str) -> Result<()> {
        if name == "root" {
            return Err(Error::new(ErrorKind::Other, "Can't delete root user"));
        }
        if name.into_array()? == self.fs_desc.users[self.user].name {
            return Err(Error::new(
                ErrorKind::Other,
                "Can't delete yourself, please login with other account",
            ));
        }

        let mut ok = false;
        for user in self.fs_desc.users.iter_mut() {
            if user.name == name.into_array()? {
                *user = User::default();
                self.fs_desc.used_dirs_count -= 1;
                ok = true;
            }
        }

        if !ok {
            return Err(Error::new(ErrorKind::Other, "User not exists."));
        }

        self.write_fs_desc()?;
        Ok(())
    }
}
