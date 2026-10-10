use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct ServiceRegistry {
    // Ключ: (номер порта, имя протокола в нижнем регистре: "tcp" или "udp")
    services: HashMap<(u16, String), String>,
}

impl ServiceRegistry {
    pub fn load() -> Self {
        let mut services = HashMap::new();
        let file = match File::open("/etc/services") {
            Ok(f) => f,
            Err(_) => return Self { services },
        };

        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            let name = parts[0];
            let port_proto: Vec<&str> = parts[1].split('/').collect();
            if port_proto.len() != 2 {
                continue;
            }

            if let Ok(port) = port_proto[0].parse::<u16>() {
                let proto = port_proto[1].to_lowercase();
                services.insert((port, proto), name.to_string());
            }
        }

        Self { services }
    }

    pub fn lookup(&self, port: u16, proto: &str) -> Option<&str> {
        let proto_clean = if proto.to_lowercase().starts_with("tcp") {
            "tcp"
        } else {
            "udp"
        };
        self.services.get(&(port, proto_clean.to_string())).map(|s| s.as_str())
    }
}
