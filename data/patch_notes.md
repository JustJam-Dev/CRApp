# Version 0.2.5 - Patch Notes

## New functions
- **Advanced Character Content Blur**: Introduced a configurable image obfuscation system for character avatars when content blurring is active. Users can customize their experience under Options -> General -> Content Blur, with three selectable options:
  - **Full Blur** (Default): Safely censors avatars with a high-contrast solid black overlay and clear "NSFW" / "BLURRED" text.
  - **Simple**: Classic, soft Gaussian blur filter dynamically applied to the avatar.
  - **Pixelize**: Vintage 16-bit blocky pixelation filter.
- **SillyTavern V3 Card Integration**: Added comprehensive support for the SillyTavern V3 character card format. Introduce a new, dedicated "Silly Tavern" tab in the character editor featuring sub-tabs for both Main and Advanced parameters. Users can now define independent properties such as creator handle, versioning, alternate greetings, world association, talkativeness index, and custom depth prompts directly inside CRApp.
- **SillyTavern PNG Character Card Exporter**: Re-engineered low-level PNG chunk writing in the export pipeline. Clicking export writes the SillyTavern card V3 data as Base64 encoded payload into standard-compliant PNG `tEXt` chunks (`chara` and `ccv3` keywords) within the character's avatar image, making exported cards fully compatible with SillyTavern and other third-party frontends.
- **Revert Changes**: Added an optional "↺ REVERT" button to the character editor toolbar. When modifying an existing character in a dirty state, clicking this button opens a confirmation popup window allowing the user to safely discard all unsaved edits and restore the character to its last saved configuration.

## UI Refinements
- **SillyTavern Data Copy**: Added the "Copy data from Online Format" button into the "Silly Tavern" tab horizontal sub-bar, using `"Main"` as the label for the source tab, added a clean delimiter separating it from the `"Main"` and `"Advanced"` sub-tabs, and styled it as a standard gray action button.
- **Editor Tab Grouping & Labeling**: Changed the "Main Data" button label in the character editor to "Online Format". Additionally, grouped the "Online Format" and "Silly Tavern" tabs separately from "Notes", "Lorebooks", and "Gallery" using a vertical separator and horizontal spacing for a cleaner structure.
- **Sidebar Folder Navigation**: Decoupled folder expansion/collapse from explorator navigation. Clicking a folder's name now strictly opens/selects it in the central browser view without changing its expansion state on the sidebar, while clicking the arrow icon strictly toggles the expansion state of the folder in the sidebar tree. Additionally, quick double-clicking on the folder name will toggle the folder's expansion state on the sidebar, preserving ease of navigation.

## Architectural Improvements
- **Lazy Image Cache Processing & Storage Safety**: Implemented helper for the Content Blur system. Processed image assets (simple Gaussian blurs and blocky pixelations) are generated on demand and stored in `data/avatars/` with `_blur` and `_pixel` suffixes to ensure ultra-smooth 60fps immediate-mode GUI rendering. Enhanced `cleanup_avatar` to automatically delete these cache files when an avatar is deleted or replaced.
- **Database Schema Expansion**: Automatically upgrades existing user databases during initialization, adding 13 new SillyTavern-specific metadata fields to the SQLite database schema while ensuring complete backward-compatibility and zero data loss.
- **Database Import Stability**: Implemented synchronization lock state and cache-invalidation handler that prevents connection pool race conditions during database imports or ZIP restorations. Silences background query failures during transit and invalidates selections and navigation history to prevent mismatched key references once the new database schema loads.
- **Pre-Import Safety Backup Auto-Cleanup**: Resolved an issue where temporary pre-import database safety backup files (`crap_data_backup_*.db`) would accumulate in the project root after failed imports. The application now automatically deletes these safety backups upon a successful rollback and database re-initialization, keeping the directory clean while preserving the file only in the extreme case of a double-fault (when the rollback itself fails).

