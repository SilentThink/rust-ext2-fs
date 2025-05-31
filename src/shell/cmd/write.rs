use crate::fs::*;
pub struct Write;

impl Write {
    fn main(fs: &mut Fs, path: &str) -> Result<()> {
        let fd = fs.open(path)?;

        let res = fs.cut(fd, 0);
        if res.is_err() {
            fs.close(fd)?;
            return res;
        }

        println!("Input content of file now, Press Ctrl+D will save the file.");
        println!("-----------------------------------------------------------");
        println!("");

        loop {
            let mut content = String::new();
            if std::io::stdin().read_line(&mut content).unwrap() == 0 {
                break;
            }
            fs.write(fd, content.as_bytes())?;
        }

        fs.close(fd)?;

        println!("");
        println!("-----------------------------------------------------------");
        println!("{} Saved.", path);
        Ok(())
    }
}

impl super::Cmd for Write {
    fn description(&self) -> String {
        "Write something into file".to_string()
    }

    fn run(&self, crate::shell::Shell { fs, .. }: &mut crate::shell::Shell, argv: &[&str]) {
        match argv.len() {
            0 => println!("Should receive a file to write"),
            1 => {
                if let Err(e) = Self::main(fs, argv[0]) {
                    println!("{}: {}", argv[0], e);
                }
            }
            _ => println!("Too many argument"),
        }
    }
}
