# SSH MCP Server Rust Implementation Blueprint

> **Цель**: Портирование TypeScript MCP сервера на Rust с использованием `rmcp` SDK и `russh` SSH библиотеки.

---

## Технологический стек

| Компонент | Библиотека | Версия |
|-----------|------------|--------|
| MCP SDK | `rmcp` | 0.12.0 |
| SSH | `russh`, `russh-keys` | 0.56.0 |
| Async runtime | `tokio` | 1.x |
| CLI | `clap` | 4.x |
| Error handling | `thiserror`, `anyhow` | 2.x, 1.x |

> [!TIP]
> **Документация**: Перед началом каждой фазы ознакомься с документами в `Docs/`:
> - [rmcp-sdk.md](file:///home/stfu/ai/ssh-mcp/ssh-mcp-rs/Docs/rmcp-sdk.md) — MCP server API, `#[tool_router]` macro
> - [russh-library.md](file:///home/stfu/ai/ssh-mcp/ssh-mcp-rs/Docs/russh-library.md) — SSH client, Channel, PTY

---

## Phase 1: Project Setup and Basic Structure

**Цель**: Инициализация проекта, базовые типы и конфигурация.

### Референсные файлы TypeScript

| Файл | Что взять |
|------|-----------|
| [src/index.ts:10-70](file:///home/stfu/ai/ssh-mcp/src/index.ts#L10-L70) | `parseArgv()`, `validateConfig()`, константы |
| [package.json](file:///home/stfu/ai/ssh-mcp/package.json) | Metadata, версия 1.4.0 |

### Задачи

- [x] **1.1** Создать `Cargo.toml` с зависимостями
- [x] **1.2** Создать `src/main.rs` — entry point с `#[tokio::main]`
- [x] **1.3** Создать `src/config.rs`:
  - Struct `Config` с полями: `host`, `port`, `user`, `password`, `key`, `timeout`, `max_chars`
  - Struct `Args` с `#[derive(Parser)]` от clap
  - `validate_config()` — проверка обязательных параметров
  - `parse_max_chars()` — поддержка "none" / 0 / число
- [x] **1.4** Создать `src/error.rs`:
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum SshMcpError {
      #[error("SSH connection error: {0}")]
      Connection(String),
      #[error("Authentication failed: {0}")]
      Authentication(String),
      #[error("Command timeout after {0}ms")]
      Timeout(u64),
      #[error("Invalid parameters: {0}")]
      InvalidParams(String),
      #[error("Elevation failed: {0}")]
      ElevationFailed(String),
  }
  ```
- [x] **1.5** Создать `src/lib.rs` — re-exports для тестов

### Результат фазы

```
ssh-mcp-rs/
├── Cargo.toml
├── Docs/
│   ├── rmcp-sdk.md
│   └── russh-library.md
└── src/
    ├── main.rs
    ├── lib.rs
    ├── config.rs
    └── error.rs
```

---

## Phase 2: SSH Connection Manager

**Цель**: Реализация persistent SSH соединения с reconnect логикой.

### Референсные файлы TypeScript

| Файл | Что взять |
|------|-----------|
| [src/index.ts:108-337](file:///home/stfu/ai/ssh-mcp/src/index.ts#L108-L337) | `SSHConfig`, `SSHConnectionManager` class |
| [test/persistent-connection.test.ts](file:///home/stfu/ai/ssh-mcp/test/persistent-connection.test.ts) | Тесты для connection manager |

### Задачи

- [ ] **2.1** Создать `src/ssh/mod.rs` — модуль SSH
- [ ] **2.2** Создать `src/ssh/config.rs`:
  ```rust
  pub struct SshConfig {
      pub host: String,
      pub port: u16,
      pub username: String,
      pub password: Option<String>,
      pub private_key: Option<String>,
      pub su_password: Option<String>,
      pub sudo_password: Option<String>,
  }
  ```
- [ ] **2.3** Создать `src/ssh/handler.rs`:
  - Implement `russh::client::Handler` trait
  - `check_server_key()` — принимать все ключи (или проверять fingerprint)
- [ ] **2.4** Создать `src/ssh/connection.rs`:
  ```rust
  pub struct SshConnectionManager {
      config: SshConfig,
      session: Arc<Mutex<Option<russh::client::Handle<MyHandler>>>>,
      is_connecting: AtomicBool,
      su_channel: Arc<Mutex<Option<russh::Channel>>>,
      is_elevated: AtomicBool,
  }

  impl SshConnectionManager {
      pub async fn new(config: SshConfig) -> Self;
      pub async fn connect(&self) -> Result<(), SshMcpError>;
      pub fn is_connected(&self) -> bool;
      pub async fn ensure_connected(&self) -> Result<(), SshMcpError>;
      pub fn close(&self);
  }
  ```

### Ключевые моменты

- Connection timeout: 30 секунд
- Concurrent connection handling через `Mutex` + `AtomicBool`
- Автоматический reconnect в `ensure_connected()`

---

## Phase 3: Command Execution

**Цель**: Выполнение команд через SSH channel.

### Референсные файлы TypeScript

| Файл | Что взять |
|------|-----------|
| [src/index.ts:487-591](file:///home/stfu/ai/ssh-mcp/src/index.ts#L487-L591) | `execSshCommandWithConnection()` |
| [src/index.ts:593-681](file:///home/stfu/ai/ssh-mcp/src/index.ts#L593-L681) | `execSshCommand()` — legacy с timeout/pkill |
| [src/index.ts:73-106](file:///home/stfu/ai/ssh-mcp/src/index.ts#L73-L106) | `sanitizeCommand()`, `escapeCommandForShell()` |

### Задачи

- [ ] **3.1** Создать `src/ssh/sanitize.rs`:
  ```rust
  pub fn sanitize_command(command: &str, max_chars: Option<usize>) -> Result<String, SshMcpError>;
  pub fn escape_command_for_shell(command: &str) -> String;
  ```
- [ ] **3.2** Создать `src/ssh/command.rs`:
  ```rust
  pub struct CommandOutput {
      pub stdout: String,
      pub stderr: String,
      pub exit_code: Option<u32>,
  }

  impl SshConnectionManager {
      pub async fn exec_command(
          &self,
          command: &str,
          timeout: Duration,
      ) -> Result<CommandOutput, SshMcpError>;
  }
  ```
- [ ] **3.3** Реализовать timeout через `tokio::time::timeout`
- [ ] **3.4** Graceful abort: при timeout отправлять `pkill -f 'command'`

### Алгоритм exec_command

```
1. ensure_connected()
2. Если есть su_channel (elevated) → использовать его
3. Иначе → channel_open_session() + exec()
4. Собрать stdout/stderr через ChannelMsg::Data
5. Обработать ChannelMsg::ExitStatus
6. timeout → pkill + reject
```

---

## Phase 4: su/sudo Elevation

**Цель**: Интерактивная elevation через su и sudo wrapping.

> [!CAUTION]
> **Сложная фаза!** Требует PTY shell и парсинга промптов.

### Референсные файлы TypeScript

| Файл | Что взять |
|------|-----------|
| [src/index.ts:231-311](file:///home/stfu/ai/ssh-mcp/src/index.ts#L231-L311) | `ensureElevated()` — su логика |
| [src/index.ts:504-538](file:///home/stfu/ai/ssh-mcp/src/index.ts#L504-L538) | Использование `suShell` для команд |
| [src/index.ts:465-476](file:///home/stfu/ai/ssh-mcp/src/index.ts#L465-L476) | sudo wrapping с password pipe |

### Задачи

- [ ] **4.1** Создать `src/ssh/elevation.rs`:
  ```rust
  impl SshConnectionManager {
      pub async fn ensure_elevated(&self) -> Result<(), SshMcpError>;
      pub fn get_sudo_password(&self) -> Option<&str>;
      pub fn get_su_password(&self) -> Option<&str>;
      pub async fn set_su_password(&self, pwd: Option<String>) -> Result<(), SshMcpError>;
  }
  ```

### su Elevation Algorithm

```
1. request_pty("xterm", 80, 24)
2. request_shell()
3. Send "su -\n"
4. Wait for /password[: ]/i
5. Send password + "\n"
6. Wait for "#" (root prompt)
7. Store channel for reuse
```

### sudo Wrapping

```rust
fn wrap_sudo_command(command: &str, password: Option<&str>) -> String {
    match password {
        None => format!("sudo -n sh -c '{}'", escape(command)),
        Some(pwd) => format!(
            "printf '%s\\n' '{}' | sudo -p \"\" -S sh -c '{}'",
            escape(pwd), escape(command)
        ),
    }
}
```

---

## Phase 5: MCP Tools Integration

**Цель**: Регистрация `exec` и `sudo-exec` tools через rmcp SDK.

### Референсные файлы TypeScript

| Файл | Что взять |
|------|-----------|
| [src/index.ts:350-412](file:///home/stfu/ai/ssh-mcp/src/index.ts#L350-L412) | `exec` tool registration |
| [src/index.ts:414-485](file:///home/stfu/ai/ssh-mcp/src/index.ts#L414-L485) | `sudo-exec` tool registration |
| [src/index.ts:341-348](file:///home/stfu/ai/ssh-mcp/src/index.ts#L341-L348) | McpServer initialization |

### Задачи

- [ ] **5.1** Создать `src/tools/mod.rs`
- [ ] **5.2** Создать `src/tools/exec.rs`:
  ```rust
  use rmcp::{tool, tool_router, handler::server::tool::ToolRouter};

  #[tool_router]
  impl SshMcpServer {
      #[tool(description = "Execute a shell command on the remote SSH server")]
      async fn exec(&self, command: String) -> Result<CallToolResult, McpError> {
          let sanitized = sanitize_command(&command, self.config.max_chars)?;
          let output = self.connection.exec_command(&sanitized, self.timeout).await?;
          Ok(CallToolResult::success(vec![Content::text(output.stdout)]))
      }
  }
  ```
- [ ] **5.3** Создать `src/tools/sudo_exec.rs`:
  - Условная регистрация (если не `--disable-sudo`)
  - sudo wrapping с паролем
- [ ] **5.4** Создать `src/server.rs`:
  ```rust
  pub struct SshMcpServer {
      config: Config,
      connection: Arc<SshConnectionManager>,
      tool_router: ToolRouter<Self>,
  }

  impl ServerHandler for SshMcpServer {
      // ... implement required methods
  }
  ```
- [ ] **5.5** Обновить `src/main.rs`:
  - Parse CLI args
  - Create SshMcpServer
  - Start stdio transport с `server.serve(stdio()).await`
  - Graceful shutdown (SIGINT, SIGTERM)

---

## Phase 6: Testing and Verification

**Цель**: Unit и integration тесты, ручная проверка.

### Референсные файлы TypeScript

| Файл | Что взять |
|------|-----------|
| [test/persistent-connection.test.ts](file:///home/stfu/ai/ssh-mcp/test/persistent-connection.test.ts) | Connection lifecycle тесты |
| [test/sudo-exec.test.ts](file:///home/stfu/ai/ssh-mcp/test/sudo-exec.test.ts) | sudo тесты |
| [test/maxChars.test.ts](file:///home/stfu/ai/ssh-mcp/test/maxChars.test.ts) | max_chars validation |
| [docker-compose.yml](file:///home/stfu/ai/ssh-mcp/docker-compose.yml) | Test SSH container config |

### Задачи

- [ ] **6.1** Создать `tests/unit.rs`:
  - `test_sanitize_command_valid`
  - `test_sanitize_command_empty`
  - `test_sanitize_command_too_long`
  - `test_escape_command_for_shell`
  - `test_config_parse_max_chars`
- [ ] **6.2** Создать `tests/integration.rs` с testcontainers:
  ```rust
  use testcontainers::{runners::AsyncRunner, GenericImage};

  #[tokio::test]
  async fn test_exec_echo() {
      let container = GenericImage::new("linuxserver/openssh-server", "latest")
          .with_env_var("PASSWORD_ACCESS", "true")
          .with_env_var("USER_PASSWORD", "test")
          .start().await.unwrap();
      // ...
  }
  ```
- [ ] **6.3** Тесты: connection, reconnect, concurrent commands, timeout, sudo
- [ ] **6.4** Ручная проверка с MCP Inspector:
  ```bash
  npx @modelcontextprotocol/inspector ./target/release/ssh-mcp-rs -- \
    --host=YOUR_HOST --user=root --password=pass
  ```

---

## Финальная структура проекта

```
ssh-mcp-rs/
├── Cargo.toml
├── Docs/
│   ├── rmcp-sdk.md
│   └── russh-library.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config.rs
│   ├── error.rs
│   ├── server.rs
│   ├── ssh/
│   │   ├── mod.rs
│   │   ├── config.rs
│   │   ├── handler.rs
│   │   ├── connection.rs
│   │   ├── command.rs
│   │   ├── elevation.rs
│   │   └── sanitize.rs
│   └── tools/
│       ├── mod.rs
│       ├── exec.rs
│       └── sudo_exec.rs
└── tests/
    ├── unit.rs
    └── integration.rs
```

---

## Оценка времени по фазам

| Фаза | Сложность | Часы |
|------|-----------|------|
| Phase 1 | 🟢 Low | 2-3 |
| Phase 2 | 🟡 Medium | 4-6 |
| Phase 3 | 🟡 Medium | 3-4 |
| Phase 4 | 🔴 High | 6-10 |
| Phase 5 | 🟢 Low | 3-4 |
| Phase 6 | 🟡 Medium | 4-6 |
| **Total** | | **22-33** |
