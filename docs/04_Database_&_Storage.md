# Database & Storage

## Technology
-   **Database**: SQLite
-   **Driver**: `sqlx` (Rust) with `migrate` feature.
-   **Schema Management**: Versioned SQL migrations embedded in the binary.
-   **Safety**: Automatic startup backups.

## Migration System

The application uses `sqlx::migrate!` to manage database schema evolution.

-   **Location**: `migrations/` directory in the project root.
-   **Format**: Plain SQL files named with a timestamp prefix (e.g., `20260101000000_initial_schema.sql`).
-   **Behavior**:
    -   Migrations are compiled strictly into the `.exe`.
    -   On startup, the app checks the `_sqlx_migrations` table.
    -   Any missing migrations are applied atomically (inside a transaction).

### Adding a Migration
To modify the database (e.g., add a column):
1.  Create a new file in `migrations/`: `YYYYMMDDHHMMSS_description.sql`.
2.  Write the `ALTER TABLE` or `CREATE TABLE` statements.
3.  Run the app.

## Safety & Backups

To prevent data loss during updates, the application performs a **Safety Backup** during initialization (`src/db/mod.rs`):

1.  **Check**: Does `crap_data.db` exist?
2.  **Backup**: If yes, copy it to `crap_data.db.bak` immediately.
3.  **Migrate**: Only *after* the backup is secured does the migration runner start.

If a migration fails, the application will panic/crash to prevent partial data corruption, and the user can restore `crap_data.db.bak`.

### Database Import & Export Safety

To ensure maximum data protection during runtime database imports and archive exports (`src/ui/controllers/export_import.rs`), CRApp enforces the following safety protocols:

1. **Pre-Import Checkpoint & Safe Shutdown**: Before starting any import, the active connection pool executes a full WAL checkpoint to ensure all uncommitted transactions are fully flushed to disk. The database pool is then gracefully closed.
2. **Pre-Import Safety Backup**: A temporary safety backup file `crap_data_backup_YYYYMMDD_HHMMSS.db` is created in the project root.
3. **Automated Rollback Recovery**: If the unzipping or database copying process fails at any point (e.g. invalid zip structure, partial extracts, disk space exhaustion):
   - The backup copy is automatically copied back to `crap_data.db` to restore the pre-import state.
   - A descriptive warning status toast is pushed to the UI to notify the user.
   - The connection pool is safely re-initialized using the restored original database file.
4. **Temporary Clutter Prevention**:
   - **On Success**: The pre-import safety backup file is automatically deleted.
   - **On Failure Cleanup**: If an export (ZIP or DB export) fails midway or is cancelled, any partial, empty, or incomplete files created at the destination path are automatically cleaned up to keep the filesystem tidy.

## Schema

### Characters Table (`characters`)
Stores the main character data, including custom SillyTavern-compatible fields.
| Column | Type | Description |
| --- | --- | --- |
| `id` | INTEGER PK | Auto-incrementing ID. |
| `name` | TEXT | Display name / File name. |
| `char_name` | TEXT | Internal character name. |
| `char_title` | TEXT | Subtitle. |
| `personality` | TEXT | Main character personality definition. |
| `scenario` | TEXT | Main character scenario definition. |
| `example_dialogue` | TEXT | Main character example dialogue. |
| `first_message` | TEXT | Main character greeting message. |
| `author_notes` | TEXT | Author's notes. |
| `avatar_path` | TEXT | Path to local file system. |
| `collection_id` | INTEGER FK | Links to `collections`. |
| `created_at` | DATETIME | Creation timestamp. |
| `updated_at` | DATETIME | Last update timestamp. |
| `blur_avatar` | BOOLEAN | Toggle for blurred avatar presentation in browser. |
| `st_name` | TEXT | SillyTavern specific: display name. |
| `st_description` | TEXT | SillyTavern specific: description / world info. |
| `st_personality` | TEXT | SillyTavern specific: personality string. |
| `st_scenario` | TEXT | SillyTavern specific: scenario scenario. |
| `st_first_mes` | TEXT | SillyTavern specific: first message. |
| `st_mes_example` | TEXT | SillyTavern specific: message examples. |
| `st_creator_notes` | TEXT | SillyTavern specific: creator's notes. |
| `st_alternate_greetings_json`| TEXT | SillyTavern specific: array of alternative greetings (JSON). |
| `st_creator` | TEXT | SillyTavern specific: character card creator name. |
| `st_character_version` | TEXT | SillyTavern specific: version identifier. |
| `st_talkativeness` | REAL | SillyTavern specific: talkativeness factor (0.0 to 1.0). |
| `st_world` | TEXT | SillyTavern specific: world / lore association. |
| `st_depth_prompt` | TEXT | SillyTavern specific: custom depth prompt. |

