//! BerkeOS builtin module for BerkeBex compiler.
//!
//! Provides access to BerkeOS syscalls through a Python-friendly API.
//!
//! Usage:
//! ```python
//! import berkeos
//! berkeos.process.sleep(1000)
//! berkeos.display.clear()
//! ```

use crate::ir::{IrBlock, IrFunction, IrInstruction, IrModule};

// Syscall numbers from BerkeOS kernel (src/syscall.rs)
pub const SYS_WINDOW_NEW: i32 = 60;
pub const SYS_FOPEN: i32 = 12;
pub const SYS_FREAD: i32 = 13;
pub const SYS_FWRITE: i32 = 14;
pub const SYS_FCLOSE: i32 = 15;
pub const SYS_FB_CLEAR: i32 = 33;
pub const SYS_FB_PIXEL: i32 = 31;
pub const SYS_FB_RECT: i32 = 32;
pub const SYS_FB_TEXT: i32 = 34;
pub const SYS_READ_KEY: i32 = 40;
pub const SYS_SLEEP: i32 = 4;
pub const SYS_GETPID: i32 = 3;

/// BerkeOS module namespace
pub mod berkeos {
    use super::*;

    /// Process management submodule
    pub mod process {
        use super::*;

        /// Sleep for specified milliseconds.
        /// Corresponds to SYS_SLEEP (4)
        pub fn sleep_ir(ms: usize) -> Vec<IrInstruction> {
            vec![
                IrInstruction::ConstInt {
                    dest: 0,
                    value: ms as i64,
                },
                IrInstruction::Call {
                    dest: 1,
                    func: "__berkeos_syscall_sleep".to_string(),
                    args: vec![0],
                },
            ]
        }

        /// Get current process ID.
        /// Corresponds to SYS_GETPID (3)
        pub fn getpid_ir() -> Vec<IrInstruction> {
            vec![IrInstruction::Call {
                dest: 0,
                func: "__berkeos_syscall_getpid".to_string(),
                args: vec![],
            }]
        }
    }

    /// File operations submodule
    pub mod file {
        use super::*;

        /// Open a file.
        /// Corresponds to SYS_FOPEN (12)
        pub fn open_ir(path: &str, mode: &str) -> Vec<IrInstruction> {
            vec![
                IrInstruction::ConstString {
                    dest: 0,
                    value: path.to_string(),
                },
                IrInstruction::ConstString {
                    dest: 1,
                    value: mode.to_string(),
                },
                IrInstruction::Call {
                    dest: 2,
                    func: "__berkeos_syscall_fopen".to_string(),
                    args: vec![0, 1],
                },
            ]
        }

        /// Read from a file handle.
        /// Corresponds to SYS_FREAD (13)
        pub fn read_ir(handle: usize, size: usize) -> Vec<IrInstruction> {
            vec![
                IrInstruction::ConstInt {
                    dest: 0,
                    value: handle as i64,
                },
                IrInstruction::ConstInt {
                    dest: 1,
                    value: size as i64,
                },
                IrInstruction::Call {
                    dest: 2,
                    func: "__berkeos_syscall_fread".to_string(),
                    args: vec![0, 1],
                },
            ]
        }

        /// Write to a file handle.
        /// Corresponds to SYS_FWRITE (14)
        pub fn write_ir(handle: usize, data: &str) -> Vec<IrInstruction> {
            vec![
                IrInstruction::ConstInt {
                    dest: 0,
                    value: handle as i64,
                },
                IrInstruction::ConstString {
                    dest: 1,
                    value: data.to_string(),
                },
                IrInstruction::Call {
                    dest: 2,
                    func: "__berkeos_syscall_fwrite".to_string(),
                    args: vec![0, 1],
                },
            ]
        }

        /// Close a file handle.
        /// Corresponds to SYS_FCLOSE (15)
        pub fn close_ir(handle: usize) -> Vec<IrInstruction> {
            vec![
                IrInstruction::ConstInt {
                    dest: 0,
                    value: handle as i64,
                },
                IrInstruction::Call {
                    dest: 1,
                    func: "__berkeos_syscall_fclose".to_string(),
                    args: vec![0],
                },
            ]
        }
    }

