# Changelog

## 0.2.9

- Added Toon format support with complete cross-format compatibility
- Fixed `:v` paste command to sync correctly across JSON/Markdown/Toon formats
- Fixed `:dd` and `:yy` commands in Edit mode for Toon files
- Fixed cross-format paste operations (9 format combinations now work correctly)
- Refactored clipboard module into organized submodules
- Improved format sync functions for better maintainability

## 0.2.8

- Fixed markdown syntax colors to match between view mode and raw edit mode
- Added markdown syntax highlighting with theme support for code blocks
- Added vim-like `a` (append) and `o` (new line) commands in View Edit mode
- Added PDF export functionality with `:pdf` command

## 0.2.7

- Added View Edit mode (`v` key) in overlay with Normal/Insert mode separation
- Added `hjkl` and arrow key navigation in View Edit mode normal mode
- Added `:outline` (`:ol`) command for card outline navigation
- Added search functionality in outline window (`/`, `n`, `N`)
- Added mouse focus and scrolling support for outline window
- Improved card scroll margins for better content visibility
- Enhanced overlay mode color indicators: blue for Normal mode, yellow for Insert mode
- Improved horizontal scroll behavior in Edit mode (preserves camera position when typing)
- Fixed `:enew` bug where previous file content appeared in new buffers
- Fixed outline search to work correctly (not triggering file search)
- Fixed outline focus restoration when closing with `q`/`Esc`
- Fixed Edit mode scroll preservation when inserting/deleting characters
- Added undo/redo buffer reset when switching files
- Added markdown support to undo/redo functionality
- Updated README with detailed View Edit mode documentation

## 0.2.6

- Improved Markdown support: entries without `###` headers are now recognized (separated by blank lines)
- Fixed `:markdown` command output format to use `**URL:**` and `**Percentage:**` instead of `####`
- Fixed Edit mode to properly display Markdown files
- Fixed Markdown file reload to correctly parse content
- Added better error handling for Markdown parsing

## 0.2.5

- Fixed percentage display priority over long URLs
- Fixed cursor position reset when opening files
- Fixed `:e` completion for relative paths
- Fixed overlay window layout

## 0.2.4

- Added Tab completion for commands
- Fixed `:e <tab>` to complete file paths
- Changed Tab completion to cycle through candidates
- Added `:op` and `:on` ordering commands with tests and documentation

## 0.2.3

- Added Visual mode and View Edit mode for enhanced text selection and editing
- Added RC file (`~/.revwrc`) configuration support
- Added comprehensive colorscheme customization for all UI elements
- Added `:set card=N` support in RC configuration
- Improved overlay context field scrolling and rendering
- Implemented vertical scrolling for card context with full-height display
- Fixed context field newline rendering across all edit modes
- Fixed View Edit mode to be restricted to context field only
- Fixed remaining hardcoded colors to use configurable colorscheme
- Updated documentation with colorscheme and configuration details

## 0.2.2

- Added file explorer with tree view and directory operations
- Added file operations: `:a` (create file), `:d` (create directory), `:m` (rename/move), `:dd` (delete), `:yy` (copy)
- Enhanced `:m` command to support relative paths for moving files/directories
- Added window navigation: `Ctrl+w w` (cycle), `Ctrl+w h` (left), `Ctrl+w l` (right)
- Added `:lx` as alias for `:Lexplore`
- Added mouse support in explorer (scroll, double-click)
- Added `x` and `X` delete commands for edit and overlay modes
- Improved overlay rendering and behavior
- Updated to Rust edition 2024

## 0.2.1

- Fixed autoreload to work when file path changes
- Auto-create new files with default entries when opening non-existent files
- Default new files include one empty outside entry and one inside entry with current timestamp
- Formatted JSON output with proper indentation for new files

## 0.2.0

- Command history buffer for `:` and `/` commands
- Enhanced filter documentation
- Updated UI components

## 0.1.8

- Settings commands (`:set number`, `:set card=N`)
- Improved card layout

## 0.1.7

- Improved filter documentation
- Fixed filter entry selection with original_index tracking

## 0.1.6

- Fixed `:yy` to duplicate entries
- Added filter protection
- Added crates.io badge

## 0.1.5

- ESC quit support
- Exit button in overlay
- Fixed tests and removed dead code
- Package optimization (excluded .github and assets)

## 0.1.4

- Double-click support
- Improved mouse handling

## 0.1.3

- Multi-platform releases
- Build.rs for git version
- `--inside`/`--outside` CLI flags

## 0.1.2

- Fixed search highlighting in Edit mode

## 0.1.1

- Edit mode enhancements
- Vim-like substitute commands
- Auto-save features
- Fixed CLI flags

## 0.1.0

- Initial release
- Vim-like TUI interface
- Clipboard integration
- View and Edit modes
- Card-based interface for managing notes
- Support for INSIDE and OUTSIDE sections
- Search functionality
- Filter support
- Undo/redo support
- Substitute commands
