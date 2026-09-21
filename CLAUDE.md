# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository

This is `cenchat-server`, one of two independent Rust repos in the `cenchat` chat project (the other is `cenchat-cli`, a separate repository at `git@github.com:khajer/cenchat-cli.git`). This repo has its own remote (`git@github.com:khajer/cenchat.git`) and is **not** part of a Cargo workspace — build, run, and commit from within this directory.

## Commands

```bash
cargo build
cargo run        # starts listening on ws://127.0.0.1:9001
cargo check       # fast type-check without building
```

There are no tests (`#[test]` / `tests/`), no lint/format config (`clippy.toml`, `rustfmt.toml`), and no CI in this repo.

To exercise the server manually, connect with `cenchat-cli` (sibling repo) or any WebSocket client and drive the protocol described below.

## Architecture

Connection lifecycle: `main.rs` binds a `TcpListener` on `127.0.0.1:9001` and spawns one task per accepted connection running `connection::handle_connection`. Each connection task performs the WS handshake, then splits the socket into a `write` half and `read` half. Writes are funneled through an `mpsc::unbounded_channel`, with a dedicated writer task draining the receiver — this is what lets other connections' broadcasts reach this socket concurrently with reads. The read loop parses each incoming `Text` message and calls `dispatch`, which owns the per-connection `name`/`room` state (kept as locals, not stored in shared state) and mutates them across calls.

Shared state (`state.rs`): `Shared = Arc<Mutex<SharedState>>`, holding a `HashMap<room_name, Room>` where each `Room` maps `SocketAddr -> (display_name, UnboundedSender<Message>)`. `join`/`leave`/`broadcast`/`user_names` are the only mutation points; `leave` garbage-collects a room once its member map is empty. There is no cross-room state and no persistence — everything resets on process restart.

Protocol (`protocol.rs`): `parse_line` turns a raw client line into a `Command` enum (`Name`, `Join`, `Leave`, `Who`, `Text`, `Unknown`); `sys`/`err`/`chat`/`users_list` build the corresponding outbound `SYS `/`ERR `/`MSG name: text`/`USERS ...` response frames. Adding a new command means extending both the `Command` enum and the `dispatch` match in `connection.rs`.

A client must `/name` before it can `/join`, and must `/join` a room before `/who` or sending chat text works — `dispatch` enforces this ordering by checking the local `name`/`room` `Option`s and returning an `ERR` response rather than by validating in `state.rs`. Full protocol/command reference is in README.md.
