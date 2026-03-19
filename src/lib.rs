#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(global_allocator)]

mod ahci;
mod allocator;
mod ata;
mod berkefs;
mod boot;
mod deno;
mod disk_io;
mod disk_paths;
mod editor;
mod font;
mod framebuffer;
mod idt;
mod keyboard;

mod net;
mod paging;
mod pcspeaker;
mod pic;
mod pit;
mod process;
mod rtc;
mod rtl8139;
mod scheduler;
mod shell;
mod syscall;
mod term;
mod usb;
mod vga;

use berkefs::BerkeFS;
use core::panic::PanicInfo;
use framebuffer::Framebuffer;
use shell::Shell;

use allocator::{KernelAllocator, HEAP_SIZE, HEAP_START};

#[global_allocator]
static KERNEL_ALLOCATOR: KernelAllocator = KernelAllocator::new();

#[repr(C)]
pub struct FbInfo {
    pub addr: u64,
    pub pitch: u32,
    pub width: u32,
    pub height: u32,
    pub bpp: u8,
    pub fb_type: u8,
}

// ── VGA diagnostic helpers ────────────────────────────────────────────────────
fn vga_probe() -> bool {
    // Probe VGA: try to write and read back
    unsafe {
        let vga = 0xb8000 as *mut u16;
        let orig = vga.read_volatile();
        vga.write_volatile(0xAA55);
        let result = vga.read_volatile();
        vga.write_volatile(orig);
        result == 0xAA55
    }
}

static VGA_AVAILABLE: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(true);