    /// Display/graphics submodule
    pub mod display {
        use super::*;

        /// Clear the framebuffer.
        /// Corresponds to SYS_FB_CLEAR (33)
        pub fn clear_ir(color: u32) -> Vec<IrInstruction> {
            vec![
                IrInstruction::ConstInt {
                    dest: 0,
                    value: color as i64,
                },
                IrInstruction::Call {
                    dest: 1,
                    func: "__berkeos_syscall_fb_clear".to_string(),
                    args: vec![0],
                },
            ]
        }

        /// Draw a single pixel.
        /// Corresponds to SYS_FB_PIXEL (31)
        pub fn draw_pixel_ir(x: usize, y: usize, color: u32) -> Vec<IrInstruction> {
            vec![
                IrInstruction::ConstInt {
                    dest: 0,
                    value: x as i64,
                },
                IrInstruction::ConstInt {
                    dest: 1,
                    value: y as i64,
                },
                IrInstruction::ConstInt {
                    dest: 2,
                    value: color as i64,
                },
                IrInstruction::Call {
                    dest: 3,
                    func: "__berkeos_syscall_fb_pixel".to_string(),
                    args: vec![0, 1, 2],
                },
            ]
        }

        /// Draw a rectangle.
        /// Corresponds to SYS_FB_RECT (32)
        pub fn draw_rect_ir(
            x: usize,
            y: usize,
            w: usize,
            h: usize,
            color: u32,
        ) -> Vec<IrInstruction> {
            vec![
                IrInstruction::ConstInt {
                    dest: 0,
                    value: x as i64,
                },
                IrInstruction::ConstInt {
                    dest: 1,
                    value: y as i64,
                },
                IrInstruction::ConstInt {
                    dest: 2,
                    value: w as i64,
                },
                IrInstruction::ConstInt {
                    dest: 3,
                    value: h as i64,
                },
                IrInstruction::ConstInt {
                    dest: 4,
                    value: color as i64,
                },
                IrInstruction::Call {
                    dest: 5,
                    func: "__berkeos_syscall_fb_rect".to_string(),
                    args: vec![0, 1, 2, 3, 4],
                },
            ]
        }

        /// Draw text at position.
        /// Corresponds to SYS_FB_TEXT (34)
        pub fn draw_text_ir(x: usize, y: usize, text: &str, color: u32) -> Vec<IrInstruction> {
            vec![
                IrInstruction::ConstInt {
                    dest: 0,
                    value: x as i64,
                },
                IrInstruction::ConstInt {
                    dest: 1,
                    value: y as i64,
                },
                IrInstruction::ConstString {
                    dest: 2,
                    value: text.to_string(),
                },
                IrInstruction::ConstInt {
                    dest: 3,
                    value: color as i64,
                },
                IrInstruction::Call {
                    dest: 4,
                    func: "__berkeos_syscall_fb_text".to_string(),
                    args: vec![0, 1, 2, 3],
                },
            ]
        }
    }

    /// Input submodule
    pub mod input {
        use super::*;

        /// Read a key from keyboard.
        /// Corresponds to SYS_READ_KEY (40)
        pub fn key_ir() -> Vec<IrInstruction> {
            vec![IrInstruction::Call {
                dest: 0,
                func: "__berkeos_syscall_read_key".to_string(),
                args: vec![],
            }]
        }
    }

    /// Window management submodule
    pub mod window {
        use super::*;

        /// Create a new window.
        /// Corresponds to SYS_WINDOW_NEW (60)
        pub fn new_ir(title: &str, width: usize, height: usize) -> Vec<IrInstruction> {
            vec![
                IrInstruction::ConstString {
                    dest: 0,
                    value: title.to_string(),
                },
                IrInstruction::ConstInt {
                    dest: 1,
                    value: width as i64,
                },
                IrInstruction::ConstInt {
                    dest: 2,
                    value: height as i64,
                },
                IrInstruction::Call {
                    dest: 3,
                    func: "__berkeos_syscall_window_new".to_string(),
                    args: vec![0, 1, 2],
                },
            ]
        }
    }
}

