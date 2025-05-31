use crate::fs::core::*;

impl Fs {
    pub fn chown(&mut self, path: &str, user: &str) -> Result<()> {
        let entry = self.path_parse(path)?;

        let mut user_id = None;
        for (i, u) in self.fs_desc.users.iter().enumerate() {
            if u.name == user.into_array()? {
                user_id = Some(i);
                break;
            }
        }

        match user_id {
            None => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Can't find user {}", user),
                ))
            }
            Some(user) => {
                let mut inode = self.get_inode(entry.dir_entry.i_node)?;

                if inode.i_mode.owner as usize != self.user && self.user != 0 {
                    return Err(Error::new(ErrorKind::PermissionDenied, "Permission Denied"));
                }
                inode.i_mode.owner = user as u8;
                self.write_inode(entry.dir_entry.i_node, inode)?;
                Ok(())
            }
        }
    }
}
