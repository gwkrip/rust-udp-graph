# UDP Monitor

Real-time UDP telemetry dashboard built with Rust + Actix-Web.  
Receives UDP packets, parses RPS metrics, and streams them live to a browser via WebSocket.

[![CI](https://github.com/your-username/rust-udp-graph/actions/workflows/ci.yml/badge.svg)](https://github.com/your-username/rust-udp-graph/actions/workflows/ci.yml)
[![Release](https://github.com/your-username/rust-udp-graph/actions/workflows/release.yml/badge.svg)](https://github.com/your-username/rust-udp-graph/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## Features

- 📡 **UDP Listener** — receives packets on port `8125`
- 🌐 **HTTP Dashboard** — served at `http://0.0.0.0:8080`
- ⚡ **WebSocket Push** — data broadcast to all clients every second
- 📊 **Live Chart** — rolling 60-point line graph with Chart.js
- 🔢 **Stat Cards** — Current RPS, Peak RPS, Average RPS, Data Points
- 🔄 **Auto-Reconnect** — browser reconnects automatically on disconnect
- 🎨 **Dark UI** — cyber-terminal aesthetic with neon accents

---

## Architecture

```
UDP Sender  ──[port 8125]──▶  UdpSocket (Tokio)
                                    │
                             Arc<Mutex<Vec<u32>>>   (shared state)
                                    │
                    ┌───────────────┴───────────────┐
                    │                               │
              GET /  (Tera HTML)         GET /ws/  (WebSocket)
                    │                               │
                 Browser  ◀────── WS push (1s) ────┘
```

---

## UDP Packet Format

Packets are plain UTF-8 strings with pipe-separated `key:value` pairs:

```
vrypt.rps:142|gvrypt.rps:98|other.rps:305
```

Each numeric value after `:` is parsed as a `u32`. Non-numeric values are silently skipped.

---

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) 1.70 or later

### Run from source

```bash
git clone https://github.com/your-username/rust-udp-graph.git
cd rust-udp-graph
cargo run --release
```

Open your browser at `http://localhost:8080`.

### Send test data

```bash
# Linux / macOS
echo -n "svc.rps:120|api.rps:340|worker.rps:88" | nc -u 127.0.0.1 8125

# Windows (PowerShell)
$udp = New-Object System.Net.Sockets.UdpClient
$bytes = [System.Text.Encoding]::UTF8.GetBytes("svc.rps:120|api.rps:340|worker.rps:88")
$udp.Send($bytes, $bytes.Length, "127.0.0.1", 8125)
$udp.Close()
```

---

## Configuration

| Variable | Default | Description |
|---|---|---|
| UDP bind address | `0.0.0.0:8125` | Where to listen for UDP packets |
| HTTP bind address | `0.0.0.0:8080` | Where to serve the dashboard |
| WebSocket push interval | `1s` | How often to push data to clients |
| Chart history | `60 points` | Rolling window in the browser |

> Configuration is currently compile-time. Environment variable support is planned for v0.2.

---

## Project Structure

```
.
├── .github/
│   └── workflows/
│       ├── ci.yml          # Lint, format check, build on every PR
│       └── release.yml     # Cross-compile & publish GitHub Release on tag push
├── templates/
│   └── index.html          # Tera template — dashboard UI
├── src/
│   └── main.rs             # Entry point
├── Cargo.toml
├── CHANGELOG.md
└── README.md
```

---

## Creating a Release

Releases are created automatically by pushing a version tag:

```bash
git tag v1.0.0
git push origin v1.0.0
```

The `release.yml` workflow will:

1. Cross-compile for 5 targets (Linux x86_64/ARM64, macOS x86_64/ARM64, Windows x86_64)
2. Bundle each binary with the `templates/` folder into a `.tar.gz` / `.zip`
3. Create a GitHub Release with all assets attached
4. Mark as **pre-release** if the tag contains a hyphen (e.g. `v1.0.0-beta.1`)

### Supported release targets

| Archive | Arch | libc | Use case |
|---|---|---|---|
| `rust_udp_graph-linux-x86_64-gnu.tar.gz` | x86_64 | glibc | Ubuntu, Debian, Fedora, most distros |
| `rust_udp_graph-linux-x86_64-musl.tar.gz` | x86_64 | musl | Alpine, static binary, containers |
| `rust_udp_graph-linux-aarch64-gnu.tar.gz` | ARM64 | glibc | AWS Graviton, Raspberry Pi OS 64-bit |
| `rust_udp_graph-linux-aarch64-musl.tar.gz` | ARM64 | musl | Alpine ARM64, static binary |

> **musl vs gnu**: musl builds produce fully static binaries with no external libc dependency — ideal for Docker scratch/alpine images. gnu builds link against glibc and are compatible with most mainstream Linux distributions.

---

## Development

```bash
# Check formatting
cargo fmt --all -- --check

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all

# Build debug
cargo build

# Build optimised
cargo build --release
```

---

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Commit your changes: `git commit -m "feat: add my feature"`
4. Push to the branch: `git push origin feat/my-feature`
5. Open a Pull Request

Please follow [Conventional Commits](https://www.conventionalcommits.org/) for commit messages.

---

## License

This project is licensed under the MIT License — see [LICENSE](LICENSE) for details.
