//! GUI模块，提供基于Web的文件系统可视化界面

use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use log::info;
use chrono::TimeZone;
use std::io::{self, Write};

use crate::shell::Shell;

/// 共享的Shell实例，用于在Web服务器中执行命令
type SharedShell = Arc<Mutex<Shell>>;

/// 文件或目录项的信息
#[derive(Serialize)]
pub struct FileItem {
    name: String,
    is_dir: bool,
    size: String,
    owner: String,
    mode: String,
    create_time: String,
    edit_time: String,
}

/// 目录内容响应
#[derive(Serialize)]
pub struct DirectoryContent {
    path: String,
    items: Vec<FileItem>,
}

/// 命令请求
#[derive(Deserialize)]
pub struct CommandRequest {
    cmd: String,
    args: Vec<String>,
}

/// 命令响应
#[derive(Serialize)]
pub struct CommandResponse {
    success: bool,
    output: String,
}

/// 获取当前目录内容
async fn get_current_directory(shell: web::Data<SharedShell>) -> Result<impl Responder> {
    let mut shell = shell.lock().unwrap();
    
    // 获取当前路径
    let path = match shell.fs.pwd() {
        Ok(path) => path,
        Err(e) => return Ok(HttpResponse::InternalServerError().json(format!("获取当前路径失败: {}", e))),
    };
    
    // 获取目录内容
    let mut items = Vec::new();
    
    // 使用fs的API直接获取目录内容
    match shell.fs.path_parse("") {
        Ok(parsed_path) => {
            let users = &shell.fs.fs_desc().users;
            
            // 处理目录项迭代器，如果出错则使用空迭代器
            if let Ok(dir_entries) = parsed_path.dir_entry.iter(&shell.fs) {
                for item in dir_entries {
                if let crate::fs::DirEntryIterItem::Using(crate::fs::Item { entry, .. }) = item {
                    let filename = crate::fs::utils::str(&entry.name).to_string();
                    
                    // 跳过重复的父目录引用
                    if filename == ".." && items.iter().any(|item: &FileItem| item.name == "..") {
                        continue;
                    }
                    
                    let is_dir = match entry.file_type.into() {
                        crate::fs::FileType::Dir => true,
                        crate::fs::FileType::File => false,
                    };
                    
                    // 获取inode信息
                    if let Ok(i_node) = shell.fs.get_inode(entry.i_node) {
                        let file_type = if is_dir { "d" } else { "f" };
                        let mode = format!("[{}].{}", file_type, i_node.i_mode);
                        
                        let owner = users
                            .get(i_node.i_mode.owner as usize)
                            .map(|s| crate::fs::utils::str(&s.name))
                            .unwrap_or("???")
                            .to_string();
                        
                        let size = crate::utils::pretty_byte(i_node.i_size);
                        
                        let create_time = chrono::Utc.timestamp_opt(i_node.i_ctime as i64, 0).unwrap().to_string();
                        let edit_time = chrono::Utc.timestamp_opt(i_node.i_mtime as i64, 0).unwrap().to_string();
                        
                        items.push(FileItem {
                            name: filename,
                            is_dir,
                            size,
                            owner,
                            mode,
                            create_time,
                            edit_time,
                        });
                    }
                }
                }
            }
        },
        Err(e) => return Ok(HttpResponse::InternalServerError().json(format!("解析路径失败: {}", e))),
    }
    
    Ok(HttpResponse::Ok().json(DirectoryContent {
        path,
        items,
    }))
}

/// 执行cd命令
async fn change_directory(shell: web::Data<SharedShell>, path: web::Json<String>) -> Result<impl Responder> {
    let mut shell = shell.lock().unwrap();
    
    match shell.fs.chdir(&path) {
        Ok(_) => {
            let current_path = shell.fs.pwd().unwrap_or_else(|_| "Unknown".to_string());
            Ok(HttpResponse::Ok().json(CommandResponse {
                success: true,
                output: current_path,
            }))
        },
        Err(e) => Ok(HttpResponse::BadRequest().json(CommandResponse {
            success: false,
            output: e.to_string(),
        })),
    }
}

