//! SSH MCP Server - A Model Context Protocol server for remote SSH command execution
//!
//! This crate provides an MCP server that allows executing commands on remote Linux
//! systems via SSH. It supports both password and key-based authentication, as well
//! as privilege elevation via `su` and `sudo`.
//!
//! # Features
//!
//! - Execute shell commands on remote SSH servers
//! - Support for `sudo` command execution with password
//! - Persistent SSH connection with auto-reconnect
//! - Configurable command timeout
//! - Command length limits for safety
//!
//! # Example Usage (CLI)
//!
//! ```bash
//! ssh-mcp --host=192.168.1.100 --user=admin --password=secret
//! ```

pub mod config;
pub mod error;

// Re-exports for convenience
pub use config::{Args, Config};
pub use error::{Result, SshMcpError};
