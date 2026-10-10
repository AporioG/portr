# portr ⚡

A high-performance, interactive TUI network socket and process auditor for Linux, written in pure Rust.

Zero runtime dependencies. Direct `/proc` kernel introspection. Fully static PIE binary.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Linux-green.svg)
![Rust](https://img.shields.io/badge/rust-2021-orange.svg)
![Binary](https://img.shields.io/badge/static-musl-purple.svg)

![portr demo](demo.gif)

## Features

- **Direct Kernel Introspection:** Queries `/proc/net/{tcp,tcp6,udp,udp6}` without external shell wrappers.
- **VFS Socket Resolution:** Automatically maps kernel socket inodes to PID, binary paths (`/proc/[pid]/exe`), and complete arguments (`cmdline`).
- **IANA Service Decoding:** Translates port numbers to human-readable network service names via `/etc/services` (RFC 6335).
- **Process Management:** Send `SIGTERM` (`t`) or `SIGKILL` (`Shift+K`) directly from the interface.
- **Fine-Grained Linux Capabilities:** Operates securely as an unprivileged user using `CAP_SYS_PTRACE` and `CAP_DAC_READ_SEARCH`. No `sudo` execution required.
- **Terminal Crash Protection:** Built-in panic hooks prevent raw mode leaks on runtime failure.

## Installation

### Arch Linux (AUR)

```bash
yay -S portr-bin
```

### Static Binary (Any Linux x86_64)

```bash
curl -fsSL https://github.com/AporioG/portr/releases/download/v0.1.0/portr-linux-x86_64.tar.gz | tar -xz
sudo install -Dm755 portr /usr/local/bin/portr
sudo setcap cap_sys_ptrace,cap_dac_read_search+ep /usr/local/bin/portr
portr
```

### From Source

```bash
git clone https://github.com/AporioG/portr.git
cd portr
cargo build --release
sudo setcap cap_sys_ptrace,cap_dac_read_search+ep target/release/portr
./target/release/portr
```

## Keybindings

| **Key** | **Action** |
| --- | --- |
| `j` / `Down` | Navigate down |
| `k` / `Up` | Navigate up |
| `/` | Live fuzzy filter (by IP, Port, Service, PID, Process, or Cmdline) |
| `r` | Manual refresh |
| `t` | Send `SIGTERM` to targeted process |
| `Shift + K` | Send `SIGKILL` to targeted process |
| `q` | Quit |

## License

MIT
