# SSH MCP Server Rust Implementation Blueprint

> **Цель**: Портирование TypeScript MCP сервера на Rust с использованием `rmcp` SDK и `russh` SSH библиотеки.

## 📊 Статус реализации

| Фаза | Статус | Описание |
|------|--------|----------|
| Phase 1 | ✅ Завершено | Project Setup and Basic Structure |
| Phase 2 | ✅ Завершено | SSH Connection Manager |
| Phase 3 | ✅ Завершено | Command Execution |
| Phase 4 | ✅ Завершено | su/sudo Elevation |
| Phase 5 | ✅ Завершено | MCP Tools Integration |
| Phase 6 | ⏳ В ожидании | Testing and Verification |

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

- [x] **2.1** Создать `src/ssh/mod.rs` — модуль SSH
- [x] **2.2** Создать `src/ssh/config.rs`:
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
- [x] **2.3** Создать `src/ssh/handler.rs`:
  - Implement `russh::client::Handler` trait
  - `check_server_key()` — принимать все ключи (или проверять fingerprint)
- [x] **2.4** Создать `src/ssh/connection.rs`:
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

- [x] **3.1** Создать `src/ssh/sanitize.rs`:
  ```rust
  pub fn sanitize_command(command: &str, max_chars: Option<usize>) -> Result<String, SshMcpError>;
  pub fn escape_command_for_shell(command: &str) -> String;
  ```
- [x] **3.2** Создать `src/ssh/command.rs`:
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
- [x] **3.3** Реализовать timeout через `tokio::time::timeout`
- [x] **3.4** Graceful abort: при timeout отправлять `pkill -f 'command'`

### Алгоритм exec_command

```
1. ensure_connected()
2. Если есть su_channel (elevated) → использовать его
3. Иначе → channel_open_session() + exec()
4. Собрать stdout/stderr через ChannelMsg::Data
5. Обработать ChannelMsg::ExitStatus
6. timeout → pkill + reject
```

### Результат фазы

✅ **Завершено**: Созданы модули для выполнения команд:

- `src/ssh/sanitize.rs`:
  - `sanitize_command()` — валидация и нормализация команд
  - `escape_command_for_shell()` — экранирование для shell (pkill)
  - 10 unit-тестов + 2 doc-теста
  
- `src/ssh/command.rs`:
  - `CommandOutput` struct с полями stdout/stderr/exit_code
  - `exec_command()` — основной метод выполнения команд
  - `exec_via_su_shell()` — выполнение через PTY shell (su)
  - `exec_via_channel()` — стандартное выполнение через exec
  - `abort_command()` — graceful abort через pkill при timeout
  - 6 unit-тестов

- Обновлены экспорты в `ssh/mod.rs` и `lib.rs`

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

- [x] **4.1** Создать `src/ssh/elevation.rs`:
  ```rust
  // Реализовано: sudo wrapping utilities
  pub fn wrap_sudo_command(command: &str, password: Option<&str>) -> String;
  pub fn escape_for_shell(s: &str) -> String;
  pub fn is_valid_password(password: &str) -> bool;
  pub fn sanitize_password(password: Option<&str>) -> Option<String>;
  
  // su elevation реализовано в connection.rs:
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

### Результат фазы

✅ **Завершено**: Созданы модули для privilege elevation:

- `src/ssh/elevation.rs`:
  - `wrap_sudo_command()` — оборачивает команду для выполнения с sudo
  - `escape_for_shell()` — экранирование строк для shell (single quotes)
  - `is_valid_password()` — валидация пароля
  - `sanitize_password()` — санитизация пароля
  - 11 unit-тестов + 2 doc-теста

- `src/ssh/connection.rs` (su elevation):
  - `ensure_elevated()` — интерактивная elevation через PTY shell + su
  - `handle_su_elevation()` — обработка prompts и authentication
  - `get_su_password()` / `get_sudo_password()` — получение паролей
  - `set_su_password()` — установка su пароля с автоматической elevation

- Обновлены экспорты в `ssh/mod.rs` и `lib.rs`

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

- [x] **5.1** Создать `src/tools/mod.rs`
- [x] **5.2** Реализовать `exec` tool в `src/server.rs`:
  - Tool definition через `Tool::new()` с JSON schema
  - Метод `execute_command()` для выполнения команд
  - Sanitization и error handling
- [x] **5.3** Реализовать `sudo-exec` tool:
  - Условная регистрация (если не `--disable-sudo`)
  - sudo wrapping через `wrap_sudo_command()`
  - Метод `execute_sudo_command()` для выполнения sudo команд
- [x] **5.4** Создать `src/server.rs`:
  - `SshMcpServer` struct с config, connection, timeout, max_chars
  - Реализация `ServerHandler` trait с `get_info()`, `list_tools()`, `call_tool()`
  - Graceful shutdown через `shutdown()` метод
- [x] **5.5** Обновить `src/main.rs`:
  - Parse CLI args через clap
  - Create SshMcpServer
  - Start stdio transport с `server.serve(rmcp::transport::io::stdio()).await`
  - Graceful shutdown (SIGINT, SIGTERM) через tokio signals

### Результат фазы

✅ **Завершено**: Полная интеграция MCP tools:

- `src/tools/mod.rs`:
  - `ExecParams` и `SudoExecParams` structs для JSON schema
  - 2 unit-теста для deserialize
  
- `src/server.rs`:
  - `SshMcpServer` struct — основной MCP сервер
  - `ServerHandler` implementation:
    - `get_info()` — возвращает ServerInfo с версией и capabilities
    - `list_tools()` — список доступных tools (exec + sudo-exec если enabled)
    - `call_tool()` — роутинг вызовов к execute_command/execute_sudo_command
  - `exec_tool()` / `sudo_exec_tool()` — Tool definitions с JSON schema
  - `execute_command()` — выполнение команд через SSH
  - `execute_sudo_command()` — выполнение команд с sudo
  - 3 unit-теста
  
- `src/main.rs`:
  - Полная интеграция с MCP server
  - stdio transport
  - Graceful shutdown с SIGINT/SIGTERM
  
- `src/lib.rs`:
  - Обновлённые экспорты: `SshMcpServer`, `ExecParams`, `SudoExecParams`

### Архитектурные решения

1. **Отказ от tool_router macro**: Вместо использования `#[tool_router]` на отдельных классах,
   tools реализованы непосредственно в `ServerHandler::call_tool()`. Это упрощает код и 
   избегает проблем с trait bounds.