/// 执行通用命令
async fn execute_command(shell: web::Data<SharedShell>, cmd_req: web::Json<CommandRequest>) -> Result<impl Responder> {
    let mut shell = shell.lock().unwrap();
    let cmds = shell.cmds.clone();
    
    // 将String转换为&str以匹配BTreeMap的键类型
    match cmds.get(cmd_req.cmd.as_str()) {
        Some(cmd) => {
            // 将String转换为&str
            let args: Vec<&str> = cmd_req.args.iter().map(|s| s.as_str()).collect();
            
            // 使用特殊处理的命令
            let mut output_text = String::new();
            let mut handled = false;
            let mut success = true;
            
            // 对于pwd命令，使用特殊处理
            if cmd_req.cmd == "pwd" {
                handled = true;
                if let Ok(path) = shell.fs.pwd() {
                    output_text = path;
                } else {
                    return Ok(HttpResponse::InternalServerError().json(CommandResponse {
                        success: false,
                        output: "获取当前路径失败".to_string(),
                    }));
                }
            } else if cmd_req.cmd == "help" || cmd_req.cmd == "?" {
                // 对help命令特殊处理
                handled = true;
                let mut help_output = String::new();
                for (&name, cmd) in shell.cmds.iter() {
                    help_output.push_str(&format!("{:12}  {}\n", name, cmd.description()));
                }
                output_text = help_output;
            } else if cmd_req.cmd == "ls" {
                // 列出当前目录内容
                handled = true;
                let mut ls_output = String::new();
                
                // 获取目录内容
                if let Ok(parsed_path) = shell.fs.path_parse("") {
                    if let Ok(dir_entries) = parsed_path.dir_entry.iter(&shell.fs) {
                        for item in dir_entries {
                            if let crate::fs::DirEntryIterItem::Using(crate::fs::Item { entry, .. }) = item {
                                let filename = crate::fs::utils::str(&entry.name).to_string();
                                
                                // 获取文件类型
                                let file_type = match entry.file_type.into() {
                                    crate::fs::FileType::Dir => "[目录]",
                                    crate::fs::FileType::File => "[文件]",
                                };
                                
                                ls_output.push_str(&format!("{} {}\n", file_type, filename));
                            }
                        }
                    }
                }
                
                output_text = ls_output;
            } else if cmd_req.cmd == "cat" {
                // 对cat命令特殊处理，直接读取文件内容
                handled = true;
                let mut cat_output = String::new();
                
                for arg in &args {
                    match shell.fs.open(arg) {
                        Ok(fd) => {
                            let mut file_content = String::new();
                            let mut has_error = false;
                            
                            loop {
                                let mut buf = [0u8; 512];
                                match shell.fs.read(fd, &mut buf) {
                                    Ok(bytes) => {
                                        if bytes == 0 {
                                            break;
                                        } else {
                                            file_content.push_str(crate::utils::str(&buf[0..bytes]));
                                        }
                                    }
                                    Err(e) => {
                                        cat_output.push_str(&format!("{}: {}\n", arg, e));
                                        has_error = true;
                                        break;
                                    }
                                }
                            }
                            
                            if !has_error {
                                cat_output.push_str(&file_content);
                            }
                            
                            shell.fs.close(fd).unwrap_or_default();
                        }
                        Err(e) => cat_output.push_str(&format!("{}: {}\n", arg, e)),
                    }
                }
                
                output_text = cat_output;
            } else if cmd_req.cmd == "whoami" {
                // 获取当前用户
                handled = true;
                let fs_desc = shell.fs.fs_desc();
                let user_index = shell.fs.current_user();
                if user_index < fs_desc.users.len() {
                    let name = &fs_desc.users[user_index].name;
                    output_text = crate::fs::utils::str(name).to_string();
                } else {
                    output_text = "未登录".to_string();
                }
            } else if cmd_req.cmd == "users" {
                // 列出所有用户
                handled = true;
                let mut users_output = String::new();
                for user in &shell.fs.fs_desc().users {
                    users_output.push_str(&format!("{} {}\n", 
                        crate::fs::utils::str(&user.name),
                        crate::fs::utils::str(&user.password)));
                }
                output_text = users_output;
            } else if cmd_req.cmd == "write" {
                // 特殊处理write命令
                handled = true;
                
                // 检查参数
                if args.is_empty() {
                    output_text = "请指定要写入的文件名".to_string();
                    success = false;
                } else {
                    let filename = args[0];
                    
                    // 如果有第二个参数，则认为是要写入的内容
                    if args.len() > 1 {
                        let content = args[1]; // 第二个参数是内容
                        
                        // 打开文件
                        match shell.fs.open(filename) {
                            Ok(fd) => {
                                // 清空文件
                                if let Err(e) = shell.fs.cut(fd, 0) {
                                    output_text = format!("清空文件失败: {}", e);
                                    success = false;
                                } else {
                                    // 写入内容
                                    match shell.fs.write(fd, content.as_bytes()) {
                                        Ok(_) => {
                                            output_text = format!("文件 {} 已保存", filename);
                                        },
                                        Err(e) => {
                                            output_text = format!("写入文件失败: {}", e);
                                            success = false;
                                        }
                                    }
                                }
                                
                                // 关闭文件
                                let _ = shell.fs.close(fd);
                            },
                            Err(e) => {
                                output_text = format!("打开文件失败: {}", e);
                                success = false;
                            }
                        }
                    } else {
                        // 如果没有提供内容，返回提示信息
                        output_text = format!("请在Web界面使用格式: write 文件名 内容");
                        success = false;
                    }
                }
            } else if cmd_req.cmd == "rmdir" || cmd_req.cmd == "rm" || cmd_req.cmd == "mkdir" || 
                    cmd_req.cmd == "touch" || cmd_req.cmd == "chmod" || cmd_req.cmd == "chown" || 
                    cmd_req.cmd == "useradd" || cmd_req.cmd == "userdel" || cmd_req.cmd == "passwd" {
                // 执行文件系统修改命令
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    cmd.run(&mut *shell, &args);
                }));
                
                if result.is_ok() {
                    output_text = format!("命令 {} 执行成功", cmd_req.cmd);
                } else {
                    success = false;
                    output_text = format!("命令 {} 执行失败", cmd_req.cmd);
                }
                handled = true;
            }
            
            // 如果没有特殊处理，使用默认处理
            if !handled {
                // 创建临时文件以捕获输出
                let temp_dir = std::env::temp_dir();
                let temp_file_path = temp_dir.join(format!("ext2fs_cmd_output_{}.txt", std::process::id()));
                
                // 创建临时文件
                let file = match std::fs::File::create(&temp_file_path) {
                    Ok(file) => file,
                    Err(_) => {
                        return Ok(HttpResponse::InternalServerError().json(CommandResponse {
                            success: false,
                            output: format!("无法创建临时文件捕获输出"),
                        }));
                    }
                };
                
                // 设置临时输出
                let mut output_handle = std::io::BufWriter::new(file);
                
                // 执行命令
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    // 备份标准输出
                    let backup_stdout = io::stdout();
                    
                    // 临时重定向标准输出
                    let _result = writeln!(output_handle, "执行命令: {} {:?}", cmd_req.cmd, args);
                    output_handle.flush().unwrap_or_default();
                    
                    // 执行命令
                    cmd.run(&mut *shell, &args);
                    
                    // 恢复标准输出
                    drop(backup_stdout);
                }));
                
                // 关闭输出文件
                drop(output_handle);
                
                // 读取临时文件内容
                match std::fs::read_to_string(&temp_file_path) {
                    Ok(content) => {
                        output_text = content;
                        success = result.is_ok();
                    },
                    Err(_) => {
                        output_text = format!("执行命令 {} 成功，但无法读取输出", cmd_req.cmd);
                        success = result.is_ok();
                    }
                }
                
                // 删除临时文件
                let _ = std::fs::remove_file(temp_file_path);
            }
            
            // 如果输出为空，但命令执行成功，添加成功消息
            if output_text.trim().is_empty() && success {
                output_text = format!("命令 {} 执行成功", cmd_req.cmd);
            }
            
            Ok(HttpResponse::Ok().json(CommandResponse {
                success,
                output: output_text,
            }))
        },
        None => Ok(HttpResponse::BadRequest().json(CommandResponse {
            success: false,
            output: format!("未知命令: {}", cmd_req.cmd),
        })),
    }
}

/// 启动Web服务器
pub async fn start_server(shell: Shell) -> std::io::Result<()> {
    let shared_shell = Arc::new(Mutex::new(shell));
    
    info!("启动Web服务器，监听 127.0.0.1:8080");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(shared_shell.clone()))
            .service(web::resource("/api/directory").route(web::get().to(get_current_directory)))
            .service(web::resource("/api/cd").route(web::post().to(change_directory)))
            .service(web::resource("/api/command").route(web::post().to(execute_command)))
            .service(fs::Files::new("/", "./src/gui/static").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}