//! SSH client handler implementation
//!
//! Implements the `russh::client::Handler` trait to handle SSH connection events.

/// SSH client handler for russh
///
/// This handler is used by russh to process SSH events such as server key
/// verification. Currently, it accepts all server keys (suitable for automated
/// connections where host key verification is handled separately).
#[derive(Debug, Clone)]
pub struct SshHandler;

impl SshHandler {
    /// Create a new SSH handler
    pub fn new() -> Self {
        Self
    }
}

impl Default for SshHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl russh::client::Handler for SshHandler {
    type Error = anyhow::Error;

    /// Verify the server's host key
    ///
    /// # Security Note
    /// Currently accepts all server keys for simplicity in automated environments.
    /// In production, you may want to implement proper host key verification
    /// (check against known_hosts file or fingerprint whitelist).
    async fn check_server_key(
        &mut self,
        _server_public_key: &russh::keys::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_creation() {
        let handler = SshHandler::new();
        assert!(format!("{:?}", handler).contains("SshHandler"));
    }

    #[test]
    fn test_handler_default() {
        let _handler: SshHandler = Default::default();
    }
}
