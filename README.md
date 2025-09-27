# Revw

Terminal JSON viewer/editor focused on the Relf JSON format, with simple Vim-like navigation.

## Install

```bash
cargo install --git https://github.com/rlelf/revw.git
```

## Usage

```bash
# Start without file
revw

# Relf mode
revw file.json

# JSON mode
revw --json file.json

# Output options
revw --output file.json
revw --stdout file.json
```

## Controls

### View Mode
- `v` paste
- `c` copy
- `r` toggle Relf/JSON
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
- `/` search
- `n/N` next/prev match
- `:ai` add inside
- `:ao` add outside
- `:o` order
- `:w` save
- `:wq` save and quit
- `:q` quit
- `:h` help

## Relf-Specific

Revw is tailored for Relf JSON. It expects a simple schema with two top-level arrays: `outside` and `inside`.


## License

MIT
