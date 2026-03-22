//! GUI detector module for BerkeBex
//!
//! Detects GUI framework usage at compile time (static) and runtime (runtime hooks).

pub mod runtime;
pub mod static_detector;

pub use runtime::{
    disable_gui_guard, emit_gui_warning_syscall, enable_gui_guard, is_gui_guard_enabled, GuiHook,
    GuiHookRegistry,
};
pub use static_detector::{detect_gui_imports, GuiWarning};
