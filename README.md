# BerkeOS

![BerkeOS Banner](BerkeOS.png)

> An indigenous, independent x86_64 operating system written in Rust from scratch.

![Rust](https://img.shields.io/badge/Rust-no__std-b7410e?style=flat-square)
![Architecture](https://img.shields.io/badge/Architecture-x86__64-4dc71f?style=flat-square)
![License](https://img.shields.io/badge/License-Apache_2.0-blue?style=flat-square)
![Lines of Code](https://img.shields.io/badge/Lines-14%2C288-brightgreen?style=flat-square)

**BerkeOS** is a modern, DOS-inspired operating system developed by a 16-year-old developer from Turkey. Built entirely from scratch using Rust, it demonstrates that with dedication and AI assistance, anyone can build an operating system.

## About the Developer

- **Developer**: Berke Oruc (Age 16, Turkey)
- **Started**: Age 14
- **Motivation**: "I wanted to prove that with dedication and AI assistance, anyone can build an operating system from scratch."
- **Cost**: 0 TL (built using free AI tools)

## Features

### Current Features (v0.6.0)
- **Custom Kernel**: Monolithic kernel written in Rust (no_std)
- **Boot**: UEFI/BIOS auto-detection and boot
- **Filesystem**: BerkeFS - custom file system with ATA PIO support
- **Shell**: Interactive command-line interface (berkesh)
- **VGA Driver**: Text mode with color support
- **Keyboard**: PS/2 keyboard driver
- **Memory**: Paging with 2 MiB huge pages
- **Interrupts**: IDT + PIC 8259 + PIT 100Hz
- **Text Editor**: Deno - built-in text editor
- **RTC**: Real-time clock support
- **Audio**: PC Speaker beep support

### Planned Features
- [ ] Network stack (TCP/IP)
- [ ] Sound card driver
- [ ] Multi-core CPU support
- [ ] USB 3.0 support
- [ ] GUI desktop environment
- [ ] Package manager
- [ ] Advanced text editor
- [ ] Web browser
- [ ] Mobile device support

## Code Statistics

| Metric | Value |
|--------|-------|
| Total Lines | ~14,288 |
| By Developer | 43% (~6,143 lines) |
| AI-Assisted | 57% (~8,145 lines) |
| Build Cost | 0 TL |

## Quick Start

### Requirements

**Arch Linux:**
```bash
sudo pacman -S rust nasm grub xorriso qemu
rustup override set nightly
rustup component add rust-src llvm-tools-preview
```

**Ubuntu/Debian:**
```bash
sudo apt install build-essential rustc nasm grub-pc-bin xorriso qemu-system-x86
rustup override set nightly
rustup component add rust-src llvm-tools-preview
```

### Build & Run

```bash
# Clone the repository
git clone https://github.com/berkeoruc/berkeos.git
cd berkeos

# Build the OS
chmod +x build.sh
./build.sh

# Run in QEMU
chmod +x run.sh
./run.sh
```

## Project Structure

```
BerkeOS/
├── src/
│   ├── main.rs              # Kernel entry point
│   ├── lib.rs               # Kernel library
│   ├── shell.rs             # Interactive shell (berkesh)
│   ├── berkefs.rs           # Custom filesystem
│   ├── vga.rs               # VGA text mode driver
│   ├── keyboard.rs          # PS/2 keyboard driver
│   ├── framebuffer.rs       # Graphics framebuffer
│   ├── font.rs              # Font renderer
│   ├── ata.rs               # ATA disk driver
│   ├── idt.rs               # Interrupt descriptor table
│   ├── pic.rs               # Programmable interrupt controller
│   ├── pit.rs               # Programmable interval timer
│   ├── paging.rs            # Memory paging
│   ├── allocator.rs         # Heap allocator
│   ├── scheduler.rs         # Process scheduler
│   ├── process.rs           # Process management
│   ├── syscall.rs           # System calls
│   ├── rtc.rs               # Real-time clock
│   ├── audio.rs             # Audio system
│   ├── pcspeaker.rs         # PC Speaker driver
│   ├── image.rs             # Image viewer
│   ├── editor.rs            # Text editor
│   ├── deno.rs              # Deno text editor
│   ├── boot.asm             # Boot assembly (NASM)
│   ├── ahci.rs              # AHCI SATA driver
│   ├── usb/                 # USB drivers
│   ├── net/                 # Network stack
│   └── rtl8139.rs           # Network card driver
├── Cargo.toml               # Rust project config
├── x86_64-berkeos.json      # Custom target spec
├── linker.ld                # Linker script
├── build.sh                 # Build script
├── run.sh                   # Run script
├── LICENSE                  # Apache 2.0 License
└── README.md                # This file
```

## Shell Commands

```
Navigation: cd, pwd, ls/dir, drives, df
File Ops:   cat, touch, mkdir, rm, cp, mv, find, stat
Editor:     deno <file>
System:     help, about, ver, uptime, mem, date, sysinfo
Network:    neofetch, ifconfig, ping
Tools:      calc, beep, play, snake, ascii
Admin:      reboot, halt, format, mkdrive, rmdrive
```

Run `help` in the shell for more commands.

## How It Works

```
UEFI/BIOS (auto-detect)
        │
        ▼
    boot.asm [32-bit]
        │
        ├── Verify boot mode
        ├── Set up page tables
        ├── Enable Long Mode
        └── Jump to 64-bit
                │
                ▼
        kernel_main() [Rust, no_std]
                │
                ├── Initialize VGA
                ├── Initialize keyboard
                ├── Mount BerkeFS
                ├── Start shell
                └── Halt loop
```

## Why Rust?

- **Memory Safety**: No garbage collector, no runtime overhead
- **No Undefined Behavior**: Critical for kernel development
- **Modern Tooling**: cargo, rustfmt, excellent error messages
- **no_std Support**: Runs on bare metal without any OS underneath
- **Zero-Cost Abstractions**: High-level code, machine efficiency

## Contributing

Contributions are welcome! Please read the Apache 2.0 license before contributing.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **Rust Community**: For the amazing no_std ecosystem
- **OSDev Wiki**: For invaluable kernel development resources
- **Free AI Tools**: For making this project possible at zero cost

---

**If this project interests you, please give it a star!**

*Made with ❤️ by Berke Oruc from Turkey*

[![GitHub stars](https://img.shields.io/github/stars/berkeoruc/berkeos?style=social)](https://github.com/berkeoruc/berkeos/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/berkeoruc/berkeos?style=social)](https://github.com/berkeoruc/berkeos/network/members)
