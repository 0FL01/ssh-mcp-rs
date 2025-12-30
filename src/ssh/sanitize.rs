//! Command sanitization and escaping utilities
//!
//! Provides functions for validating and escaping commands before SSH execution.

use crate::error::{Result, SshMcpError};

/// Sanitize a command before execution
///
/// This function:
/// - Validates that the command is not empty
/// - Trims whitespace
/// - Checks length against max_chars limit
///
/// # Arguments
/// * `command` - The raw command string
/// * `max_chars` - Optional maximum character limit (None = unlimited)
///
/// # Returns
/// * `Ok(String)` - The sanitized command
/// * `Err(SshMcpError::InvalidParams)` - If command is empty or too long
///
/// # Examples
/// ```
/// use ssh_mcp::ssh::sanitize::sanitize_command;
///
/// let cmd = sanitize_command("  ls -la  ", Some(1000)).unwrap();
/// assert_eq!(cmd, "ls -la");
///
/// // Too long command
/// let result = sanitize_command("a".repeat(100).as_str(), Some(50));
/// assert!(result.is_err());
/// ```
pub fn sanitize_command(command: &str, max_chars: Option<usize>) -> Result<String> {
    let trimmed = command.trim();

    if trimmed.is_empty() {
        return Err(SshMcpError::invalid_params("Command cannot be empty"));
    }

    // Check length limit
    if let Some(max) = max_chars {
        if trimmed.len() > max {
            return Err(SshMcpError::invalid_params(format!(
                "Command is too long (max {} characters, got {})",
                max,
                trimmed.len()
            )));
        }
    }

    Ok(trimmed.to_string())
}

/// Escape a command for use in shell contexts (like pkill -f)
///
/// This escapes single quotes in the command so it can be safely
/// used inside single-quoted shell strings.
///
/// # Example
/// ```
/// use ssh_mcp::ssh::sanitize::escape_command_for_shell;
///
/// let escaped = escape_command_for_shell("echo 'hello'");
/// assert_eq!(escaped, "echo '\"'\"'hello'\"'\"'");
/// ```
pub fn escape_command_for_shell(command: &str) -> String {
    // Replace single quotes with escaped single quotes
    // 'word' becomes '"'"'word'"'"'
    // This effectively: end quote, add literal quote via double quotes, start quote again
    command.replace('\'', "'\"'\"'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_command_valid() {
        let result = sanitize_command("ls -la", Some(1000));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "ls -la");
    }

    #[test]
    fn test_sanitize_command_trims_whitespace() {
        let result = sanitize_command("  ls -la  ", Some(1000));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "ls -la");
    }

    #[test]
    fn test_sanitize_command_empty() {
        let result = sanitize_command("", Some(1000));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_sanitize_command_whitespace_only() {
        let result = sanitize_command("   ", Some(1000));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_sanitize_command_too_long() {
        let long_cmd = "a".repeat(100);
        let result = sanitize_command(&long_cmd, Some(50));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too long"));
    }

    #[test]
    fn test_sanitize_command_exactly_at_limit() {
        let cmd = "a".repeat(50);
        let result = sanitize_command(&cmd, Some(50));
        assert!(result.is_ok());
    }

    #[test]
    fn test_sanitize_command_unlimited() {
        let long_cmd = "a".repeat(10000);
        let result = sanitize_command(&long_cmd, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_escape_command_for_shell_no_quotes() {
        let escaped = escape_command_for_shell("ls -la");
        assert_eq!(escaped, "ls -la");
    }

    #[test]
    fn test_escape_command_for_shell_with_quotes() {
        let escaped = escape_command_for_shell("echo 'hello'");
        assert_eq!(escaped, "echo '\"'\"'hello'\"'\"'");
    }

    #[test]
    fn test_escape_command_for_shell_multiple_quotes() {
        let escaped = escape_command_for_shell("echo 'a' 'b'");
        assert_eq!(escaped, "echo '\"'\"'a'\"'\"' '\"'\"'b'\"'\"'");
    }

    #[test]
    fn test_escape_command_for_shell_empty() {
        let escaped = escape_command_for_shell("");
        assert_eq!(escaped, "");
    }
}
