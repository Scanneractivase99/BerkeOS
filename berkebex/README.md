# BerkeBex - BerkeOS Executable Compiler

A cross-compiler that compiles Python, C, and Rust programs to `.bex` format for BerkeOS.

## Features

- **Python Support** - Compile a subset of Python to .bex bytecode
- **Stack-based Bytecode VM** - Efficient virtual machine for BerkeOS
- **Simple Syntax** - C-like syntax with Python-like simplicity

## Installation

```bash
cd berkebex
cargo build --release
cargo install --path .
```

## Usage

```bash
# Compile Python to .bex
berkebex compile examples/hello.bepy -o hello.bex

# Run a .bex file
berkebex run hello.bex

# Show file info
berkebex info hello.bex
```

## Supported Syntax

### Variables
```python
let x = 10
let name = "BerkeOS"
let is_active = true
```

### Functions
```python
fn add(a, b) {
    return a + b
}
```

### Control Flow
```python
if x > 0 {
    println("positive")
}

while x < 10 {
    x = x + 1
}
```

### Built-in Functions
- `print(value)` - Print without newline
- `println(value)` - Print with newline
- `sleep(ms)` - Sleep for milliseconds

## Berkepython

Berkepython is a Python-like language that compiles to .bex bytecode for BerkeOS. You can write programs using Berkepython syntax and run them either directly or compile them to .bex files.

### Usage

```bash
# Run a Berkepython script directly
berkepython examples/hello.bepy

# Compile a Berkepython script to .bex
berkebex --python examples/hello.bepy -o hello.bex

# Run the compiled .bex file
berkebex run hello.bex
```

### BerkeOS API

Access BerkeOS system calls through the `berkeos` module:

```python
import berkeos

fn main() {
    # Process
    berkeos.process.sleep(1000)
    let pid = berkeos.process.getpid()

    # File operations
    let handle = berkeos.file.open("/data.txt", "r")
    let content = berkeos.file.read(handle, 256)
    berkeos.file.write(handle, "hello")
    berkeos.file.close(handle)

    # Display (framebuffer)
    berkeos.display.clear(0x000000)
    berkeos.display.draw_pixel(100, 100, 0xFF0000)
    berkeos.display.draw_rect(50, 50, 200, 100, 0x00FF00)
    berkeos.display.draw_text(10, 10, "Hello!", 0xFFFFFF)

    # Input
    let key = berkeos.input.key()

    # Windows
    let win = berkeos.window.new("My Window", 640, 480)
}
```

#### Process Module
| Function | Description |
|:---|:---|
| `berkeos.process.sleep(ms)` | Sleep for specified milliseconds |
| `berkeos.process.getpid()` | Get current process ID |

#### File Module
| Function | Description |
|:---|:---|
| `berkeos.file.open(path, mode)` | Open file (modes: "r", "w", "a") |
| `berkeos.file.read(handle, size)` | Read from file handle |
| `berkeos.file.write(handle, data)` | Write string to file |
| `berkeos.file.close(handle)` | Close file handle |

#### Display Module
| Function | Description |
|:---|:---|
| `berkeos.display.clear(color)` | Clear framebuffer (0xRRGGBB) |
| `berkeos.display.draw_pixel(x, y, color)` | Draw single pixel |
| `berkeos.display.draw_rect(x, y, w, h, color)` | Draw filled rectangle |
| `berkeos.display.draw_text(x, y, text, color)` | Draw text at position |

#### Input Module
| Function | Description |
|:---|:---|
| `berkeos.input.key()` | Read key from keyboard |

#### Window Module
| Function | Description |
|:---|:---|
| `berkeos.window.new(title, width, height)` | Create new window |

### Example Programs

**Hello World** (`examples/hello.bepy`):
```python
fn main() {
    println("Hello, BerkeOS!")
    println("Welcome to .bex runtime!")
}
```

**Calculator** (`examples/calculator.bepy`):
```python
fn main() {
    let a = 10
    let b = 20
    
    println("Calculator Demo")
    println("a + b = ")
    println(a + b)
    println("a * b = ")
    println(a * b)
}
```

**Snake Game** (`examples/snake.bepy`):
```python
fn main() {
    println("=== SNAKE ===")
    println("Use: up, down, left, right")
    println("Eat food to grow!")

    let score = 0
    let game_over = 0

    while game_over == 0 {
        println("Score: ")
        println(score)
        berkeos.process.sleep(500)
    }

    println("Game Over!")
}
```

### Color Format

Colors are 24-bit RGB values in hexadecimal format: `0xRRGGBB`

Examples:
- `0xFF0000` - Red
- `0x00FF00` - Green
- `0x0000FF` - Blue
- `0xFFFFFF` - White
- `0x000000` - Black

### Error Messages

| Error | Cause | Solution |
|:---|:---|:---|
| `Syntax error: unexpected token` | Invalid syntax | Check for missing braces, parentheses |
| `Unknown function 'xyz'` | Undefined function | Define function with `fn xyz() {}` |
| `Type mismatch` | Wrong operand types | Ensure numbers with numbers, strings with strings |
| `Division by zero` | Dividing by 0 | Add check before division |
| `File not found` | Invalid path | Check file path exists |

### Limitations

- No classes or objects (procedural only)
- No imports (stdlib not yet available)
- No floating-point support (integers only)
- No recursion (function calls within same function)
- No string manipulation beyond concatenation
- No arrays or complex data structures
- Framebuffer requires graphical mode (not VGA text mode)
- Window system requires GUI desktop (not yet implemented)

## .bex File Format

```
┌─────────────────────┐
│ Magic: 0x42455831   │  "BEX1"
│ Version: 1          │
├─────────────────────┤
│ Name Length + Name  │
├─────────────────────┤
│ Constants Pool      │
│ (integers, strings) │
├─────────────────────┤
│ Functions Pool      │
│ (instructions)      │
└─────────────────────┘
```

## Roadmap

- [x] Bytecode IR definition
- [x] Python frontend (Berkepython)
- [x] BerkeOS API module
- [x] Snake game example
- [ ] C frontend
- [ ] Rust frontend
- [ ] BerkeOS kernel VM module
- [ ] Standard library functions

## License

Apache 2.0
