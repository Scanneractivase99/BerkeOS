// BerkeOS — scheduler.rs
// Round-robin preemptive scheduler
// Called from IRQ0 (PIT timer, 100Hz)
// Saves current context, picks next ready process, restores its context

use crate::process::{ProcessState, ProcessTable, MAX_PROCESSES};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

// Scheduler enabled flag — disabled during init
pub static SCHEDULER_ENABLED: AtomicBool = AtomicBool::new(false);
pub static SCHEDULE_COUNT: AtomicUsize = AtomicUsize::new(0);

// Global process table — allocated statically
pub static mut PTABLE: ProcessTable = ProcessTable::new();

// Kernel stacks for processes — 8 KiB each, 16 processes
// Total: 128 KiB — placed in BSS
#[repr(align(16))]
pub struct KernelStacks {
    pub data: [[u8; 8192]; MAX_PROCESSES],
}

pub static mut KSTACKS: KernelStacks = KernelStacks {
    data: [[0u8; 8192]; MAX_PROCESSES],
};

// Enable the scheduler
pub fn enable() {
    SCHEDULER_ENABLED.store(true, Ordering::Release);
}

// Disable the scheduler
pub fn disable() {
    SCHEDULER_ENABLED.store(false, Ordering::Release);
}

pub fn is_enabled() -> bool {
    SCHEDULER_ENABLED.load(Ordering::Acquire)
}

// Called from IRQ0 handler — performs context switch if needed
// This is called in interrupt context — must be very fast
pub unsafe fn tick() {
    if !is_enabled() {
        return;
    }

    let ptable = &mut *(&raw mut PTABLE);

    // Increment current process tick count
    let cur = ptable.current;
    if ptable.procs[cur].state == ProcessState::Running {
        ptable.procs[cur].ticks += 1;
    }

    SCHEDULE_COUNT.fetch_add(1, Ordering::Relaxed);
}

// Voluntary yield — switch to next ready process
pub unsafe fn schedule() {
    let ptable = &mut *(&raw mut PTABLE);

    let cur = ptable.current;

    // Mark current as ready (if it was running)
    if ptable.procs[cur].state == ProcessState::Running {
        ptable.procs[cur].state = ProcessState::Ready;
    }

    // Find next ready process
    if let Some(next) = ptable.next_ready(cur) {
        ptable.current = next;
        ptable.procs[next].state = ProcessState::Running;
    } else {
        // No other process ready — keep running current
        ptable.procs[cur].state = ProcessState::Running;
    }
}

// Initialize scheduler — register the idle process (kernel main)
pub unsafe fn init() {
    let ptable = &mut *(&raw mut PTABLE);

    // Process 0 = kernel/idle process
    ptable.procs[0].pid = 0;
    ptable.procs[0].state = ProcessState::Running;
    ptable.procs[0].ticks = 0;
    let name = b"kernel";
    ptable.procs[0].name_len = name.len();
    ptable.procs[0].name[..name.len()].copy_from_slice(name);
    ptable.current = 0;
    ptable.count = 1;
}

// Get current PID
pub fn current_pid() -> u32 {
    unsafe { (&*(&raw const PTABLE)).current_proc().pid }
}

// Get schedule count
pub fn schedule_count() -> usize {
    SCHEDULE_COUNT.load(Ordering::Relaxed)
}

// Get process count
pub fn process_count() -> usize {
    unsafe { (&*(&raw const PTABLE)).count }
}
