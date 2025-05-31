use crate::{utils::str, shell::Shell};
use super::Cmd;

pub struct Cat;

impl Cmd for Cat {
    fn description(&self) -> String {
        "Show content of file".into()
    }

    fn run(&self, shell: &mut Shell, argv: &[&str]) {
        for arg in argv {
            match shell.fs.open(arg) {
                Ok(fd) => {
                    loop {
                        let mut buf = [0u8; 512];
                        match shell.fs.read(fd, &mut buf) {
                            Ok(bytes) => {
                                if bytes == 0 {
                                    break;
                                } else {
                                    print!("{}", str(&buf))
                                }
                            }
                            Err(e) => {
                                println!("{}: {}", arg, e);
                                break;
                            }
                        }
                    }
                    shell.fs.close(fd).unwrap()
                }
                Err(e) => println!("{}: {}", arg, e),
            }
        }
    }
}
