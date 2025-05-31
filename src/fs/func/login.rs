use crate::fs::core::*;

impl Fs {
    pub fn login(&mut self, username: &str, password: &str) -> Result<()> {
        if username.is_empty() || password.is_empty() {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "Wrong username or password",
            ));
        }

        for (i, user) in self.fs_desc.users.iter().enumerate() {
            if user.name == username.into_array()? && user.password == password.into_array()? {
                self.user = i;
                return Ok(());
            }
        }

        Err(Error::new(
            ErrorKind::PermissionDenied,
            "Wrong username or password",
        ))
    }
}