2. **Tool definitions**: Используется `Tool::new()` метод с `Arc<JsonObject>` для input schema,
   что соответствует API rmcp 0.12.

3. **Error handling**: Все ошибки возвращаются как `CallToolResult::error()` вместо 
   `Err(McpError)`, что соответствует TypeScript реализации (ошибки не прерывают сессию).



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

- [x] **6.1** Исправление clippy warnings (redundant closures, manual_async_fn, is_none_or, etc.)
- [ ] **6.2** Создать `tests/unit.rs`:
  - `test_sanitize_command_valid`
  - `test_sanitize_command_empty`
  - `test_sanitize_command_too_long`
  - `test_escape_command_for_shell`
  - `test_config_parse_max_chars`
- [ ] **6.3** Создать `tests/integration.rs` с testcontainers:
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
- [ ] **6.4** Тесты: connection, reconnect, concurrent commands, timeout, sudo
- [ ] **6.5** Ручная проверка с MCP Inspector:
  ```bash
  npx @modelcontextprotocol/inspector ./target/release/ssh-mcp-rs -- \
    --host=YOUR_HOST --user=root --password=pass
  ```

---

## Phase 7: Code Polishing & Quality Assurance

**Цель**: Улучшение качества кода, производительности и безопасности.

### Задачи

- [ ] **7.1** **Zero-copy optimization**: Использование `Cow<'a, str>` или `&str` в санитизации команд.
- [ ] **7.2** **Logging enhancement**: Замена `println!` на `tracing` макросы во всех модулях.
- [ ] **7.3** **Secure sensitive data**: Использование `secrecy` crate для хранения паролей в памяти.
- [ ] **7.4** **Custom error types**: Более детальная типизация ошибок для каждого модуля.
- [ ] **7.5** **Resource management**: Проверка на утечки дескрипторов при частых реконнектах.

---

## Phase 8: Documentation & Examples

**Цель**: Создание качественной документации для пользователей и разработчиков.

### Задачи

- [x] **8.1** Создать `README.md` с гайдом по сборке и настройке.
- [ ] **8.2** Добавить примеры конфигурации для различных MCP клиентов (Cursor, Claude Desktop, VS Code).
- [ ] **8.3** Оформить `examples/` с использованием библиотеки программно.
- [ ] **8.4** Проверить doc-комментарии (`cargo doc --open`).

---

## Phase 9: Production Readiness & Deployment

**Цель**: Подготовка к релизу и CI/CD.

### Задачи

- [ ] **9.1** Создать `Dockerfile` для мультиплатформенной сборки.
- [ ] **9.2** Настроить GitHub Actions для автоматического тестирования и сборки релизов.
- [ ] **9.3** Оптимизация размера бинарного файла (LTO, stripping).
- [ ] **9.4** Публикация на crates.io (опционально).

---

## Финальная структура проекта

```
ssh-mcp-rs/
├── Cargo.toml
├── IMPLEMENTATION-BLUEPRINT.md
├── README.md            # (Phase 8)
├── Docs/
│   ├── rmcp-sdk.md
│   └── russh-library.md
├── src/
│   ├── main.rs          # Entry point, CLI parsing, MCP server startup
│   ├── lib.rs           # Re-exports for library usage
│   ├── config.rs        # Config/Args structs, validation
│   ├── error.rs         # SshMcpError enum
│   ├── server.rs        # SshMcpServer, ServerHandler impl
│   ├── ssh/
│   │   ├── mod.rs       # SSH module exports
│   │   ├── config.rs    # SshConfig struct
│   │   ├── handler.rs   # SshHandler (russh client::Handler)
│   │   ├── connection.rs # SshConnectionManager
│   │   ├── command.rs   # Command execution, CommandOutput
│   │   ├── elevation.rs # su/sudo utilities
│   │   └── sanitize.rs  # Command sanitization
│   └── tools/
│       └── mod.rs       # ExecParams, SudoExecParams (tools in server.rs)
└── tests/               # (Phase 6)
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
| Phase 7 | 🟡 Medium | 3-4 |
| Phase 8 | 🟢 Low | 2-3 |
| Phase 9 | 🟡 Medium | 3-4 |
| **Total** | | **30-44** |
