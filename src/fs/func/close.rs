use crate::fs::core::*;

impl Fs {
    pub fn close(&mut self, fd: usize) -> Result<()> {
        if fd >= self.fds.len() || self.fds[fd].is_none() {
            return Err(Error::new(ErrorKind::Other, "Bad file description"));
        }

        self.fds[fd] = None;
        self.opened_len -= 1;

        Ok(())
    }
}
