# SSH MCP Server (Rust Implementation)

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Protocol: MCP](https://img.shields.io/badge/Protocol-MCP-blue.svg)](https://modelcontextprotocol.io)

A high-performance Rust implementation of the SSH Model Context Protocol (MCP) server. This tool allows AI models to securely interact with remote Linux systems over SSH, providing tools for command execution and administrative tasks.

> This is a Rust port of the original [TypeScript SSH MCP Server](https://github.com/tufantunc/ssh-mcp), optimized for performance and reliability.

## ✨ Features

- **Persistent Connections**: Maintains a single SSH session across multiple tool calls for maximum speed.
- **Auto-Reconnect**: Automatically restores the connection if it drops.
- **Interactive Elevation**: Supports `su` elevation with PTY shell for full root access.
- **Sudo Integration**: Provides a `sudo-exec` tool with password wrapping.
- **Command Sanitization**: Built-in safety checks for command inputs.
- **Output Control**: Configurable output length limits to prevent token overflow.
- **Cross-Platform**: Compiled binary runs on any system with SSH access.

## 🛠 Installation

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (cargo, rustc)
- `pkg-config` and OpenSSL headers (usually `libssl-dev` on Ubuntu/Debian)

### Build from Source

```bash
git clone https://github.com/tufantunc/ssh-mcp.git
cd ssh-mcp/ssh-mcp-rs
cargo build --release
```

The binary will be available at `./target/release/ssh-mcp`.

## ⚙️ Configuration

The server is configured via CLI arguments or environment variables.

| Argument | Environment Variable | Description |
|----------|----------------------|-------------|
| `--host` | `SSH_MCP_HOST` | SSH host (required) |
| `--user` | `SSH_MCP_USER` | SSH username (required) |
| `--port` | `SSH_MCP_PORT` | SSH port (default: 22) |
| `--password` | `SSH_MCP_PASSWORD` | SSH password (alt to key) |
| `--key` | `SSH_MCP_KEY` | Path to private key file |
| `--su-password` | `SSH_MCP_SU_PASSWORD` | Password for `su` elevation |
| `--sudo-password` | `SSH_MCP_SUDO_PASSWORD` | Password for `sudo` pipes |
| `--timeout` | `SSH_MCP_TIMEOUT` | Command timeout in ms (default: 60000) |
| `--maxChars` | `SSH_MCP_MAX_CHARS` | Output limit (default: 1000, "none" to disable) |
| `--disable-sudo` | `SSH_MCP_DISABLE_SUDO` | Disable the `sudo-exec` tool |

## 🚀 Adding to MCP Clients

### Claude Desktop

Add this to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "ssh-remote": {
      "command": "/absolute/path/to/ssh-mcp",
      "args": [
        "--host=192.168.1.10",
        "--port=22",
        "--user=agent-nc",
        "--key=/path/to/private/key",
        "--timeout=30000",
        "--maxChars=1000"
      ]
    }
  }
}
```

## 🛠 Tools

The server exposes the following MCP tools:

### `exec`
Execute a shell command as the connected user.
- **Arguments**:
  - `command` (string): The shell command to execute.

### `sudo-exec`
Execute a command with root privileges using `sudo`.
- **Arguments**:
  - `command` (string): The shell command to execute with sudo.
- **Note**: This tool uses the `--sudo-password` provided at startup.

## 🔒 Security

- **Stdio Transport**: Communicates using JSON-RPC over stdin/stdout, ensuring no exposed ports.
- **Credential Storage**: Passwords and keys are only kept in memory and never logged.
- **Logging**: All internal logs are sent to `stderr` to avoid interfering with the MCP protocol.

## 🧪 Development

### Running Tests
```bash
cargo test
```

### Linting
```bash
cargo clippy
```

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.
