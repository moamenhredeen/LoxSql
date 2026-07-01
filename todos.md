# LoxQL TODOs

This file tracks what has been built so far and the path toward a full-featured
native PostgreSQL client.

## Done So Far

- Created the initial Rust/GPUI desktop app shell.
- Added gpui-component setup and client-side window decorations.
- Registered bundled JetBrains Mono fonts for the UI/editor.
- Built the main app layout:
  - Title bar
  - Connection picker
  - Database/catalog panel
  - SQL workspace
  - Result grid
  - Bottom status/event panel
- Added a local PostgreSQL development profile:
  - `local-dev`
  - `localhost:5432`
  - `app_db`
  - `postgres` / `postgres`
- Added `docker-compose.yml` for a matching PostgreSQL 17 development database.
- Added a PostgreSQL runtime backed by Tokio.
- Added command/event channels between the UI and PostgreSQL runtime.
- Implemented basic PostgreSQL commands:
  - Connect
  - Execute SQL
  - Load catalog node
  - Cancel placeholder
- Added SQL parse validation with `pg_query` before execution.
- Rendered query results into a simple table.
- Added result conversion for common PostgreSQL value types:
  - booleans
  - integers
  - floats
  - JSON/JSONB
  - text-like fallback values
  - NULL display
- Added catalog loading from `pg_catalog.pg_tables`.
- Added a bottom panel that records recent PostgreSQL events.
- Converted the connection picker into a GPUI entity so it can own internal UI
  state.
- Moved connection picker state into the picker:
  - open/closed state
  - search input state
  - selected profile display
  - local profile list
- Moved the connection picker trigger and popover into one cohesive component.
- Removed direct `AppShell` access from the connection picker.
- Added typed connection picker events:
  - `ConnectionSelected`
  - `CreateNewConnection`
- Wired `AppShell` to subscribe to connection picker events.
- Added a project README with setup, architecture, limitations, and usage notes.

## Current Known Limitations

- Only one hard-coded connection profile exists.
- The "Create a new Connection" action is a placeholder.
- Connection credentials are hard-coded.
- The SQL editor is still a rendered sample buffer, not a real editable editor.
- Query cancellation is only a notice and does not cancel an active query.
- The catalog tree is shallow.
- Result rendering is simple and not virtualized.
- The result grid does not support sorting, copying, filtering, resizing, or
  pagination.
- There is no persistent settings or connection storage.
- There is no transaction/session state UI beyond simple status messages.
- Error handling is basic.
- There are no automated tests yet.

## Next Plan

### 1. Stabilize The Core App Model

- Define clear ownership boundaries between:
  - `AppShell`
  - connection picker
  - workspace/editor
  - database panel
  - PostgreSQL runtime
- Keep child UI components event-driven.
- Avoid child components reading or mutating `AppShell` directly.
- Replace temporary sample state with explicit app/session state structs.
- Remove debug placeholder behavior from `CreateNewConnection`.
- Clean up unused imports and warnings.

### 2. Build Real Connection Management

- Add a connection profile model that supports:
  - name
  - host
  - port
  - database
  - username
  - password handling
  - SSL mode
  - optional connection parameters
- Add a create/edit connection dialog.
- Add connection validation/test flow.
- Persist profiles locally.
- Support deleting and duplicating profiles.
- Support recent connections.
- Show connection state clearly:
  - disconnected
  - connecting
  - connected
  - failed
- Avoid storing plaintext secrets long term if a platform credential store is
  available.

### 3. Replace The Sample SQL Editor

- Introduce a real editable SQL buffer.
- Add cursor, selection, editing, and scrolling behavior.
- Add keyboard shortcuts for:
  - run query
  - run selected query
  - format SQL
  - explain
  - cancel
- Add SQL syntax highlighting.
- Add query boundary detection.
- Support multiple query tabs.
- Track dirty state and tab titles.

### 4. Improve Query Execution

- Execute the selected statement or current statement.
- Stream or progressively handle large results where possible.
- Add real query cancellation.
- Track active query handles per session.
- Improve error display with PostgreSQL error fields.
- Add query timing and row count metadata.
- Support commands that return no rows.
- Support multiple result sets if needed.
- Add query history.

### 5. Build A Real Result Grid

- Virtualize large result sets.
- Add column resizing.
- Add cell selection.
- Add row selection.
- Add copy cell/row/selection as TSV/CSV/JSON.
- Add sorting and filtering where appropriate.
- Add NULL-specific styling.
- Add type-aware rendering for:
  - timestamps
  - dates
  - UUIDs
  - arrays
  - JSON
  - bytea
- Add result export.

### 6. Expand The Catalog Browser

- Load databases, schemas, tables, views, functions, indexes, extensions, and
  constraints.
- Make the catalog tree lazy-loaded and refreshable per node.
- Add object details panels.
- Add table preview actions.
- Add context menu actions:
  - copy name
  - copy qualified name
  - select rows
  - inspect DDL
  - refresh
- Track selected object in app state.

### 7. Add PostgreSQL Session Features

- Show current database, schema, user, server version, and transaction status.
- Support changing search path.
- Support transaction controls:
  - begin
  - commit
  - rollback
- Surface notices and warnings clearly.
- Add connection keepalive/health checks.
- Handle disconnects and reconnects.

### 8. Add Application Persistence

- Store window/layout state.
- Store saved connections.
- Store recent SQL files or scratch buffers.
- Store query history.
- Store UI preferences.
- Add import/export for configuration.

### 9. Polish The UI

- Replace placeholder text and temporary controls.
- Add proper empty states.
- Add loading states for connect, catalog refresh, and query execution.
- Add keyboard navigation for the connection picker and catalog.
- Add accessible focus handling.
- Add consistent icons and tooltips.
- Make result/status text fit reliably at small window sizes.

### 10. Testing And Reliability

- Add unit tests for PostgreSQL result conversion.
- Add tests for event handling between components.
- Add tests for connection profile serialization.
- Add integration tests against the Docker PostgreSQL service.
- Add smoke tests for startup and basic query execution.
- Add CI checks:
  - `cargo fmt --check`
  - `cargo check`
  - `cargo test`
  - clippy, once warnings are under control

### 11. Packaging

- Decide target platforms.
- Add app metadata and icons.
- Add release builds.
- Add installer/package flow per platform.
- Document system dependencies.
- Add versioning and changelog.

## Near-Term Priority

1. Remove placeholder/debug behavior from `CreateNewConnection`.
2. Add a real connection dialog and profile storage.
3. Replace the sample editor with an editable SQL buffer.
4. Implement real query cancellation.
5. Make the catalog tree lazy-loaded and object-aware.
6. Upgrade the result grid for large result sets.
