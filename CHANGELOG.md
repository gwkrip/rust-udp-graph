# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [v0.1.0] - 2024-01-01

### Added
- UDP listener on port `8125` parsing `key:value|key:value` format
- HTTP server on port `8080` serving the dashboard via Actix-Web
- WebSocket endpoint `/ws/` broadcasting parsed data every 1 second
- Real-time line chart powered by Chart.js
- Live stat cards: Current RPS, Peak RPS, Average RPS, Data Points
- Auto-reconnect WebSocket client with 3-second retry
- Dark cyber-terminal UI with scanline effects and neon accents
- Rolling 60-point history on the chart
