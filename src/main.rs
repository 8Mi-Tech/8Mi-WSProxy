mod args;
mod proxy;
mod utils;
mod log;

use crate::args::Args;
use crate::proxy::run_proxy;
use clap::Parser;
use std::process;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.version {
        println!("BMi-WSProxy 0.1.0");
        return;
    }

    if args.aes_only && args.secret.is_none() {
        eprintln!("错误: --aes-only 需要 --secret 参数");
        std::process::exit(1);
    }

    // 获取当前进程ID
    let pid = process::id();
    
    // 打印启动信息
    log::print_startup_info(&args, pid);

    if let Err(e) = run_proxy(args).await {
        eprintln!("[!] 启动代理失败: {e}");
    }
}