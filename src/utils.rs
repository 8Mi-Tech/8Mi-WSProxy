use std::net::{IpAddr, SocketAddr};
use byteorder::{NetworkEndian, WriteBytesExt};
use std::str::FromStr;

pub fn build_proxy_v2(src_ip: &str, src_port: u16, dest: &str) -> anyhow::Result<Vec<u8>> {
    // 解析目标地址为 SocketAddr 以支持 IPv6 格式
    let dest_addr = if dest.starts_with('[') && dest.contains(']') {
        // IPv6 地址格式 [::1]:8080
        dest.parse::<SocketAddr>()
    } else {
        // IPv4 地址格式 127.0.0.1:8080
        SocketAddr::from_str(dest)
    }?;
    
    let dest_ip = dest_addr.ip();
    let dest_port = dest_addr.port();
    
    // 解析源IP地址（支持IPv6）
    let src_ip: IpAddr = src_ip.parse()?;

    let mut header = Vec::new();
    header.extend(b"\r\n\r\n\0\r\nQUIT\n");
    header.push(0x21);

    // 统一转换为IPv6地址处理
    let src_ip_v6 = match src_ip {
        IpAddr::V4(ipv4) => ipv4.to_ipv6_mapped(),
        IpAddr::V6(ipv6) => ipv6,
    };

    let dest_ip_v6 = match dest_ip {
        IpAddr::V4(ipv4) => ipv4.to_ipv6_mapped(),
        IpAddr::V6(ipv6) => ipv6,
    };

    header.push(0x21);
    header.write_u16::<NetworkEndian>(36)?;
    header.extend_from_slice(&src_ip_v6.octets());
    header.extend_from_slice(&dest_ip_v6.octets());

    header.write_u16::<NetworkEndian>(src_port)?;
    header.write_u16::<NetworkEndian>(dest_port)?;
    Ok(header)
}