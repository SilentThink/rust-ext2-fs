use super::Cmd;
use crate::fs::*;
use utils::str;

pub struct Chown;

impl Chown {
    fn chown_recursively(fs: &mut Fs, path: &str, user: &str) {
        match fs.open(path) {
            Ok(fd) => {
                if let Err(e) = fs.chown(path, user) {
                    println!("{}: {}", path, e.to_string())
                }
                fs.close(fd).unwrap();
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

        for to_chown in names {
            Self::chown_recursively(fs, &format!("{}/{}", path, to_chown), user);
        }

        if let Err(e) = fs.chown(path, user) {
            println!("{}: {}", path, e);
        }
    }
}

impl Cmd for Chown {
    fn description(&self) -> String {
        "Change owrner of files.".into()
    }

    fn run(&self, shell: &mut crate::shell::Shell, argv: &[&str]) {
        let mut argv = argv.to_vec();
        let mut recursively = false;

        if argv.len() == 0 {
            return;
        }

        if let Some(&"-r") = argv.get(0) {
            recursively = true;
            argv.remove(0);
        }

        let mut user = None;
        if let Some(&u) = argv.get(0) {
            argv.remove(0);
            user = Some(u);
        }

        if let Some(user) = user {
            for path in argv {
                match recursively {
                    true => Chown::chown_recursively(&mut shell.fs, path, user),
                    false => {
                        if let Err(e) = shell.fs.chown(path, user) {
                            println!("{}: {}", path, e);
                        }
                    }
                }
            }
        }
    }

    fn help(&self) -> String {
        self.description()
            + "\n chown [-r] [username] [files...]"
            + "\n -r change owner recursively."
    }
}
