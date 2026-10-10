# portr

A high-performance, interactive TUI network socket and process auditor for Linux, written in pure Rust.

Zero runtime dependencies. Direct `/proc` kernel introspection. No overhead.

## Features

- **Direct Kernel Introspection:** Queries `/proc/net/{tcp,tcp6,udp,udp6}` without external shell wrappers.
- **VFS Socket Resolution:** Automatically maps kernel socket inodes to PID, binary paths (`/proc/[pid]/exe`), and complete arguments (`cmdline`).
- **IANA Service Decoding:** Translates port numbers to human-readable network service names via `/etc/services` (RFC 6335).
- **Process Management:** Send `SIGTERM` or `SIGKILL` directly from the interface.
- **Fine-Grained Linux Capabilities:** Operates securely as an unprivileged user using `CAP_SYS_PTRACE` and `CAP_DAC_READ_SEARCH`. No `sudo` execution required.
- **Terminal Crash Protection:** Built-in panic hooks prevent raw mode leaks on runtime failure.

## Installation & Build

```bash
git clone https://github.com/AporioG/portr.git
cd portr
cargo build --release
sudo setcap cap_sys_ptrace,cap_dac_read_search+ep target/release/portr
./target/release/portr
```

## Keybindings

| Key | Action |
| --- | --- |
| `j` / `Down` | Navigate down |
| `k` / `Up` | Navigate up |
| `/` | Live search filter |
| `r` | Manual refresh |
| `t` | Send `SIGTERM` to targeted process |
| `Shift + K` | Send `SIGKILL` to targeted process |
| `q` | Quit |

## License

MIT