## Bug Fixes
- **Tag Layout Wrapping**: Fixed an issue where a large number of tags would overflow horizontally past the editor boundaries, pushing and clipping right-hand elements (such as buttons, notes, and the picture panel). Tags are now dynamically measured and wrapped cleanly across multiple lines.
- **SpicyChat Lorebook Import**: Fixed an issue where SpicyChat's updated web page structure prevented the importer from detecting lorebook entries. Also resolved a UI collision bug where imported entries lacked distinct in-memory IDs before being saved, causing multiple entries to highlight simultaneously and overwrite each other's data when clicked.

# Version 0.2.4 - Patch Notes

## New functions
- **Regional Dictionaries**: Added toggleable dictionary selection between American (`en_US`) and British (`en_GB`) English in the Options window under the Dictionary tab.
- **Context-Menu Spellcheck Corrections**: Right-clicking on any red-underlined misspelled word in the text editor now displays up to 5 suggested corrections. Clicking a suggestion instantly replaces the misspelled word while preserving text layout and cursor positioning.

## Architectural Improvements
- **Asynchronous Threading & Task Supervision**: Refactored the core application initialization and background task execution. All async Tokio tasks are now managed by a robust supervisor (`spawn_supervised`) that intercepts both expected errors and unexpected panics, immediately notifying the UI thread and forcing instant canvas repaints without locking up.
- **Pre-GUI Gatekeeper & Crash Handler**: Implemented a strict startup initialization gatekeeper ensuring storage, logging, Tokio, and database subsystems boot cleanly before the GUI launches. Added a low-overhead panic hook dumping unhandled exceptions directly to `data/logs/crash.log` and exiting gracefully.
- **Database Corruption Resilience**: Enhanced database initialization with an automatic corruption isolation routine. If the SQLite database file or schema is malformed or corrupted, the system automatically isolates the corrupted file (`crap_data.corrupted.<timestamp>.db`) and boots a fresh database to prevent permanent lockout.
- **Improved Error Capture**: Integrated a professional logging system (`tracing`) that records application events and errors to `data/logs/`. Logs are rotated daily, and the application automatically cleans up old logs (keeping only the 5 most recent files).
- **Enhanced UI Feedback**: All background operations (CRUD actions, importing, exporting, settings) now propagate domain-specific errors (`AppError`, `DbError`) directly to the UI, displaying color-coded status toasts in real-time.

## UI & UX Improvements
- **Lorebook Layout Fixes**: Redesigned the Lorebook Entry editor to use a pinned layout. The "Save Entry" and "Delete" buttons are now anchored to the bottom of the screen, ensuring they are always accessible regardless of the content length or window scaling.
- **Granular Dirty State Tracking**: Lorebook entries now track unsaved changes individually.
    - Added visual indicators (`*`) in the entry list to show which entries have unsaved modifications.
    - Improved navigation: You can now switch between entries, add new ones, or paste entries from the clipboard without being interrupted by popups. Unsaved changes are preserved in memory and merged seamlessly.
- **Enhanced Logging**: Added comprehensive diagnostic logging (`tracing`) for all lorebook and character management operations, making it easier to troubleshoot background tasks.
- **Logging Visibility**: Background operations now provide better diagnostic information in case of failures.

## Bugfixes
- **Folder Renaming**: Fixed a bug where renaming a subfolder would move it to the root and reset its display order.
- **UI Scaling**: Fixed an issue where the Lorebook "Save Entry" button could be obscured when the content field was large.
- **Migration Stability**: Improved the robustness of the database migration system to better handle existing schemas.
- **Robust Path Sanitization**: Implemented a more comprehensive `sanitize_filename` system that automatically strips trailing spaces and periods, handles Windows reserved filenames (CON, PRN, etc.), and filters out control characters to improve filesystem compatibility across all platforms.
- **Export Diagnostic and Compatibility**: Enhanced mass-export operations with detailed logging and real-time UI error feedback for easier troubleshooting.
- **SillyTavern Export Compatibility**: Fixed compatibility issues when exporting lorebooks to SillyTavern. Corrected the `position` field format from string to integer mapping and added the required `enabled` boolean field to prevent SillyTavern from silently skipping exported entries.