### Character URLs Table (`character_urls`)
Stores multiple source links per character.
| Column | Type | Description |
| --- | --- | --- |
| `id` | INTEGER PK | |
| `character_id` | INTEGER FK | Links to `characters`. |
| `url` | TEXT | |
| `label` | TEXT | Optional service name. |

### Collections Table (`collections`)
Hierarchical folder structure.
| Column | Type | Description |
| --- | --- | --- |
| `id` | INTEGER PK | |
| `name` | TEXT | |
| `image_path` | TEXT | Path to custom icon file. |
| `parent_id` | INTEGER FK | Self-referencing FK for nesting. |

### Tags Tables
-   `tags`: Internal app tags.
-   `external_tags`: Tags imported from external sources.
-   `character_tags` / `character_external_tags`: Many-to-Many link tables.

### Lorebooks Table (`lorebooks`)
World info entries.
| Column | Type | Description |
| --- | --- | --- |
| `id` | INTEGER PK | |
| `title` | TEXT | |
| `description` | TEXT | Legacy column, synced with `content`. |
| `content` | TEXT | Main body text. |
| `cover_path` | TEXT | |

### Lorebook Entries Table (`lorebook_entries`)
Individual lore pieces within a book.
| Column | Type | Description |
| --- | --- | --- |
| `id` | INTEGER PK | |
| `lorebook_id` | INTEGER FK | Links to `lorebooks`. |
| `name` | TEXT | |
| `keywords` | TEXT | |
| `content` | TEXT | |

### Lorebook Tags
-   `lorebook_tags`: Links `lorebooks` to `tags`.

### Links
-   `character_lore_link`: Many-to-Many link between Characters and Lorebooks.

## File Storage
Non-text data is stored on the local filesystem, with paths stored in the database.
-   **Avatars**: Stored in `data/avatars/`.
-   **Collection Images**: Stored in `data/collection_images/`.
-   **Lorebook Covers**: Stored in `data/covers/`.
-   **Gallery**: Stored in `data/gallery/` (Reserved for character galleries, managed separately).
-   **Background**: Stored in `data/background/` (`default.png` and `custom.png`).
-   **Exports**: Saved to `exports/` (default dialog path). To ensure cross-platform compatibility (especially for Windows), all exported file and directory names are processed through a robust sanitization system that strips trailing spaces/dots and handles reserved filenames.
-   **Logs**: Application events and errors are stored in `data/logs/`.
    -   **Rotation**: Logs are rotated daily (`crapp.log.YYYY-MM-DD`).
    -   **Cleanup**: Only the 5 most recent log files are kept to save space.

## Automated Media Cleanup

To keep the storage clean, the application includes a **Media Cleanup** system (`src/cleaner.rs`).

-   **Trigger**: Runs automatically on every application startup.
-   **Logic**: 
    1. Scans `data/avatars/`, `data/collection_images/`, and `data/covers/`.
    2. Compares files on disk with paths stored in the SQLite database.
    3. Deletes any file found on disk that is no longer referenced in the database (orphaned files).
-   **Exclusions**: Specifically ignores `data/gallery/` to prevent data loss in managed galleries.
-   **Safety**: If an error occurs during cleanup, it is logged, but the application startup continues.

## Async Operations
All database operations are asynchronous (`async`/`await`). The UI thread spawns `tokio` tasks to perform DB writes or reads, preventing UI freezes. Results are communicated back via channels or by updating shared state wrapped in `Arc<Mutex>` (though `CrapApp` mostly reloads data after events).
