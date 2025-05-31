use chrono::TimeZone;
use super::Cmd;
use super::Shell;
use crate::fs::*;
use crate::utils::pretty_byte;
use crossterm::style::Stylize;

#[derive(Clone)]
pub struct Ls;

impl Ls {
    fn main(fs: &mut Fs, l_option: bool, path: Option<&str>) -> Result<()> {
        let users = &fs.fs_desc().users;

        let mut output: Vec<[String; 6]> = Vec::new();
        output.push([
            "Name".blue().to_string(),
            "Mode".into(),
            "Owner".into(),
            "Size".into(),
            "Create Time".into(),
            "Edit Time".into()
        ]);
        let (mut file_w, mut size_w, mut owner_w, mut mode_w, mut time_w) =
            ("Name".blue().to_string().len(), 4, 5, 4, 0);

        for item in fs
            .path_parse(path.unwrap_or_default())?
            .dir_entry
            .iter(fs)?
        {
            if let DirEntryIterItem::Using(Item { entry, .. }) = item {
                let mut filename = utils::str(&entry.name).to_string();
                match entry.file_type.into() {
                    FileType::Dir => filename = filename.blue().to_string(),
                    FileType::File => filename = filename.green().to_string(),
                }
                file_w = file_w.max(filename.len());

                let i_node = fs.get_inode(entry.i_node)?;
                let file_type = match entry.file_type.into() {
                    FileType::File => "f",
                    FileType::Dir => "d",
                };
                let mode = format!("[{}].{}", file_type, i_node.i_mode);
                mode_w = mode_w.max(mode.len());

                let owner = users
                    .get(i_node.i_mode.owner as usize)
                    .map(|s| utils::str(&s.name))
                    .unwrap_or("???")
                    .to_string();
                owner_w = owner_w.max(owner.len());

                let size = pretty_byte(i_node.i_size);
                size_w = size.len().max(size_w);

                let create_time = chrono::Utc.timestamp_opt(i_node.i_ctime as i64, 0).unwrap().to_string();
                let edit_time = chrono::Utc.timestamp_opt(i_node.i_mtime as i64, 0).unwrap().to_string();
                time_w = time_w.max(edit_time.len());

                output.push([filename, mode, owner, size, create_time, edit_time])
            }
        }

        for line in output {
            if l_option {
                println!(
                    "{:<file_w$}  {:<mode_w$}  {:<owner_w$}  {:<size_w$}  {:<time_w$}  {:<time_w$}",
                    line[0], line[1], line[2], line[3], line[4], line[5]
                );
            } else {
                println!("{}", line[0])
            }
        }
        Ok(())
    }
}

impl Cmd for Ls {
    fn description(&self) -> String {
        "List contents in directory".into()
    }

    fn run(&self, Shell { fs, .. }: &mut Shell, argv: &[&str]) {
        let mut filename: Option<&str> = None;
        let mut l_option = false;

        for &arg in argv {
            match arg {
                "-l" => l_option = true,
                "-h" => {
                    println!("{}", self.help());
                    return;
                }
                name => filename = Some(name),
            }
        }
        if let Err(e) = Ls::main(fs, l_option, filename) {
            println!("{}", e.to_string())
        }
    }

    fn help(&self) -> String {
        self.description()
            + r#"

ls [-lh] <file>
-l  Display extend information about file
-h  Display this help messega"#
    }
}
