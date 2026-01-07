# Revw

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Repo](https://img.shields.io/badge/repo-rlelf%2Frevw-blue?logo=github)](https://github.com/rlelf/revw)
[![Built With Ratatui](https://img.shields.io/badge/Built_With_Ratatui-000?logo=ratatui&logoColor=fff)](https://ratatui.rs/)

A vim-like TUI for managing notes and resources.

![Revw](https://raw.githubusercontent.com/rlelf/revw/main/assets/revw.gif)

## Features

- Vim-like terminal user interface
- Clipboard integration
- Clean card-based interface
- Toggle between View mode and raw Edit mode

## Format

### Outside
External resources and references:
- **Name**: Title or identifier of the resource
- **Context**: Description or notes about the resource
- **URL**: Web address or link
- **Percentage**: Score or progress indicator, sortable for ordering

### Inside
Internal notes or thoughts with timestamps:
- **Date**: Timestamp of the entry, sortable for ordering
- **Context**: notes or thoughts

### Markdown Format

```markdown
## OUTSIDE
### Rust Programming Language
A systems programming language focused on safety, speed, and concurrency.

**URL:** https://www.rust-lang.org/

**Percentage:** 100%

## INSIDE
### 2025-01-01 00:00:00
Finally learned how to use cargo! Running 'cargo new my_project' creates such a clean project structure.
```

### JSON Format

```json
{
  "outside": [
    {
      "name": "Rust Programming Language",
      "context": "A systems programming language focused on safety, speed, and concurrency.",
      "url": "https://www.rust-lang.org/",
      "percentage": 100
    }
  ],
  "inside": [
    {
      "date": "2025-01-01 00:00:00",
      "context": "Finally learned how to use cargo! Running 'cargo new my_project' creates such a clean project structure."
    }
  ]
}
```

### Toon Format

```yaml
outside[1]{name,context,url,percentage}:
  "Rust Programming Language","A systems programming language focused on safety, speed, and concurrency.",https://www.rust-lang.org/,100

inside[1]{date,context}:
  "2025-01-01 00:00:00","Finally learned how to use cargo! Running 'cargo new my_project' creates such a clean project structure."
```

## Install

```bash
cargo install --git https://github.com/rlelf/revw.git --locked
```

Or install a specific version:
```bash
cargo install --git https://github.com/rlelf/revw.git --tag v0.x.x --locked
```

Or download from [Releases](https://github.com/rlelf/revw/releases)

## Usage

### Notes and resources Management
Track learning resources, books, articles, and tools you're exploring.

### Learning diary
Document your daily learning progress with timestamped notes.

### LLM-Assisted Workflows
Revw integrates seamlessly with AI assistants:

**Workflow 1: LLM generates → You review**
1. Ask an LLM to generate a reading list or resource collection
2. Paste the Markdown, JSON, or Toon into Revw (`:v`, `:vi`, `:vo`, `:vai`, `:vao`)
3. Browse and organize visually

**Workflow 2: You create → LLM assists**
1. Maintain your notes in Revw's clean interface
2. Copy sections to clipboard (`:c`, `:ci`, `:co`)
3. Share with LLM for analysis, summarization, or questions

### Command Line Options

**Basic Usage:**
```bash
# View help
revw --help

# Show version
revw --version

# Start without file
revw

# View mode
revw file.json
revw file.md
revw file.toon

# Edit mode
revw --edit file.json
revw --edit file.md
revw --edit file.toon
```

**File Operations:**

```bash
# Output
revw --stdout file.json                     # Output to stdout
revw --stdout file.md
revw --stdout file.toon
revw --output output.txt file.json          # Output to file
revw --output output.txt file.md
revw --output output.txt file.toon
revw --stdout --inside file.json            # Output only INSIDE section
revw --stdout --inside file.md
revw --stdout --inside file.toon
revw --stdout --outside file.json           # Output only OUTSIDE section
revw --stdout --outside file.md
revw --stdout --outside file.toon
revw --stdout --markdown file.json          # Output in Markdown format
revw --stdout --markdown file.md
revw --stdout --markdown file.toon
revw --stdout --json file.json              # Output in JSON format
revw --stdout --json file.md
revw --stdout --json file.toon
revw --stdout --toon file.json              # Output in Toon format
revw --stdout --toon file.md
revw --stdout --toon file.toon

# Export to PDF
revw --pdf file.json                        # Export JSON to PDF
revw --pdf file.md                          # Export Markdown to PDF
revw --pdf file.toon                        # Export Toon to PDF

# Token count
revw --token file.json                      # Show token counts for all formats
revw --token file.md
revw --token file.toon

# Input (overwrite)
revw --input data.json file.json            # Overwrite entire content
revw --input data.json file.md
revw --input data.json file.toon
revw --input data.md file.json
revw --input data.md file.md
revw --input data.md file.toon
revw --input data.toon file.json
revw --input data.toon file.md
revw --input data.toon file.toon
revw --input data.json --inside file.json   # Overwrite only INSIDE section
revw --input data.json --inside file.md
revw --input data.json --inside file.toon
revw --input data.md --inside file.json
revw --input data.md --inside file.md
revw --input data.md --inside file.toon
revw --input data.toon --inside file.json
revw --input data.toon --inside file.md
revw --input data.toon --inside file.toon
revw --input data.json --outside file.json  # Overwrite only OUTSIDE section
revw --input data.json --outside file.md
revw --input data.json --outside file.toon
revw --input data.md --outside file.json
revw --input data.md --outside file.md
revw --input data.md --outside file.toon
revw --input data.toon --outside file.json
revw --input data.toon --outside file.md
revw --input data.toon --outside file.toon

# Input (append)
revw --input data.json --append file.json             # Append entire content
revw --input data.json --append file.md
revw --input data.json --append file.toon
revw --input data.md --append file.json
revw --input data.md --append file.md
revw --input data.md --append file.toon
revw --input data.toon --append file.json
revw --input data.toon --append file.md
revw --input data.toon --append file.toon
revw --input data.json --append --inside file.json    # Append to INSIDE only
revw --input data.json --append --inside file.md
revw --input data.json --append --inside file.toon
revw --input data.md --append --inside file.json
revw --input data.md --append --inside file.md
revw --input data.md --append --inside file.toon
revw --input data.toon --append --inside file.json
revw --input data.toon --append --inside file.md
revw --input data.toon --append --inside file.toon
revw --input data.json --append --outside file.json   # Append to OUTSIDE only
revw --input data.json --append --outside file.md
revw --input data.json --append --outside file.toon
revw --input data.md --append --outside file.json
revw --input data.md --append --outside file.md
revw --input data.md --append --outside file.toon
revw --input data.toon --append --outside file.json
revw --input data.toon --append --outside file.md
revw --input data.toon --append --outside file.toon
```

## Controls

### View Mode
**Navigation:**
- `j/k` or `↑/↓` select card (or mouse wheel)
- `h/l` or `f/b` scroll card content
- `gg` select first card
- `G` select last card
- `:gi` jump to first INSIDE entry
- `:go` jump to first OUTSIDE entry
- `/` search forward
- `n/N` next/prev match (jumps to card)
- `:noh` clear search highlighting

**Editing:**
- `Enter` open edit overlay for selected card
- `:ai` add new INSIDE entry (jumps to it)
- `:ao` add new OUTSIDE entry (jumps to it)
- `:dd` delete selected entry (entire object)
- `:yy` duplicate selected entry (entire object)
- `:o` order entries (by percentage then name) and auto-save
- `:op` order by percentage only and auto-save
- `:on` order by name only and auto-save
- `:or` order randomly and auto-save
- `:f pattern` filter entries by pattern

**Visual Mode (multi-card selection):**
- `v` enter Visual mode
- `j/k` extend selection
- `:cc` copy selected cards (rendered format)
- `:ccj` copy selected cards (JSON format)
- `:ccm` copy selected cards (Markdown format)
- `:cct` copy selected cards (Toon format)
- `:dc` delete selected cards
- `Esc` or `Ctrl+[` exit Visual mode

**Copy/Paste:**
- `:c` copy all rendered content (with OUTSIDE/INSIDE headers)
- `:ci` copy INSIDE section only
- `:co` copy OUTSIDE section only
- `:cj` copy all content (JSON format)
- `:cm` copy all content (Markdown format)
- `:ct` copy all content (Toon format)
- `:cu` copy URL from selected card
- `:v` paste file path or JSON content
- `:vu` paste URL from clipboard to selected card
- `:vi` paste INSIDE from clipboard (overwrite)
- `:vo` paste OUTSIDE from clipboard (overwrite)
- `:va` paste both INSIDE and OUTSIDE from clipboard (append)
- `:vai` paste INSIDE from clipboard (append)
- `:vao` paste OUTSIDE from clipboard (append)
- `:xi` clear INSIDE section
- `:xo` clear OUTSIDE section

**Filter:**
- `:f pattern` filter entries by pattern
- `:nof` clear filter

**Settings:**
- `:set number` or `:set nu` enable line numbers (Edit mode)
- `:set nonumber` or `:set nonu` disable line numbers
- `:set relativenumber` or `:set rnu` enable relative line numbers (Edit mode)
- `:set norelativenumber` or `:set nornu` disable relative line numbers
- `:set card=N` set max visible cards (1-10, default: 5)
- `:set border=rounded` use rounded border style (default)
- `:set border=plain` use plain border style
- `:set extension` show file extensions in explorer (default)
- `:set noextension` hide file extensions in explorer
- `:set json` set format to JSON (for unnamed files)
- `:set markdown` set format to Markdown (for unnamed files)
- `:set toon` set format to Toon (for unnamed files)

**Other:**
- `r` toggle View/Edit mode
- `:Lexplore` or `:Lex` or `:lx` toggle file explorer (left)
- `:outline` or `:ol` toggle card outline panel (right)
- `Ctrl+w w` cycle between windows (explorer → content → outline)
- `Ctrl+w h` move to explorer (left)
- `Ctrl+w l` move to outline (right)
- `Ctrl+w j/k` move to file (center)
- `:x` clear content
- `:h` or `?` toggle help mode
- `q` or `Esc` quit

**File Explorer:**
- `j/k` or `↑/↓` navigate files/directories
- `h/l` or `←/→` scroll left/right (for long filenames)
- `gg/G` jump to first/last entry
- `/` search files by name
- `n/N` next/prev search match
- `go` preview entry
- `Enter` open file (JSON only) or expand/collapse directory
- `q` close explorer

**Outline Panel:**
- `j/k` or `↑/↓` navigate entries
- `h/l` or `←/→` scroll left/right (for long entry names)
- `/` search entries
- `n/N` next/prev search match
- `gg/G` jump to first/last entry
- `go` preview entry
- `Enter` jump to entry and release focus
- `q` close outline

**Explorer File Operations (when explorer has focus):**
- `Enter` or `o` open file or navigate into directory
- `:a` create new file in current directory (supports .json, .md, .toon)
- `:d` create new directory
- `:m` rename/move selected file/directory (supports relative paths like `./folder/file.json`, or just `newname.json`)
- `:dd` delete selected file (confirms with yes/no)
- `:yy` copy selected file (prompts for new filename)

#### Edit Overlay
**Field Selection Mode (default):**
- `j/k` or `↑/↓` navigate between fields
- `Enter` enter Normal mode (renders `\n` as newlines, allows navigation)
- `i` enter Insert mode (renders `\n` as newlines, allows editing)
- `w` save changes
- `Esc` or `q` cancel

**Normal Mode (after pressing `Enter`):**
- Renders `\n` as actual newlines for multi-line viewing
- `h/j/k/l` or arrow keys navigate
- `gg` jump to start, `G` jump to end
- `w/b/e` word navigation
- `0/$` start/end of line
- `x/X` delete character
- `i` enter Insert mode
- `Esc` or `Ctrl+[` exit to field selection mode

**Insert Mode (after pressing `i`):**
- Renders `\n` as actual newlines for multi-line editing
- Type to edit text
- `↑/↓/←/→` move cursor
- `Enter` insert literal newline (`\n`)
- `Backspace` delete character (including `\n`)
- `Esc` or `Ctrl+[` exit to field selection mode

### Edit Mode
**Navigation:**
- `h/j/k/l` or arrow keys - move cursor
- `w` next word start
- `e` next word end
- `b` previous word start
- `0` move to start of line
- `$` move to end of line
- `gg` jump to top
- `G` jump to bottom
- `:gi` jump to first INSIDE entry
- `:go` jump to first OUTSIDE entry
- Mouse wheel - scroll (drag disabled)

**Editing:**
- `i` enter insert mode
- `x` delete character at cursor
- `X` delete character before cursor
- `dd` delete current line
- `yy` yank (copy) current line
- `p` paste yanked line after current line
- `Esc` or `Ctrl+[` exit insert mode
- `:dd` delete current entry (entire object)
- `:yy` duplicate current entry (entire object)
- `u` undo
- `Ctrl+r` redo
- `g-` undo
- `g+` redo

**Search:**
- `/` search forward
- `n/N` next/prev match
- `:noh` clear search highlighting

**Commands:**
- `:ai` add INSIDE entry
- `:ao` add OUTSIDE entry
- `:o` order entries (by percentage then name)
- `:op` order by percentage only
- `:on` order by name only
- `:or` order randomly
- `:dd` delete current entry (entire object)
- `:yy` duplicate current entry (entire object)
- `:c` copy all content
- `:ci` copy INSIDE section (JSON format)
- `:co` copy OUTSIDE section (JSON format)
- `:cj` copy all content (JSON format)
- `:cm` copy all content (Markdown format)
- `:ct` copy all content (Toon format)
- `:v` paste from clipboard
- `:vi` paste INSIDE from clipboard (overwrite)
- `:vo` paste OUTSIDE from clipboard (overwrite)
- `:va` paste both INSIDE and OUTSIDE from clipboard (append)
- `:vai` paste INSIDE from clipboard (append)
- `:vao` paste OUTSIDE from clipboard (append)
- `:x` clear all content
- `:xi` clear INSIDE section
- `:xo` clear OUTSIDE section
- `:nof` clear filter
- `:w` save
- `:wq` save and quit
- `:q` quit
- `:e` reload file
- `:ar` toggle auto-reload (default: on)
- `:markdown` export current file to Markdown format (same folder, .md extension)
- `:json` export current file to JSON format (same folder, .json extension)
- `:toon` export current file to Toon format (same folder, .toon extension)
- `:pdf` export current file to PDF format (same folder, .pdf extension)
- `:Lexplore` or `:Lex` or `:lx` toggle file explorer
- `:outline` or `:ol` toggle card outline view
- `Ctrl+w w` cycle between explorer and file window
- `Ctrl+w h` move to explorer window (left)
- `Ctrl+w l` move to file window (right)
- `:h` or `?` help

**Settings:**
- `:set number` or `:set nu` enable line numbers
- `:set nonumber` or `:set nonu` disable line numbers
- `:set relativenumber` or `:set rnu` enable relative line numbers
- `:set norelativenumber` or `:set nornu` disable relative line numbers
- `:set card=N` set max visible cards (1-10, default: 5)
- `:set border=rounded` use rounded border style (default)
- `:set border=plain` use plain border style
- `:set extension` show file extensions in explorer (default)
- `:set noextension` hide file extensions in explorer
- `:set json` set format to JSON (for unnamed files)
- `:set markdown` set format to Markdown (for unnamed files)
- `:set toon` set format to Toon (for unnamed files)

**Substitute:**
- `:s/foo/bar/` substitute first occurrence in current line
- `:s/foo/bar/g` substitute all occurrences in current line
- `:%s/foo/bar/` substitute first occurrence in all lines
- `:%s/foo/bar/g` substitute all occurrences in all lines

## Configuration

Revw can be configured using a `.revwrc` file in your home directory (`~/.revwrc`).

### Configuration Options

**Line Numbers:**
```vim
set number        # Enable line numbers (Edit mode)
set nonumber      # Disable line numbers (default)
set relativenumber # Enable relative line numbers (Edit mode)
set norelativenumber # Disable relative line numbers (default)
```

**Max Visible Cards:**
```vim
set card=5        # Set max visible cards (1-10, default: 5)
```

**Border Style:**
```vim
set border=rounded # Use rounded border style (default)
set border=plain   # Use plain border style
```

**File Extensions:**
```vim
set extension     # Show file extensions in explorer (default)
set noextension   # Hide file extensions in explorer
```

**Color Schemes:**
```vim
colorscheme Default      # Default color scheme
```

You can also change the color scheme at runtime using `:colorscheme <name>`.

**Available themes:** Default, Morning, Evening, Pablo, Ron, Blue

### Example `.revwrc`

```vim
# Example configuration
colorscheme Default
set number
set card=5
```

## Changelog

See [CHANGELOG](https://github.com/rlelf/revw/blob/main/CHANGELOG.md) for version history and changes.

## License

MIT
