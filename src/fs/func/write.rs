use crate::fs::core::*;

impl Fs {
    pub fn write(&mut self, fd: usize, buf: &[u8]) -> Result<usize> {
        if fd >= self.fds.len() || self.fds[fd].is_none() {
            return Err(Error::new(ErrorKind::Other, "Bad file description"));
        }

        let mut file = self.fds[fd].clone().unwrap();

        if !file.inode.i_mode.can_write(self.user) {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "Permission Denied.",
            ));
        }

        let mut counter = 0;

        loop {
            if counter == buf.len() {
                break;
            }

            if file.current_pos >= file.inode.i_size as usize {
                file.inode.i_size = (file.current_pos + 1) as u32;

                let mut new_blocks = (file.inode.i_size / BLOCK_SIZE as u32) as u16;
                if file.inode.i_size % BLOCK_SIZE as u32 != 0 {
                    new_blocks += 1;
                }

                let block_to_alloc = new_blocks - file.inode.i_blocks;

                for _ in 0..block_to_alloc {
                    file.inode.alloc_data_block(self)?;
                }
            }

            let addr = file
                .inode
                .convert_addr(&self.disk, file.current_pos as u64)?;
            self.disk.write_at(&[buf[counter]], addr.addr)?;

            counter += 1;
            file.current_pos += 1;
        }

        file.inode.i_mtime = utils::now();

        // 写入更新后的索引节点
        self.write_inode(file.inode_i, file.inode.clone())?;

        self.fds[fd] = Some(file);

        Ok(counter)
    }
}

#[test]
fn test_read_write() {
    let part1 = r#"fn main() {
    println!("Hello World");
}

// This is simple Hello World program in Rust
// You can use `cargo new` to create rust project.

// So, seems this functions works properly........
// Seems great.

fn main() {
    println!("Hello World");
}

// This is simple Hello World program in Rust
// You can use `cargo new` to create rust project.

// So, seems this functions works properly........
// Seems great.
fn main() {
    println!("Hello World");
}

// This is simple Hello World program in Rust
// You can use `cargo new` to create rust project.

// So, seems this functions works properly........
// Seems great.
"#;

    let mut fs = Fs::format().unwrap();
    fs.create("test.txt").unwrap();
    let fd = fs.open("test.txt").unwrap();
    fs.write(fd, part1.as_bytes()).unwrap();
    fs.write(fd, part1.as_bytes()).unwrap();
    let fd2 = fs.open("test.txt").unwrap();

    let mut str: Vec<u8> = Vec::new();
    let mut buf = [0u8; 13];
    while fs.read(fd2, &mut buf).unwrap() != 0 {
        str.extend(buf.iter());
        buf.fill(0);
    }
    let str = std::str::from_utf8(&str).unwrap_or("invaild utf-8 string");
    println!("{}", str);
    assert_eq!(str.trim_matches('\0'), format!("{}{}", part1, part1));
}
