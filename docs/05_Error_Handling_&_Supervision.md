# Architectural Documentation: Error Handling, Supervision & Corruption Resilience

This document outlines the robust error handling, asynchronous task supervision, and database corruption resilience architecture implemented in CRApp version 0.2.4.

## 1. Architectural Philosophy
Desktop applications operating over local file systems and SQLite databases must be exceptionally resilient. A single malformed file or unexpected power loss should never result in an unhandled crash or permanent application lockout. 

Our error handling architecture enforces strict boundaries between:
1. **Expected (Recoverable) Errors:** Network timeouts, SQL constraint violations, missing optional files, or user input errors.
2. **Unexpected (Fatal/Unrecoverable) Errors:** Memory corruption, panic unwindings in third-party crates, or critical hardware failure.

## 2. Core Components

### A. Pre-GUI Initialization Gatekeeper (`src/main.rs`)
Before launching the graphical user interface (`eframe::run_native`), `main.rs` executes a strictly ordered initialization chain via `run_gatekeeper()`:
1. Initializes storage directories and the structured logging subsystem (`tracing`).
2. Bootstraps the multi-threaded Tokio asynchronous runtime and enters its context guard.
3. Establishes the SQLite connection pool and executes schema migrations.

If any foundational step fails, the error is intercepted on the clean main thread and displayed to the user via a blocking OS dialog (`rfd::MessageDialog`). This guarantees the user is clearly informed of why initialization failed without locking up or crashing the OS window system.

### B. Low-Overhead Crash Handler & Panic Hook
To capture unexpected panics anywhere in the application without deadlocking UI threads:
- A custom hook (`setup_crash_handler`) is registered at the absolute start of `main()`.
- On panic, the hook captures the panic payload and backtrace location, formats them into a structured log entry, writes directly to `data/logs/crash.log`, and immediately calls `std::process::exit(1)`.
- **Strict Rule:** Native blocking UI dialogs (`rfd`) are strictly prohibited inside the panic hook to prevent OS window system deadlocks on broken threads.

### C. Unified Domain Errors (`src/error.rs`)
All internal operations bubble up standardized error enums deriving `Clone, Debug, thiserror::Error`:
- `DbError`: Maps exact SQLx error variants (record not found, constraint violations, connection failures, schema migration failures) while preserving contextual strings.
- `AppError`: Encapsulates database errors (`AppError::Database`), IO failures, serialization failures, Tokio task failures, and background panics.

### D. Asynchronous Task Supervision (`src/task.rs`)
In an `eframe` GUI application, background asynchronous tasks running on Tokio worker threads must communicate back to the main UI thread.
- All background tasks are spawned via `crate::task::spawn_supervised(ctx, future, tx)`.
- The supervisor wrapper owns a clone of `egui::Context` and the UI event loop `Sender<UiEvent>`.
- **Expected Failures:** If the underlying async task returns an `Err(AppError)`, the supervisor transmits `UiEvent::AppError` through the channel and immediately calls `ctx.request_repaint()`.
- **Unexpected Panics:** The supervisor safely awaits Tokio's `JoinHandle`. If `JoinError::is_panic()` is detected, the supervisor intercepts the panic boundary, logs a critical failure, transmits a panic status message to the UI channel, and calls `ctx.request_repaint()`.

### E. Database Corruption Resilience (`src/db/mod.rs`)
When SQLite databases encounter bit rot or sudden machine shutdown, connection or migration execution returns `sqlite::Error::Corrupt`.
- `Database::init()` wraps the connection and migration sequence in a robust retry and isolation loop.
- If initialization fails on an existing `crap_data.db`, the system detects the failure, isolates the corrupted file by renaming it to `crap_data.corrupted.<timestamp>.db`, logs a critical tracing warning, and automatically creates and migrates a fresh database.
- This ensures the application launches successfully and remains functional even in extreme system failure scenarios.

## 3. UI Reactivity & Feedback (`src/ui/events.rs`)
When `UiEvent::AppError` or status messages are received by the main event loop:
- The UI controller instantly sets a high-visibility red error status toast or modal on the canvas.
- The user receives immediate visual feedback, while developers can inspect `data/logs/crapp.log` and `crash.log` for full structured context.

## 4. Database Import State Synchronization, Rollback & Error Silencing

During database import operations (DB file swaps or full ZIP restorations):
1.  **The Race Condition:** The connection pool is closed via `db.close().await` to allow safe overwriting of the underlying SQLite file. However, the multi-threaded Tokio background pool and the eframe/egui GUI thread (which renders at 60 FPS) remain active. Any pre-existing, concurrent, or freshly-spawned background tasks holding cloned database connections would previously attempt to execute queries on the closed pool, resulting in query failures and error popups.
2.  **Synchronization Lock (`is_importing`):** A state lock flag `pub is_importing: bool` is added to `CrapApp` to serialize and guard database transitions.
   - When the user confirms the database import, `is_importing` is set to `true`.
   - Any background task failure (such as connection pool closed errors) triggered by concurrent threads during this transition is intercepted in the event loop and safely silenced/ignored as a warning instead of raising critical UI errors.
3.  **Self-Healing Rollback Recovery & Visual Alerting:** If the database copy or zip archive extraction encounters any failure:
   - A copy rollback automatically restores `crap_data.db` from the pre-import safety backup.
   - If the rollback is successful, `UiEvent::StatusMessage` publishes a warning notification to the GUI status bar in high-visibility yellow: `"Import failed! Successfully restored original database state from safety backup."`
   - If the rollback itself fails (extreme filesystem failure), `UiEvent::StatusMessage` publishes a critical warning in bold red: `"CRITICAL: Import failed and rollback also failed! Please restore your backup manually..."`
   - In all recovery scenarios, a database pool initialization (`Database::init().await`) is triggered to ensure the UI can continue to function gracefully using the restored active connection.
4.  **In-Memory Cache Invalidation:** Upon a successful reload, all cached in-memory selections (`selected_character`, `selected_lorebook`, `selected_template`, `selected_entry`) and the complete `navigation_history` are invalidated and cleared to prevent mismatched IDs or out-of-sync references pointing to the old database schema.
5.  **Dialog Cancellation Pathway:** If the file picker dialog is closed or cancelled by the user, the original database pool is re-dispatched to the reload event, which safely resets the lock flag without disrupting active operations.
