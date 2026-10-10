use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use crate::process::{map_inodes_to_processes, ProcessMetadata};
use crate::services::ServiceRegistry;

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    Tcp,
    Tcp6,
    Udp,
    Udp6,
}

impl Protocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Protocol::Tcp => "TCP",
            Protocol::Tcp6 => "TCP6",
            Protocol::Udp => "UDP",
            Protocol::Udp6 => "UDP6",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SocketEntry {
    pub proto: Protocol,
    pub local: String,
    pub remote: String,
    pub service: Option<String>,
    pub state: String,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
    pub exe: Option<String>,
    pub cmdline: Option<String>,
}

pub fn scan_all_sockets(registry: &ServiceRegistry) -> Result<Vec<SocketEntry>, std::io::Error> {
    let inode_map = map_inodes_to_processes();
    let mut entries = Vec::new();

    parse_proc_file("/proc/net/tcp", Protocol::Tcp, &inode_map, registry, &mut entries)?;
    let _ = parse_proc_file("/proc/net/tcp6", Protocol::Tcp6, &inode_map, registry, &mut entries);
    let _ = parse_proc_file("/proc/net/udp", Protocol::Udp, &inode_map, registry, &mut entries);
    let _ = parse_proc_file("/proc/net/udp6", Protocol::Udp6, &inode_map, registry, &mut entries);

    Ok(entries)
}

fn parse_proc_file(
    path: &str,
    proto: Protocol,
    inode_map: &HashMap<u64, ProcessMetadata>,
    registry: &ServiceRegistry,
    entries: &mut Vec<SocketEntry>,
) -> Result<(), std::io::Error> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return Ok(()),
    };
    let reader = BufReader::new(file);

    for line in reader.lines().skip(1) {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            continue;
        }

        let is_ipv6 = matches!(proto, Protocol::Tcp6 | Protocol::Udp6);
        let is_udp = matches!(proto, Protocol::Udp | Protocol::Udp6);

        let local_addr = parse_endpoint(parts[1], is_ipv6)?;
        let remote_addr = parse_endpoint(parts[2], is_ipv6)?;
        let state = if is_udp {
            udp_state_name(parts[3])
        } else {
            tcp_state_name(parts[3])
        };

        let inode: u64 = parts[9].parse().unwrap_or(0);
        let proc_meta = inode_map.get(&inode);

        let service = registry
            .lookup(local_addr.port(), proto.as_str())
            .map(|s| s.to_string());

        entries.push(SocketEntry {
            proto: proto.clone(),
            local: local_addr.to_string(),
            remote: remote_addr.to_string(),
            service,
            state: state.to_string(),
            pid: proc_meta.map(|m| m.pid),
            process_name: proc_meta.map(|m| m.name.clone()),
            exe: proc_meta.map(|m| m.exe.clone()),
            cmdline: proc_meta.map(|m| m.cmdline.clone()),
        });
    }

    Ok(())
}

fn parse_endpoint(raw: &str, is_ipv6: bool) -> Result<SocketAddr, std::io::Error> {
    let mut split = raw.split(':');
    let ip_hex = split.next().unwrap_or("");
    let port_hex = split.next().unwrap_or("0000");

    let port = u16::from_str_radix(port_hex, 16)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    if is_ipv6 {
        if ip_hex.len() != 32 {
            return Ok(SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), port));
        }

        let mut octets = [0u8; 16];
        for i in 0..4 {
            let chunk = &ip_hex[i * 8..(i + 1) * 8];
            let val = u32::from_str_radix(chunk, 16)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            let bytes = val.to_le_bytes();
            octets[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
        }

        Ok(SocketAddr::new(Ipv6Addr::from(octets).into(), port))
    } else {
        let ip_u32 = u32::from_str_radix(ip_hex, 16)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let ip = Ipv4Addr::from(ip_u32.to_be());
        Ok(SocketAddr::new(ip.into(), port))
    }
}

fn tcp_state_name(state_hex: &str) -> &'static str {
    match state_hex {
        "01" => "ESTABLISHED",
        "02" => "SYN_SENT",
        "03" => "SYN_RECV",
        "04" => "FIN_WAIT1",
        "05" => "FIN_WAIT2",
        "06" => "TIME_WAIT",
        "07" => "CLOSE",
        "08" => "CLOSE_WAIT",
        "09" => "LAST_ACK",
        "0A" => "LISTEN",
        "0B" => "CLOSING",
        _ => "UNKNOWN",
    }
}

fn udp_state_name(state_hex: &str) -> &'static str {
    match state_hex {
        "07" => "UNCONN",
        "01" => "ESTAB",
        _ => "UNKNOWN",
    }
}
