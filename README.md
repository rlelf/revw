# Revw

A vim-like TUI for viewing and editing personal data.

## Relf Format

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

This format is also available in [github.com/rlelf/relf](https://github.com/rlelf/relf)

## Install

```bash
cargo install --git https://github.com/rlelf/revw.git
```

## Usage

### Daily Usage with LLM
Revw is designed for LLM-assisted data management workflows:

#### Method 1: LLM → JSON → Revw
1. LLM generates a relf file
2. View the generated relf file in Revw
3. Edit the relf file in Revw

#### Method 2: Revw → JSON → LLM
1. Make a relf file in Revw
2. Edit the relf file in Revw
3. LLM assists with the relf file

### Command Line Options
```bash
# Start without file
revw

# View mode
revw file.json

# Edit mode
revw --json file.json

# Output options
revw --output file.json
revw --stdout file.json
```

## Controls

### View Mode
- `v` paste
- `c` copy
- `r` toggle View/Edit
- `x` clear
- `/` search
- `n/N` next/prev match
- `q` quit

### Edit Mode
- `i` insert
- `h/j/k/l` move
- `e` next word end
- `b` previous word start
- `dd` delete entry
- `u` undo
- `Ctrl+r` redo
- `g-` undo
- `g+` redo
- `/` search
- `n/N` next/prev match
- `:ai` add inside
- `:ao` add outside
- `:o` order
- `:w` save
- `:wq` save and quit
- `:q` quit
- `:e` reload file
- `:ar` toggle auto-reload (default: on)
- `:h` help

## License

MIT
