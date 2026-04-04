# Lific — Agent Instructions

## What is this

Lific is a local-first, SQLite-backed issue tracker with a REST API and MCP interface. Single Rust binary, no external dependencies.

## Project structure

```
src/
  main.rs          — entry point, CLI dispatch, server setup
  api/mod.rs       — REST API routes (axum)
  mcp/
    mod.rs         — MCP server setup
    tools.rs       — MCP tool implementations
    schemas.rs     — MCP input schemas
  db/
    mod.rs         — connection pool, test helpers
    models.rs      — data structs
    queries/       — all SQL (issues, projects, pages, resources, search)
    migrate.rs     — migration runner
  cli/mod.rs       — clap CLI definitions
  config.rs        — TOML config loading
  auth.rs          — API key auth
  oauth.rs         — OAuth 2.1 flow
  backup.rs        — SQLite backup + WAL checkpoint
  error.rs         — error types
migrations/        — SQL migration files
deploy.sh          — deploy to basement server
```

## Building and testing

```bash
cargo build           # debug build
cargo build --release # release build
cargo test            # run all tests (110+, ~1.5s)
```

## Testing philosophy

Every new MCP tool or API endpoint must ship with tests. See the Testing Philosophy page in Lific (LIF-DOC-2) for full conventions. Key points:

- All tests use in-memory SQLite via `crate::db::open_memory()`
- MCP tool tests call methods directly via `Parameters(...)` on a `LificMcp` instance
- API tests use `tower::ServiceExt::oneshot` against the axum router
- Test names describe behavior, not implementation

## Deploying

The production instance runs on the basement server (`blake@basement`) at `/opt/ada/lific/`.

**To deploy after making changes:**

```bash
./deploy.sh
```

This builds a release binary, copies it to the basement server, swaps it in place, and restarts the `lific.service` systemd unit. The database and config are untouched — only the binary is replaced.

To deploy a pre-built binary without rebuilding:

```bash
./deploy.sh --skip
```

**Do not** deploy without running `cargo test` first. All tests must pass.

## Production details

- **Binary location**: `/opt/ada/lific/lific`
- **Database**: `/opt/ada/lific/lific.db`
- **Config**: `/opt/ada/lific/lific.toml`
- **Service**: `lific.service` (systemd, runs as user `blake`)
- **Backups**: `/opt/ada/lific/backups/` (every 30min, retain 48)
- **Access**: behind Tailscale Serve at `https://fedora.tailb93ac8.ts.net/lific`
- **Syncthing**: Vaultwarden data is synced, not Lific (Lific has its own backup system)

## Issue tracking

Lific tracks its own development. The project identifier is `LIF`. Use the MCP tools or REST API to manage issues. Modules include Testing and others as needed.

When finishing work on a Lific issue, mark it as `done`. When removing a feature that has an associated test issue, mark that issue as `cancelled`.

## Things NOT in this project

- No Plane import (removed — was a one-time migration tool)
- No Docker setup (single binary, no need)
- No frontend (LIF-9 is backlog for a future web UI)
