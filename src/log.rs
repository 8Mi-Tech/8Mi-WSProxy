use chrono::Local;

pub fn print_startup_info(args: &crate::args::Args, pid: u32) {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S %z");
    let version = "0.1.0";
    let addr = format!("{}:{}", args.addr, args.port);
    let ssl_support = "不支持"; // 目前没有SSL支持
    let proxy_proto = if args.haproxy_protocol { "启用" } else { "禁用" };
    let passphrase = args.secret.as_deref().unwrap_or("无");
    
    // frkey的显示：如果存在则显示为`?{frkey}=`，否则显示`?`
    let url_request = if let Some(frkey) = &args.frkey {
        format!("/?{}=", frkey)
    } else {
        "/?".to_string()
    };

    println!(
        r#"============= WSproxy 运行中: 正常 , [{}] =============
进程ID: {}
版本:        {}
地址:       {}
SSL/TLS:   {}
代理协议:   {}
连接超时:  {}秒
最大连接数:  65536
缓冲区大小:   {}
密钥短语:    {}
URL请求:   {}
============="#,
        now, pid, version, addr, ssl_support, proxy_proto, args.timeout, args.buffer, passphrase, url_request
    );
}

pub fn log_connection(
    start_time: std::time::Instant,
    client_ip: &str,
    src: &str,
    dest: &str,
    method: &str,
    path: &str,
    status: u16,
    referer: &str,
    forwarded_for: &str,
    user_agent: &str,
    user_id: Option<&str>,
) {
    let elapsed = start_time.elapsed();
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    // 将处理时间转换为微秒
    let elapsed_micros = elapsed.as_micros();
    
    // 构造网络路径：格式为 src->dest
    let net_path = format!("{}->{}", src, dest);
    
    // 构造请求行
    let request_line = format!("{} {}", method, path);
    
    // 引用，如果没有则为"-"
    let referer = if referer.is_empty() { "-" } else { referer };
    
    // 用户ID，如果有的话，格式为 "User-Id:xxx"，否则为空字符串
    let user_id_str = if let Some(id) = user_id {
        format!("\"User-Id:{}\"", id)
    } else {
        "\"\"".to_string()
    };
    
    // 确保 forwarded_for 不为空
    let forwarded_for = if forwarded_for.is_empty() {
        "-"
    } else {
        forwarded_for
    };
    
    // 转义引号
    let user_agent_escaped = user_agent.replace('"', "\\\"");
    
    println!(
        "{} [信息] {}µs {} \"网络路径:{}\" \"{}\" {} \"{}\" \"{}\" \"{}\" {}",
        now,
        elapsed_micros,
        client_ip,
        net_path,
        request_line,
        status,
        referer,
        forwarded_for,
        user_agent_escaped,
        user_id_str
    );
}