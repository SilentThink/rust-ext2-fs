use super::Cmd;
use crate::fs::core::*;
use crate::utils::str;

pub struct Rm;

impl Rm {
    fn rm_recursively(fs: &mut Fs, path: &str) {
        if path == "." || path == ".." {
            println!("{}: Can't delete directory . and ..", path)
        }

        match fs.open(path) {
            Ok(fd) => {
                if let Err(e) = fs.rm(fd) {
                    println!("{}: {}", path, e.to_string())
                }
                return;
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::Other => {}
                _ => println!("{}: {}", path, e.to_string()),
            },
        }

        let mut names = Vec::new();

        match fs.path_parse(path) {
            Ok(path_res) => match path_res.dir_entry.iter(fs) {
                Ok(iter) => {
                    for item in iter {
                        if let DirEntryIterItem::Using(Item { entry, .. }) = item {
                            if entry.name == ".".into_array().unwrap()
                                || entry.name == "..".into_array().unwrap()
                            {
                                continue;
                            }
                            names.push(str(&entry.name).to_string());
                        }
                    }
                }
                Err(e) => println!("{}: {}", path, e.to_string()),
            },
            Err(e) => println!("{}: {}", path, e.to_string()),
        }

        for to_delete in names {
            Self::rm_recursively(fs, &format!("{}/{}", path, to_delete));
        }

        if let Err(e) = fs.rmdir(path) {
            println!("{}: {}", path, e);
        }
    }
}

impl Cmd for Rm {
    fn description(&self) -> String {
        "Delete file".to_string()
    }

    fn run(&self, crate::shell::Shell { fs, .. }: &mut crate::shell::Shell, argv: &[&str]) {
        let dir = fs.pwd().unwrap();

        if argv.len() == 0 {
            return;
        }

        if argv[0] == "-r" {
            if argv.len() >= 2 {
                for &arg in &argv[1..] {
                    Self::rm_recursively(fs, &format!("{}/{}", dir, arg));
                }
                return;
            }
        }

        if argv[0] == "-h" {
            println!("{}", self.help());
            return;
        }

        for arg in argv {
            let mut err = None;
            match fs.open(arg) {
                Ok(fd) => {
                    if let Err(e) = fs.rm(fd) {
                        println!("{}", e.to_string())
                    }
                }
                Err(e) => match e.kind() {
                    std::io::ErrorKind::Other => {
                        if let Err(e) = fs.rmdir(arg) {
                            err = Some(e.to_string())
                        }
                    }
                    _ => err = Some(e.to_string()),
                },
            }
            if let Some(err) = err {
                println!("{}/{}: {}", dir, arg, err);
            }
        }
    }

    fn help(&self) -> String {
        self.description()
            + r#"
 -h show this message
 -r delete files recursively"#
    }
}
