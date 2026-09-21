# cenchat-server

WebSocket chat server written in Rust with `tokio` and `tokio-tungstenite`. Supports named users, rooms, and broadcast messaging.

## Running

```bash
cargo build
cargo run       # starts listening on ws://127.0.0.1:9001
```

## Protocol

The server communicates over WebSocket text frames. Client-to-server messages are plain lines; lines starting with `/` are commands.

### Client commands

| Command | Description |
| --- | --- |
| `/name <name>` | Set your display name (must be non-empty, no whitespace). Required before joining a room or chatting. |
| `/join <room>` | Join a room, creating it if it doesn't exist. Leaves any room you're currently in first. |
| `/leave` | Leave your current room. |
| `/who` | List the names of everyone in your current room. |
| *(anything else)* | Sent as a chat message, broadcast to everyone in your current room. |

### Server responses

| Prefix | Meaning |
| --- | --- |
| `SYS <text>` | System notification (e.g. name set, joined/left room). |
| `ERR <text>` | Error, usually from a missing name/room or malformed command. |
| `MSG <name>: <text>` | A chat message from `<name>`. |
| `USERS <name1> <name2> ...` | Response to `/who`, space-separated and sorted. |

An unrecognized `/command` returns `ERR unknown command: <command>`.

## Example session

```bash
cargo run

# in another terminal, using cenchat-cli or any WebSocket client:
cargo run -- ws://127.0.0.1:9001
/name alice
/join lobby
hello everyone!
/who
/leave
```

## Architecture

- `src/main.rs` — binds the TCP listener and spawns a task per connection
- `src/connection.rs` — per-connection read/write loop and command dispatch
- `src/protocol.rs` — line parsing into `Command`s and response message formatting
- `src/state.rs` — shared, mutex-protected room/member state (`Shared` = `Arc<Mutex<SharedState>>`)

State is fully in-memory; there is no persistence or authentication.

## Development notes / current gaps

- No tests (no `#[test]` or `tests/` dirs)
- No lint/format config (no `clippy.toml`, no `rustfmt.toml`)
- No CI (no `.github/workflows`)
- No env/config files, no database

## Related

- [`cenchat-cli`](https://github.com/khajer/cenchat-cli) — terminal client for this server
