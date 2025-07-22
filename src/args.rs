use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "BMi-WSProxy", version, about = "WebSocket Gateway Proxy with HAProxy support")]
pub struct Args {
    #[arg(long = "addr", default_value = "0.0.0.0")]
    pub addr: String,

    #[arg(long = "port")]
    pub port: u16,

    #[arg(long = "frkey")]
    pub frkey: Option<String>,  // 改为可选
    
    #[arg(long = "secret")]
    pub secret: Option<String>,

    #[arg(long = "aes-only")]
    pub aes_only: bool,

    #[arg(long = "haproxy-protocol")]
    pub haproxy_protocol: bool,

    #[arg(long = "stream", default_value = "bin")]
    pub stream: String,

    #[arg(long = "buffer", default_value = "1024")]
    pub buffer: usize,

    #[arg(long = "timeout", default_value = "3")]
    pub timeout: u64,

    #[arg(long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    // 新增 realip_header 配置
    #[arg(long = "realip-header", default_value = "X-Real-IP")]
    pub realip_header: String,
}