# Version 0.2.3 - Patch Notes

## New functions

### Edit Character Section Improvements
- **Text Editor Improvements**: Remade text editor to enable customization options (font, font size, brightness) and better handling of selection and context menu. Optimized large file handling with culling and caching.
- **Quick Notes**: Added quick character notes.
- **Global Dictionary**: Implemented a global dictionary management system in settings for the spellchecker. You can also add your own words to the dictionary either through the settings or the context menu of the character editor.
- **NSFW Marking**: Added NSFW marking to characters. For now this only affects the blur system but it might be expanded in the future.
- **Blur/Unblur System**: Implemented a blur system. This include global blur setting, per-character blur setting and bluring characters marked as NSFW. You can also change blur state temporarily by right-clicking on the character image.

### Integration
- **SillyTavern/Chub.ai Lorebook Export**: Added support for lorebook exports compatible with SillyTavern, Chub.ai and afterhour.app.
- **Chub.ai Character Import**: Added clipboard import support for Chub.ai character edit pages.

## UI & UX Improvements
- **Embedded Patch Notes**: You can view these patch notes directly in the application settings.
- **Context Menu**: Added icons to some context menu options across the application.
- **Context Menu Sizing**: Refined context menu layout to prevent width issues and disabled text wrapping.
- **Performance**: Implemented asynchronous background thumbnailing and pre-calculated URIs for the character gallery to prevent freezes.

## Bugfixes
- **JPG Support**: Restored full support for JPG images.

# Version 0.2.2 - Patch Notes

## New functions

### Auto-Update System
Implemented a complete end-to-end auto-update system. This is the last update you need to download and swap .exe manually!

### Advanced Export Capabilities
- Added mass export for collections with multiple format selections.
- Added Export to one file option (Grid PNG, Detailed HTML list).
- Implemented context-aware export buttons for "All Characters" and "Favorites" views.

### New Importing from clipboard
- Added support for clipboard import from afterhour.app.

### Statistics and Token Counting
- Added a generalized statistics popup with token breakdowns for folders and current views.
- Implemented granular token counting settings.
- Added character and token counters specifically for the Lorebook view.

### Gallery Features
- Added Lightbox zoom and pan functionality to the image gallery.
- Implemented dynamic gallery navigation and clipboard support.

### UI & Navigation
- **Options Window**: Refactored into a tabbed interface (General, Tokens, Update, About) and made the window movable/fixed-size.
- **Sidebar**: Added "Unfold all" navigation option. (Right click on 'Uncategorized' section)
- **History**: Implemented browser-style navigation history and smart tab switching. Look under right click of back button.
- **Spell Check**: Added optional spell check with global settings and per-section overrides.
- **Avatar**: Added a context menu to the avatar image.
- **Background**: Added configurable background image scaling options.

## Minor changes/functions

### UX Improvements
- Gallery image deletion now requires confirmation.
- Lorebook entry deletion now automatically selects the nearest remaining entry.
- Added redirection to the Templates view if a user tries to apply a template but none exist.
- Added version number and GitHub link to the settings "About" section.
- Added ability to Copy and Paste entire Lorebook entries.

### Visual Changes
- Character tags are now displayed in the List View.
- Improved Character Editor UI layout and scrollbar behavior.

## Bugfixes
- Fixed gallery refresh bugs and image loading failures.
- Fixed character name import logic to correctly separate file name from display name.
- Fixed an issue where Character IDs were not preserved during import, causing duplicates.
- Fixed navigation history not properly saving Lorebook ID when navigating from the Lorebook Characters view.
- Ensured unique avatar filenames by automatically appending the Character ID preventing deleting avatars for multiple characters.
- Fixed navigation history not properly saving Lorebook ID when navigating from the Lorebook Characters view.
- Fixed issues with Character dirty state persistence (unsaved changes warning).
- Fixed Lorebook save logic consistency.