/// Generate syscall wrapper functions for BerkeOS.
/// These functions are added to the IR module and generate syscall bytecode.
pub fn generate_syscall_wrappers(module: &mut IrModule) {
    // __berkeos_syscall_sleep (SYS_SLEEP = 4)
    let mut sleep_fn = IrFunction::new("__berkeos_syscall_sleep".to_string());
    sleep_fn.add_param("ms".to_string());
    let mut sleep_block = IrBlock::new("entry".to_string());
    sleep_block.push(IrInstruction::Arg { dest: 0, index: 0 });
    sleep_block.push(IrInstruction::Return { value: Some(0) });
    sleep_fn.add_block(sleep_block);
    module.add_function(sleep_fn);

    // __berkeos_syscall_getpid (SYS_GETPID = 3)
    let mut getpid_fn = IrFunction::new("__berkeos_syscall_getpid".to_string());
    let mut getpid_block = IrBlock::new("entry".to_string());
    getpid_block.push(IrInstruction::Return { value: Some(0) });
    getpid_fn.add_block(getpid_block);
    module.add_function(getpid_fn);

    // __berkeos_syscall_fopen (SYS_FOPEN = 12)
    let mut fopen_fn = IrFunction::new("__berkeos_syscall_fopen".to_string());
    fopen_fn.add_param("path".to_string());
    fopen_fn.add_param("mode".to_string());
    let mut fopen_block = IrBlock::new("entry".to_string());
    fopen_block.push(IrInstruction::Arg { dest: 0, index: 0 });
    fopen_block.push(IrInstruction::Arg { dest: 1, index: 1 });
    fopen_block.push(IrInstruction::Return { value: Some(0) });
    fopen_fn.add_block(fopen_block);
    module.add_function(fopen_fn);

    // __berkeos_syscall_fread (SYS_FREAD = 13)
    let mut fread_fn = IrFunction::new("__berkeos_syscall_fread".to_string());
    fread_fn.add_param("handle".to_string());
    fread_fn.add_param("size".to_string());
    let mut fread_block = IrBlock::new("entry".to_string());
    fread_block.push(IrInstruction::Arg { dest: 0, index: 0 });
    fread_block.push(IrInstruction::Arg { dest: 1, index: 1 });
    fread_block.push(IrInstruction::Return { value: Some(0) });
    fread_fn.add_block(fread_block);
    module.add_function(fread_fn);

    // __berkeos_syscall_fwrite (SYS_FWRITE = 14)
    let mut fwrite_fn = IrFunction::new("__berkeos_syscall_fwrite".to_string());
    fwrite_fn.add_param("handle".to_string());
    fwrite_fn.add_param("data".to_string());
    let mut fwrite_block = IrBlock::new("entry".to_string());
    fwrite_block.push(IrInstruction::Arg { dest: 0, index: 0 });
    fwrite_block.push(IrInstruction::Arg { dest: 1, index: 1 });
    fwrite_block.push(IrInstruction::Return { value: Some(0) });
    fwrite_fn.add_block(fwrite_block);
    module.add_function(fwrite_fn);

    // __berkeos_syscall_fclose (SYS_FCLOSE = 15)
    let mut fclose_fn = IrFunction::new("__berkeos_syscall_fclose".to_string());
    fclose_fn.add_param("handle".to_string());
    let mut fclose_block = IrBlock::new("entry".to_string());
    fclose_block.push(IrInstruction::Arg { dest: 0, index: 0 });
    fclose_block.push(IrInstruction::Return { value: Some(0) });
    fclose_fn.add_block(fclose_block);
    module.add_function(fclose_fn);

    // __berkeos_syscall_fb_clear (SYS_FB_CLEAR = 33)
    let mut fb_clear_fn = IrFunction::new("__berkeos_syscall_fb_clear".to_string());
    fb_clear_fn.add_param("color".to_string());
    let mut fb_clear_block = IrBlock::new("entry".to_string());
    fb_clear_block.push(IrInstruction::Arg { dest: 0, index: 0 });
    fb_clear_block.push(IrInstruction::ReturnVoid);
    fb_clear_fn.add_block(fb_clear_block);
    module.add_function(fb_clear_fn);

    // __berkeos_syscall_fb_pixel (SYS_FB_PIXEL = 31)
    let mut fb_pixel_fn = IrFunction::new("__berkeos_syscall_fb_pixel".to_string());
    fb_pixel_fn.add_param("x".to_string());
    fb_pixel_fn.add_param("y".to_string());
    fb_pixel_fn.add_param("color".to_string());
    let mut fb_pixel_block = IrBlock::new("entry".to_string());
    fb_pixel_block.push(IrInstruction::Arg { dest: 0, index: 0 });
    fb_pixel_block.push(IrInstruction::Arg { dest: 1, index: 1 });
    fb_pixel_block.push(IrInstruction::Arg { dest: 2, index: 2 });
    fb_pixel_block.push(IrInstruction::ReturnVoid);
    fb_pixel_fn.add_block(fb_pixel_block);
    module.add_function(fb_pixel_fn);

    // __berkeos_syscall_fb_rect (SYS_FB_RECT = 32)
    let mut fb_rect_fn = IrFunction::new("__berkeos_syscall_fb_rect".to_string());
    fb_rect_fn.add_param("x".to_string());
    fb_rect_fn.add_param("y".to_string());
    fb_rect_fn.add_param("w".to_string());
    fb_rect_fn.add_param("h".to_string());
    fb_rect_fn.add_param("color".to_string());
    let mut fb_rect_block = IrBlock::new("entry".to_string());
    fb_rect_block.push(IrInstruction::Arg { dest: 0, index: 0 });
    fb_rect_block.push(IrInstruction::Arg { dest: 1, index: 1 });
    fb_rect_block.push(IrInstruction::Arg { dest: 2, index: 2 });
    fb_rect_block.push(IrInstruction::Arg { dest: 3, index: 3 });
    fb_rect_block.push(IrInstruction::Arg { dest: 4, index: 4 });
    fb_rect_block.push(IrInstruction::ReturnVoid);
    fb_rect_fn.add_block(fb_rect_block);
    module.add_function(fb_rect_fn);

    // __berkeos_syscall_fb_text (SYS_FB_TEXT = 34)
    let mut fb_text_fn = IrFunction::new("__berkeos_syscall_fb_text".to_string());
    fb_text_fn.add_param("x".to_string());
    fb_text_fn.add_param("y".to_string());
    fb_text_fn.add_param("text".to_string());
    fb_text_fn.add_param("color".to_string());
    let mut fb_text_block = IrBlock::new("entry".to_string());
    fb_text_block.push(IrInstruction::Arg { dest: 0, index: 0 });
    fb_text_block.push(IrInstruction::Arg { dest: 1, index: 1 });
    fb_text_block.push(IrInstruction::Arg { dest: 2, index: 2 });
    fb_text_block.push(IrInstruction::Arg { dest: 3, index: 3 });
    fb_text_block.push(IrInstruction::ReturnVoid);
    fb_text_fn.add_block(fb_text_block);
    module.add_function(fb_text_fn);

    // __berkeos_syscall_read_key (SYS_READ_KEY = 40)
    let mut read_key_fn = IrFunction::new("__berkeos_syscall_read_key".to_string());
    let mut read_key_block = IrBlock::new("entry".to_string());
    read_key_block.push(IrInstruction::Return { value: Some(0) });
    read_key_fn.add_block(read_key_block);
    module.add_function(read_key_fn);

    // __berkeos_syscall_window_new (SYS_WINDOW_NEW = 60)
    let mut window_new_fn = IrFunction::new("__berkeos_syscall_window_new".to_string());
    window_new_fn.add_param("title".to_string());
    window_new_fn.add_param("width".to_string());
    window_new_fn.add_param("height".to_string());
    let mut window_new_block = IrBlock::new("entry".to_string());
    window_new_block.push(IrInstruction::Arg { dest: 0, index: 0 });
    window_new_block.push(IrInstruction::Arg { dest: 1, index: 1 });
    window_new_block.push(IrInstruction::Arg { dest: 2, index: 2 });
    window_new_block.push(IrInstruction::Return { value: Some(0) });
    window_new_fn.add_block(window_new_block);
    module.add_function(window_new_fn);
}

