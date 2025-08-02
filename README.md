# LuaBuild - Modern Build Automation System

A powerful, extensible build automation tool that combines the simplicity of Lua scripting with high-performance Rust backend.

## Features

- **Lua Scripting Engine**: Write build scripts in Lua with full async support
- **File Operations**: Complete file and directory management
- **Path Utilities**: Smart path resolution and manipulation
- **Text Processing**: Advanced string operations and regex support
- **Modular Architecture**: Clean separation of concerns with extensible modules

## Installation

```bash
cargo build --release
```

## Usage

### Basic Usage

```bash
# Run a build script
./solver build.lua

# Specify a target
./solver build.lua --target release

# Enable verbose output
./solver build.lua --verbose

# List available targets
./solver build.lua --list-targets
```

### Command Line Options

- `script` - Path to the Lua build script (required)
- `-t, --target <TARGET>` - Specify build target
- `-l, --list-targets` - List available targets
- `-v, --verbose` - Enable verbose output

## API Reference

### Basic Variables

| Variable | Type | Description |
|----------|------|-------------|
| `verbose` | boolean | Whether verbose mode is enabled |
| `target` | string | Current build target (if specified) |
| `lua_script_path` | string | Full path to the current Lua script |

### File Operations

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `file_exists(path)` | string | boolean | Check if file exists |
| `dir_exists(path)` | string | boolean | Check if directory exists |
| `read_file(path)` | string | string\|nil | Read file content |
| `write_file(path, content)` | string, string | void | Write content to file |
| `create_dir(path)` | string | void | Create directory (recursive) |
| `delete_file(path)` | string | void | Delete file |
| `delete_dir(path)` | string | void | Delete directory (recursive) |

### Path Operations

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `get_full_path(path)` | string | string | Convert to absolute path |
| `resolve_path(path)` | string | string | Resolve path relative to script |

### Text Processing

#### String Formatting
| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `trim(text)` | string | string | Remove leading/trailing whitespace |
| `trim_start(text)` | string | string | Remove leading whitespace |
| `trim_end(text)` | string | string | Remove trailing whitespace |
| `to_upper(text)` | string | string | Convert to uppercase |
| `to_lower(text)` | string | string | Convert to lowercase |
| `capitalize(text)` | string | string | Capitalize first character |
| `split(text, delimiter)` | string, string | table | Split string by delimiter |
| `join(parts, delimiter)` | table, string | string | Join strings with delimiter |
| `pad_left(text, width, char?)` | string, number, string? | string | Pad left to width |
| `pad_right(text, width, char?)` | string, number, string? | string | Pad right to width |

#### Regular Expressions
| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `regex_match(text, pattern)` | string, string | boolean | Test if pattern matches |
| `regex_find(text, pattern)` | string, string | string\|nil | Find first match |
| `regex_find_all(text, pattern)` | string, string | table | Find all matches |
| `regex_replace(text, pattern, replacement)` | string, string, string | string | Replace matches |
| `regex_capture(text, pattern)` | string, string | table\|nil | Capture groups |

#### Text Analysis
| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `word_count(text)` | string | number | Count words |
| `line_count(text)` | string | number | Count lines |
| `char_count(text)` | string | number | Count characters |
| `contains(text, substring)` | string, string | boolean | Check if contains substring |
| `starts_with(text, prefix)` | string, string | boolean | Check if starts with prefix |
| `ends_with(text, suffix)` | string, string | boolean | Check if ends with suffix |

#### Text Transformation
| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `reverse(text)` | string | string | Reverse string |
| `repeat_text(text, count)` | string, number | string | Repeat string |
| `remove_whitespace(text)` | string | string | Remove all whitespace |
| `normalize_whitespace(text)` | string | string | Normalize whitespace |
| `substring(text, start, length?)` | string, number, number? | string | Extract substring |
| `find_index(text, substring)` | string, string | number\|nil | Find substring index |
| `replace_text(text, from, to)` | string, string, string | string | Replace text |

### Utility Functions

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `to_string(value)` | any | string | Convert value to string |
| `include(path)` | string | void | Execute external Lua file |
| `include_local(path)` | string | void | Execute Lua file relative to script |
| `print_success(text)` | string | void | Print green success message |
| `print_error(text)` | string | void | Print red error message |

## Example Build Scripts

### Simple Build Script

```lua
print("Building project...")

if not dir_exists("build") then
    create_dir("build")
    print_success("Created build directory")
end

local source = read_file("src/main.cpp")
if source then
    local processed = regex_replace(source, "DEBUG", "RELEASE")
    write_file("build/main.cpp", processed)
    print_success("Processed source file")
else
    print_error("Failed to read source file")
end
```

### Target-Specific Build

```lua
if target == "debug" then
    print("Building debug version...")
    create_dir("debug")
elseif target == "release" then
    print("Building release version...")
    create_dir("release")
    local content = read_file("config.txt")
    content = regex_replace(content, "DEBUG=1", "DEBUG=0")
    write_file("release/config.txt", content)
else
    print_error("Unknown target: " .. (target or "none"))
end
```

### Advanced Text Processing

```lua
local readme = read_file("README.md")
if readme then
    local word_count = word_count(readme)
    local line_count = line_count(readme)
    
    print("Documentation stats:")
    print("  Words: " .. word_count)
    print("  Lines: " .. line_count)
    
    -- Extract all headers
    local headers = regex_find_all(readme, "^#+\\s+(.+)$")
    print("  Headers: " .. #headers)
    
    -- Generate TOC
    local toc = "# Table of Contents\n"
    for _, header in ipairs(headers) do
        local clean = trim(regex_replace(header, "^#+\\s+", ""))
        local link = to_lower(regex_replace(clean, "[^%w%s]", ""))
        link = regex_replace(link, "%s+", "-")
        toc = toc .. "- [" .. clean .. "](#" .. link .. ")\n"
    end
    
    write_file("TOC.md", toc)
    print_success("Generated table of contents")
end
```

### Including External Scripts

```lua
-- Include common utilities
include_local("utils/common.lua")

-- Include from absolute path
include("/path/to/shared/build-tools.lua")

-- Use functions from included scripts
if verbose then
    print("Running with utilities loaded")
end
```

## Architecture

The system is built with a modular architecture:

- **`main.rs`** - CLI interface and entry point
- **`lua_engine.rs`** - Core Lua execution engine
- **`io.rs`** - File and directory operations
- **`basic.rs`** - Basic variables and utilities
- **`utility.rs`** - Script inclusion and type conversion
- **`text.rs`** - Text processing and regex operations

Each module follows the pattern of exposing a `setup_globals_*` function that registers its API with the Lua runtime.

## Requirements

- Rust 2024 edition
- Tokio async runtime
- MLua for Lua integration
- Regex crate for pattern matching

## License

This project is open source. See LICENSE file for details.