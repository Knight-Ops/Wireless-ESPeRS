# ESP32-C6 Modular Networking Firmware

## Project Overview
This project aims to provide a modular, efficient, and flexible firmware for the ESP32-C6, enabling it to act as a networking co-processor for a secondary microcontroller via SPI. The firmware supports both WiFi and Thread networking, and exposes a simple SPI-based API for network operations and configuration.

## Features
- **WiFi Networking**
  - Connect to WiFi networks
  - TCP/UDP client connections over WiFi
  - Send/receive data over WiFi
- **Thread Networking**
  - Connect to Thread networks
  - TCP/UDP client connections over Thread
  - Send/receive data over Thread
- **SPI API**
  - Simple, coherent SPI interface for secondary MCU
  - Data transfer and command interface
- **HTTP Server**
  - Configuration and status endpoints

## Repository Structure
- `xiao-firmware/` — Rust firmware for ESP32-C6
  - `src/`
    - `ap_mode.rs` — Access Point mode logic
    - `web_server.rs` — HTTP server for configuration
    - `lib.rs` — Main library entry point
    - `bin/main.rs` — Firmware entry point
  - `Cargo.toml`, `rust-toolchain.toml` — Rust project configuration
- `xiao-web/` — Svelte web UI for configuration
  - `src/routes/` — Web pages for WiFi and Thread configuration
  - `static/`, `build/` — Static assets and build output

## Current Progress
### Implemented
- Project structure and modular Rust codebase
- Basic HTTP server for configuration (`web_server.rs`)
- Initial WiFi and Thread connection logic (in progress)
- Svelte web UI skeleton for configuration

### In Progress
- Modularization of networking code for WiFi/Thread
- SPI API design and implementation
- TCP/UDP client logic for both WiFi and Thread
- API endpoints for status/configuration

### To Do
- Complete SPI API (init, send/receive, error handling)
- Finalize Thread networking integration
- Implement TCP/UDP data transfer logic
- Expand HTTP server endpoints for full configuration
- Add unit and integration tests
- Documentation for SPI protocol and API usage

## Getting Started
1. **Firmware**
   - Requires Rust toolchain for embedded development
   - Build and flash instructions coming soon
2. **Web UI**
   - Requires Node.js and npm
   - `cd xiao-web && npm install && npm run dev`

## Contributing
Contributions are welcome! Please follow the general coding standards:
- Refactor for readability and maintainability
- Avoid code duplication (DRY principle)
- Add comments to major AI-generated code blocks
