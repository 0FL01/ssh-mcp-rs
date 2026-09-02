# Goal: SSH jump host support

Status: complete
Source: user-approved SSH jump-host plan in the current task
Last updated: 2026-09-02

## Objective
Allow one configured MCP server to reach a target SSH host through one independently authenticated SSH jump host, including safe transfer behavior and documented OpenCode configuration.

## Execution Directive
Complete the frozen Required Outcomes using the listed Change Envelope and Primary Evidence. Work on the smallest unresolved outcome. Do not add requirements from reviews, tests, tools, speculative risks, or optional source text. Finish when every required outcome is resolved and affected constraints remain satisfied.

## Frozen Contract

### Required Outcomes
- R1: Configure one jump endpoint with credentials independent from the target.
  - Source: approved plan and key/password configuration examples.
  - Acceptance: `--jump=USER@HOST[:PORT]` plus exactly one of `--jump-key` or `--jump-password` is parsed and validated; target credentials are not inherited.
  - Primary evidence: focused configuration unit tests.
  - Status: verified
  - Evidence: `cargo test config::tests --lib` passed 25 focused configuration tests, including jump parsing, independent key expansion, credential exclusivity, and empty-password normalization.
- R2: Route the persistent target SSH session through the jump host.
  - Source: approved native `russh` design.
  - Acceptance: commands authenticate both hops independently and target-relative addresses such as `127.0.0.1:2222` are reached through `direct-tcpip`; direct mode remains supported.
  - Primary evidence: focused Docker integration test plus existing direct tests compiling.
  - Status: verified
  - Evidence: `cargo test --test docker_integration_test jump_tests -- --nocapture` passed the distinct-key command route and password-jump command route.
- R3: Preserve transfer access through the route.
  - Source: approved transfer behavior.
  - Acceptance: `exec-raw` uses the persistent route; key-authenticated local transports receive a generated jump ProxyCommand; password-only routes fall back safely to `exec-raw` and explicit unsupported transports return an error.
  - Primary evidence: transfer option unit tests and focused Docker transfer test.
  - Status: verified
  - Evidence: transfer unit tests passed; the focused Docker tests passed explicit SFTP through distinct keys, password-route `auto` fallback to `exec-raw`, and explicit unsupported SFTP error behavior.
- R4: Document the supported configuration and limitations.
  - Source: explicit request to supplement documentation.
  - Acceptance: README contains OpenCode examples for distinct keys and passwords, explains key handling, host-key behavior, and transfer limitations.
  - Primary evidence: README diff reviewed against implemented flags.
  - Status: verified
  - Evidence: README now documents the exact OpenCode key/key and password configuration, local key handling, host-key identities, transfer fallback, and platform limitation.
- R5: Build and publish the completed change.
  - Source: explicit instruction to commit, push, and build.
  - Acceptance: required Rust checks and release build succeed; implementation commits are pushed to the current remote branch.
  - Primary evidence: command results, commit hashes, and remote push result.
  - Status: verified
  - Evidence: `cargo fmt --all -- --check`, Clippy with `-D warnings`, `cargo test --all-features`, and `cargo build --release` passed; implementation commit `34cae4a` was pushed to `origin/dev`.

### Constraints
- C1: Use the existing `russh` and OpenSSH mechanisms; add no dependency, forwarding service, or raw user ProxyCommand.
- C2: Keep private keys local and preserve separate host-key verification for jump and target.
- C3: Preserve unrelated working-tree changes and do not include them in this objective's commits.
- C4: Keep direct non-jump behavior intact.

### Non-goals
- Multiple jump hops, SSH config discovery, agent forwarding, key passphrases, `sshpass`, and per-tool routes.
- Windows ProxyCommand support; `auto` may use `exec-raw` there.
- Refactoring unrelated target dual-auth or reconnect-generation behavior.

## Change Envelope
- Target: startup configuration, SSH route establishment, transfer endpoint options, focused tests, README, and this goal document.
- Expected paths, symbols, and direct consumers: `src/config.rs`, `src/ssh/{config,connection,handler}.rs`, `src/server.rs`, `src/server/testing.rs`, `src/transfer/{mod,openssh,rsync}.rs`, nearest tests/fixtures, `README.md`, and public re-exports.
- Allowed and forbidden artifacts: source, tests, fixture, and documentation edits are allowed; new dependencies, services, persistent state, and generic proxy abstractions are forbidden.
- User or harness budget: smallest complete one-hop implementation; no unrelated cleanup.

## Current Checkpoint
- Closes: none; objective complete.
- Smallest next action: stop.
- Expected evidence: all required outcomes remain verified.
- Stop or replan if: a new authoritative request creates a separate objective.

## Current State
- Resolved: R1-R5 configuration, native route, transfers, documentation, build, commit, and push.
- Last relevant evidence: all-feature tests and release build passed; `34cae4a` is on `origin/dev`.
- Blocker: none.
- Next: none.

## Material Decisions
- 2026-09-02: Jump credentials are explicit and exactly one of key/password; no inheritance from target.
- 2026-09-02: Native routing uses nested `russh`; local key transports use a generated structured ProxyCommand; password routes use `exec-raw`.
- 2026-09-02: Public `Config` gains one optional nested jump field; no compatibility wrapper is added.

## Checkpoint History
- 2026-09-02: Contract frozen after parallel configuration, native-route, and transfer reviews; next checkpoint is R1.
- 2026-09-02: R1 verified by focused configuration tests; next checkpoint is native route R2.
- 2026-09-02: R2-R4 verified by transfer unit tests and two Docker jump tests; next checkpoint is build and publication R5.
- 2026-09-02: R5 verified; implementation commit `34cae4a` pushed to `origin/dev`; closure check passed.

## Completion
- Resolved outcomes: R1-R5.
- Commands and artifacts: focused config/transfer tests, two Docker jump tests, full all-feature suite, Clippy, formatting, release build, README, and implementation commit `34cae4a`.
- Constraint and diff-scope check: no dependency, proxy service, raw ProxyCommand, credential inheritance, or unrelated working-tree changes were included.
- Final status: complete