/// Check if a function call is a berkeos builtin and generate appropriate IR
pub fn handle_berkeos_builtin(func_name: &str, args: &[String]) -> Option<Vec<IrInstruction>> {
    let parts: Vec<&str> = func_name.split('.').collect();

    if parts.len() < 2 || parts[0] != "berkeos" {
        return None;
    }

    match parts[1] {
        "process" => {
            if parts.len() == 3 {
                match parts[2] {
                    "sleep" => {
                        if let Some(ms_str) = args.first() {
                            if let Ok(ms) = ms_str.parse::<usize>() {
                                return Some(berkeos::process::sleep_ir(ms));
                            }
                        }
                    }
                    "getpid" => {
                        return Some(berkeos::process::getpid_ir());
                    }
                    _ => {}
                }
            }
        }
        "file" => {
            if parts.len() == 3 {
                match parts[2] {
                    "open" => {
                        if args.len() >= 2 {
                            return Some(berkeos::file::open_ir(&args[0], &args[1]));
                        }
                    }
                    "read" => {
                        if let Some(handle_str) = args.first() {
                            if let Ok(handle) = handle_str.parse::<usize>() {
                                let size = args
                                    .get(1)
                                    .and_then(|s| s.parse::<usize>().ok())
                                    .unwrap_or(512);
                                return Some(berkeos::file::read_ir(handle, size));
                            }
                        }
                    }
                    "write" => {
                        if args.len() >= 2 {
                            if let Ok(handle) = args[0].parse::<usize>() {
                                return Some(berkeos::file::write_ir(handle, &args[1]));
                            }
                        }
                    }
                    "close" => {
                        if let Some(handle_str) = args.first() {
                            if let Ok(handle) = handle_str.parse::<usize>() {
                                return Some(berkeos::file::close_ir(handle));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        "display" => {
            if parts.len() == 3 {
                match parts[2] {
                    "clear" => {
                        let color = args
                            .first()
                            .and_then(|s| s.parse::<u32>().ok())
                            .unwrap_or(0x000000);
                        return Some(berkeos::display::clear_ir(color));
                    }
                    "draw_pixel" => {
                        if args.len() >= 3 {
                            if let (Ok(x), Ok(y), Ok(color)) = (
                                args[0].parse::<usize>(),
                                args[1].parse::<usize>(),
                                args[2].parse::<u32>(),
                            ) {
                                return Some(berkeos::display::draw_pixel_ir(x, y, color));
                            }
                        }
                    }
                    "draw_rect" => {
                        if args.len() >= 5 {
                            if let (Ok(x), Ok(y), Ok(w), Ok(h), Ok(color)) = (
                                args[0].parse::<usize>(),
                                args[1].parse::<usize>(),
                                args[2].parse::<usize>(),
                                args[3].parse::<usize>(),
                                args[4].parse::<u32>(),
                            ) {
                                return Some(berkeos::display::draw_rect_ir(x, y, w, h, color));
                            }
                        }
                    }
                    "draw_text" => {
                        if args.len() >= 4 {
                            if let (Ok(x), Ok(y), Ok(color)) = (
                                args[0].parse::<usize>(),
                                args[1].parse::<usize>(),
                                args[3].parse::<u32>(),
                            ) {
                                return Some(berkeos::display::draw_text_ir(x, y, &args[2], color));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        "input" => {
            if parts.len() == 3 {
                match parts[2] {
                    "key" => {
                        return Some(berkeos::input::key_ir());
                    }
                    _ => {}
                }
            }
        }
        "window" => {
            if parts.len() == 3 {
                match parts[2] {
                    "new" => {
                        if args.len() >= 3 {
                            if let (Ok(width), Ok(height)) =
                                (args[1].parse::<usize>(), args[2].parse::<usize>())
                            {
                                return Some(berkeos::window::new_ir(&args[0], width, height));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }

    None
}
