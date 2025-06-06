use chrono::{TimeZone, FixedOffset}; // 引入 chrono 库，用于处理时间和时区
use super::Cmd; // 引入 Cmd 特性
use super::Shell; // 引入 Shell 结构体
use crate::fs::*; // 引入文件系统模块
use crate::utils::pretty_byte; // 引入工具模块中的 pretty_byte 函数，用于格式化字节大小
use crossterm::style::Stylize; // 引入 crossterm 库的 Stylize 特性，用于格式化文本颜色

#[derive(Clone)] // 为 Ls 结构体派生 Clone 特性
pub struct Ls; // 定义一个名为 Ls 的结构体

impl Ls {
    fn main(fs: &mut Fs, l_option: bool, path: Option<&str>) -> Result<()> {
        // 定义 Ls 的主逻辑函数，接收文件系统引用、是否显示详细信息的标志和路径
        let users = &fs.fs_desc().users; // 获取文件系统描述符中的用户信息

        let mut output: Vec<[String; 6]> = Vec::new(); // 创建一个用于存储输出内容的二维字符串数组
        output.push([
            "Name".blue().to_string(), // 添加表头，文件名用蓝色显示
            "Mode".into(), // 文件模式
            "Owner".into(), // 文件所有者
            "Size".into(), // 文件大小
            "Create Time".into(), // 文件创建时间
            "Edit Time".into() // 文件修改时间
        ]);
        let (mut file_w, mut size_w, mut owner_w, mut mode_w, mut time_w) =
            ("Name".blue().to_string().len(), 4, 5, 4, 0);
        // 初始化字段宽度变量，用于后续对齐输出

        for item in fs
            .path_parse(path.unwrap_or_default())? // 解析路径，如果路径为空则使用默认值
            .dir_entry
            .iter(fs)? // 遍历目录条目
        {
            if let DirEntryIterItem::Using(Item { entry, .. }) = item {
                // 如果是有效的目录条目
                let mut filename = utils::str(&entry.name).to_string(); // 获取文件名
                match entry.file_type.into() { // 根据文件类型设置颜色
                    FileType::Dir => filename = filename.blue().to_string(), // 目录用蓝色显示
                    FileType::File => filename = filename.green().to_string(), // 文件用绿色显示
                    FileType::Symlink => filename = filename.yellow().to_string(), // 软链接用黄色显示
                }
                file_w = file_w.max(filename.len()); // 更新文件名字段宽度

                let i_node = fs.get_inode(entry.i_node)?; // 获取文件的 inode 信息
                let file_type = match entry.file_type.into() { // 获取文件类型标识
                    FileType::Dir => "d", // 目录
                    FileType::File => "-", // 文件
                    FileType::Symlink => "l", // 软链接
                };
                let mode = format!("{}{}", file_type, i_node.i_mode); // 组合文件类型和权限模式
                mode_w = mode_w.max(mode.len()); // 更新模式字段宽度

                let owner = users
                    .get(i_node.i_mode.owner as usize) // 获取文件所有者信息
                    .map(|s| utils::str(&s.name))
                    .unwrap_or("???")
                    .to_string();
                owner_w = owner_w.max(owner.len()); // 更新所有者字段宽度

                let size = pretty_byte(i_node.i_size); // 格式化文件大小
                size_w = size.len().max(size_w); // 更新大小字段宽度

                // 创建中国时区（东八区，UTC+8）
                let china_tz = FixedOffset::east_opt(8 * 3600).unwrap();
                let create_time = chrono::Utc.timestamp_opt(i_node.i_ctime as i64, 0).unwrap() // 获取创建时间
                    .with_timezone(&china_tz) // 转换为东八区时区
                    .format("%Y-%m-%d %H:%M:%S CST") // 格式化时间
                    .to_string();
                let edit_time = chrono::Utc.timestamp_opt(i_node.i_mtime as i64, 0).unwrap() // 获取修改时间
                    .with_timezone(&china_tz) // 转换为东八区时区
                    .format("%Y-%m-%d %H:%M:%S CST") // 格式化时间
                    .to_string();
                time_w = time_w.max(edit_time.len()); // 更新时间字段宽度

                // 如果是软链接，显示链接目标
                let symlink_type: u8 = FileType::Symlink.into();
                if entry.file_type == symlink_type {
                    // 构建完整路径
                    let full_path = if path.unwrap_or_default().is_empty() {
                        utils::str(&entry.name).to_string()
                    } else {
                        format!("{}/{}", path.unwrap_or_default(), utils::str(&entry.name))
                    };
                    
                    // 读取软链接的目标路径
                    let target = match fs.read_symlink_target(&full_path) {
                        Ok(target) => target,
                        Err(_) => "invalid link".to_string(),
                    };
                    
                    // 修改文件名以显示链接目标
                    let filename_with_target = format!("{} -> {}", filename, target);
                    output.push([filename_with_target, mode, owner, size, create_time, edit_time]);
                } else {
                    output.push([filename, mode, owner, size, create_time, edit_time])
                }
            }
        }

        for line in output {
            if l_option {
                // 如果启用了详细信息选项，按照字段宽度对齐输出
                println!(
                    "{:<file_w$}  {:<mode_w$}  {:<owner_w$}  {:<size_w$}  {:<time_w$}  {:<time_w$}",
                    line[0], line[1], line[2], line[3], line[4], line[5]
                );
            } else {
                println!("{}", line[0]) // 否则只输出文件名
            }
        }
        Ok(())
    }
}

impl Cmd for Ls {
    fn description(&self) -> String {
        "List contents in directory".into() // 返回命令的描述信息
    }

    fn run(&self, Shell { fs, .. }: &mut Shell, argv: &[&str]) {
        // 命令的运行逻辑
        let mut filename: Option<&str> = None; // 初始化文件名变量
        let mut l_option = false; // 初始化详细信息选项标志

        for &arg in argv {
            match arg {
                "-l" => l_option = true, // 如果是 "-l" 选项，启用详细信息模式
                "-h" => {
                    println!("{}", self.help()); // 如果是 "-h" 选项，显示帮助信息
                    return;
                }
                name => filename = Some(name), // 其他参数视为文件名
            }
        }
        if let Err(e) = Ls::main(fs, l_option, filename) {
            // 调用主逻辑函数，如果出错则打印错误信息
            println!("{}", e.to_string())
        }
    }

    fn help(&self) -> String {
        // 返回命令的帮助信息
        self.description()
            + r#"

ls [-lh] <file>
-l  Display extend information about file
-h  Display this help message"#
    }
}