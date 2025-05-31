use crate::fs::*;
use super::*;

pub struct Cp;

impl Cp {
    fn cp(fs: &mut Fs, src: &str, dest: &str) -> Result<()> {
        if let Err(e) = fs.create(dest) {
            println!("{}: {}", dest, e);
            return Err(e);
        };

        let fd_src = fs.open(src)?;
        let fd_dest = fs.open(dest)?;

        loop {
            let mut c = [0u8; 1];
            if fs.read(fd_src, &mut c)? == 0 {
                break;
            }
            fs.write(fd_dest, &c)?;
        }

        fs.close(fd_src)?;
        fs.close(fd_dest)?;
        Ok(())
    }
}

impl Cmd for Cp {
    fn description(&self) -> String {
        "Clone content of type".into()
    }

    fn run(&self, shell: &mut Shell, argv: &[&str]) {
        if argv.len() != 2 {
            println!("Need two arguments.")
        }

        if let Err(e) = Self::cp(&mut shell.fs, argv[0], argv[1]) {
            println!("Copy failed. {}", e)
        }
    }

    fn help(&self) -> String {
        self.description() + "\n cp [src] [dest]"
    }
}
