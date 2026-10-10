use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
pub struct ProcessMetadata {
    pub pid: u32,
    pub name: String,
    pub exe: String,
    pub cmdline: String,
}

pub fn map_inodes_to_processes() -> HashMap<u64, ProcessMetadata> {
    let mut map = HashMap::new();

    let proc_dir = match fs::read_dir("/proc") {
        Ok(dir) => dir,
        Err(_) => return map,
    };

    for entry in proc_dir.flatten() {
        let file_name = entry.file_name();
        let pid_str = match file_name.to_str() {
            Some(s) => s,
            None => continue,
        };

        let pid: u32 = match pid_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let pid_path = entry.path();

        // Имя процесса (comm)
        let comm_path = pid_path.join("comm");
        let proc_name = fs::read_to_string(&comm_path)
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        // Абсолютный путь к бинарнику через symlink /proc/[pid]/exe
        let exe_path = fs::read_link(pid_path.join("exe"))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "-".to_string());

        // Полная строка аргументов из /proc/[pid]/cmdline (разделены \0)
        let cmdline = fs::read(pid_path.join("cmdline"))
            .map(|bytes| {
                bytes
                    .split(|&b| b == 0)
                    .filter(|b| !b.is_empty())
                    .map(|b| String::from_utf8_lossy(b).to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();

        let fd_path = pid_path.join("fd");
        let fd_entries = match fs::read_dir(&fd_path) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        let metadata = ProcessMetadata {
            pid,
            name: proc_name,
            exe: exe_path,
            cmdline: if cmdline.is_empty() { "-".to_string() } else { cmdline },
        };

        for fd_entry in fd_entries.flatten() {
            if let Ok(target) = fs::read_link(fd_entry.path()) {
                if let Some(target_str) = target.to_str() {
                    if target_str.starts_with("socket:[") && target_str.ends_with(']') {
                        let inode_str = &target_str[8..target_str.len() - 1];
                        if let Ok(inode) = inode_str.parse::<u64>() {
                            map.insert(inode, metadata.clone());
                        }
                    }
                }
            }
        }
    }

    map
}
