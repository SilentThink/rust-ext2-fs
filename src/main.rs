use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    // 解析命令行参数
    let args: Vec<String> = env::args().collect();
    
    // 创建Shell实例
    let mut shell = simulate_unixlike_fs::shell::Shell::new();
    
    // 如果有--web参数，启动Web服务器，否则启动传统Shell
    if args.len() > 1 && args[1] == "--web" {
        println!("启动Web界面模式，请访问 http://localhost:8080");
        simulate_unixlike_fs::gui::start_server(shell).await
    } else {
        // 传统Shell模式
        shell.run();
        Ok(())
    }
}
