use super::Cmd;
use crate::fs::*;
use utils::str;

pub struct Chmod;

impl Chmod {
    fn chmod_recursively(fs: &mut Fs, path: &str, mode: u8) {
        match fs.open(path) {
            Ok(fd) => {
                if let Err(e) = fs.chmod(path, mode) {
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

        for to_delete in names {
            Self::chmod_recursively(fs, &format!("{}/{}", path, to_delete), mode);
        }

        if let Err(e) = fs.chmod(path, mode) {
            println!("{}: {}", path, e);
        }
    }
}

impl Cmd for Chmod {
    fn description(&self) -> String {
        "Change permission mode for files.".into()
    }

    fn run(&self, shell: &mut crate::shell::Shell, argv: &[&str]) {
        let mut argv = argv.to_vec();
        let mut recursively = false;
        let mut mode: u8 = 0xff;

        if argv.len() == 0 {
            return;
        }

        if let Some(&"-r") = argv.get(0) {
            recursively = true;
            argv.remove(0);
        }

        if let Some(m) = argv.get(0) {
            match FileMode::str_to_mode(m) {
                Ok(m) => mode = m,
                Err(e) => {
                    println!("{}", e);
                    return;
                }
            }
            argv.remove(0);
        }

        for path in argv {
            match recursively {
                true => Chmod::chmod_recursively(&mut shell.fs, path, mode),
                false => {
                    if let Err(e) = shell.fs.chmod(path, mode) {
                        println!("{}: {}", path, e);
                    }
                }
            }
        }
    }

    fn help(&self) -> String {
        self.description()
            + "\n chmod [-r] [mode] [files...]"
            + "\n -r change permission recursively."
            + "\n\n [mode] is a string like this rwx:rwx"
    }
}
