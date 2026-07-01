# LoxQL

LoxQL is an experimental native PostgreSQL client built with Rust, GPUI, and
gpui-component. The current app is a workbench-style desktop UI with a
connection picker, catalog panel, SQL workspace, result grid, and status/event
panel.

The project is early and intentionally small. Today it is wired primarily around
a local development PostgreSQL profile named `local-dev`.

## Features

- Native GPUI desktop shell with client-side window decorations.
- Searchable connection picker in the title bar.
- Built-in `local-dev` PostgreSQL connection profile.
- SQL workspace with tabs, a sample query editor, and Run, Stop, and Explain
  actions.
- Query execution through `tokio-postgres`.
- SQL parsing before execution with `pg_query`.
- Result rendering for common scalar and JSON PostgreSQL values.
- Catalog refresh for user tables from `pg_catalog.pg_tables`.
- Bottom status panel showing recent PostgreSQL/runtime events.
- Bundled JetBrains Mono font assets.

## Current Limitations

- Only the `local-dev` connection profile is implemented.
- The "Create a new Connection" picker action is a placeholder.
- Cancel currently records a notice; it does not cancel an in-flight PostgreSQL
  query yet.
- The editor is currently a rendered sample buffer, not a full text-editing
  surface.
- The catalog view is shallow and lists up to 64 user tables.
- Connection credentials are hard-coded for local development.

## Requirements

- Rust toolchain with edition 2024 support.
- Docker, if you want to use the included local PostgreSQL service.
- Network access on the first build, because GPUI and gpui-component are pulled
  from Git dependencies.

## Local Database

The repository includes a `docker-compose.yml` for PostgreSQL 17:

```powershell
docker compose up -d postgres
```

It starts a database that matches the built-in connection profile:

- Host: `localhost`
- Port: `5432`
- Database: `app_db`
- User: `postgres`
- Password: `postgres`

The sample query in the workspace uses `generate_series`, so it can run without
creating any tables first.

## Running

```powershell
cargo run
```

In the app:

1. Open the connection picker in the title bar.
2. Select `local-dev`.
3. Use `Run` to execute the sample SQL.
4. Use the catalog refresh control to reload visible database objects after
   connecting.

## Development

Check the project:

```powershell
cargo check
```

Format the project:

```powershell
cargo fmt
```

## Project Structure

- `src/main.rs` initializes GPUI, registers fonts, and opens the main window.
- `src/app.rs` owns top-level app state and coordinates child component events.
- `src/pg.rs` contains connection profiles, PostgreSQL commands/events, query
  execution, catalog loading, and result conversion.
- `src/ui/connection_picker.rs` renders the title-bar picker and emits
  connection events.
- `src/ui/workspace.rs` renders the SQL workspace, query toolbar, and result
  grid.
- `src/ui/database_panel.rs` renders the database/catalog panel.
- `src/ui/bottom_panel.rs` renders status and recent PostgreSQL events.
- `assets/fonts/jetbrains-mono/` contains the bundled editor/UI font.

## Architecture Notes

UI components are GPUI entities where they need internal state. Child components
emit typed events, and `AppShell` decides how those events affect application
state. PostgreSQL work runs on a dedicated Tokio runtime and communicates back to
the UI through command and event channels.

The connection picker owns only picker UI state: open/closed state, search input
state, local profile list, and selected profile display. It emits
`ConnectionPickerEvent` rather than mutating `AppShell` directly.
