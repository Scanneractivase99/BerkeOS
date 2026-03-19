// BerkeOS — ata.rs
// ATA PIO Mode 28-bit LBA disk driver
// Polls the ATA bus — no IRQ, no DMA, no BIOS
// Works on QEMU's default IDE controller (primary bus, master drive)

use core::hint::spin_loop;

// ── ATA I/O port base addresses ───────────────────────────────────────────────
const ATA_PRIMARY_DATA: u16 = 0x1F0;
const ATA_PRIMARY_ERR: u16 = 0x1F1;
const ATA_PRIMARY_COUNT: u16 = 0x1F2;
const ATA_PRIMARY_LBA_LO: u16 = 0x1F3;
const ATA_PRIMARY_LBA_MID: u16 = 0x1F4;
const ATA_PRIMARY_LBA_HI: u16 = 0x1F5;
const ATA_PRIMARY_DRIVE: u16 = 0x1F6;
const ATA_PRIMARY_CMD: u16 = 0x1F7;
const ATA_PRIMARY_STATUS: u16 = 0x1F7;
const ATA_PRIMARY_CTRL: u16 = 0x3F6;

// ── ATA status bits ───────────────────────────────────────────────────────────
const ATA_STATUS_ERR: u8 = 0x01;
const ATA_STATUS_DRQ: u8 = 0x08;
const ATA_STATUS_SRV: u8 = 0x10;
const ATA_STATUS_DF: u8 = 0x20;
const ATA_STATUS_RDY: u8 = 0x40;
const ATA_STATUS_BSY: u8 = 0x80;

// ── ATA commands ──────────────────────────────────────────────────────────────
const ATA_CMD_READ_SECTORS: u8 = 0x20;
const ATA_CMD_WRITE_SECTORS: u8 = 0x30;
const ATA_CMD_FLUSH_CACHE: u8 = 0xE7;
const ATA_CMD_IDENTIFY: u8 = 0xEC;

pub const SECTOR_SIZE: usize = 512;

// ── Drive IDs and LBA offsets ─────────────────────────────────────────────────
/// Drive identifier for Alpha drive (LBA 0-32768, 16MB)
pub const DRIVE_ALPHA: u8 = 0;
/// Drive identifier for Beta drive (LBA 32768-1081344, 512MB offset)
pub const DRIVE_BETA: u8 = 1;

/// LBA offset for Alpha drive
const LBA_OFFSET_ALPHA: u32 = 0;
/// LBA offset for Beta drive (32768 sectors = 16MB)
const LBA_OFFSET_BETA: u32 = 32768;

/// Get LBA offset for a given drive ID
#[inline]
const fn get_lba_offset(drive_id: u8) -> u32 {
    match drive_id {
        DRIVE_ALPHA => LBA_OFFSET_ALPHA,
        DRIVE_BETA => LBA_OFFSET_BETA,
        _ => LBA_OFFSET_ALPHA, // Default to Alpha for unknown drives
    }
}

// ── I/O helpers ───────────────────────────────────────────────────────────────
#[inline]
unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    core::arch::asm!("in al, dx", out("al") val, in("dx") port, options(nomem, nostack));
    val
}

#[inline]
unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack));
}

#[inline]
unsafe fn inw(port: u16) -> u16 {
    let val: u16;
    core::arch::asm!("in ax, dx", out("ax") val, in("dx") port, options(nomem, nostack));
    val
}

#[inline]
unsafe fn outw(port: u16, val: u16) {
    core::arch::asm!("out dx, ax", in("dx") port, in("ax") val, options(nomem, nostack));
}

// ── Wait for BSY to clear ─────────────────────────────────────────────────────
unsafe fn wait_not_busy() -> bool {
    let mut timeout = 0u32;
    loop {
        let status = inb(ATA_PRIMARY_STATUS);
        if status & ATA_STATUS_BSY == 0 {
            return true;
        }
        timeout += 1;
        if timeout > 100_000_000 {
            return false;
        }
        spin_loop();
    }
}

// ── Wait for DRQ (data ready) ─────────────────────────────────────────────────
unsafe fn wait_drq() -> bool {
    let mut timeout = 0u32;
    loop {
        let status = inb(ATA_PRIMARY_STATUS);
        if status & ATA_STATUS_ERR != 0 {
            return false;
        }
        if status & ATA_STATUS_DF != 0 {
            return false;
        }
        if status & ATA_STATUS_DRQ != 0 {
            return true;
        }
        timeout += 1;
        if timeout > 100_000_000 {
            return false;
        }
        spin_loop();
    }
}

// ── 400ns delay (read alt status 4 times) ─────────────────────────────────────
unsafe fn delay400ns() {
    inb(ATA_PRIMARY_CTRL);
    inb(ATA_PRIMARY_CTRL);
    inb(ATA_PRIMARY_CTRL);
    inb(ATA_PRIMARY_CTRL);
}

// ── Detect if ATA drive present ───────────────────────────────────────────────
pub static mut DISK_COUNT: usize = 0;

