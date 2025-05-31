//! GUI模块，提供基于Web的文件系统可视化界面

use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use log::info;
use chrono::TimeZone;

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

/// 用于捕获命令输出的结构
struct OutputCapture {
    output: String,
}

impl OutputCapture {
    fn new() -> Self {
        Self { output: String::new() }
    }
    
    fn add_line(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
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
            
            // 创建输出捕获
            let output_capture = OutputCapture::new();
            
            // 使用闭包捕获输出
            let original_print = std::io::stdout();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                // 执行命令
                cmd.run(&mut *shell, &args);
            }));
            
            // 处理执行结果
            match result {
                Ok(_) => {
                    // 如果是pwd命令，直接返回当前路径
                    if cmd_req.cmd == "pwd" {
                        if let Ok(path) = shell.fs.pwd() {
                            Ok(HttpResponse::Ok().json(CommandResponse {
                                success: true,
                                output: path,
                            }))
                        } else {
                            Ok(HttpResponse::InternalServerError().json(CommandResponse {
                                success: false,
                                output: "获取当前路径失败".to_string(),
                            }))
                        }
                    } else {
                        // 对于其他命令，返回成功
                        Ok(HttpResponse::Ok().json(CommandResponse {
                            success: true,
                            output: format!("执行命令: {} {:?} 成功", cmd_req.cmd, args),
                        }))
                    }
                },
                Err(_) => Ok(HttpResponse::InternalServerError().json(CommandResponse {
                    success: false,
                    output: format!("执行命令: {} {:?} 失败", cmd_req.cmd, args),
                })),
            }
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