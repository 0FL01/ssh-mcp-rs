# russh - Async SSH Library (v0.56.0)

> Source: https://docs.rs/russh/latest/russh/

## Overview

Server and client SSH asynchronous library, based on tokio/futures.

The library uses **handlers**: types that implement `client::Handler` for clients and `server::Handler` for servers.

## Writing SSH Clients

Use the `russh::client` module:

1. Implement `client::Handler` trait
2. Use `client::connect()` to establish connection
3. Open channels with `channel_open_session()`
4. Execute commands with `Channel::exec()`

### Basic SSH Client Flow

```rust
use russh::*;
use russh::client::*;
use russh_keys::*;

struct MyHandler;

impl client::Handler for MyHandler {
    type Error = anyhow::Error;

    async fn check_server_key(
        &mut self,
        _key: &ssh_key::PublicKey
    ) -> Result<bool, Self::Error> {
        Ok(true) // Accept all keys (don't do this in production!)
    }
}

async fn connect_and_exec() -> Result<()> {
    let config = client::Config::default();
    let mut session = client::connect(
        Arc::new(config),
        ("hostname", 22),
        MyHandler
    ).await?;

    // Authenticate
    let auth_res = session.authenticate_password("user", "password").await?;

    // Open channel and execute command
    let mut channel = session.channel_open_session().await?;
    channel.exec(true, "ls -la").await?;

    // Read output
    while let Some(msg) = channel.wait().await {
        match msg {
            ChannelMsg::Data { data } => println!("{}", String::from_utf8_lossy(&data)),
            ChannelMsg::ExitStatus { exit_status } => println!("Exit: {}", exit_status),
            _ => {}
        }
    }

    Ok(())
}
```

## Key Types

### Structs

| Type | Description |
|------|-------------|
| `Channel` | SSH channel for command execution |
| `ChannelId` | Unique channel identifier |
| `ChannelReadHalf` | Read half of split channel |
| `ChannelWriteHalf` | Write half of split channel |
| `ChannelStream` | AsyncRead/AsyncWrite stream |
| `Limits` | Connection limits |
| `Preferred` | Preferred algorithms |

### Enums

| Type | Description |
|------|-------------|
| `ChannelMsg` | Messages received from channel (Data, Eof, ExitStatus, etc.) |
| `Error` | SSH errors |
| `Disconnect` | Disconnect reasons |
| `Sig` | Signal types |
| `Pty` | PTY modes |

## Channel Messages (`ChannelMsg`)

Used with `Channel::wait()`:

```rust
enum ChannelMsg {
    Data { data: CryptoVec },
    ExtendedData { data: CryptoVec, ext: u32 },
    Eof,
    Close,
    ExitStatus { exit_status: u32 },
    ExitSignal { signal_name: Sig, ... },
    // ... more variants
}
```

## PTY Shell (for interactive commands like `su`)

```rust
// Request PTY for interactive shell
channel.request_pty(
    true,           // want_reply
    "xterm",        // terminal type
    80,             // columns
    24,             // rows
    0, 0,           // pixel dimensions
    &[]             // terminal modes
).await?;

// Request shell
channel.request_shell(true).await?;

// Now you can write commands and read responses interactively
channel.data(b"su -\n").await?;
```

## Modules

- `client` - SSH client implementation
- `server` - SSH server implementation  
- `cipher` - Encryption ciphers
- `kex` - Key exchange
- `keys` - Key handling
- `mac` - Message authentication
- `compression` - Compression support

## Design Principles

- Uses buffered I/O for encryption
- Event loop with handler callbacks
- Internally manages encryption/decryption buffers
