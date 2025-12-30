//! MCP Tools module
//!
//! This module previously provided separate tool classes with #[tool_router].
//! Now, tools are implemented directly in the SshMcpServer via ServerHandler trait.
//!
//! Available tools:
//! - `exec` - Execute shell commands on the remote SSH server
//! - `sudo-exec` - Execute shell commands with sudo privileges
//!
//! See `server.rs` for the implementation.

// The tools are now implemented directly in server.rs as part of ServerHandler.
// This module is kept for potential future expansion with additional tools
// or utility functions.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for the exec tool
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ExecParams {
    /// Shell command to execute on the remote SSH server
    pub command: String,
}

/// Parameters for the sudo-exec tool
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SudoExecParams {
    /// Shell command to execute with sudo on the remote SSH server
    pub command: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exec_params_deserialize() {
        let json = r#"{"command": "echo hello"}"#;
        let params: ExecParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.command, "echo hello");
    }

    #[test]
    fn test_sudo_exec_params_deserialize() {
        let json = r#"{"command": "apt update"}"#;
        let params: SudoExecParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.command, "apt update");
    }
}
