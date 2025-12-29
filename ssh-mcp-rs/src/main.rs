//! SSH MCP Server - Entry point
//!
//! This is the main entry point for the SSH MCP Server.
//! It parses CLI arguments, validates configuration, and will start the MCP server.

use clap::Parser;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

use ssh_mcp::config::{Args, Config};
use ssh_mcp::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing/logging
    let _subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    // Parse CLI arguments
    let args = Args::parse();

    // Validate and create config
    let config = Config::from_args(args)?;

    info!(
        "SSH MCP Server v{} starting...",
        env!("CARGO_PKG_VERSION")
    );
    info!(
        "Connecting to {}@{}:{}",
        config.user, config.host, config.port
    );
    info!(
        "Timeout: {}ms, Max chars: {}",
        config.timeout_ms,
        config.max_chars.map_or("unlimited".to_string(), |n| n.to_string())
    );

    if config.disable_sudo {
        info!("sudo-exec tool is disabled");
    }

    // TODO: Phase 2+ implementation
    // 1. Create SSH connection manager
    // 2. Create MCP server with tools
    // 3. Start stdio transport
    // 4. Handle graceful shutdown

    info!("SSH MCP Server running on stdio");

    // Setup graceful shutdown
    tokio::signal::ctrl_c().await.map_err(|e| {
        error!("Failed to setup signal handler: {}", e);
        ssh_mcp::SshMcpError::Io(e)
    })?;

    info!("Shutting down SSH MCP Server...");

    Ok(())
}
