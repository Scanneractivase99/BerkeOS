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
- [x] Python frontend
- [ ] C frontend
- [ ] Rust frontend
- [ ] BerkeOS kernel VM module
- [ ] Standard library functions
- [ ] Snake game example

## License

Apache 2.0
