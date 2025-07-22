use crate::args::Args;
use crate::log;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, time::timeout};
use warp::Filter;
use futures_util::{SinkExt, StreamExt};
use warp::http::{Method, HeaderMap};

pub async fn run_proxy(args: Args) -> anyhow::Result<()> {
    let listen = format!("{}:{}", args.addr, args.port);
    let args = Arc::new(args);

    let ws_route = warp::ws()
        .and(warp::addr::remote())
        .and(warp::header::headers_cloned()) // 获取所有头部
        .and(warp::method())
        .and(warp::path::full())
        .and(warp::query::<HashMap<String, String>>())
        .and(warp::any().map(move || Arc::clone(&args)))
        .map(|ws: warp::ws::Ws, 
             remote: Option<SocketAddr>, 
             headers: HeaderMap,
             method: Method,
             path: warp::path::FullPath,
             query: HashMap<String, String>, 
             args: Arc<Args>| {
            let start = std::time::Instant::now();
            ws.on_upgrade(move |socket| async move {
                // 解析客户端真实IP
                let client_ip = get_client_ip(&headers, remote);
                
                // 处理 token 逻辑
                let target_addr = if let Some(frkey) = &args.frkey {
                    query.get(frkey).cloned()
                } else {
                    // 如果没有设置 frkey，尝试获取第一个参数
                    query.values().next().cloned()
                };
                
                if let Some(target) = target_addr {
                    // 获取User-Agent
                    let user_agent = headers.get("user-agent")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("-");
                    
                    // 获取Referer
                    let referer = headers.get("referer")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("-");
                    
                    // 获取User-Id
                    let user_id = headers.get("user-id")
                        .and_then(|v| v.to_str().ok());
                    
                    // 处理WebSocket连接
                    if let Err(e) = handle_ws(
                        socket, 
                        target.clone(), 
                        client_ip.clone(), 
                        remote, 
                        args.clone()
                    ).await {
                        eprintln!("[!] 错误: {e}");
                    }
                    
                    // 记录连接日志
                    let src_port = remote.map(|r| r.port()).unwrap_or(0);
                    let src = format!("{}:{}", client_ip, src_port);
                    log::log_connection(
                        start,
                        &client_ip,
                        &src,
                        &target,
                        method.as_str(),
                        path.as_str(),
                        200,
                        referer,
                        &client_ip, // 使用解析出的客户端IP作为forwarded_for
                        user_agent,
                        user_id,
                    );
                } else {
                    eprintln!("[!] 缺少目标地址查询参数");
                }
            })
        });

    warp::serve(ws_route)
        .run(listen.parse::<SocketAddr>()?)
        .await;

    Ok(())
}

// 获取客户端真实IP（按照优先级顺序）
fn get_client_ip(headers: &HeaderMap, remote: Option<SocketAddr>) -> String {
    // 1. 尝试从 X-Forwarded-For 获取
    if let Some(xff) = headers.get("x-forwarded-for") {
        if let Ok(xff_str) = xff.to_str() {
            // 取第一个IP（可能有多个用逗号分隔）
            if let Some(ip) = xff_str.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }
    
    // 2. 尝试从 X-Real-IP 获取
    if let Some(xri) = headers.get("x-real-ip") {
        if let Ok(ip) = xri.to_str() {
            return ip.trim().to_string();
        }
    }
    
    // 3. 尝试从 REMOTE_HOST 获取
    if let Some(rh) = headers.get("remote_host") {
        if let Ok(ip) = rh.to_str() {
            return ip.trim().to_string();
        }
    }
    
    // 4. 最后使用连接地址
    remote.map(|r| r.ip().to_string())
        .unwrap_or_else(|| "127.0.0.1".to_string())
}

// 处理WebSocket连接
async fn handle_ws(
    ws_stream: warp::ws::WebSocket,
    target_addr: String,
    client_ip: String, // 使用解析出的客户端IP
    remote: Option<SocketAddr>,
    args: Arc<Args>,
) -> anyhow::Result<()> {
    let (mut ws_tx, mut ws_rx) = ws_stream.split();
    let mut tcp_stream = timeout(
        std::time::Duration::from_secs(args.timeout),
        TcpStream::connect(&target_addr),
    )
    .await??;

    if args.haproxy_protocol {
        // 使用解析出的客户端IP
        let src_port = remote.map(|r| r.port()).unwrap_or(12345);
        let header = crate::utils::build_proxy_v2(&client_ip, src_port, &target_addr)?;
        tcp_stream.write_all(&header).await?;
    }

    let (mut r_tcp, mut w_tcp) = tokio::io::split(tcp_stream);

    let to_tcp = async {
        while let Some(result) = ws_rx.next().await {
            let msg = result?;
            // 统一处理所有消息为二进制数据
            let data = msg.as_bytes().to_vec();
            w_tcp.write_all(&data).await?;
        }
        Ok::<(), anyhow::Error>(())
    };

    let to_ws = async {
        let mut buf = vec![0u8; args.buffer];
        loop {
            let n = r_tcp.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            // 使用warp的Message类型发送二进制数据
            ws_tx.send(warp::ws::Message::binary(buf[..n].to_vec())).await?;
        }
        Ok::<(), anyhow::Error>(())
    };

    tokio::try_join!(to_tcp, to_ws)?;
    Ok(())
}