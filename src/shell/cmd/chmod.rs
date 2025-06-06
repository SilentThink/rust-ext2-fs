use super::Cmd;
use crate::fs::*;
use utils::str;

pub struct Chmod;

impl Chmod {
    fn chmod_recursively(fs: &mut Fs, path: &str, mode: u8) {       
        // 尝试打开文件
        match fs.open(path) {
            // 文件打开成功
            Ok(fd) => {
                // 尝试修改文件权限
                if let Err(e) = fs.chmod(path, mode) {
                    println!("{}: {}", path, e.to_string())
                }
                // 关闭文件
                fs.close(fd).unwrap();
                // 返回
                return;
            }
            // 文件打开失败
            Err(e) => match e.kind() {
                // 其他错误
                std::io::ErrorKind::Other => {}
                // 其他错误
                _ => println!("{}: {}", path, e.to_string()),
            },
        }

        // 创建一个空数组用于存储文件名
        let mut names = Vec::new();

        // 解析路径
        match fs.path_parse(path) {
            Ok(path_res) => match path_res.dir_entry.iter(fs) {
                Ok(iter) => {
                    // 遍历目录项
                    for item in iter {
                        // 如果目录项是使用中的
                        if let DirEntryIterItem::Using(Item { entry, .. }) = item {
                            // 如果目录项是当前目录或父目录
                            if entry.name == ".".into_array().unwrap()
                                || entry.name == "..".into_array().unwrap()
                            {
                                // 跳过
                                continue;
                            }
                            // 添加文件名到数组
                            names.push(str(&entry.name).to_string());
                        }
                    }
                }
                Err(e) => println!("{}: {}", path, e.to_string()),
            },
            Err(e) => println!("{}: {}", path, e.to_string()),
        }

        // 递归修改文件权限
        for to_delete in names {
            // 递归修改文件权限
            Self::chmod_recursively(fs, &format!("{}/{}", path, to_delete), mode);
        }

        // 尝试修改文件权限
        if let Err(e) = fs.chmod(path, mode) {
            println!("{}: {}", path, e);
        }
    }
}

impl Cmd for Chmod {
    // 返回命令的描述信息
    fn description(&self) -> String {
        "Change permission mode for files.".into()
    }

    // 实现命令的运行逻辑
    fn run(&self, shell: &mut crate::shell::Shell, argv: &[&str]) {
        // 将参数转换为向量
        let mut argv = argv.to_vec();
        // 是否递归
        let mut recursively = false;
        // 文件权限
        let mut mode: u8 = 0xff;

        // 如果参数为空，返回
        if argv.len() == 0 {
            return;
        }

        // 如果参数为 -r，设置递归
        if let Some(&"-r") = argv.get(0) {
            recursively = true;
            argv.remove(0);
        }

        // 如果参数为文件权限，设置文件权限
        if let Some(m) = argv.get(0) {
            match FileMode::str_to_mode(m) {
                // 如果文件权限转换成功，设置文件权限
                Ok(m) => mode = m,
                // 如果文件权限转换失败，打印错误信息并返回
                Err(e) => {
                    println!("{}", e);
                    return;
                }
            }
            // 移除文件权限
            argv.remove(0);
        }

        // 遍历参数
        for path in argv {
            match recursively {
                // 如果递归，递归修改文件权限
                true => Chmod::chmod_recursively(&mut shell.fs, path, mode),
                // 如果非递归，修改文件权限
                false => {
                    // 如果修改文件权限失败，打印错误信息
                    if let Err(e) = shell.fs.chmod(path, mode) {
                        println!("{}: {}", path, e);
                    }
                }
            }
        }
    }

    // 返回命令的帮助信息
    fn help(&self) -> String {
        // 返回命令的描述信息
        self.description()
            // 返回命令的帮助信息
            + "\n chmod [-r] [mode] [files...]"
            // 返回命令的帮助信息
            + "\n -r change permission recursively."
            // 返回命令的帮助信息
            + "\n\n [mode] is a string like this rwx:rwx"
    }
}