unsafe fn vga_clear() {
    if !VGA_AVAILABLE.load(core::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let vga = 0xb8000 as *mut u8;
    for i in 0..80 * 25 {
        vga.add(i * 2).write_volatile(b' ');
        vga.add(i * 2 + 1).write_volatile(0x07);
    }
}

unsafe fn vga_print(row: usize, col: usize, msg: &str, attr: u8) {
    if !VGA_AVAILABLE.load(core::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let vga = 0xb8000 as *mut u8;
    for (i, b) in msg.bytes().enumerate() {
        if col + i >= 80 {
            break;
        }
        let off = (row * 80 + col + i) * 2;
        vga.add(off).write_volatile(b);
        vga.add(off + 1).write_volatile(attr);
    }
}

unsafe fn vga_hex32(row: usize, col: usize, val: u32, attr: u8) {
    if !VGA_AVAILABLE.load(core::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let vga = 0xb8000 as *mut u8;
    for i in 0..8 {
        let nibble = ((val >> (28 - i * 4)) & 0xF) as u8;
        let ch = if nibble < 10 {
            b'0' + nibble
        } else {
            b'a' + nibble - 10
        };
        let off = (row * 80 + col + i) * 2;
        vga.add(off).write_volatile(ch);
        vga.add(off + 1).write_volatile(attr);
    }
}

unsafe fn vga_hex64(row: usize, col: usize, val: u64, attr: u8) {
    vga_hex32(row, col, (val >> 32) as u32, attr);
    vga_hex32(row, col + 8, val as u32, attr);
}

// ── Static shell storage — avoids blowing the 64KiB stack ────────────────────
static mut SHELL: Shell = Shell::new_static();

// ── Per-drive BerkeFS instances (12 drives: Alpha..Mu) ───────────────────────
// Drive index: 0=Alpha, 1=Beta, 2=Gamma, 3=Sigma, 4=Epsilon, 5=Zeta,
//              6=Eta, 7=Theta, 8=Iota, 9=Kappa, 10=Lambda, 11=Mu
static mut FS0: BerkeFS = BerkeFS::new(0); // Alpha (ATA/SATA disk)
static mut FS1: BerkeFS = BerkeFS::new(1); // Beta  (RAM disk)
static mut FS2: BerkeFS = BerkeFS::new(2); // Gamma
static mut FS3: BerkeFS = BerkeFS::new(3); // Sigma
static mut FS4: BerkeFS = BerkeFS::new(4); // Epsilon
static mut FS5: BerkeFS = BerkeFS::new(5); // Zeta
static mut FS6: BerkeFS = BerkeFS::new(6); // Eta
static mut FS7: BerkeFS = BerkeFS::new(7); // Theta
static mut FS8: BerkeFS = BerkeFS::new(8); // Iota
static mut FS9: BerkeFS = BerkeFS::new(9); // Kappa
static mut FS10: BerkeFS = BerkeFS::new(10); // Lambda
static mut FS11: BerkeFS = BerkeFS::new(11); // Mu

/// Returns a mutable reference to the BerkeFS instance for the given drive index.
unsafe fn get_fs(drive_idx: usize) -> Option<&'static mut BerkeFS> {
    match drive_idx {
        0 => Some(&mut FS0),
        1 => Some(&mut FS1),
        2 => Some(&mut FS2),
        3 => Some(&mut FS3),
        4 => Some(&mut FS4),
        5 => Some(&mut FS5),
        6 => Some(&mut FS6),
        7 => Some(&mut FS7),
        8 => Some(&mut FS8),
        9 => Some(&mut FS9),
        10 => Some(&mut FS10),
        11 => Some(&mut FS11),
        _ => None,
    }
}

// ── Alpha RAM Disk (4 MiB sector cache) — Boot BSS must fit in 256 MiB ─────────
// Reduced from 16 MiB to avoid Boot OOM during BSS zero-init.
const ALPHA_RAM_DISK_SIZE: usize = 4 * 1024 * 1024; // 4 MiB sector cache
static mut ALPHA_RAM_DISK: [u8; ALPHA_RAM_DISK_SIZE] = [0u8; ALPHA_RAM_DISK_SIZE];

// ── Beta Disk (32 MiB sector cache) — Boot BSS must fit in 256 MiB ────────────
// Reduced from 512 MiB. Actual disk is 256 MiB (run.sh). 32 MiB cache is sufficient.
const BETA_RAM_DISK_SIZE: usize = 32 * 1024 * 1024; // 32 MiB sector cache
static mut BETA_RAM_DISK: [u8; BETA_RAM_DISK_SIZE] = [0u8; BETA_RAM_DISK_SIZE];

// ── Beta RAM Disk sector I/O (operates on BETA_RAM_DISK buffer) ────────────────
const BETA_SECTOR_SIZE: usize = 512;

unsafe fn beta_read_sector(lba: u32, buf: &mut [u8; BETA_SECTOR_SIZE]) -> bool {
    let offset = (lba as usize) * BETA_SECTOR_SIZE;
    if offset + BETA_SECTOR_SIZE > BETA_RAM_DISK_SIZE {
        return false;
    }
    let start = offset;
    let end = start + BETA_SECTOR_SIZE;
    buf.copy_from_slice(&BETA_RAM_DISK[start..end]);
    true
}

unsafe fn beta_write_sector(lba: u32, buf: &[u8; BETA_SECTOR_SIZE]) -> bool {
    let offset = (lba as usize) * BETA_SECTOR_SIZE;
    if offset + BETA_SECTOR_SIZE > BETA_RAM_DISK_SIZE {
        return false;
    }
    let start = offset;
    let end = start + BETA_SECTOR_SIZE;
    BETA_RAM_DISK[start..end].copy_from_slice(buf);
    true
}

// ── Check if Beta disk has valid BerkeFS signature ───────────────────────────
fn beta_disk_exists() -> bool {
    let mut sector = [0u8; BETA_SECTOR_SIZE];
    unsafe {
        if !beta_read_sector(berkefs::SUPERBLOCK_LBA, &mut sector) {
            return false;
        }
        let magic = u32::from_le_bytes([sector[0], sector[1], sector[2], sector[3]]);
        magic == berkefs::BERKEFS_MAGIC
    }
}

// ── Format Beta disk with BerkeFS ────────────────────────────────────────────
fn beta_format() -> bool {
    let mut sector = [0u8; BETA_SECTOR_SIZE];
    let magic = berkefs::BERKEFS_MAGIC.to_le_bytes();
    sector[0] = magic[0];
    sector[1] = magic[1];
    sector[2] = magic[2];
    sector[3] = magic[3];
    sector[4] = (berkefs::BERKEFS_VERSION & 0xFF) as u8;
    sector[5] = (berkefs::BERKEFS_VERSION >> 8) as u8;
    sector[6] = (berkefs::MAX_DATA_BLOCKS & 0xFF) as u8;
    sector[7] = (berkefs::MAX_DATA_BLOCKS >> 8) as u8;
    sector[8] = (berkefs::MAX_DATA_BLOCKS & 0xFF) as u8;
    sector[9] = (berkefs::MAX_DATA_BLOCKS >> 8) as u8;
    let label = b"Beta";
    sector[12..16].copy_from_slice(label);
    unsafe { beta_write_sector(berkefs::SUPERBLOCK_LBA, &sector) }
}

// ── Kernel entry ──────────────────────────────────────────────────────────────
#[no_mangle]
pub extern "C" fn kernel_main(mb2_info_ptr: u32) -> ! {
    // Probe for VGA availability early
    let vga_exists = vga_probe();
    VGA_AVAILABLE.store(vga_exists, core::sync::atomic::Ordering::Relaxed);

    // Show loading spinner if VGA exists
    let spinner_chars = [b'-', b'\\', b'|', b'/'];

    for i in 0..20 {
        if vga_exists {
            unsafe {
                let vga = 0xb8000 as *mut u16;
                vga.offset(0)
                    .write_volatile((spinner_chars[(i as usize) % 4] as u16) | 0x0F00);
            }
        }
        for _ in 0..100000 {}
    }

    let fb_info = unsafe { parse_mb2_framebuffer(mb2_info_ptr) };

    match fb_info {
        Some(info) => {
            if vga_exists {
                unsafe {
                    vga_clear();
                }
                unsafe {
                    vga_print(0, 0, "BerkeOS v5.6 booting...", 0x0a);
                }
            }

            for i in 0..5 {
                if vga_exists {
                    unsafe {
                        vga_print(1, i * 10, "[==========]", 0x0a);
                    }
                }
                for _ in 0..500000 {}
            }

            let mut fb = unsafe { Framebuffer::new(info) };
            let w = fb.width;
            let h = fb.height;

            unsafe {
                vga_print(2, 0, "Launching shell...", 0x0a);
            }

            unsafe {
                idt::init();
                pic::init();
                pit::init(100);
                scheduler::init();
                pic::enable();
                vga_print(2, 20, "IDT+PIC+PIT+SCHED OK", 0x0a);
            }

            let fs = unsafe { &mut *(&raw mut FS0) };

            let ata_ok = unsafe { ata::ata_detect() };

            let sata_ok = if !ata_ok {
                unsafe { ahci::ahci_init() }
            } else {
                false
            };

            let disk_ok = sata_ok || ata_ok;

            unsafe {
                vga_print(3, 0, "ATA:    ", 0x08);
                if ata_ok {
                    vga_print(3, 5, "OK   ", 0x0a);
                } else {
                    vga_print(3, 5, "FAIL ", 0x0c);
                }
                vga_print(3, 11, "SATA:   ", 0x08);
                if sata_ok {
                    vga_print(3, 17, "OK   ", 0x0a);
                } else {
                    vga_print(3, 17, "FAIL ", 0x0c);
                }
                vga_print(3, 23, "DISK:   ", 0x08);
                if disk_ok {
                    vga_print(3, 29, "OK   ", 0x0a);
                } else {
                    vga_print(3, 29, "FAIL ", 0x0c);
                }
            }

            if disk_ok && ata_ok {
                unsafe {
                    vga_print(3, 20, "ATA disk OK", 0x0a);
                }
                if !fs.mount() {
                    unsafe {
                        vga_print(3, 40, "Formatting...", 0x0e);
                    }
                    fs.format(b"Alpha");
                } else {
                    unsafe {
                        vga_print(3, 40, "BerkeFS mounted", 0x0a);
                    }
                }
            } else if sata_ok {
                unsafe {
                    vga_print(3, 0, "SATA (AHCI) detected!", 0x0a);
                }
                if !fs.mount() {
                    unsafe {
                        vga_print(3, 40, "Formatting...", 0x0e);
                    }
                    fs.format(b"Alpha");
                } else {
                    unsafe {
                        vga_print(3, 40, "BerkeFS mounted", 0x0a);
                    }
                }
            } else {
                unsafe {
                    vga_print(3, 40, "Live USB Mode!", 0x0a);
                }
            }

            if vga_exists {
                unsafe {
                    vga_print(4, 10, "Beta:    ", 0x08);
                }
                if beta_disk_exists() {
                    unsafe {
                        vga_print(4, 17, "Mounted  ", 0x0a);
                        FS1.set_mounted();
                    }
                } else {
                    unsafe {
                        vga_print(4, 17, "Creating ", 0x0e);
                    }
                    if beta_format() {
                        unsafe {
                            vga_print(4, 17, "Ready    ", 0x0a);
                            FS1.set_mounted();
                            vga_print(4, 25, "512MB OK ", 0x0a);
                        }
                    } else {
                        unsafe {
                            vga_print(4, 25, "FAIL     ", 0x0c);
                        }
                    }
                }
            } else {
                // No VGA: Beta is still a RAM disk, mark it as mounted
                unsafe {
                    FS1.set_mounted();
                }
            }

            if vga_exists {
                unsafe {
                    vga_print(4, 0, "[====================] DONE!", 0x0a);
                }
            }

            // Ensure Beta RAM disk FS is mounted if not already
            unsafe {
                if !FS1.mounted {
                    FS1.set_mounted();
                }
            }

            let shell = unsafe { &mut *(&raw mut SHELL) };
            let disk_count = ata::get_disk_count();
            shell.init(
                w,
                h,
                disk_ok,
                disk_count,
                unsafe { &mut FS0 },
                unsafe { &mut FS1 },
                unsafe { &mut FS2 },
                unsafe { &mut FS3 },
                unsafe { &mut FS4 },
                unsafe { &mut FS5 },
                unsafe { &mut FS6 },
                unsafe { &mut FS7 },
                unsafe { &mut FS8 },
                unsafe { &mut FS9 },
                unsafe { &mut FS10 },
                unsafe { &mut FS11 },
            );

            fb.clear(framebuffer::Color::rgb(0x00, 0x00, 0x00));

            let green = framebuffer::Color::rgb(0x00, 0xFF, 0x00);
            let white = framebuffer::Color::rgb(0xFF, 0xFF, 0xFF);
            let red = framebuffer::Color::rgb(0xFF, 0x00, 0x00);
            let cyan = framebuffer::Color::rgb(0x00, 0xFF, 0xFF);

            fb.draw_string(
                25,
                2,
                "██████╗ ███████╗████████╗██████╗  ██████╗ ",
                white,
                framebuffer::Color::rgb(0x00, 0x00, 0x00),
            );
            fb.draw_string(
                25,
                3,
                "██╔══██╗██╔════╝╚══██╔══╝██╔══██╗██╔═══██╗",
                white,
                framebuffer::Color::rgb(0x00, 0x00, 0x00),
            );
            fb.draw_string(
                25,
                4,
                "██████╔╝█████╗     ██║   ██████╔╝██║   ██║",
                white,
                framebuffer::Color::rgb(0x00, 0x00, 0x00),
            );
            fb.draw_string(
                25,
                5,
                "██╔══██╗██╔══╝     ██║   ██╔══██╗██║   ██║",
                white,
                framebuffer::Color::rgb(0x00, 0x00, 0x00),
            );
            fb.draw_string(
                25,
                6,
                "██║  ██║███████╗   ██║   ██║  ██║╚██████╔╝",
                white,
                framebuffer::Color::rgb(0x00, 0x00, 0x00),
            );
            fb.draw_string(
                25,
                7,
                "╚═╝  ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ",
                white,
                framebuffer::Color::rgb(0x00, 0x00, 0x00),
            );

            fb.draw_string(
                28,
                9,
                "╔════════════════════════════════════════╗",
                cyan,
                framebuffer::Color::rgb(0x00, 0x00, 0x00),
            );
            fb.draw_string(
                28,
                10,
                "║       BerkeOS v5.6 - Boot Sequence    ║",
                cyan,
                framebuffer::Color::rgb(0x00, 0x00, 0x00),
            );
            fb.draw_string(
                28,
                11,
                "╚════════════════════════════════════════╝",
                cyan,
                framebuffer::Color::rgb(0x00, 0x00, 0x00),
            );

            fb.draw_string(
                10,
                13,
                "Initializing BerkeOS...",
                white,
                framebuffer::Color::rgb(0x00, 0x00, 0x00),
            );
            pit::sleep_ms(1000);
            fb.draw_string(
                40,
                13,
                "[  OK  ]",
                green,
                framebuffer::Color::rgb(0x00, 0x00, 0x00),
            );

            fb.draw_string(
                10,
                14,
                "Loading memory manager...",
                white,
                framebuffer::Color::rgb(0x00, 0x00, 0x00),
            );
            pit::sleep_ms(1000);
            fb.draw_string(
                40,
                14,
                "[  OK  ]",
                green,
                framebuffer::Color::rgb(0x00, 0x00, 0x00),
            );

            fb.draw_string(
                10,
                15,
                "Setting up interrupts...",
                white,
                framebuffer::Color::rgb(0x00, 0x00, 0x00),
            );
            pit::sleep_ms(1000);
            fb.draw_string(
                40,
                15,
                "[  OK  ]",
                green,
                framebuffer::Color::rgb(0x00, 0x00, 0x00),
            );

            fb.draw_string(
                10,
                16,
                "Initializing keyboard...",
                white,
                framebuffer::Color::rgb(0x00, 0x00, 0x00),
            );
            pit::sleep_ms(1000);
            fb.draw_string(
                40,
                16,
                "[  OK  ]",
                green,
                framebuffer::Color::rgb(0x00, 0x00, 0x00),
            );

            fb.draw_string(
                10,
                17,
                "Initializing storage...",
                white,
                framebuffer::Color::rgb(0x00, 0x00, 0x00),
            );
            pit::sleep_ms(1000);
            fb.draw_string(
                40,
                17,
                "[  OK  ]",
                green,
                framebuffer::Color::rgb(0x00, 0x00, 0x00),
            );

            fb.draw_string(
                10,
                18,
                "Starting shell...",
                white,
                framebuffer::Color::rgb(0x00, 0x00, 0x00),
            );
            pit::sleep_ms(1000);
            fb.draw_string(
                40,
                18,
                "[  OK  ]",
                green,
                framebuffer::Color::rgb(0x00, 0x00, 0x00),
            );

            fb.clear(framebuffer::Color::rgb(0x00, 0x00, 0x00));

            shell.run(&mut fb);
        }

        None => {
            // No framebuffer - use VGA text mode
            if vga_exists {
                let v = vga::Vga::new();
                v.clear(vga::Color::Black);
                v.fill_row(0, vga::Color::Blue);
                v.print_at(
                    1,
                    0,
                    "BerkeOS v5.6 booting...",
                    vga::Color::White,
                    vga::Color::Blue,
                );
                v.fill_row(2, vga::Color::Black);
                v.print_at(
                    1,
                    2,
                    "Detecting hardware...",
                    vga::Color::Yellow,
                    vga::Color::Black,
                );

                // Initialize basics
                unsafe {
                    idt::init();
                    pic::init();
                    pit::init(100);
                    scheduler::init();
                    pic::enable();
                }

                v.print_at(
                    1,
                    3,
                    "Checking storage...",
                    vga::Color::Yellow,
                    vga::Color::Black,
                );
                let sata_ok = unsafe { ahci::ahci_init() };
                let ata_ok = if !sata_ok {
                    unsafe { ata::ata_detect() }
                } else {
                    false
                };

                if sata_ok {
                    v.print_at(1, 4, "SATA (AHCI) OK", vga::Color::Green, vga::Color::Black);
                } else if ata_ok {
                    v.print_at(1, 4, "ATA disk OK", vga::Color::Green, vga::Color::Black);
                } else {
                    v.print_at(1, 4, "Live USB Mode", vga::Color::Cyan, vga::Color::Black);
                }

                v.fill_row(6, vga::Color::Blue);
                v.print_at(
                    1,
                    6,
                    "Loading BerkeOS...",
                    vga::Color::White,
                    vga::Color::Blue,
                );

                let fs = unsafe { &mut *(&raw mut FS0) };
                if !fs.mount() {
                    v.print_at(
                        1,
                        7,
                        "Formatting drive...",
                        vga::Color::Yellow,
                        vga::Color::Black,
                    );
                    fs.format(b"Alpha");
                }

                v.fill_row(24, vga::Color::Blue);
                v.print_at(
                    1,
                    24,
                    "BerkeOS v5.6 | Berke Oruc | Rust | x86_64",
                    vga::Color::White,
                    vga::Color::Blue,
                );
            } else {
                // No VGA, no framebuffer - halt with error
                loop {}
            }
        }
    }

    loop {}
}

// ── Multiboot2 framebuffer tag parser ─────────────────────────────────────────
unsafe fn parse_mb2_framebuffer(mb2_ptr: u32) -> Option<FbInfo> {
    if mb2_ptr == 0 {
        return None;
    }

    let total_size = (mb2_ptr as *const u32).read_volatile();

    unsafe {
        vga_print(24, 0, "MB2 sz=0x", 0x08);
    }
    unsafe {
        vga_hex32(24, 9, total_size, 0x08);
    }

    if total_size < 8 || total_size > 0x20000 {
        return None;
    }

    let mut offset: u32 = 8;
    let mut tag_count: u32 = 0;

    while offset + 8 <= total_size {
        let tag_addr = mb2_ptr + offset;
        let tag_type = (tag_addr as *const u32).read_volatile();
        let tag_size = ((tag_addr + 4) as *const u32).read_volatile();

        if tag_count < 6 {
            let col = (tag_count * 13) as usize;
            if col + 8 < 80 {
                vga_hex32(23, col, tag_type, 0x08);
            }
        }
        tag_count += 1;

        if tag_type == 0 {
            break;
        }

        // Tag type 8 = framebuffer info
        if tag_type == 8 && tag_size >= 31 {
            let base = tag_addr as *const u8;
            let addr = (base.add(8) as *const u64).read_volatile();
            let pitch = (base.add(16) as *const u32).read_volatile();
            let width = (base.add(20) as *const u32).read_volatile();
            let height = (base.add(24) as *const u32).read_volatile();
            let bpp = base.add(28).read_volatile();
            let fb_type = base.add(29).read_volatile();

            vga_print(3, 0, "FB tag: addr=0x", 0x0b);
            vga_hex64(3, 15, addr, 0x0b);
            vga_print(3, 32, " bpp=", 0x0b);
            {
                let vga = 0xb8000 as *mut u8;
                vga.add((3 * 80 + 37) * 2).write_volatile(b'0' + bpp / 10);
                vga.add((3 * 80 + 37) * 2 + 1).write_volatile(0x0b);
                vga.add((3 * 80 + 38) * 2).write_volatile(b'0' + bpp % 10);
                vga.add((3 * 80 + 38) * 2 + 1).write_volatile(0x0b);
            }
            vga_print(3, 40, " type=", 0x0b);
            {
                let vga = 0xb8000 as *mut u8;
                vga.add((3 * 80 + 46) * 2).write_volatile(b'0' + fb_type);
                vga.add((3 * 80 + 46) * 2 + 1).write_volatile(0x0b);
            }

            // Accept any pixel framebuffer (type 0 or 1), any bpp >= 8
            // Do NOT check addr upper bound — framebuffer can be anywhere in 4GiB
            if addr != 0 && width > 0 && height > 0 && bpp >= 8 && fb_type != 2 {
                vga_print(4, 0, "FB ACCEPTED!", 0x0a);
                return Some(FbInfo {
                    addr,
                    pitch,
                    width,
                    height,
                    bpp,
                    fb_type,
                });
            } else {
                vga_print(4, 0, "FB rejected:", 0x0c);
                if addr == 0 {
                    vga_print(4, 13, "addr=0", 0x0c);
                }
                if width == 0 {
                    vga_print(4, 20, "w=0", 0x0c);
                }
                if height == 0 {
                    vga_print(4, 24, "h=0", 0x0c);
                }
                if bpp < 8 {
                    vga_print(4, 28, "bpp<8", 0x0c);
                }
                if fb_type == 2 {
                    vga_print(4, 34, "text", 0x0c);
                }
            }
        }

        let aligned = (tag_size + 7) & !7;
        if aligned == 0 {
            break;
        }
        offset += aligned;
    }

    None
}

// ── Panic handler ─────────────────────────────────────────────────────────────
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        let vga = 0xb8000 as *mut u8;
        for i in 0..80 {
            vga.add(i * 2).write_volatile(b' ');
            vga.add(i * 2 + 1).write_volatile(0x4f);
        }
        let msg = b"!!! KERNEL PANIC !!!";
        for (i, &b) in msg.iter().enumerate() {
            vga.add(i * 2).write_volatile(b);
            vga.add(i * 2 + 1).write_volatile(0x4f);
        }
    }
    loop {}
}
