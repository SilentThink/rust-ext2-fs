use crate::fs::core::*;

impl Fs {
    pub fn useradd(&mut self, name: &str, passwd: &str) -> Result<()> {
        if self.fs_desc.users_len as usize >= self.fs_desc.users.len() {
            return Err(Error::new(ErrorKind::Other, "Can't add more user"));
        }

        for (_, user) in self.fs_desc.users.iter_mut().enumerate() {
            if user.name == name.into_array()? {
                return Err(Error::new(ErrorKind::Other, "User exists yet."));
            }

            if user.name[0] == 0 {
                user.name = name.into_array()?;
                user.password = passwd.into_array()?;
                self.fs_desc.users_len += 1;
                break;
            }
        }

        let path = format!("/home/{}", name);

        self.mkdir(&path)?;
        self.chown(&path, name)?;

        self.write_fs_desc()?;

        Ok(())
    }
}
