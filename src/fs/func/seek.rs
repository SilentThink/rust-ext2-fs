use crate::fs::core::*;

pub enum Seek {
    FromStart(usize),
    FromEnd(usize),
    Current(i64),
}

impl Fs {
    pub fn seek(&mut self, fd: usize, seek: Seek) -> Result<usize> {
        if fd >= self.fds.len() || self.fds[fd].is_none() {
            return Err(Error::new(ErrorKind::Other, "Bad file description"));
        }

        let file = self.fds[fd].as_mut().unwrap();

        // seek 需要写权限
        if !file.inode.i_mode.can_write(self.user) {
            return Err(Error::new(ErrorKind::PermissionDenied, "Permission Denied"));
        }

        match seek {
            Seek::FromStart(pos) => file.current_pos = pos,
            Seek::FromEnd(size) => {
                if size > file.inode.i_size as usize {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "Seek failed. Can't set cursor of file to negative",
                    ));
                }
                file.current_pos -= size;
            }
            Seek::Current(offset) => {
                if file.inode.i_size as i64 + offset < 0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Seek failed. Can't set cursor of file to negative",
                    ));
                }
                file.current_pos = (file.current_pos as i64 + offset) as usize;
            }
        }

        Ok(file.current_pos)
    }
}