unsafe fn detect_drive(drive_sel: u8) -> bool {
    outb(ATA_PRIMARY_DRIVE, drive_sel);
    delay400ns();

    outb(ATA_PRIMARY_CTRL, 0x04);
    delay400ns();
    outb(ATA_PRIMARY_CTRL, 0x00);
    delay400ns();

    let mut retries = 10;
    while retries > 0 {
        if wait_not_busy() {
            break;
        }
        retries -= 1;
    }

    outb(ATA_PRIMARY_COUNT, 0);
    outb(ATA_PRIMARY_LBA_LO, 0);
    outb(ATA_PRIMARY_LBA_MID, 0);
    outb(ATA_PRIMARY_LBA_HI, 0);
    outb(ATA_PRIMARY_CMD, ATA_CMD_IDENTIFY);

    delay400ns();
    let status = inb(ATA_PRIMARY_STATUS);

    if status == 0 {
        return false;
    }

    retries = 100;
    while retries > 0 {
        let s = inb(ATA_PRIMARY_STATUS);
        if s & ATA_STATUS_BSY == 0 {
            break;
        }
        retries -= 1;
    }

    if status & ATA_STATUS_ERR != 0 {
        return false;
    }

    let mid = inb(ATA_PRIMARY_LBA_MID);
    let hi = inb(ATA_PRIMARY_LBA_HI);
    if mid == 0x14 && hi == 0xEB {
        return false;
    }

    retries = 100;
    while retries > 0 {
        let s = inb(ATA_PRIMARY_STATUS);
        if s & ATA_STATUS_DRQ != 0 {
            break;
        }
        if s & ATA_STATUS_ERR != 0 {
            return false;
        }
        retries -= 1;
    }

    for _ in 0..256 {
        inw(ATA_PRIMARY_DATA);
    }

    true
}

pub unsafe fn ata_detect() -> bool {
    DISK_COUNT = 0;

    if detect_drive(0xA0) {
        DISK_COUNT += 1;
    }

    if detect_drive(0xB0) {
        DISK_COUNT += 1;
    }

    DISK_COUNT > 0
}

pub fn get_disk_count() -> usize {
    unsafe { DISK_COUNT }
}

// ── Read one sector (512 bytes) from LBA address ──────────────────────────────
pub unsafe fn read_sector(drive_id: u8, lba: u32, buf: &mut [u8; SECTOR_SIZE]) -> bool {
    let offset_lba = lba + get_lba_offset(drive_id);
    outb(ATA_PRIMARY_DRIVE, 0xE0 | ((offset_lba >> 24) as u8 & 0x0F));
    delay400ns();

    if !wait_not_busy() {
        return false;
    }

    outb(ATA_PRIMARY_ERR, 0);
    outb(ATA_PRIMARY_COUNT, 1);
    outb(ATA_PRIMARY_LBA_LO, (offset_lba & 0xFF) as u8);
    outb(ATA_PRIMARY_LBA_MID, ((offset_lba >> 8) & 0xFF) as u8);
    outb(ATA_PRIMARY_LBA_HI, ((offset_lba >> 16) & 0xFF) as u8);
    outb(ATA_PRIMARY_CMD, ATA_CMD_READ_SECTORS);

    delay400ns();

    if !wait_drq() {
        return false;
    }

    // Read 256 words = 512 bytes
    let ptr = buf.as_mut_ptr() as *mut u16;
    for i in 0..256 {
        let word = inw(ATA_PRIMARY_DATA);
        ptr.add(i).write_volatile(word);
    }

    true
}

// ── Write one sector (512 bytes) to LBA address ───────────────────────────────
pub unsafe fn write_sector(drive_id: u8, lba: u32, buf: &[u8; SECTOR_SIZE]) -> bool {
    let offset_lba = lba + get_lba_offset(drive_id);
    outb(ATA_PRIMARY_DRIVE, 0xE0 | ((offset_lba >> 24) as u8 & 0x0F));
    delay400ns();

    if !wait_not_busy() {
        return false;
    }

    outb(ATA_PRIMARY_ERR, 0);
    outb(ATA_PRIMARY_COUNT, 1);
    outb(ATA_PRIMARY_LBA_LO, (offset_lba & 0xFF) as u8);
    outb(ATA_PRIMARY_LBA_MID, ((offset_lba >> 8) & 0xFF) as u8);
    outb(ATA_PRIMARY_LBA_HI, ((offset_lba >> 16) & 0xFF) as u8);
    outb(ATA_PRIMARY_CMD, ATA_CMD_WRITE_SECTORS);

    delay400ns();

    if !wait_drq() {
        return false;
    }

    // Write 256 words = 512 bytes
    let ptr = buf.as_ptr() as *const u16;
    for i in 0..256 {
        outw(ATA_PRIMARY_DATA, ptr.add(i).read_volatile());
    }

    // Flush write cache
    outb(ATA_PRIMARY_CMD, ATA_CMD_FLUSH_CACHE);
    if !wait_not_busy() {
        return false;
    }

    true
}
