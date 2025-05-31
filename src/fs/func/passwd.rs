use crate::fs::core::*;

impl Fs {
    pub fn passwd(&mut self, user: usize, passwd: &str) -> Result<()> {
        if self.user != 0 && user != self.user {
            return Err(Error::new(ErrorKind::PermissionDenied, "Permission Denied"));
        }

        if let Some(user) = self.fs_desc.users.get_mut(user) {
            user.password = passwd.into_array()?
        } else {
            return Err(Error::new(ErrorKind::PermissionDenied, "User not found"));
        }
        self.write_fs_desc()?;
        Ok(())
    }
}
